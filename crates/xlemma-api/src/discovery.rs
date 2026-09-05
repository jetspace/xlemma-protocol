use crate::{event_store::ApiJournalEvent, ApiError, AppState, VerificationJobRecord};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use xlemma_core::{FormalStatus, MessageId, ResearchCertificationStatus, VerificationProfileClass};
use xlemma_economics::{
    DeclaredEvidenceStatus, DiscoveryEnvelope, DiscoveryEvidenceSource, DiscoverySettlementPlan,
    QueueEntry, ResolvedDiscoveryEvidence, ServiceError,
};
use xlemma_xlmp::{XlmpEnvelope, XlmpMessage};

pub(crate) struct EvidenceSource<'a> {
    pub jobs: &'a BTreeMap<String, VerificationJobRecord>,
    pub messages: &'a BTreeMap<String, XlmpEnvelope>,
    pub projection: &'a xlemma_xlmp::ProtocolProjection,
}

fn invalid(_: impl std::fmt::Debug) -> ServiceError {
    ServiceError::Invalid("invalid authenticated research evidence")
}
fn digest(value: &str) -> String {
    format!("0x{}", value.rsplit(':').next().unwrap_or_default())
}

impl DiscoveryEvidenceSource for EvidenceSource<'_> {
    fn resolve(
        &self,
        id: &MessageId,
        at: DateTime<Utc>,
    ) -> Result<ResolvedDiscoveryEvidence, ServiceError> {
        let message = self.messages.get(id.as_str()).ok_or(ServiceError::Invalid(
            "evidence has not crossed authenticated XLMP ingress",
        ))?;
        match &message.message {
            XlmpMessage::Certificate(value) => {
                let c = &value.certificate;
                let job = self
                    .jobs
                    .get(c.job_id.as_str())
                    .ok_or(ServiceError::Invalid("missing verification job"))?;
                crate::validate_certificate_evidence(job, c, self.messages, at).map_err(invalid)?;
                let held = self
                    .projection
                    .quarantines
                    .values()
                    .any(|q| q.affected_claim_id == c.claim_id)
                    || self
                        .projection
                        .challenges
                        .values()
                        .any(|q| q.certificate_id == c.certificate_id);
                Ok(ResolvedDiscoveryEvidence {
                    job_id: c.job_id.clone(),
                    claim_id: c.claim_id.clone(),
                    artifact_id: c.artifact_id.clone(),
                    profile_id: c.verification_policy_id.clone(),
                    class: VerificationProfileClass::Formal,
                    evidence_roots: xlemma_economics::formal_discovery_roots(
                        &c.claim_id,
                        &c.proof_id,
                        &c.artifact_root,
                        &c.environment_root,
                        &c.dependency_root,
                        &c.axiom_set_root,
                    )?,
                    status: if held {
                        DeclaredEvidenceStatus::Inconclusive
                    } else {
                        DeclaredEvidenceStatus::Supported
                    },
                    final_after: c.challenge_window_ends_at,
                    verifier_clusters: c.operator_cluster_ids.iter().cloned().collect(),
                    certificate_digest: digest(c.certificate_id.as_str()),
                    observation_digest: digest(
                        xlemma_core::ArtifactId::derive(&c.observation_receipt_ids)
                            .map_err(invalid)?
                            .as_str(),
                    ),
                })
            }
            XlmpMessage::ResearchCertificate(value) => {
                if value.profile.class == VerificationProfileClass::Formal {
                    return Err(ServiceError::Invalid(
                        "formal research requires exact PoIR/checker evidence",
                    ));
                }
                let mut observations = BTreeMap::new();
                for prior in self.messages.values() {
                    if let XlmpMessage::ReproductionObservation(reveal) = &prior.message {
                        if reveal.job.job_id == value.job.job_id {
                            if reveal.job != value.job || reveal.profile != value.profile {
                                return Err(ServiceError::Invalid(
                                    "research job or profile changed",
                                ));
                            }
                            observations.insert(
                                reveal.observation.receipt_id.clone(),
                                reveal.observation.clone(),
                            );
                        }
                    }
                }
                let expected: BTreeSet<_> = value
                    .observations
                    .iter()
                    .map(|o| o.receipt_id.clone())
                    .collect();
                if observations.keys().cloned().collect::<BTreeSet<_>>() != expected
                    || value
                        .observations
                        .iter()
                        .any(|o| observations.get(&o.receipt_id) != Some(o))
                {
                    return Err(ServiceError::Invalid(
                        "research certificate omitted authenticated evidence",
                    ));
                }
                value
                    .certificate
                    .validate_against(&value.job, &value.profile, &value.observations)
                    .map_err(invalid)?;
                if value.certificate.issued_at > at {
                    return Err(ServiceError::Invalid(
                        "research certificate unavailable or quarantined",
                    ));
                }
                let mut status = match value.certificate.status {
                    ResearchCertificationStatus::Certified => DeclaredEvidenceStatus::Supported,
                    ResearchCertificationStatus::Failed => DeclaredEvidenceStatus::Rejected,
                    ResearchCertificationStatus::Divergent => DeclaredEvidenceStatus::Divergent,
                    ResearchCertificationStatus::Inconclusive => {
                        DeclaredEvidenceStatus::Inconclusive
                    }
                };
                if self
                    .projection
                    .quarantines
                    .values()
                    .any(|q| q.affected_claim_id == value.job.claim_id)
                {
                    status = DeclaredEvidenceStatus::Divergent;
                }
                Ok(ResolvedDiscoveryEvidence {
                    job_id: value.job.job_id.clone(),
                    claim_id: value.job.claim_id.clone(),
                    artifact_id: value.job.artifact_id.clone(),
                    profile_id: value.profile.policy_id.clone(),
                    class: value.profile.class,
                    evidence_roots: value
                        .job
                        .evidence_roots
                        .iter()
                        .map(|(kind, root)| {
                            Ok((
                                *kind,
                                root.parse()
                                    .or_else(|_| xlemma_core::ArtifactId::derive(root))
                                    .map_err(invalid)?,
                            ))
                        })
                        .collect::<Result<_, ServiceError>>()?,
                    status,
                    final_after: value.certificate.challenge_window_ends_at,
                    verifier_clusters: value
                        .observations
                        .iter()
                        .map(|o| o.operator_cluster_id.clone())
                        .collect(),
                    certificate_digest: digest(value.certificate.certificate_id.as_str()),
                    observation_digest: digest(
                        xlemma_core::ArtifactId::derive(&value.observations)
                            .map_err(invalid)?
                            .as_str(),
                    ),
                })
            }
            XlmpMessage::ObservationReveal(reveal) => {
                let job = self
                    .jobs
                    .get(reveal.observation.job_id.as_str())
                    .ok_or(ServiceError::Invalid("missing formal job"))?;
                let mut observations = BTreeMap::new();
                for o in &job.observations {
                    observations.insert(o.receipt_id.clone(), o.clone());
                }
                for prior in self.messages.values() {
                    if let XlmpMessage::ObservationReveal(r) = &prior.message {
                        if r.observation.job_id == job.job_id {
                            observations
                                .insert(r.observation.receipt_id.clone(), r.observation.clone());
                        }
                    }
                }
                let observations: Vec<_> = observations.into_values().collect();
                for o in &observations {
                    crate::validate_job_observation(job, o).map_err(invalid)?;
                }
                let result =
                    xlemma_consensus::evaluate_formal_consensus(&job.policy, &observations)
                        .map_err(invalid)?;
                // Passing observations without a certificate cannot unlock a discovery award.
                let status = match result.status {
                    FormalStatus::Rejected => DeclaredEvidenceStatus::Rejected,
                    FormalStatus::Divergent | FormalStatus::Quarantined => {
                        DeclaredEvidenceStatus::Divergent
                    }
                    _ => DeclaredEvidenceStatus::Inconclusive,
                };
                let proof = self
                    .messages
                    .values()
                    .find_map(|m| match &m.message {
                        XlmpMessage::ProofCandidate(p) if p.job_id == job.job_id => {
                            Some(&p.proof_id)
                        }
                        _ => None,
                    })
                    .ok_or(ServiceError::Invalid("missing formal proof candidate"))?;
                Ok(ResolvedDiscoveryEvidence {
                    job_id: job.job_id.clone(),
                    claim_id: job.claim_id.clone(),
                    artifact_id: job.artifact_id.clone(),
                    profile_id: job.policy_id.clone(),
                    class: VerificationProfileClass::Formal,
                    evidence_roots: xlemma_economics::formal_discovery_roots(
                        &job.claim_id,
                        proof,
                        &reveal.observation.artifact_root,
                        &reveal.observation.environment_root,
                        &reveal.observation.dependency_root,
                        &reveal.observation.axiom_set_root,
                    )?,
                    status,
                    final_after: at,
                    verifier_clusters: observations
                        .iter()
                        .map(|o| o.operator_cluster_id.clone())
                        .collect(),
                    certificate_digest: format!("0x{}", "0".repeat(64)),
                    observation_digest: digest(
                        xlemma_core::ArtifactId::derive(&observations)
                            .map_err(invalid)?
                            .as_str(),
                    ),
                })
            }
            _ => Err(ServiceError::Invalid(
                "message is not accepted verification evidence",
            )),
        }
    }
}

pub(crate) struct StrictDiscovery(DiscoveryEnvelope);
impl<'de> Deserialize<'de> for StrictDiscovery {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = serde_json::Value::deserialize(deserializer)?;
        let value: DiscoveryEnvelope =
            serde_json::from_value(raw.clone()).map_err(serde::de::Error::custom)?;
        if serde_json::to_value(&value).map_err(serde::de::Error::custom)? != raw {
            return Err(serde::de::Error::custom(
                "unknown or noncanonical discovery fields",
            ));
        }
        Ok(Self(value))
    }
}

pub(crate) async fn accept(
    State(state): State<AppState>,
    Json(StrictDiscovery(envelope)): Json<StrictDiscovery>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let at = Utc::now();
    let messages = state.messages.read().await;
    let jobs = state.jobs.read().await;
    let projection = state.projection.read().await;
    let source = EvidenceSource {
        jobs: &jobs,
        messages: &messages,
        projection: &projection,
    };
    let mut ledger = state.discovery.write().await;
    let mut next = ledger.clone();
    next.apply(&envelope, at, &source)
        .map_err(|e| ApiError::Invalid(e.to_string()))?;
    state.persist(ApiJournalEvent::DiscoveryAccepted {
        envelope: envelope.clone(),
        received_at: at,
    })?;
    *ledger = next;
    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({"command_id":envelope.command_id,"accepted_at":at})),
    ))
}
pub(crate) async fn overview(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(
        state
            .discovery
            .read()
            .await
            .overview()
            .map_err(|e| ApiError::Invalid(e.to_string()))?,
    ))
}
pub(crate) async fn queue(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<QueueEntry>>, ApiError> {
    let id = id
        .parse()
        .map_err(|_| ApiError::Invalid("invalid round identifier".into()))?;
    Ok(Json(
        state
            .discovery
            .read()
            .await
            .queue(&id)
            .map_err(|e| ApiError::Invalid(e.to_string()))?,
    ))
}
pub(crate) async fn settlement(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DiscoverySettlementPlan>, ApiError> {
    let id = id
        .parse()
        .map_err(|_| ApiError::Invalid("invalid round identifier".into()))?;
    state
        .discovery
        .read()
        .await
        .plan(&id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn history(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<DiscoveryEnvelope>>, ApiError> {
    let id = id
        .parse()
        .map_err(|_| ApiError::Invalid("invalid round identifier".into()))?;
    Ok(Json(
        state
            .discovery
            .read()
            .await
            .history(&id)
            .map_err(|e| ApiError::Invalid(e.to_string()))?
            .to_vec(),
    ))
}

pub(crate) async fn evidence(
    State(state): State<AppState>,
    Path((round, submission)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let round = round
        .parse()
        .map_err(|_| ApiError::Invalid("invalid round identifier".into()))?;
    let submission = submission
        .parse()
        .map_err(|_| ApiError::Invalid("invalid submission identifier".into()))?;
    let messages = state.messages.read().await;
    let jobs = state.jobs.read().await;
    let projection = state.projection.read().await;
    let source = EvidenceSource {
        jobs: &jobs,
        messages: &messages,
        projection: &projection,
    };
    Ok(Json(
        state
            .discovery
            .read()
            .await
            .evidence_publication(&round, &submission, &source, Utc::now())
            .map_err(|e| ApiError::Invalid(e.to_string()))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::EventJournal;
    use chrono::TimeZone;
    use xlemma_economics::{DiscoveryLedger, DiscoveryTrust};

    #[test]
    fn journal_replays_signed_discovery_and_rejects_replay_or_changed_trust() {
        let trust: DiscoveryTrust = serde_json::from_str(include_str!(
            "../../../examples/discovery/service-trust.json"
        ))
        .unwrap();
        let envelope: DiscoveryEnvelope = serde_json::from_str(include_str!(
            "../../../examples/discovery/service-create-envelope.json"
        ))
        .unwrap();
        let path = std::env::temp_dir().join(format!(
            "xlemma-discovery-journal-{}.jsonl",
            uuid::Uuid::new_v4()
        ));
        let (journal, _) = EventJournal::open_with_discovery(&path, Some(trust.clone())).unwrap();
        let event = ApiJournalEvent::DiscoveryAccepted {
            envelope: envelope.clone(),
            received_at: Utc.timestamp_opt(900, 0).unwrap(),
        };
        journal.append(event.clone()).unwrap();
        assert!(journal.append(event).is_err());
        drop(journal);
        let (journal, recovered) =
            EventJournal::open_with_discovery(&path, Some(trust.clone())).unwrap();
        assert!(recovered.discovery.contains(&envelope.command_id));
        assert_eq!(
            recovered.discovery.overview().unwrap()["accepted_commands"],
            1
        );
        drop(journal);
        let mut changed = trust;
        changed.network = "another-network".into();
        assert!(EventJournal::open_with_discovery(&path, Some(changed)).is_err());
        assert!(EventJournal::open_with_discovery(&path, None).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn formal_resolver_preserves_exact_roots_and_rejects_changed_artifacts() {
        let (job, certificate, mut messages) = crate::tests::certificate_fixture();
        let envelope = XlmpEnvelope::new(
            None,
            "fixture-source",
            Utc::now(),
            XlmpMessage::Certificate(xlemma_xlmp::CertificateMessage {
                certificate: certificate.clone(),
            }),
            "fixture-signature",
        )
        .unwrap();
        let id = envelope.message_id.clone();
        messages.insert(id.to_string(), envelope);
        let mut jobs = BTreeMap::from([(job.job_id.to_string(), job)]);
        let projection = Default::default();
        let source = EvidenceSource {
            jobs: &jobs,
            messages: &messages,
            projection: &projection,
        };
        let resolved = source.resolve(&id, Utc::now()).unwrap();
        assert_eq!(resolved.status, DeclaredEvidenceStatus::Supported);
        assert_eq!(resolved.profile_id, certificate.verification_policy_id);
        assert_eq!(resolved.evidence_roots.len(), 4);
        jobs.values_mut().next().unwrap().artifact_root = "blake3:substituted".into();
        assert!(EvidenceSource {
            jobs: &jobs,
            messages: &messages,
            projection: &projection
        }
        .resolve(&id, Utc::now())
        .is_err());
    }

    #[test]
    fn discovery_json_rejects_unknown_nested_fields() {
        let mut value: serde_json::Value = serde_json::from_str(include_str!(
            "../../../examples/discovery/service-create-envelope.json"
        ))
        .unwrap();
        assert!(serde_json::from_value::<StrictDiscovery>(value.clone()).is_ok());
        value["command"]["policy"]["economics"]["hidden_reward_multiplier"] =
            serde_json::json!(100);
        assert!(serde_json::from_value::<StrictDiscovery>(value).is_err());
    }

    #[test]
    fn unsupported_or_absent_evidence_cannot_become_a_reward_certificate() {
        let source = EvidenceSource {
            jobs: &BTreeMap::new(),
            messages: &BTreeMap::new(),
            projection: &Default::default(),
        };
        assert!(source
            .resolve(&MessageId::derive(&"invented").unwrap(), Utc::now())
            .is_err());
        assert_eq!(
            DiscoveryLedger::default().overview().unwrap()["rounds"],
            serde_json::json!([])
        );
    }
}

//! Append-only, deterministic projection of canonical XLMP research state.
//!
//! The projection is deliberately storage-engine neutral. A service can replay
//! the hash-chained XLMP journal into this structure, compare its state root
//! with another indexer, and answer research-graph queries without treating a
//! database row as protocol truth.

use crate::{FinalizeMessage, XlmpEnvelope, XlmpError, XlmpMessage};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xlemma_core::{
    AvailabilityReceipt, CertificateId, Challenge, ChallengeId, ClaimId, ClaimManifest,
    ComputeReceipt, CreditId, DependencyDividend, DividendId, LemmaCapsule, LemmaId, License,
    LicenseId, MessageId, PoIRCertificate, ProofId, ProofManifest, PublicationId,
    PublicationRecord, QuarantineId, QuarantineRecord, ResearchCredit, ResearchVault, ResearcherId,
    ResearcherNodeManifest, RevenueEvent, RevenueEventId, RightsManifest, TheoryId, TheoryManifest,
    VaultId,
};

#[derive(Clone, Debug, Default)]
pub struct ProtocolProjection {
    message_ids: BTreeSet<MessageId>,
    pub researchers: BTreeMap<ResearcherId, ResearcherNodeManifest>,
    pub theories: BTreeMap<TheoryId, TheoryManifest>,
    pub claims: BTreeMap<ClaimId, ClaimManifest>,
    pub claim_roots: BTreeMap<ClaimId, (String, String)>,
    pub contribution_manifests: BTreeMap<String, xlemma_core::ContributionManifest>,
    pub rights_manifests: BTreeMap<String, RightsManifest>,
    pub proofs: BTreeMap<ProofId, ProofManifest>,
    pub certificates: BTreeMap<CertificateId, PoIRCertificate>,
    pub challenges: BTreeMap<ChallengeId, Challenge>,
    pub quarantines: BTreeMap<QuarantineId, QuarantineRecord>,
    pub finalizations: BTreeMap<CertificateId, FinalizeMessage>,
    pub compute_receipts: BTreeMap<xlemma_core::ReceiptId, ComputeReceipt>,
    pub research_credits: BTreeMap<CreditId, ResearchCredit>,
    pub vault_snapshots: BTreeMap<VaultId, Vec<ResearchVault>>,
    pub revenue_events: BTreeMap<RevenueEventId, RevenueEvent>,
    pub dividends: BTreeMap<DividendId, DependencyDividend>,
    pub licenses: BTreeMap<LicenseId, License>,
    pub capsules: BTreeMap<LemmaId, LemmaCapsule>,
    pub publications: BTreeMap<PublicationId, PublicationRecord>,
    pub availability: BTreeMap<xlemma_core::ReceiptId, AvailabilityReceipt>,
}

impl ProtocolProjection {
    pub fn replay<'a>(
        envelopes: impl IntoIterator<Item = &'a XlmpEnvelope>,
    ) -> Result<Self, ProjectionError> {
        let mut projection = Self::default();
        for envelope in envelopes {
            projection.apply(envelope)?;
        }
        Ok(projection)
    }

    pub fn apply(&mut self, envelope: &XlmpEnvelope) -> Result<(), ProjectionError> {
        envelope.validate_integrity()?;
        if self.message_ids.contains(&envelope.message_id) {
            return Err(ProjectionError::DuplicateMessage(
                envelope.message_id.to_string(),
            ));
        }

        match &envelope.message {
            XlmpMessage::Researcher(message) => insert_unique(
                &mut self.researchers,
                message.researcher.researcher_id.clone(),
                message.researcher.clone(),
            )?,
            XlmpMessage::Theory(message) => insert_unique(
                &mut self.theories,
                message.theory_id.clone(),
                message.theory.clone(),
            )?,
            XlmpMessage::Claim(message) => {
                require_key(&self.theories, &message.claim.theory_id, "theory")?;
                insert_unique(
                    &mut self.claims,
                    message.claim_id.clone(),
                    message.claim.clone(),
                )?;
                self.claim_roots.insert(
                    message.claim_id.clone(),
                    (
                        message.contribution_manifest_hash.clone(),
                        message.rights_manifest_hash.clone(),
                    ),
                );
            }
            XlmpMessage::Contribution(message) => {
                require_key(&self.claims, &message.manifest.claim_id, "claim")?;
                let claim_message = self.claim_message(&message.manifest.claim_id);
                if claim_message.is_none_or(|roots| roots.0 != message.manifest_hash) {
                    return Err(ProjectionError::ReferenceMismatch("contribution manifest"));
                }
                insert_unique(
                    &mut self.contribution_manifests,
                    message.manifest_hash.clone(),
                    message.manifest.clone(),
                )?;
            }
            XlmpMessage::Rights(message) => {
                require_key(&self.claims, &message.manifest.claim_id, "claim")?;
                let claim_message = self.claim_message(&message.manifest.claim_id);
                if claim_message.is_none_or(|roots| roots.1 != message.manifest_hash) {
                    return Err(ProjectionError::ReferenceMismatch("rights manifest"));
                }
                insert_unique(
                    &mut self.rights_manifests,
                    message.manifest_hash.clone(),
                    message.manifest.clone(),
                )?;
            }
            XlmpMessage::ProofCandidate(message) => {
                require_key(&self.claims, &message.proof.claim_id, "claim")?;
                if message.proof.artifact_id != message.artifact_id {
                    return Err(ProjectionError::ReferenceMismatch("proof candidate"));
                }
                insert_unique(
                    &mut self.proofs,
                    message.proof_id.clone(),
                    message.proof.clone(),
                )?;
            }
            XlmpMessage::Certificate(message) => {
                let proof = require_key(&self.proofs, &message.certificate.proof_id, "proof")?;
                let claim = require_key(&self.claims, &message.certificate.claim_id, "claim")?;
                if proof.claim_id != message.certificate.claim_id
                    || proof.artifact_id != message.certificate.artifact_id
                    || claim.theory_id != message.certificate.theory_id
                    || !message.certificate.has_independent_reproduction()
                {
                    return Err(ProjectionError::ReferenceMismatch("PoIR certificate"));
                }
                insert_unique(
                    &mut self.certificates,
                    message.certificate.certificate_id.clone(),
                    message.certificate.clone(),
                )?;
            }
            XlmpMessage::Challenge(message) => {
                require_key(
                    &self.certificates,
                    &message.challenge.certificate_id,
                    "certificate",
                )?;
                validate_parent(
                    &self.challenges,
                    message.challenge.supersedes.as_ref(),
                    "challenge",
                )?;
                if let Some(parent) = &message.challenge.supersedes {
                    let prior = &self.challenges[parent];
                    if prior.certificate_id != message.challenge.certificate_id
                        || prior.challenger != message.challenge.challenger
                        || prior.kind != message.challenge.kind
                        || prior.opened_at != message.challenge.opened_at
                        || self
                            .challenges
                            .values()
                            .any(|c| c.supersedes.as_ref() == Some(parent))
                    {
                        return Err(ProjectionError::InvalidSupersession("challenge"));
                    }
                }
                insert_unique(
                    &mut self.challenges,
                    message.challenge.challenge_id.clone(),
                    message.challenge.clone(),
                )?;
            }
            XlmpMessage::Quarantine(message) => {
                let certificate = require_key(
                    &self.certificates,
                    &message.record.certificate_id,
                    "certificate",
                )?;
                if certificate.claim_id != message.record.affected_claim_id
                    || message.record.challenge_id.as_ref().is_some_and(|id| {
                        self.challenges.get(id).is_none_or(|challenge| {
                            challenge.certificate_id != message.record.certificate_id
                        })
                    })
                {
                    return Err(ProjectionError::ReferenceMismatch("quarantine"));
                }
                validate_parent(
                    &self.quarantines,
                    message.record.supersedes.as_ref(),
                    "quarantine",
                )?;
                if message.record.supersedes.as_ref().is_some_and(|parent| {
                    self.quarantines[parent].certificate_id != message.record.certificate_id
                }) {
                    return Err(ProjectionError::InvalidSupersession("quarantine"));
                }
                insert_unique(
                    &mut self.quarantines,
                    message.record.quarantine_id.clone(),
                    message.record.clone(),
                )?;
            }
            XlmpMessage::Finalize(message) => {
                let certificate =
                    require_key(&self.certificates, &message.certificate_id, "certificate")?;
                let unresolved_challenge = self.has_unresolved_challenge(&message.certificate_id);
                let quarantined = self
                    .quarantines
                    .values()
                    .any(|record| record.certificate_id == message.certificate_id);
                if certificate.claim_id != message.claim_id
                    || message.finalized_at < certificate.challenge_window_ends_at
                    || unresolved_challenge
                    || quarantined
                {
                    return Err(ProjectionError::ReferenceMismatch("finalization"));
                }
                insert_unique(
                    &mut self.finalizations,
                    message.certificate_id.clone(),
                    message.clone(),
                )?;
            }
            XlmpMessage::ComputeReceipt(message) => insert_unique(
                &mut self.compute_receipts,
                message.receipt.receipt_id.clone(),
                message.receipt.clone(),
            )?,
            XlmpMessage::ResearchCredit(message) => {
                require_key(
                    &self.researchers,
                    &message.credit.researcher_id,
                    "researcher",
                )?;
                insert_unique(
                    &mut self.research_credits,
                    message.credit.credit_id.clone(),
                    message.credit.clone(),
                )?;
            }
            XlmpMessage::ResearchVault(message) => {
                require_key(
                    &self.researchers,
                    &message.vault.researcher_id,
                    "researcher",
                )?;
                if self
                    .vault_snapshots
                    .get(&message.vault.vault_id)
                    .is_some_and(|snapshots| {
                        snapshots.iter().any(|snapshot| {
                            snapshot.state_root == message.vault.state_root
                                || snapshot.observed_at >= message.vault.observed_at
                        })
                    })
                {
                    return Err(ProjectionError::InvalidSupersession("vault snapshot"));
                }
                self.vault_snapshots
                    .entry(message.vault.vault_id.clone())
                    .or_default()
                    .push(message.vault.clone());
            }
            XlmpMessage::Revenue(message) => {
                require_key(&self.claims, &message.event.claim_id, "claim")?;
                if !self
                    .publications
                    .values()
                    .any(|record| record.claim_id == message.event.claim_id)
                {
                    return Err(ProjectionError::MissingPrerequisite("publication"));
                }
                insert_unique(
                    &mut self.revenue_events,
                    message.event.revenue_event_id.clone(),
                    message.event.clone(),
                )?;
            }
            XlmpMessage::DependencyDividend(message) => {
                let revenue = require_key(
                    &self.revenue_events,
                    &message.dividend.revenue_event_id,
                    "revenue event",
                )?;
                require_key(
                    &self.claims,
                    &message.dividend.upstream_claim_id,
                    "upstream claim",
                )?;
                if revenue.claim_id != message.dividend.downstream_claim_id {
                    return Err(ProjectionError::ReferenceMismatch("dependency dividend"));
                }
                insert_unique(
                    &mut self.dividends,
                    message.dividend.dividend_id.clone(),
                    message.dividend.clone(),
                )?;
            }
            XlmpMessage::License(message) => {
                if !self
                    .rights_manifests
                    .contains_key(&message.license.rights_manifest_hash)
                {
                    return Err(ProjectionError::MissingPrerequisite("rights manifest"));
                }
                validate_parent(
                    &self.licenses,
                    message.license.supersedes.as_ref(),
                    "license",
                )?;
                insert_unique(
                    &mut self.licenses,
                    message.license.license_id.clone(),
                    message.license.clone(),
                )?;
            }
            XlmpMessage::Capsule(message) => {
                let claim = require_key(&self.claims, &message.capsule.claim_id, "claim")?;
                require_key(&self.theories, &message.capsule.theory_id, "theory")?;
                let roots = self
                    .claim_message(&message.capsule.claim_id)
                    .ok_or(ProjectionError::MissingPrerequisite("claim roots"))?;
                if claim.theory_id != message.capsule.theory_id
                    || roots.0 != message.capsule.contribution_manifest_hash
                    || roots.1 != message.capsule.rights_manifest_hash
                    || message.capsule.proof_id.as_ref().is_some_and(|id| {
                        self.proofs.get(id).is_some_and(|proof| {
                            proof.claim_id != message.capsule.claim_id
                                || proof.artifact_id != message.capsule.artifact_id
                        })
                    })
                {
                    return Err(ProjectionError::ReferenceMismatch("capsule"));
                }
                if message
                    .capsule
                    .proof_id
                    .as_ref()
                    .is_some_and(|id| !self.proofs.contains_key(id))
                    || !self
                        .contribution_manifests
                        .contains_key(&message.capsule.contribution_manifest_hash)
                    || !self
                        .rights_manifests
                        .contains_key(&message.capsule.rights_manifest_hash)
                {
                    return Err(ProjectionError::MissingPrerequisite(
                        "capsule proof, contribution, or rights",
                    ));
                }
                validate_parent(
                    &self.capsules,
                    message.capsule.supersedes.as_ref(),
                    "capsule",
                )?;
                insert_unique(
                    &mut self.capsules,
                    message.capsule.lemma_id.clone(),
                    message.capsule.clone(),
                )?;
            }
            XlmpMessage::Publish(message) => {
                let certificate = require_key(
                    &self.certificates,
                    &message.publication.certificate_id,
                    "certificate",
                )?;
                let finalization = require_key(
                    &self.finalizations,
                    &message.publication.certificate_id,
                    "finalization",
                )?;
                let capsule_matches = self.capsules.values().any(|capsule| {
                    capsule.claim_id == message.publication.claim_id
                        && capsule.proof_id.as_ref() == Some(&message.publication.proof_id)
                        && capsule.artifact_id == message.publication.artifact_id
                        && capsule.rights_manifest_hash == message.publication.rights_manifest_hash
                });
                let licenses_known = message.publication.license_ids.iter().all(|id| {
                    self.licenses.get(id).is_some_and(|license| {
                        license.rights_manifest_hash == message.publication.rights_manifest_hash
                    })
                });
                let quarantined = self.quarantines.values().any(|record| {
                    record.certificate_id == message.publication.certificate_id
                        || record.affected_claim_id == message.publication.claim_id
                });
                if certificate.claim_id != message.publication.claim_id
                    || certificate.proof_id != message.publication.proof_id
                    || certificate.artifact_id != message.publication.artifact_id
                    || !capsule_matches
                    || !licenses_known
                    || quarantined
                    || self.has_unresolved_challenge(&message.publication.certificate_id)
                    || message.publication.published_at < finalization.finalized_at
                {
                    return Err(ProjectionError::ReferenceMismatch("publication"));
                }
                validate_parent(
                    &self.publications,
                    message.publication.supersedes.as_ref(),
                    "publication",
                )?;
                insert_unique(
                    &mut self.publications,
                    message.publication.publication_id.clone(),
                    message.publication.clone(),
                )?;
            }
            XlmpMessage::Availability(message) => {
                let known_artifact = self
                    .proofs
                    .values()
                    .any(|proof| proof.artifact_id == message.receipt.artifact_id)
                    || self
                        .capsules
                        .values()
                        .any(|capsule| capsule.artifact_id == message.receipt.artifact_id);
                if !known_artifact {
                    return Err(ProjectionError::MissingPrerequisite("artifact"));
                }
                insert_unique(
                    &mut self.availability,
                    message.receipt.receipt_id.clone(),
                    message.receipt.clone(),
                )?;
            }
            _ => {}
        }

        self.message_ids.insert(envelope.message_id.clone());
        Ok(())
    }

    pub fn state_root(&self) -> Result<String, ProjectionError> {
        let digest =
            xlemma_core::canonical_json_hash("xlmp-protocol-projection-v1", &self.message_ids)?;
        Ok(format!("blake3:{}", hex::encode(digest)))
    }

    fn claim_message(&self, claim_id: &ClaimId) -> Option<(String, String)> {
        // The canonical claim roots are recovered from the accepted message
        // log so the projection never invents rights or contribution links.
        // The compact projection stores them temporarily in the maps below.
        self.claim_roots.get(claim_id).cloned()
    }

    fn has_unresolved_challenge(&self, certificate_id: &CertificateId) -> bool {
        self.challenges.values().any(|challenge| {
            challenge.certificate_id == *certificate_id
                && matches!(
                    challenge.status,
                    xlemma_core::ChallengeStatus::Open
                        | xlemma_core::ChallengeStatus::EvidenceRequested
                )
                && !self
                    .challenges
                    .values()
                    .any(|candidate| candidate.supersedes.as_ref() == Some(&challenge.challenge_id))
        })
    }
}

fn require_key<'a, K: Ord, V>(
    map: &'a BTreeMap<K, V>,
    key: &K,
    label: &'static str,
) -> Result<&'a V, ProjectionError> {
    map.get(key)
        .ok_or(ProjectionError::MissingPrerequisite(label))
}

fn insert_unique<K: Ord, V>(
    map: &mut BTreeMap<K, V>,
    key: K,
    value: V,
) -> Result<(), ProjectionError> {
    if map.contains_key(&key) {
        return Err(ProjectionError::DuplicateObject);
    }
    map.insert(key, value);
    Ok(())
}

fn validate_parent<K: Ord, V>(
    map: &BTreeMap<K, V>,
    parent: Option<&K>,
    label: &'static str,
) -> Result<(), ProjectionError> {
    if parent.is_some_and(|id| !map.contains_key(id)) {
        return Err(ProjectionError::InvalidSupersession(label));
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum ProjectionError {
    #[error(transparent)]
    Envelope(#[from] XlmpError),
    #[error(transparent)]
    Canonicalization(#[from] xlemma_core::CanonicalizationError),
    #[error("XLMP message already exists: {0}")]
    DuplicateMessage(String),
    #[error("content-derived protocol object already exists")]
    DuplicateObject,
    #[error("protocol object requires a prior {0}")]
    MissingPrerequisite(&'static str),
    #[error("protocol object has a mismatched {0} reference")]
    ReferenceMismatch(&'static str),
    #[error("append-only {0} supersession is invalid")]
    InvalidSupersession(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        manifest_root, AvailabilityMessage, CapsuleMessage, CertificateMessage, ChallengeMessage,
        ClaimMessage, ComputeReceiptMessage, ContributionMessage, DependencyDividendMessage,
        FinalizeMessage, LicenseMessage, ProofCandidateMessage, PublishMessage, QuarantineMessage,
        ResearchCreditMessage, ResearchVaultMessage, ResearcherMessage, RevenueMessage,
        RightsMessage, TheoryMessage,
    };
    use chrono::{Duration, TimeZone, Utc};
    use std::collections::BTreeMap;
    use xlemma_core::{
        Amount, ArtifactId, AssuranceLevel, CapsuleEconomicMode, ChallengeKind, ChallengeStatus,
        CheckerFamily, ComputeQuoteId, ComputeService, ContributionManifest, ContributionRole,
        ContributionShare, FormalStatus, NoveltyDecision, OperatorClusterId, ReceiptId,
        RevenueRoute, RevenueWaterfall, RightsClaim, RightsKind,
    };

    fn envelope(message: XlmpMessage) -> XlmpEnvelope {
        XlmpEnvelope::new(
            None,
            "did:key:projection-test",
            Utc.with_ymd_and_hms(2026, 9, 4, 12, 0, 0).unwrap(),
            message,
            "test-signature",
        )
        .unwrap()
    }

    fn amount(units: u128) -> Amount {
        Amount::new(units, "USDC", 6)
    }

    #[test]
    fn complete_native_research_lifecycle_replays_without_losing_lineage() {
        let at = Utc.with_ymd_and_hms(2026, 9, 4, 12, 0, 0).unwrap();
        let policy_id = xlemma_core::PolicyId::derive(&"policy").unwrap();
        let theory = TheoryManifest {
            protocol_version: "XLMP/1".into(),
            lean_toolchain: "leanprover/lean4:v4.33.1".into(),
            dependency_merkle_root: "blake3:dependency-root".into(),
            trust_policy_id: policy_id.clone(),
            checker_policy_id: policy_id.clone(),
            permitted_axioms: vec!["propext".into()],
            canonical_encoding: "xlemma-lean-expr-v1".into(),
        };
        let theory_id = theory.derive_theory_id().unwrap();
        let claim = ClaimManifest {
            protocol_version: "XLMP/1".into(),
            theory_id: theory_id.clone(),
            canonical_elaborated_type: "forall p : Prop, p -> p".into(),
            declaration_name: "XLemma.identity".into(),
            source_artifact: None,
            created_at: at,
        };
        let claim_id = claim.derive_claim_id().unwrap();
        let researcher_id = ResearcherId::derive(&"researcher").unwrap();
        let researcher = ResearcherNodeManifest {
            researcher_id: researcher_id.clone(),
            verified_user_id: None,
            user_credential_id: None,
            display_name: Some("Sovereign Researcher".into()),
            identity_keys: vec!["did:key:researcher".into()],
            research_credit_asset: "RSEARCH".into(),
            research_vault: "vault:researcher".into(),
            governance_policy_id: policy_id.clone(),
            contribution_identity_root: "blake3:contribution-identity".into(),
            reputation_root: "blake3:researcher-reputation".into(),
            supported_domains: vec!["formal-mathematics".into()],
            created_at: at,
        };
        let contribution = ContributionManifest {
            claim_id: claim_id.clone(),
            contributors: vec![ContributionShare {
                contributor: researcher_id.clone(),
                roles: vec![ContributionRole::ProofDiscoverer],
                share_bps: 10_000,
                evidence_root: "blake3:contribution-evidence".into(),
                signed_at: at,
                signature: "researcher-signature".into(),
            }],
            machine_contributions: vec![],
            amendment_parent: None,
            dispute_status: "undisputed".into(),
        };
        let contribution_root = manifest_root("contribution-manifest-v1", &contribution).unwrap();
        let rights = RightsManifest {
            claim_id: claim_id.clone(),
            originator_attribution_nontransferable: true,
            claims: vec![RightsClaim {
                kind: RightsKind::NoExclusiveRightClaimed,
                controller: "did:key:researcher".into(),
                jurisdiction: None,
                source_agreement_hash: None,
                transferable: false,
                sublicensable: false,
                limitations: vec![],
            }],
            employer_university_grant_clearance: "self-authored; no conflicting claim".into(),
            clearance_evidence_root: Some("blake3:rights-clearance".into()),
            legal_wrapper: None,
            signed_by: vec!["did:key:researcher".into()],
            signed_at: at,
        };
        let rights_root = manifest_root("rights-manifest-v1", &rights).unwrap();
        let upstream_claim = ClaimManifest {
            protocol_version: "XLMP/1".into(),
            theory_id: theory_id.clone(),
            canonical_elaborated_type: "forall p : Prop, p = p".into(),
            declaration_name: "XLemma.reflexive".into(),
            source_artifact: None,
            created_at: at,
        };
        let upstream_claim_id = upstream_claim.derive_claim_id().unwrap();
        let artifact_id = ArtifactId::derive(&"artifact").unwrap();
        let proof = ProofManifest {
            protocol_version: "XLMP/1".into(),
            claim_id: claim_id.clone(),
            canonical_proof_object: "fun p h => h".into(),
            artifact_id: artifact_id.clone(),
            direct_dependencies: vec![],
            dependency_root: "blake3:proof-dependencies".into(),
            observed_axioms: vec!["propext".into()],
        };
        let proof_id = proof.derive_proof_id().unwrap();
        let job_id = xlemma_core::JobId::derive(&"job").unwrap();
        let compute_receipt = {
            let mut value = ComputeReceipt {
                receipt_id: ReceiptId::derive(&"placeholder").unwrap(),
                job_id: job_id.clone(),
                quote_id: Some(ComputeQuoteId::derive(&"quote").unwrap()),
                service: ComputeService::ProofSearch,
                provider: "provider-neutral-prover".into(),
                implementation_id: "prover:reference".into(),
                implementation_snapshot: Some("sha256:prover-snapshot".into()),
                execution_parameters: BTreeMap::from([("attempts".into(), "4".into())]),
                request_hash: "blake3:compute-request".into(),
                context_root: "blake3:compute-context".into(),
                metering: BTreeMap::from([("attempts".into(), 4)]),
                charged_amount: amount(75_000),
                output_artifact_roots: vec!["blake3:proof-candidate".into()],
                completed_at: at + Duration::minutes(5),
                signature: "compute-provider-signature".into(),
            };
            value.receipt_id = value.derive_receipt_id().unwrap();
            value
        };
        let credit = {
            let mut value = ResearchCredit {
                credit_id: CreditId::derive(&"placeholder").unwrap(),
                researcher_id: researcher_id.clone(),
                credit_amount: Amount::new(500_000, "RSEARCH", 6),
                backing_asset_amount: amount(500_000),
                backing_value_in_credit_units: 500_000,
                valuation_policy_id: policy_id.clone(),
                backing_reference: "settlement:backing-deposit".into(),
                issued_at: at,
                signature: "vault-authority-signature".into(),
            };
            value.credit_id = value.derive_credit_id().unwrap();
            value
        };
        let vault = {
            let mut value = ResearchVault {
                vault_id: VaultId::derive(&"placeholder").unwrap(),
                researcher_id: researcher_id.clone(),
                credit_asset: "RSEARCH".into(),
                backing_assets: BTreeMap::from([("USDC".into(), amount(500_000))]),
                backing_value_in_credit_units: 500_000,
                outstanding_credit_units: 500_000,
                valuation_policy_id: policy_id.clone(),
                state_root: "blake3:vault-state".into(),
                observed_at: at,
                signature: "vault-authority-signature".into(),
            };
            value.vault_id = value.derive_vault_id().unwrap();
            value
        };
        let observation_ids = vec![
            ReceiptId::derive(&"observation-a").unwrap(),
            ReceiptId::derive(&"observation-b").unwrap(),
        ];
        let mut certificate = PoIRCertificate {
            certificate_id: CertificateId::derive(&"certificate").unwrap(),
            job_id: job_id.clone(),
            theory_id: theory_id.clone(),
            claim_id: claim_id.clone(),
            proof_id: proof_id.clone(),
            artifact_id: artifact_id.clone(),
            verification_policy_id: policy_id,
            observation_receipt_ids: observation_ids.clone(),
            checker_families: vec![CheckerFamily::LeanKernel, CheckerFamily::Nanoda],
            operator_cluster_ids: vec![
                OperatorClusterId::derive(&"operator-a").unwrap(),
                OperatorClusterId::derive(&"operator-b").unwrap(),
            ],
            artifact_root: "blake3:artifact-root".into(),
            environment_root: "blake3:environment-root".into(),
            dependency_root: "blake3:proof-dependencies".into(),
            axiom_set_root: "blake3:axiom-root".into(),
            formal_status: FormalStatus::Certified,
            assurance_level: AssuranceLevel::FormallyCertified,
            issued_at: at,
            challenge_window_ends_at: at + Duration::hours(1),
            aggregate_signature: "committee-signature".into(),
        };
        certificate.certificate_id = certificate.derive_certificate_id().unwrap();
        let license = {
            let mut value = License {
                license_id: LicenseId::derive(&"placeholder").unwrap(),
                rights_manifest_hash: rights_root.clone(),
                licensor: "did:key:researcher".into(),
                licensee: "did:key:reuser".into(),
                mode: CapsuleEconomicMode::Commons,
                scope: vec!["copy and verify public artifact".into()],
                economic_terms: None,
                consideration_receipt_id: None,
                effective_at: at + Duration::hours(2),
                expires_at: None,
                supersedes: None,
                signatures: vec!["bilateral-signature".into()],
            };
            value.license_id = value.derive_license_id().unwrap();
            value
        };
        let capsule = {
            let mut value = LemmaCapsule {
                lemma_id: LemmaId::derive(&"placeholder").unwrap(),
                theory_id: theory_id.clone(),
                claim_id: claim_id.clone(),
                proof_id: Some(proof_id.clone()),
                artifact_id: artifact_id.clone(),
                presentation_ids: vec!["xlpresentation:identity".into()],
                origin_certificate_id: ReceiptId::derive(&"origin").unwrap(),
                contribution_manifest_hash: contribution_root.clone(),
                rights_manifest_hash: rights_root.clone(),
                dependency_root: "blake3:proof-dependencies".into(),
                verification_receipt_ids: observation_ids,
                novelty_receipt_ids: vec![],
                statement_alignment_receipt_ids: vec![],
                economic_mode: CapsuleEconomicMode::Commons,
                revenue_route: RevenueRoute {
                    settlement_asset: "USDC".into(),
                    researcher_vault: "vault:researcher".into(),
                    waterfall: RevenueWaterfall {
                        creator_pool_bps: 7_000,
                        upstream_dependency_pool_bps: 0,
                        reverification_security_pool_bps: 1_000,
                        open_research_pool_bps: 1_000,
                        dispute_insurance_pool_bps: 500,
                        protocol_operations_bps: 500,
                    },
                    contributor_manifest_hash: contribution_root.clone(),
                    economic_policy_root: "blake3:commons-policy".into(),
                    dependency_reward_cap_bps: 0,
                    auto_compound_bps_by_researcher: BTreeMap::new(),
                },
                formal_status: FormalStatus::Certified,
                novelty_status: NoveltyDecision::Incremental,
                parent_capsule: None,
                supersedes: None,
                created_at: at + Duration::hours(2),
                metadata: BTreeMap::new(),
            };
            value.lemma_id = value.derive_lemma_id().unwrap();
            value
        };
        let publication = {
            let mut value = PublicationRecord {
                publication_id: PublicationId::derive(&"placeholder").unwrap(),
                claim_id: claim_id.clone(),
                proof_id: proof_id.clone(),
                certificate_id: certificate.certificate_id.clone(),
                artifact_id: artifact_id.clone(),
                rights_manifest_hash: rights_root.clone(),
                license_ids: vec![license.license_id.clone()],
                locations: vec!["ipfs:bafy-publication".into()],
                published_at: at + Duration::hours(2),
                supersedes: None,
                signature: "publisher-signature".into(),
            };
            value.publication_id = value.derive_publication_id().unwrap();
            value
        };
        let revenue = {
            let mut value = RevenueEvent {
                revenue_event_id: RevenueEventId::derive(&"placeholder").unwrap(),
                claim_id: claim_id.clone(),
                source: "licensed research service".into(),
                related_party: false,
                settlement_receipt_id: ReceiptId::derive(&"settlement").unwrap(),
                gross_collected: amount(1_000_000),
                refunds: amount(50_000),
                service_costs: amount(100_000),
                reserves: amount(50_000),
                realized_at: at + Duration::hours(3),
                evidence_root: "blake3:settlement-evidence".into(),
                signature: "settlement-signature".into(),
            };
            value.revenue_event_id = value.derive_revenue_event_id().unwrap();
            value
        };
        let dividend = {
            let mut value = DependencyDividend {
                dividend_id: DividendId::derive(&"placeholder").unwrap(),
                revenue_event_id: revenue.revenue_event_id.clone(),
                downstream_claim_id: claim_id.clone(),
                upstream_claim_id: upstream_claim_id.clone(),
                used_in_final_proof: true,
                final_dependency_root: "blake3:proof-dependencies".into(),
                eligible_economic_edge_root: "blake3:economic-edge".into(),
                economic_policy_root: "blake3:reciprocal-policy".into(),
                settlement_receipt_id: revenue.settlement_receipt_id.clone(),
                compute_savings_evidence_root: "blake3:bounded-impact-evidence".into(),
                downstream_net_revenue: amount(800_000),
                upstream_pool: amount(40_000),
                payout: amount(40_000),
                cap_bps: 500,
                non_recursive: true,
                finalized_at: at + Duration::hours(3),
                signature: "dividend-settlement-signature".into(),
            };
            value.dividend_id = value.derive_dividend_id().unwrap();
            value
        };
        let availability = {
            let mut value = AvailabilityReceipt {
                receipt_id: ReceiptId::derive(&"placeholder").unwrap(),
                artifact_id: artifact_id.clone(),
                storage_node_id: xlemma_core::NodeId::derive(&"storage-node").unwrap(),
                operator_cluster_id: OperatorClusterId::derive(&"storage-operator").unwrap(),
                provider: "provider-a".into(),
                region: "region-a".into(),
                custody_challenge_root: "blake3:custody-challenge".into(),
                available_until: at + Duration::days(30),
                observed_at: at + Duration::hours(2),
                signature: "storage-signature".into(),
            };
            value.receipt_id = value.derive_receipt_id().unwrap();
            value
        };
        let challenge = {
            let mut value = Challenge {
                challenge_id: ChallengeId::derive(&"placeholder").unwrap(),
                certificate_id: certificate.certificate_id.clone(),
                challenger: "did:key:watcher".into(),
                kind: ChallengeKind::PolicyViolation,
                evidence_root: "blake3:challenge-evidence".into(),
                bond: amount(10_000),
                status: ChallengeStatus::Open,
                opened_at: at + Duration::minutes(10),
                resolved_at: None,
                resolution_evidence_root: None,
                supersedes: None,
                signature: "watcher-signature".into(),
            };
            value.challenge_id = value.derive_challenge_id().unwrap();
            value
        };
        let resolved_challenge = {
            let mut value = Challenge {
                challenge_id: ChallengeId::derive(&"placeholder").unwrap(),
                certificate_id: certificate.certificate_id.clone(),
                challenger: "did:key:watcher".into(),
                kind: ChallengeKind::PolicyViolation,
                evidence_root: "blake3:challenge-evidence".into(),
                bond: amount(10_000),
                status: ChallengeStatus::Dismissed,
                opened_at: challenge.opened_at,
                resolved_at: Some(at + Duration::minutes(30)),
                resolution_evidence_root: Some("blake3:challenge-resolution".into()),
                supersedes: Some(challenge.challenge_id.clone()),
                signature: "watcher-resolution-signature".into(),
            };
            value.challenge_id = value.derive_challenge_id().unwrap();
            value
        };
        let revalidation_challenge = {
            let mut value = Challenge {
                challenge_id: ChallengeId::derive(&"placeholder").unwrap(),
                certificate_id: certificate.certificate_id.clone(),
                challenger: "did:key:post-publication-watcher".into(),
                kind: ChallengeKind::CheckerCompromise,
                evidence_root: "blake3:revalidation-evidence".into(),
                bond: amount(10_000),
                status: ChallengeStatus::Open,
                opened_at: at + Duration::hours(4),
                resolved_at: None,
                resolution_evidence_root: None,
                supersedes: None,
                signature: "revalidation-watcher-signature".into(),
            };
            value.challenge_id = value.derive_challenge_id().unwrap();
            value
        };
        let quarantine = {
            let mut value = QuarantineRecord {
                quarantine_id: QuarantineId::derive(&"placeholder").unwrap(),
                certificate_id: certificate.certificate_id.clone(),
                challenge_id: Some(revalidation_challenge.challenge_id.clone()),
                affected_claim_id: claim_id.clone(),
                reason: "expanded reproduction required".into(),
                evidence_roots: vec!["blake3:revalidation-evidence".into()],
                quarantined_at: at + Duration::hours(4) + Duration::minutes(1),
                supersedes: None,
                authority_signature: "quarantine-authority-signature".into(),
            };
            value.quarantine_id = value.derive_quarantine_id().unwrap();
            value
        };

        let messages = vec![
            XlmpMessage::Researcher(ResearcherMessage {
                researcher: researcher.clone(),
            }),
            XlmpMessage::Theory(TheoryMessage { theory_id, theory }),
            XlmpMessage::Claim(ClaimMessage {
                claim_id,
                claim,
                contribution_manifest_hash: contribution_root.clone(),
                rights_manifest_hash: rights_root.clone(),
            }),
            XlmpMessage::Claim(ClaimMessage {
                claim_id: upstream_claim_id,
                claim: upstream_claim,
                contribution_manifest_hash: "blake3:upstream-contribution".into(),
                rights_manifest_hash: "blake3:upstream-rights".into(),
            }),
            XlmpMessage::Contribution(ContributionMessage {
                manifest_hash: contribution_root,
                manifest: contribution,
            }),
            XlmpMessage::Rights(RightsMessage {
                manifest_hash: rights_root,
                manifest: rights,
            }),
            XlmpMessage::ProofCandidate(ProofCandidateMessage {
                job_id,
                proof_id,
                artifact_id,
                proof,
            }),
            XlmpMessage::ComputeReceipt(ComputeReceiptMessage {
                receipt: compute_receipt.clone(),
            }),
            XlmpMessage::ResearchCredit(ResearchCreditMessage {
                credit: credit.clone(),
            }),
            XlmpMessage::ResearchVault(ResearchVaultMessage {
                vault: vault.clone(),
            }),
            XlmpMessage::Certificate(CertificateMessage {
                certificate: certificate.clone(),
            }),
            XlmpMessage::Challenge(ChallengeMessage {
                challenge: challenge.clone(),
            }),
            XlmpMessage::Challenge(ChallengeMessage {
                challenge: resolved_challenge,
            }),
            XlmpMessage::Finalize(FinalizeMessage {
                certificate_id: certificate.certificate_id,
                claim_id: certificate.claim_id,
                finalization_root: "blake3:finalization-root".into(),
                finalized_at: at + Duration::hours(2),
                signature: "finalizer-signature".into(),
            }),
            XlmpMessage::License(LicenseMessage {
                license: license.clone(),
            }),
            XlmpMessage::Capsule(CapsuleMessage { capsule }),
            XlmpMessage::Publish(PublishMessage {
                publication: publication.clone(),
            }),
            XlmpMessage::Availability(AvailabilityMessage {
                receipt: availability,
            }),
            XlmpMessage::Revenue(RevenueMessage {
                event: revenue.clone(),
            }),
            XlmpMessage::DependencyDividend(DependencyDividendMessage {
                dividend: dividend.clone(),
            }),
            XlmpMessage::Challenge(ChallengeMessage {
                challenge: revalidation_challenge,
            }),
            XlmpMessage::Quarantine(QuarantineMessage { record: quarantine }),
        ];
        let envelopes = messages.into_iter().map(envelope).collect::<Vec<_>>();
        let capsule_index = envelopes
            .iter()
            .position(|e| matches!(e.message, XlmpMessage::Capsule(_)))
            .unwrap();
        let mut before_capsule = ProtocolProjection::replay(&envelopes[..capsule_index]).unwrap();
        let mut substituted = envelopes[capsule_index].message.clone();
        if let XlmpMessage::Capsule(message) = &mut substituted {
            message.capsule.artifact_id = ArtifactId::derive(&"unrelated-artifact").unwrap();
            message.capsule.lemma_id = message.capsule.derive_lemma_id().unwrap();
        }
        let original_root = before_capsule.state_root().unwrap();
        assert!(matches!(
            before_capsule.apply(&envelope(substituted)),
            Err(ProjectionError::ReferenceMismatch("capsule"))
        ));
        assert_eq!(before_capsule.state_root().unwrap(), original_root);

        let publication_index = envelopes
            .iter()
            .position(|e| matches!(e.message, XlmpMessage::Publish(_)))
            .unwrap();
        let mut before_publication =
            ProtocolProjection::replay(&envelopes[..publication_index]).unwrap();
        let mut early = publication.clone();
        early.published_at = at;
        early.publication_id = early.derive_publication_id().unwrap();
        assert!(matches!(
            before_publication.apply(&envelope(XlmpMessage::Publish(PublishMessage {
                publication: early
            }))),
            Err(ProjectionError::ReferenceMismatch("publication"))
        ));
        let mut projection = ProtocolProjection::replay(&envelopes).unwrap();

        assert_eq!(
            projection.publications.get(&publication.publication_id),
            Some(&publication)
        );
        assert_eq!(projection.licenses.get(&license.license_id), Some(&license));
        assert_eq!(
            projection.researchers.get(&researcher.researcher_id),
            Some(&researcher)
        );
        assert_eq!(
            projection.compute_receipts.get(&compute_receipt.receipt_id),
            Some(&compute_receipt)
        );
        assert_eq!(
            projection.research_credits.get(&credit.credit_id),
            Some(&credit)
        );
        assert_eq!(
            projection
                .vault_snapshots
                .get(&vault.vault_id)
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            projection.revenue_events.get(&revenue.revenue_event_id),
            Some(&revenue)
        );
        assert_eq!(
            projection.dividends.get(&dividend.dividend_id),
            Some(&dividend)
        );
        assert_eq!(projection.challenges.len(), 3);
        assert_eq!(projection.quarantines.len(), 1);
        assert!(projection.state_root().unwrap().starts_with("blake3:"));

        let root_before_rejected_publish = projection.state_root().unwrap();
        let mut superseding_publication = publication.clone();
        superseding_publication.supersedes = Some(publication.publication_id.clone());
        superseding_publication.published_at = at + Duration::hours(5);
        superseding_publication.publication_id =
            superseding_publication.derive_publication_id().unwrap();
        let attempt = envelope(XlmpMessage::Publish(PublishMessage {
            publication: superseding_publication,
        }));
        assert!(matches!(
            projection.apply(&attempt),
            Err(ProjectionError::ReferenceMismatch("publication"))
        ));
        assert_eq!(
            projection.state_root().unwrap(),
            root_before_rejected_publish
        );
    }

    #[test]
    fn publication_cannot_arrive_before_certificate_finalization() {
        let publication = PublicationRecord {
            publication_id: PublicationId::derive(&"publication").unwrap(),
            claim_id: ClaimId::from_canonical_elaborated_type(
                &TheoryId::derive(&"theory").unwrap(),
                "claim",
            )
            .unwrap(),
            proof_id: ProofId::from_canonical_proof_object(
                &ClaimId::from_canonical_elaborated_type(
                    &TheoryId::derive(&"theory").unwrap(),
                    "claim",
                )
                .unwrap(),
                "proof",
            )
            .unwrap(),
            certificate_id: CertificateId::derive(&"certificate").unwrap(),
            artifact_id: ArtifactId::derive(&"artifact").unwrap(),
            rights_manifest_hash: "blake3:rights".into(),
            license_ids: vec![],
            locations: vec!["ipfs:bafy".into()],
            published_at: Utc::now(),
            supersedes: None,
            signature: "signature".into(),
        };
        let mut publication = publication;
        publication.publication_id = publication.derive_publication_id().unwrap();
        let message = envelope(XlmpMessage::Publish(PublishMessage { publication }));
        assert!(matches!(
            ProtocolProjection::default().apply(&message),
            Err(ProjectionError::MissingPrerequisite("certificate"))
        ));
    }
}

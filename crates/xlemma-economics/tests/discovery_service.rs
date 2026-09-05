use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, TimeZone, Utc};
use ed25519_dalek::{Signer, SigningKey};
use std::collections::{BTreeMap, BTreeSet};
use xlemma_core::*;
use xlemma_economics::*;

fn at(seconds: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(seconds, 0).unwrap()
}
fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}
fn signer(n: u8) -> String {
    format!(
        "ed25519:{}",
        URL_SAFE_NO_PAD.encode(key(n).verifying_key().to_bytes())
    )
}
fn cluster(n: u8) -> OperatorClusterId {
    OperatorClusterId::derive(&n).unwrap()
}
fn root(label: &str) -> ArtifactId {
    ArtifactId::derive(&label).unwrap()
}
fn principal(n: u8, role: DiscoveryRole) -> DiscoveryPrincipal {
    DiscoveryPrincipal {
        researcher_id: ResearcherId::derive(&n).unwrap(),
        cluster_id: cluster(n),
        payout_address: format!("0x{n:040x}"),
        credential_root: root(&format!("credential-{n}")),
        roles: BTreeSet::from([role]),
    }
}

struct Fixture {
    ledger: DiscoveryLedger,
    trust: DiscoveryTrust,
    policy: ServiceRoundPolicy,
    id: DiscoveryRoundId,
    submission: ResearchSubmission,
    source: Source,
}
#[derive(Default)]
struct Source(BTreeMap<MessageId, ResolvedDiscoveryEvidence>);
impl DiscoveryEvidenceSource for Source {
    fn resolve(
        &self,
        id: &MessageId,
        _at: DateTime<Utc>,
    ) -> Result<ResolvedDiscoveryEvidence, ServiceError> {
        self.0
            .get(id)
            .cloned()
            .ok_or(ServiceError::Invalid("no authenticated evidence"))
    }
}
fn envelope(
    trust: &DiscoveryTrust,
    n: u8,
    command: DiscoveryCommand,
    time: i64,
) -> DiscoveryEnvelope {
    let mut e = DiscoveryEnvelope {
        command_id: ReceiptId::derive(&"pending").unwrap(),
        trust_root: trust.root().unwrap(),
        nonce: format!("{n}-{time}"),
        issued_at: at(time),
        expires_at: at(time + 60),
        signer: signer(n),
        command,
        signature: String::new(),
    };
    e.command_id = e.expected_id().unwrap();
    e.signature = format!(
        "ed25519:{}",
        URL_SAFE_NO_PAD.encode(key(n).sign(&e.signing_bytes().unwrap()).to_bytes())
    );
    e
}
impl Fixture {
    fn apply(
        &mut self,
        n: u8,
        command: DiscoveryCommand,
        time: i64,
    ) -> Result<ReceiptId, ServiceError> {
        let e = envelope(&self.trust, n, command, time);
        let id = e.command_id.clone();
        self.ledger.apply(&e, at(time), &self.source)?;
        Ok(id)
    }
    fn prepared() -> Self {
        Self::prepared_with(None, None)
    }
    fn prepared_with(
        trust_override: Option<DiscoveryTrust>,
        funding_override: Option<Vec<DiscoveryCommand>>,
    ) -> Self {
        let sim: DiscoverySimulation =
            serde_json::from_str(include_str!("../../../examples/discovery/pilot.json")).unwrap();
        let mut economics = sim.policy;
        economics.opens_at = 1000;
        economics.submissions_close_at = 1100;
        economics.appeals_close_at = 1200;
        economics.review_deadline = 1300;
        economics.maximum_submissions = 16;
        economics.maximum_appeals = 8;
        economics.assessors = [cluster(6), cluster(7)].into();
        economics.appeal_reviewers = [cluster(8), cluster(9)].into();
        economics.budgets.insert(
            RewardCategory::FoundationalResearch,
            economics.budgets[&RewardCategory::Discovery].clone(),
        );
        let trust = trust_override.unwrap_or(DiscoveryTrust {
            network: "local-development".into(),
            chain_id: 31337,
            escrow_address: format!("0x{:040x}", 100),
            usdc_asset: economics.usdc_asset.clone(),
            principals: [
                (1, DiscoveryRole::Administrator),
                (2, DiscoveryRole::Researcher),
                (3, DiscoveryRole::Researcher),
                (4, DiscoveryRole::FundingObserver),
                (5, DiscoveryRole::FundingObserver),
                (6, DiscoveryRole::Assessor),
                (7, DiscoveryRole::Assessor),
                (8, DiscoveryRole::AppealReviewer),
                (9, DiscoveryRole::AppealReviewer),
                (10, DiscoveryRole::SettlementObserver),
                (11, DiscoveryRole::SettlementObserver),
                (12, DiscoveryRole::Researcher),
                (13, DiscoveryRole::Researcher),
                (30, DiscoveryRole::Verifier),
                (31, DiscoveryRole::Verifier),
            ]
            .into_iter()
            .map(|(n, role)| (signer(n), principal(n, role)))
            .collect(),
        });
        economics.usdc_asset = trust.usdc_asset.clone();
        let formal_policy_bindings = economics
            .profiles
            .iter()
            .filter(|p| p.class == VerificationProfileClass::Formal)
            .map(|p| (p.policy_id.clone(), p.policy_id.clone()))
            .collect();
        let policy = ServiceRoundPolicy {
            formal_policy_bindings,
            economics,
            settlement_expires_at: 100_000,
            simultaneous_window_seconds: 60,
            reproduction_fee_units: 200_000,
            maximum_verifier_clusters: 4,
            domain: "foundational-mathematics".into(),
            minimum_foundational_bps: 1000,
            calibration: BTreeMap::from([(
                "reference".into(),
                CalibrationTier {
                    reference_cost_units: 100,
                    attempts: 20,
                    accepted_results: 10,
                    uncertainty_bps: 500,
                    outcomes_root: root("independent-outcomes"),
                },
            )]),
            calibration_root: root("calibration"),
            prior_art_cutoff: at(800),
            assisted_submission_slots: 4,
            per_researcher_submission_cap: 4,
            max_contributors_per_submission: 4,
        };
        let id = policy.id().unwrap();
        let DiscoveryEvent::Submit { submission: s, .. } = &sim.events[0] else {
            panic!()
        };
        let submission = ResearchSubmission {
            claim_id: s.claims.iter().next().unwrap().clone(),
            artifact_id: root("actual-artifact"),
            profile_id: s.profile_id.clone(),
            category: RewardCategory::FoundationalResearch,
            contributors: vec![
                ContributorShare {
                    researcher_id: principal(2, DiscoveryRole::Researcher).researcher_id,
                    operator_cluster_id: cluster(2),
                    share_bps: 7000,
                },
                ContributorShare {
                    researcher_id: principal(3, DiscoveryRole::Researcher).researcher_id,
                    operator_cluster_id: cluster(3),
                    share_bps: 3000,
                },
            ],
            manifest_root: root("team-agreement"),
            evidence_roots: s.evidence_roots.clone(),
            registered_study: None,
            assisted: true,
            research_context_root: root("unsolicited-no-buyer"),
        };
        let mut f = Self {
            ledger: DiscoveryLedger::new(trust.clone()).unwrap(),
            trust,
            policy: policy.clone(),
            id: id.clone(),
            submission,
            source: Source::default(),
        };
        f.apply(1, DiscoveryCommand::CreateRound { policy }, 900)
            .unwrap();
        f.apply(
            6,
            DiscoveryCommand::ApproveCalibration {
                round_id: id.clone(),
            },
            901,
        )
        .unwrap();
        f.apply(
            7,
            DiscoveryCommand::ApproveCalibration {
                round_id: id.clone(),
            },
            902,
        )
        .unwrap();
        let categories: Vec<_> = f.policy.economics.budgets.keys().copied().collect();
        for (i, category) in categories.into_iter().enumerate() {
            let mut receipt: FundingReceipt = serde_json::from_str(include_str!(
                "../../../examples/no-arbitrage/funding-receipt.json"
            ))
            .unwrap();
            receipt.policy_id = PolicyId::derive(&f.policy).unwrap();
            receipt.destination_vault = f.trust.escrow_address.clone();
            receipt.settled_at = at(905);
            receipt.settled_amount = Amount::new(500_000_000, &f.trust.usdc_asset, 6);
            receipt.related_party = false;
            receipt.settlement_receipt_id =
                ReceiptId::derive(&("actual-chain-funding", i)).unwrap();
            receipt.funding_receipt_id = receipt.derive_funding_receipt_id().unwrap();
            let command = DiscoveryCommand::ObserveFunding {
                round_id: id.clone(),
                receipt,
                category,
                donor_cluster: cluster(20),
                administrator_cluster: cluster(21),
            };
            let mut command = funding_override
                .as_ref()
                .map(|commands| commands[i].clone())
                .unwrap_or(command);
            if let DiscoveryCommand::ObserveFunding { receipt, .. } = &mut command {
                receipt.funding_receipt_id = receipt.derive_funding_receipt_id().unwrap();
            }
            f.apply(4, command.clone(), 910 + i as i64 * 2).unwrap();
            f.apply(5, command, 911 + i as i64 * 2).unwrap();
        }
        f.apply(1, DiscoveryCommand::OpenRound { round_id: id }, 1000)
            .unwrap();
        f
    }
    fn reveal(&mut self) -> ReceiptId {
        let salt = "a-private-commit-salt-with-at-least-32-characters";
        self.apply(
            2,
            DiscoveryCommand::Commit {
                round_id: self.id.clone(),
                commitment: self.submission.commitment(&self.id, salt).unwrap(),
            },
            1001,
        )
        .unwrap();
        self.apply(
            2,
            DiscoveryCommand::Reveal {
                round_id: self.id.clone(),
                submission: self.submission.clone(),
                salt: salt.into(),
            },
            1002,
        )
        .unwrap();
        self.submission.id(&self.id).unwrap()
    }
    fn assess(&mut self, status: DeclaredEvidenceStatus) -> ReceiptId {
        self.assess_independent(status, None)
    }
    fn assess_independent(
        &mut self,
        status: DeclaredEvidenceStatus,
        independence: Option<ArtifactId>,
    ) -> ReceiptId {
        let id = self.reveal();
        self.apply(
            3,
            DiscoveryCommand::Consent {
                round_id: self.id.clone(),
                submission_id: id.clone(),
            },
            1003,
        )
        .unwrap();
        let message = MessageId::derive(&"authenticated-certificate").unwrap();
        self.source.0.insert(
            message.clone(),
            ResolvedDiscoveryEvidence {
                job_id: JobId::derive(&"authenticated-checking-job").unwrap(),
                claim_id: self.submission.claim_id.clone(),
                artifact_id: self.submission.artifact_id.clone(),
                profile_id: self.submission.profile_id.clone(),
                class: VerificationProfileClass::Formal,
                evidence_roots: self.submission.evidence_roots.clone(),
                status,
                final_after: at(1190),
                verifier_clusters: [cluster(30), cluster(31)].into(),
                certificate_digest: format!("0x{}", "a".repeat(64)),
                observation_digest: format!("0x{}", "b".repeat(64)),
            },
        );
        self.apply(
            2,
            DiscoveryCommand::AttachEvidence {
                round_id: self.id.clone(),
                submission_id: id.clone(),
                certificate_message_id: message,
            },
            1004,
        )
        .unwrap();
        let assessment = RewardAssessment {
            submission_id: id.clone(),
            group_id: ContributionGroupId::derive(&"new-contribution").unwrap(),
            eligible: true,
            calibration_tier: "reference".into(),
            prior_art_root: root("searched-corpus"),
            additional_contribution_root: root("new-evidence"),
            independent_discovery_root: independence,
            reasons_root: root("assessment-reasons"),
        };
        self.apply(
            6,
            DiscoveryCommand::Assess {
                round_id: self.id.clone(),
                assessment: assessment.clone(),
            },
            1005,
        )
        .unwrap();
        self.apply(
            7,
            DiscoveryCommand::Assess {
                round_id: self.id.clone(),
                assessment,
            },
            1006,
        )
        .unwrap();
        id
    }
}

#[test]
fn authenticated_round_consent_assessment_appeal_and_settlement() {
    let mut f = Fixture::prepared();
    let submission_id = f.assess(DeclaredEvidenceStatus::Supported);
    let appeal = f
        .apply(
            2,
            DiscoveryCommand::Appeal {
                round_id: f.id.clone(),
                submission_id,
                grounds: AppealGround::Allocation,
                evidence_root: root("appeal"),
            },
            1110,
        )
        .unwrap();
    assert!(f
        .apply(
            1,
            DiscoveryCommand::Finalize {
                round_id: f.id.clone()
            },
            1210
        )
        .is_err());
    for (n, time) in [(8, 1120), (9, 1121)] {
        f.apply(
            n,
            DiscoveryCommand::ResolveAppeal {
                round_id: f.id.clone(),
                appeal_id: appeal.clone(),
                reasons_root: root("upheld"),
                remedy: DiscoveryRemedy::Uphold,
            },
            time,
        )
        .unwrap();
    }
    f.apply(
        1,
        DiscoveryCommand::Finalize {
            round_id: f.id.clone(),
        },
        1210,
    )
    .unwrap();
    let plan = f.ledger.plan(&f.id).unwrap();
    assert_eq!(plan.total_units, 303_200_000);
    assert_eq!(plan.items.iter().filter(|i| i.completed_review).count(), 6);
    let command = DiscoveryCommand::ObserveSettlement {
        round_id: f.id.clone(),
        plan_id: plan.plan_id.clone(),
        transaction_hash: format!("0x{}", "b".repeat(64)),
        block_hash: format!("0x{}", "c".repeat(64)),
    };
    f.apply(10, command.clone(), 1220).unwrap();
    f.apply(11, command, 1221).unwrap();
    assert_vector(
        "service-round-history.json",
        serde_json::to_value(f.ledger.history(&f.id).unwrap()).unwrap(),
    );
    assert_vector(
        "service-settlement-plan.json",
        serde_json::to_value(f.ledger.plan(&f.id).unwrap()).unwrap(),
    );
}

#[test]
fn failed_research_still_pays_completed_review_work() {
    let mut f = Fixture::prepared();
    f.assess(DeclaredEvidenceStatus::Rejected);
    f.apply(
        1,
        DiscoveryCommand::Finalize {
            round_id: f.id.clone(),
        },
        1210,
    )
    .unwrap();
    let plan = f.ledger.plan(&f.id).unwrap();
    assert_eq!(plan.total_units, 1_200_000);
    assert!(plan.items.iter().all(|i| i.completed_review));
}

#[test]
fn command_forgery_replay_and_cross_network_replay_are_rejected_atomically() {
    let mut f = Fixture::prepared();
    let before = f.ledger.overview().unwrap();
    let mut e = envelope(
        &f.trust,
        1,
        DiscoveryCommand::Expire {
            round_id: f.id.clone(),
        },
        1301,
    );
    e.signature = "ed25519:forged".into();
    assert!(f.ledger.apply(&e, at(1301), &f.source).is_err());
    assert_eq!(before, f.ledger.overview().unwrap());
    let e = envelope(
        &f.trust,
        1,
        DiscoveryCommand::Expire {
            round_id: f.id.clone(),
        },
        1301,
    );
    f.ledger.apply(&e, at(1301), &f.source).unwrap();
    assert!(f.ledger.apply(&e, at(1301), &f.source).is_err());
    let mut other = f.trust.clone();
    other.network = "other-network".into();
    assert!(e.authenticate(&other, at(1301)).is_err());
}

#[test]
fn missing_consent_unbound_evidence_and_false_roles_cannot_advance() {
    let mut f = Fixture::prepared();
    let id = f.reveal();
    assert!(f
        .apply(
            2,
            DiscoveryCommand::AttachEvidence {
                round_id: f.id.clone(),
                submission_id: id.clone(),
                certificate_message_id: MessageId::derive(&"unknown").unwrap()
            },
            1004
        )
        .is_err());
    let assessment = RewardAssessment {
        submission_id: id,
        group_id: ContributionGroupId::derive(&"fake").unwrap(),
        eligible: true,
        calibration_tier: "reference".into(),
        prior_art_root: root("prior-art"),
        additional_contribution_root: root("basis"),
        independent_discovery_root: None,
        reasons_root: root("reason"),
    };
    assert!(f
        .apply(
            2,
            DiscoveryCommand::Assess {
                round_id: f.id.clone(),
                assessment: assessment.clone()
            },
            1005
        )
        .is_err());
    assert!(f
        .apply(
            6,
            DiscoveryCommand::Assess {
                round_id: f.id.clone(),
                assessment
            },
            1005
        )
        .is_err());
}

#[test]
fn quarantine_after_assessment_blocks_payout_plan() {
    let mut f = Fixture::prepared();
    f.assess(DeclaredEvidenceStatus::Supported);
    f.source.0.values_mut().next().unwrap().status = DeclaredEvidenceStatus::Divergent;
    assert!(f
        .apply(
            1,
            DiscoveryCommand::Finalize {
                round_id: f.id.clone()
            },
            1210
        )
        .is_err());
    assert!(f.ledger.plan(&f.id).is_none());
}

#[test]
fn unassessed_research_has_an_independent_process_appeal() {
    let mut f = Fixture::prepared();
    let id = f.reveal();
    let appeal = f
        .apply(
            2,
            DiscoveryCommand::Appeal {
                round_id: f.id.clone(),
                submission_id: id,
                grounds: AppealGround::Process,
                evidence_root: root("missing-review"),
            },
            1110,
        )
        .unwrap();
    assert!(f
        .apply(
            1,
            DiscoveryCommand::Finalize {
                round_id: f.id.clone()
            },
            1210
        )
        .is_err());
    for (n, time) in [(8, 1120), (9, 1121)] {
        f.apply(
            n,
            DiscoveryCommand::ResolveAppeal {
                round_id: f.id.clone(),
                appeal_id: appeal.clone(),
                reasons_root: root("process-remedy"),
                remedy: DiscoveryRemedy::RequireRevalidation,
            },
            time,
        )
        .unwrap();
    }
    f.apply(
        1,
        DiscoveryCommand::Finalize {
            round_id: f.id.clone(),
        },
        1210,
    )
    .unwrap();
    assert_eq!(f.ledger.plan(&f.id).unwrap().total_units, 2_000_000);
}

#[test]
fn expiry_pays_completed_work_without_awarding_unresolved_research() {
    let mut f = Fixture::prepared();
    let id = f.assess(DeclaredEvidenceStatus::Supported);
    f.apply(
        2,
        DiscoveryCommand::Appeal {
            round_id: f.id.clone(),
            submission_id: id,
            grounds: AppealGround::Grouping,
            evidence_root: root("pending"),
        },
        1110,
    )
    .unwrap();
    f.apply(
        1,
        DiscoveryCommand::Expire {
            round_id: f.id.clone(),
        },
        1301,
    )
    .unwrap();
    let plan = f.ledger.plan(&f.id).unwrap();
    assert_eq!(plan.total_units, 1_200_000);
    assert!(plan.items.iter().all(|item| item.completed_review));
}

#[test]
fn wrong_profile_class_and_replaced_evidence_roots_hold_rewards() {
    for wrong_class in [true, false] {
        let mut f = Fixture::prepared();
        f.assess(DeclaredEvidenceStatus::Supported);
        let evidence = f.source.0.values_mut().next().unwrap();
        if wrong_class {
            evidence.class = VerificationProfileClass::Empirical;
        } else {
            evidence.evidence_roots.clear();
        }
        assert!(f
            .apply(
                1,
                DiscoveryCommand::Finalize {
                    round_id: f.id.clone()
                },
                1210
            )
            .is_err());
        assert!(f.ledger.plan(&f.id).is_none());
    }
}

#[test]
fn public_configuration_vectors_match_the_signed_fixture() {
    let f = Fixture::prepared();
    let values = [
        (
            "service-trust.json",
            serde_json::to_value(&f.trust).unwrap(),
        ),
        (
            "service-policy.json",
            serde_json::to_value(&f.policy).unwrap(),
        ),
        (
            "service-submission.json",
            serde_json::to_value(&f.submission).unwrap(),
        ),
        (
            "service-create-command.json",
            serde_json::to_value(DiscoveryCommand::CreateRound {
                policy: f.policy.clone(),
            })
            .unwrap(),
        ),
        (
            "service-create-envelope.json",
            serde_json::to_value(envelope(
                &f.trust,
                1,
                DiscoveryCommand::CreateRound {
                    policy: f.policy.clone(),
                },
                900,
            ))
            .unwrap(),
        ),
    ];
    for (name, value) in values {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/discovery")
            .join(name);
        if std::env::var_os("XLEMMA_UPDATE_DISCOVERY_FIXTURES").is_some() {
            std::fs::write(
                &path,
                format!("{}\n", serde_json::to_string_pretty(&value).unwrap()),
            )
            .unwrap();
        }
        let actual: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(actual, value, "{name}");
    }
}

#[test]
fn independent_simultaneous_discoveries_share_one_group_budget() {
    let mut f = Fixture::prepared();
    let mut other = f.submission.clone();
    other.contributors = vec![
        ContributorShare {
            researcher_id: principal(12, DiscoveryRole::Researcher).researcher_id,
            operator_cluster_id: cluster(12),
            share_bps: 5000,
        },
        ContributorShare {
            researcher_id: principal(13, DiscoveryRole::Researcher).researcher_id,
            operator_cluster_id: cluster(13),
            share_bps: 5000,
        },
    ];
    other.manifest_root = root("independent-team-manifest");
    let salt = "independent-team-commitment-salt-with-entropy";
    f.apply(
        12,
        DiscoveryCommand::Commit {
            round_id: f.id.clone(),
            commitment: other.commitment(&f.id, salt).unwrap(),
        },
        1001,
    )
    .unwrap();
    f.assess_independent(
        DeclaredEvidenceStatus::Supported,
        Some(root("independent-original")),
    );
    let id = other.id(&f.id).unwrap();
    f.apply(
        12,
        DiscoveryCommand::Reveal {
            round_id: f.id.clone(),
            submission: other.clone(),
            salt: salt.into(),
        },
        1007,
    )
    .unwrap();
    f.apply(
        13,
        DiscoveryCommand::Consent {
            round_id: f.id.clone(),
            submission_id: id.clone(),
        },
        1008,
    )
    .unwrap();
    let evidence_id = MessageId::derive(&"independent-certificate").unwrap();
    let mut evidence = f.source.0.values().next().unwrap().clone();
    evidence.evidence_roots = other.evidence_roots.clone();
    f.source.0.insert(evidence_id.clone(), evidence);
    f.apply(
        12,
        DiscoveryCommand::AttachEvidence {
            round_id: f.id.clone(),
            submission_id: id.clone(),
            certificate_message_id: evidence_id,
        },
        1009,
    )
    .unwrap();
    let mut assessment = f
        .ledger
        .history(&f.id)
        .unwrap()
        .iter()
        .find_map(|e| match &e.command {
            DiscoveryCommand::Assess { assessment, .. } => Some(assessment.clone()),
            _ => None,
        })
        .unwrap();
    assessment.submission_id = id;
    assessment.independent_discovery_root = Some(root("independently-established-work"));
    for (n, t) in [(6, 1010), (7, 1011)] {
        f.apply(
            n,
            DiscoveryCommand::Assess {
                round_id: f.id.clone(),
                assessment: assessment.clone(),
            },
            t,
        )
        .unwrap();
    }
    f.apply(
        1,
        DiscoveryCommand::Finalize {
            round_id: f.id.clone(),
        },
        1210,
    )
    .unwrap();
    let plan = f.ledger.plan(&f.id).unwrap();
    assert_eq!(plan.total_units, 302_200_000);
    assert_eq!(
        plan.items
            .iter()
            .filter(|i| !i.completed_review)
            .map(|i| i.amount_units)
            .sum::<u64>(),
        300_000_000
    );
    assert_eq!(
        plan.items
            .iter()
            .find(|i| i.recipient == principal(12, DiscoveryRole::Researcher).payout_address)
            .unwrap()
            .amount_units,
        75_000_000
    );
}

#[test]
fn appeal_dissent_is_preserved_and_paid_but_cannot_unlock_rewards() {
    let mut f = Fixture::prepared();
    let id = f.assess(DeclaredEvidenceStatus::Supported);
    let appeal = f
        .apply(
            2,
            DiscoveryCommand::Appeal {
                round_id: f.id.clone(),
                submission_id: id,
                grounds: AppealGround::Grouping,
                evidence_root: root("disputed"),
            },
            1110,
        )
        .unwrap();
    f.apply(
        8,
        DiscoveryCommand::ResolveAppeal {
            round_id: f.id.clone(),
            appeal_id: appeal.clone(),
            reasons_root: root("first-review"),
            remedy: DiscoveryRemedy::Uphold,
        },
        1120,
    )
    .unwrap();
    f.apply(
        9,
        DiscoveryCommand::ResolveAppeal {
            round_id: f.id.clone(),
            appeal_id: appeal,
            reasons_root: root("dissent"),
            remedy: DiscoveryRemedy::RequireRevalidation,
        },
        1121,
    )
    .unwrap();
    assert!(f
        .apply(
            1,
            DiscoveryCommand::Finalize {
                round_id: f.id.clone()
            },
            1210
        )
        .is_err());
    f.apply(
        1,
        DiscoveryCommand::Expire {
            round_id: f.id.clone(),
        },
        1301,
    )
    .unwrap();
    let plan = f.ledger.plan(&f.id).unwrap();
    assert_eq!(plan.total_units, 3_200_000);
    assert!(plan.items.iter().all(|i| i.completed_review));
    assert!(f.ledger.history(&f.id).unwrap().iter().any(|e|matches!(&e.command,DiscoveryCommand::ResolveAppeal{reasons_root,..} if *reasons_root==root("dissent"))));
}

#[test]
#[ignore = "executed by scripts/test_discovery_evm.py with isolated Anvil funding receipts"]
fn funded_evm_round_fixture() {
    let dir = std::path::PathBuf::from(std::env::var("XLEMMA_EVM_TEST_DIR").unwrap());
    let trust = serde_json::from_slice(&std::fs::read(dir.join("trust.json")).unwrap()).unwrap();
    let funding =
        serde_json::from_slice(&std::fs::read(dir.join("funding.json")).unwrap()).unwrap();
    let mut f = Fixture::prepared_with(Some(trust), Some(funding));
    let id = f.assess(DeclaredEvidenceStatus::Supported);
    f.apply(
        1,
        DiscoveryCommand::Finalize {
            round_id: f.id.clone(),
        },
        1210,
    )
    .unwrap();
    let plan = serde_json::to_vec_pretty(f.ledger.plan(&f.id).unwrap()).unwrap();
    std::fs::write(dir.join("plan.json"), plan).unwrap();
    std::fs::write(
        dir.join("publication.json"),
        serde_json::to_vec_pretty(
            &f.ledger
                .evidence_publication(&f.id, &id, &f.source, at(1210))
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec_pretty(f.ledger.history(&f.id).unwrap()).unwrap(),
    )
    .unwrap();
}

#[test]
fn every_signed_funding_command_survives_json_round_trip() {
    let f = Fixture::prepared();
    let history = f.ledger.history(&f.id).unwrap();
    let encoded = serde_json::to_vec(history).unwrap();
    let decoded: Vec<DiscoveryEnvelope> = serde_json::from_slice(&encoded).unwrap();
    for e in decoded {
        e.authenticate(&f.trust, e.issued_at).unwrap();
    }
}

#[test]
fn abandoned_awards_remain_reserved_until_independent_unpaid_expiry() {
    let mut f = Fixture::prepared();
    f.assess(DeclaredEvidenceStatus::Supported);
    f.apply(
        1,
        DiscoveryCommand::Finalize {
            round_id: f.id.clone(),
        },
        1210,
    )
    .unwrap();
    let observe = DiscoveryCommand::ObserveExpiry {
        round_id: f.id.clone(),
        transaction_hash: format!("0x{}", "8".repeat(64)),
        block_hash: format!("0x{}", "9".repeat(64)),
    };
    assert!(f.apply(10, observe.clone(), 1301).is_err());
    f.apply(10, observe.clone(), 100_001).unwrap();
    assert_eq!(
        f.ledger.overview().unwrap()["rewarded_groups"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    f.apply(11, observe, 100_002).unwrap();
    assert!(f.ledger.overview().unwrap()["rewarded_groups"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(
        f.ledger.overview().unwrap()["rounds"][0]["reservations_released"]
            .as_bool()
            .unwrap()
    );
    assert!(f.ledger.plan(&f.id).is_some()); // Immutable historical proposal retained.
}

#[test]
fn evidence_appeal_and_revalidation_hold_publication_without_erasing_history() {
    let mut f = Fixture::prepared();
    let id = f.assess(DeclaredEvidenceStatus::Supported);
    assert!(f
        .ledger
        .evidence_publication(&f.id, &id, &f.source, at(1100))
        .is_ok());
    let appeal = f
        .apply(
            2,
            DiscoveryCommand::Appeal {
                round_id: f.id.clone(),
                submission_id: id.clone(),
                grounds: AppealGround::Evidence,
                evidence_root: root("evidence-challenge"),
            },
            1110,
        )
        .unwrap();
    assert!(f
        .ledger
        .evidence_publication(&f.id, &id, &f.source, at(1111))
        .is_err());
    for (n, t) in [(8, 1120), (9, 1121)] {
        f.apply(
            n,
            DiscoveryCommand::ResolveAppeal {
                round_id: f.id.clone(),
                appeal_id: appeal.clone(),
                reasons_root: root("rerun-required"),
                remedy: DiscoveryRemedy::RequireRevalidation,
            },
            t,
        )
        .unwrap();
    }
    assert!(f
        .ledger
        .evidence_publication(&f.id, &id, &f.source, at(1122))
        .is_err());
    f.apply(
        1,
        DiscoveryCommand::Finalize {
            round_id: f.id.clone(),
        },
        1210,
    )
    .unwrap();
    assert!(f
        .ledger
        .plan(&f.id)
        .unwrap()
        .items
        .iter()
        .all(|i| i.completed_review));
}

fn assert_vector(name: &str, value: serde_json::Value) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/discovery")
        .join(name);
    if std::env::var_os("XLEMMA_UPDATE_DISCOVERY_FIXTURES").is_some() {
        std::fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&value).unwrap()),
        )
        .unwrap();
    }
    let actual: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(actual, value, "{name}");
}

#[test]
fn original_checker_cannot_review_its_own_evidence_appeal() {
    let mut trust = Fixture::prepared().trust;
    trust
        .principals
        .get_mut(&signer(8))
        .unwrap()
        .roles
        .insert(DiscoveryRole::Verifier);
    let mut f = Fixture::prepared_with(Some(trust), None);
    let id = f.assess(DeclaredEvidenceStatus::Supported);
    f.source.0.values_mut().next().unwrap().verifier_clusters =
        BTreeSet::from([cluster(8), cluster(31)]);
    let history = f.ledger.history(&f.id).unwrap().to_vec();
    let mut replay = DiscoveryLedger::new(f.trust.clone()).unwrap();
    for e in history {
        replay.apply(&e, e.issued_at, &f.source).unwrap();
    }
    f.ledger = replay;
    let appeal = f
        .apply(
            2,
            DiscoveryCommand::Appeal {
                round_id: f.id.clone(),
                submission_id: id,
                grounds: AppealGround::Evidence,
                evidence_root: root("checker-review-conflict"),
            },
            1110,
        )
        .unwrap();
    let error = f
        .apply(
            8,
            DiscoveryCommand::ResolveAppeal {
                round_id: f.id.clone(),
                appeal_id: appeal,
                reasons_root: root("self-review"),
                remedy: DiscoveryRemedy::Uphold,
            },
            1120,
        )
        .unwrap_err();
    assert!(error.to_string().contains("original verifier"));
}

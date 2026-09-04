use crate::{
    Amount, ArtifactId, CheckerFamily, ClaimId, ComputeQuoteId, JobId, NodeCredentialId, NodeId,
    ObservationVerdict, OperatorClusterId, OperatorCredentialId, OperatorId, PolicyId, ProofId,
    ReceiptId, ResearcherId, TheoryId, UserCredentialId, VerifiedUserId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstraComputeReceipt {
    pub receipt_id: ReceiptId,
    pub job_id: JobId,
    pub provider: String,
    pub model_id: String,
    pub model_snapshot: Option<String>,
    pub reasoning_effort: Option<String>,
    pub request_hash: String,
    pub context_root: String,
    pub input_units: u64,
    pub cached_input_units: u64,
    pub output_units: u64,
    pub tool_calls: u64,
    pub wall_time_ms: u64,
    pub retry_count: u32,
    pub charged_amount: Amount,
    pub candidate_artifact_roots: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckerExecution {
    pub checker_family: CheckerFamily,
    pub checker_name: String,
    pub checker_version: String,
    pub binary_digest: String,
    pub node_id: NodeId,
    pub operator_cluster_id: OperatorClusterId,
    pub infrastructure_provider: Option<String>,
    pub region: Option<String>,
    pub verdict: ObservationVerdict,
    pub execution_trace_root: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeanVerificationReceipt {
    pub receipt_id: ReceiptId,
    pub job_id: JobId,
    pub claim_id: ClaimId,
    pub proof_id: ProofId,
    pub theory_id: TheoryId,
    pub artifact_id: ArtifactId,
    pub exact_challenge_matched: bool,
    pub lean_toolchain: String,
    pub dependency_root: String,
    pub axiom_policy_id: PolicyId,
    pub observed_axioms: Vec<String>,
    pub sandbox_image_digest: String,
    pub checker_executions: Vec<CheckerExecution>,
    pub verdict: ObservationVerdict,
    pub verified_at: DateTime<Utc>,
    pub aggregate_signature: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoveltyReviewReceipt {
    pub receipt_id: ReceiptId,
    pub claim_id: ClaimId,
    pub reviewer_node_id: NodeId,
    pub operator_cluster_id: OperatorClusterId,
    pub corpus_root: String,
    pub corpus_cutoff: DateTime<Utc>,
    pub known_equivalent_probability: f64,
    pub material_novelty_probability: f64,
    pub useful_simplification_probability: f64,
    pub prior_art_coverage: f64,
    pub confidence: f64,
    pub evidence_root: String,
    pub conflicts_disclosed: Vec<String>,
    pub reviewed_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentVerdict {
    Aligned,
    PartiallyAligned,
    Misaligned,
    Inconclusive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatementAlignmentReviewer {
    pub reviewer_id: ResearcherId,
    pub operator_cluster_id: OperatorClusterId,
    pub credential_reference: String,
    pub conflict_disclosures: Vec<String>,
    pub signature: String,
}

/// Human/domain evidence that a formal statement matches a particular
/// informal claim and presentation. This receipt is deliberately independent
/// of a Lean verification receipt: neither status implies the other.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatementAlignmentReceipt {
    pub receipt_id: ReceiptId,
    pub claim_id: ClaimId,
    pub informal_claim_hash: String,
    pub latex_presentation_hash: String,
    pub assumptions_disclosed: Vec<String>,
    pub definitions_reviewed: Vec<String>,
    pub domain_reviewers: Vec<StatementAlignmentReviewer>,
    pub alignment_verdict: AlignmentVerdict,
    pub limitations: Vec<String>,
    pub evidence_root: String,
    pub reviewed_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum StatementAlignmentIntegrityError {
    #[error("statement-alignment receipt contains an empty required field")]
    EmptyField,
    #[error("statement-alignment receipt requires disclosed assumptions and reviewed definitions")]
    MissingReviewScope,
    #[error("statement-alignment receipt contains a duplicate domain reviewer")]
    DuplicateReviewer,
    #[error("statement-alignment reviewers share a beneficial-control cluster")]
    DuplicateIndependenceDomain,
    #[error("statement-alignment ReceiptID is not content-derived")]
    ReceiptIdMismatch,
    #[error("statement-alignment reviewer signature could not be verified")]
    InvalidReviewerSignature,
    #[error(transparent)]
    Canonical(#[from] crate::CanonicalizationError),
    #[error(transparent)]
    Id(#[from] crate::IdError),
}

/// Deployment boundary for credential/key resolution. Implementations must
/// cryptographically verify each reviewer against the referenced credential;
/// a non-empty signature string is never sufficient.
pub trait StatementAlignmentSignatureVerifier {
    fn verify(&self, reviewer: &StatementAlignmentReviewer, signing_bytes: &[u8]) -> bool;
}

impl<F> StatementAlignmentSignatureVerifier for F
where
    F: Fn(&StatementAlignmentReviewer, &[u8]) -> bool,
{
    fn verify(&self, reviewer: &StatementAlignmentReviewer, signing_bytes: &[u8]) -> bool {
        self(reviewer, signing_bytes)
    }
}

impl StatementAlignmentReceipt {
    fn identity_value(&self) -> Result<serde_json::Value, StatementAlignmentIntegrityError> {
        let mut value = serde_json::to_value(self).map_err(crate::CanonicalizationError::from)?;
        let object = value
            .as_object_mut()
            .ok_or(StatementAlignmentIntegrityError::EmptyField)?;
        object.remove("receipt_id");
        let reviewers = object
            .get_mut("domain_reviewers")
            .and_then(serde_json::Value::as_array_mut)
            .ok_or(StatementAlignmentIntegrityError::EmptyField)?;
        for reviewer in reviewers {
            reviewer
                .as_object_mut()
                .ok_or(StatementAlignmentIntegrityError::EmptyField)?
                .remove("signature");
        }
        Ok(value)
    }

    pub fn expected_receipt_id(&self) -> Result<ReceiptId, StatementAlignmentIntegrityError> {
        Ok(ReceiptId::derive(&self.identity_value()?)?)
    }

    /// Canonical bytes every listed reviewer signs. Reviewer signatures are
    /// removed, while the content-derived ReceiptID remains bound.
    pub fn signing_bytes(&self) -> Result<Vec<u8>, StatementAlignmentIntegrityError> {
        let mut value = serde_json::to_value(self).map_err(crate::CanonicalizationError::from)?;
        let reviewers = value
            .as_object_mut()
            .and_then(|object| object.get_mut("domain_reviewers"))
            .and_then(serde_json::Value::as_array_mut)
            .ok_or(StatementAlignmentIntegrityError::EmptyField)?;
        for reviewer in reviewers {
            reviewer
                .as_object_mut()
                .ok_or(StatementAlignmentIntegrityError::EmptyField)?
                .remove("signature");
        }
        Ok(crate::canonical_json_bytes(&value)?)
    }

    pub fn validate_integrity(&self) -> Result<(), StatementAlignmentIntegrityError> {
        self.receipt_id.validate()?;
        self.claim_id.validate()?;
        if [
            self.informal_claim_hash.as_str(),
            self.latex_presentation_hash.as_str(),
            self.evidence_root.as_str(),
        ]
        .iter()
        .any(|field| field.trim().is_empty())
        {
            return Err(StatementAlignmentIntegrityError::EmptyField);
        }
        if self.assumptions_disclosed.is_empty()
            || self.definitions_reviewed.is_empty()
            || self.domain_reviewers.is_empty()
        {
            return Err(StatementAlignmentIntegrityError::MissingReviewScope);
        }
        let mut reviewers = BTreeSet::new();
        let mut operator_clusters = BTreeSet::new();
        for reviewer in &self.domain_reviewers {
            reviewer.reviewer_id.validate()?;
            reviewer.operator_cluster_id.validate()?;
            if reviewer.credential_reference.trim().is_empty()
                || reviewer.signature.trim().is_empty()
            {
                return Err(StatementAlignmentIntegrityError::EmptyField);
            }
            if !reviewers.insert(reviewer.reviewer_id.clone()) {
                return Err(StatementAlignmentIntegrityError::DuplicateReviewer);
            }
            if !operator_clusters.insert(reviewer.operator_cluster_id.clone()) {
                return Err(StatementAlignmentIntegrityError::DuplicateIndependenceDomain);
            }
        }
        if self.receipt_id != self.expected_receipt_id()? {
            return Err(StatementAlignmentIntegrityError::ReceiptIdMismatch);
        }
        Ok(())
    }

    pub fn validate_with<V: StatementAlignmentSignatureVerifier>(
        &self,
        verifier: &V,
    ) -> Result<(), StatementAlignmentIntegrityError> {
        self.validate_integrity()?;
        let signing_bytes = self.signing_bytes()?;
        if self
            .domain_reviewers
            .iter()
            .any(|reviewer| !verifier.verify(reviewer, &signing_bytes))
        {
            return Err(StatementAlignmentIntegrityError::InvalidReviewerSignature);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentReceipt {
    pub receipt_id: ReceiptId,
    pub job_id: JobId,
    pub payment_identifier: String,
    pub scheme: String,
    pub network: String,
    pub payer: String,
    pub payee: String,
    pub authorized: Amount,
    pub settled: Amount,
    pub settlement_reference: String,
    pub settled_at: DateTime<Utc>,
    pub facilitator_signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationReceipt {
    pub receipt_id: ReceiptId,
    pub job_id: JobId,
    pub node_id: NodeId,
    pub verified_user_id: VerifiedUserId,
    pub operator_id: OperatorId,
    pub operator_cluster_id: OperatorClusterId,
    pub user_credential_id: UserCredentialId,
    pub operator_credential_id: OperatorCredentialId,
    pub node_credential_id: NodeCredentialId,
    pub credential_chain_root: String,
    pub checker_family: Option<CheckerFamily>,
    pub checker_name: String,
    pub checker_version: String,
    pub checker_binary_digest: String,
    pub infrastructure_provider: String,
    pub region: String,
    pub artifact_root: String,
    pub environment_root: String,
    pub dependency_root: String,
    pub axiom_set_root: String,
    pub execution_trace_root: String,
    pub observation_root: String,
    pub verdict: ObservationVerdict,
    pub commitment: String,
    /// Public nonce disclosed only during the reveal phase.
    pub reveal_salt: String,
    pub committed_at: DateTime<Utc>,
    pub revealed_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Debug, Error)]
pub enum ObservationIntegrityError {
    #[error("observation receipt contains an empty required field")]
    EmptyField,
    #[error("observation receipt timing is invalid")]
    InvalidTiming,
    #[error("observation root does not bind the complete execution evidence")]
    ObservationRootMismatch,
    #[error("observation commitment does not match its canonical reveal")]
    CommitmentMismatch,
    #[error("observation ReceiptID is not content-derived")]
    ReceiptIdMismatch,
    #[error(transparent)]
    Canonical(#[from] crate::CanonicalizationError),
    #[error(transparent)]
    Id(#[from] crate::IdError),
}

impl ObservationReceipt {
    /// Derives the execution-evidence root from every identity, checker,
    /// environment, artifact, trace and verdict field. Committers cannot
    /// choose this root independently of the receipt they later reveal.
    pub fn expected_observation_root(&self) -> Result<String, ObservationIntegrityError> {
        let mut value = serde_json::to_value(self).map_err(crate::CanonicalizationError::from)?;
        let object = value
            .as_object_mut()
            .ok_or(ObservationIntegrityError::EmptyField)?;
        for metadata in [
            "receipt_id",
            "observation_root",
            "commitment",
            "reveal_salt",
            "committed_at",
            "revealed_at",
            "signature",
        ] {
            object.remove(metadata);
        }
        let hash = crate::canonical_json_hash("observation-evidence-v1", &value)?;
        Ok(format!("blake3:{}", hex::encode(hash)))
    }

    pub fn expected_receipt_id(&self) -> Result<ReceiptId, ObservationIntegrityError> {
        let mut value = serde_json::to_value(self).map_err(crate::CanonicalizationError::from)?;
        let object = value
            .as_object_mut()
            .ok_or(ObservationIntegrityError::EmptyField)?;
        object.remove("receipt_id");
        object.remove("signature");
        Ok(ReceiptId::derive(&value)?)
    }

    /// Canonical bytes covered by the node signature.
    pub fn signing_bytes(&self) -> Result<Vec<u8>, ObservationIntegrityError> {
        let mut value = serde_json::to_value(self).map_err(crate::CanonicalizationError::from)?;
        value
            .as_object_mut()
            .ok_or(ObservationIntegrityError::EmptyField)?
            .remove("signature");
        Ok(crate::canonical_json_bytes(&value)?)
    }

    pub fn validate_integrity(&self) -> Result<(), ObservationIntegrityError> {
        self.receipt_id.validate()?;
        self.job_id.validate()?;
        self.node_id.validate()?;
        self.verified_user_id.validate()?;
        self.operator_id.validate()?;
        self.operator_cluster_id.validate()?;
        self.user_credential_id.validate()?;
        self.operator_credential_id.validate()?;
        self.node_credential_id.validate()?;
        if [
            self.credential_chain_root.as_str(),
            self.checker_name.as_str(),
            self.checker_version.as_str(),
            self.checker_binary_digest.as_str(),
            self.infrastructure_provider.as_str(),
            self.region.as_str(),
            self.artifact_root.as_str(),
            self.environment_root.as_str(),
            self.dependency_root.as_str(),
            self.axiom_set_root.as_str(),
            self.execution_trace_root.as_str(),
            self.observation_root.as_str(),
            self.commitment.as_str(),
            self.reveal_salt.as_str(),
            self.signature.as_str(),
        ]
        .iter()
        .any(|field| field.trim().is_empty())
        {
            return Err(ObservationIntegrityError::EmptyField);
        }
        if self.committed_at > self.revealed_at {
            return Err(ObservationIntegrityError::InvalidTiming);
        }
        if self.observation_root != self.expected_observation_root()? {
            return Err(ObservationIntegrityError::ObservationRootMismatch);
        }
        if !verify_observation_reveal(self, self.reveal_salt.as_bytes()) {
            return Err(ObservationIntegrityError::CommitmentMismatch);
        }
        if self.receipt_id != self.expected_receipt_id()? {
            return Err(ObservationIntegrityError::ReceiptIdMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvailabilityReceipt {
    pub receipt_id: ReceiptId,
    pub artifact_id: ArtifactId,
    pub storage_node_id: NodeId,
    pub operator_cluster_id: OperatorClusterId,
    pub provider: String,
    pub region: String,
    pub custody_challenge_root: String,
    pub available_until: DateTime<Utc>,
    pub observed_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OriginCertificate {
    pub receipt_id: ReceiptId,
    pub claim_id: ClaimId,
    pub researcher_id: ResearcherId,
    pub commitment_root: String,
    pub committed_at: DateTime<Utc>,
    pub ordering_reference: String,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueAllocationReceipt {
    pub receipt_id: ReceiptId,
    pub revenue_event_id: String,
    pub claim_id: ClaimId,
    pub gross_collected: Amount,
    pub service_cost: Amount,
    pub compute_cost: Amount,
    pub refunds: Amount,
    pub reserves: Amount,
    pub net_distributable: Amount,
    pub allocations: BTreeMap<String, Amount>,
    pub credits_minted: BTreeMap<ResearcherId, Amount>,
    pub cash_payouts: BTreeMap<ResearcherId, Amount>,
    pub finalized_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComputeQuoteReceipt {
    pub quote_id: ComputeQuoteId,
    pub job_id: JobId,
    pub policy_id: PolicyId,
    pub delivery_deadline: DateTime<Utc>,
    pub expected_astra_units: u64,
    pub expected_lean_units: u64,
    pub expected_review_units: u64,
    pub expected_storage_units: u64,
    pub quoted_amount: Amount,
    pub maximum_authorization: Amount,
    pub success_probability_bps: u16,
    pub novelty_clearance_probability_bps: u16,
    pub risk_premium_bps: u16,
    pub valid_until: DateTime<Utc>,
    pub provider_offer_roots: Vec<String>,
    /// Independently calibrated protocol estimates; provider claims alone are
    /// not permitted to set routing probability.
    pub success_estimate_roots: Vec<String>,
    pub signature: String,
}

/// Canonical XLMP/1 commitment for an independently produced observation.
pub fn observation_commitment(
    job_id: &JobId,
    verdict: ObservationVerdict,
    observation_root: &str,
    salt: &[u8],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"xlemma-poir-commit-v1\0");
    update_commitment_field(&mut hasher, job_id.as_str().as_bytes());
    update_commitment_field(&mut hasher, verdict_label(verdict).as_bytes());
    update_commitment_field(&mut hasher, observation_root.as_bytes());
    update_commitment_field(&mut hasher, salt);
    format!("blake3:{}", hasher.finalize().to_hex())
}

pub fn verify_observation_reveal(receipt: &ObservationReceipt, salt: &[u8]) -> bool {
    observation_commitment(
        &receipt.job_id,
        receipt.verdict,
        &receipt.observation_root,
        salt,
    ) == receipt.commitment
}

fn update_commitment_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn verdict_label(verdict: ObservationVerdict) -> &'static str {
    match verdict {
        ObservationVerdict::Pass => "pass",
        ObservationVerdict::Fail => "fail",
        ObservationVerdict::Error => "error",
        ObservationVerdict::Abstain => "abstain",
    }
}

#[cfg(test)]
mod statement_alignment_tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use ed25519_dalek::{Signer, SigningKey};

    fn receipt() -> StatementAlignmentReceipt {
        let theory_id = TheoryId::derive(&"alignment-theory").unwrap();
        let mut receipt = StatementAlignmentReceipt {
            receipt_id: ReceiptId::derive(&"placeholder").unwrap(),
            claim_id: ClaimId::from_canonical_elaborated_type(&theory_id, "forall n : Nat, n = n")
                .unwrap(),
            informal_claim_hash: "blake3:informal-claim".into(),
            latex_presentation_hash: "blake3:latex-presentation".into(),
            assumptions_disclosed: vec!["No assumptions beyond the pinned theory".into()],
            definitions_reviewed: vec!["Nat equality".into()],
            domain_reviewers: vec![StatementAlignmentReviewer {
                reviewer_id: ResearcherId::derive(&"reviewer").unwrap(),
                operator_cluster_id: OperatorClusterId::derive(&"reviewer-cluster").unwrap(),
                credential_reference: "credential:domain-reviewer".into(),
                conflict_disclosures: vec![],
                signature: "reviewer-signature".into(),
            }],
            alignment_verdict: AlignmentVerdict::Aligned,
            limitations: vec!["Formal validity is a separate status".into()],
            evidence_root: "blake3:alignment-evidence".into(),
            reviewed_at: Utc::now(),
        };
        receipt.receipt_id = receipt.expected_receipt_id().unwrap();
        receipt
    }

    #[test]
    fn alignment_receipt_has_content_identity_and_distinct_reviewers() {
        let mut receipt = receipt();
        assert!(receipt.validate_integrity().is_ok());

        receipt
            .domain_reviewers
            .push(receipt.domain_reviewers[0].clone());
        assert!(matches!(
            receipt.validate_integrity(),
            Err(StatementAlignmentIntegrityError::DuplicateReviewer)
        ));
    }

    #[test]
    fn formal_claim_presentation_mutation_changes_alignment_identity() {
        let mut receipt = receipt();
        receipt.informal_claim_hash = "blake3:different-informal-claim".into();
        assert!(matches!(
            receipt.validate_integrity(),
            Err(StatementAlignmentIntegrityError::ReceiptIdMismatch)
        ));
    }

    #[test]
    fn two_reviewer_ids_under_common_control_are_not_independent() {
        let mut receipt = receipt();
        let mut second = receipt.domain_reviewers[0].clone();
        second.reviewer_id = ResearcherId::derive(&"second-reviewer").unwrap();
        receipt.domain_reviewers.push(second);
        receipt.receipt_id = receipt.expected_receipt_id().unwrap();
        assert!(matches!(
            receipt.validate_integrity(),
            Err(StatementAlignmentIntegrityError::DuplicateIndependenceDomain)
        ));
    }

    #[test]
    fn nonempty_reviewer_signature_is_not_treated_as_authenticated() {
        struct RejectAll;
        impl StatementAlignmentSignatureVerifier for RejectAll {
            fn verify(
                &self,
                _reviewer: &StatementAlignmentReviewer,
                _signing_bytes: &[u8],
            ) -> bool {
                false
            }
        }

        assert!(matches!(
            receipt().validate_with(&RejectAll),
            Err(StatementAlignmentIntegrityError::InvalidReviewerSignature)
        ));
    }

    #[test]
    fn published_alignment_vector_has_content_identity_and_signature() {
        let signing_key = SigningKey::from_bytes(&[11_u8; 32]);
        let receipt: StatementAlignmentReceipt = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/statement-alignment-receipt.json"
        ))
        .unwrap();
        let credential_reference = format!(
            "ed25519:{}",
            URL_SAFE_NO_PAD.encode(signing_key.verifying_key().as_bytes())
        );
        assert!(receipt
            .validate_with(
                &|reviewer: &StatementAlignmentReviewer, signing_bytes: &[u8]| {
                    reviewer.credential_reference == credential_reference
                        && reviewer.signature
                            == format!(
                                "ed25519:{}",
                                URL_SAFE_NO_PAD.encode(signing_key.sign(signing_bytes).to_bytes())
                            )
                }
            )
            .is_ok());
    }
}

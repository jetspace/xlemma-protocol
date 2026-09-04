use crate::{
    Amount, ArtifactId, CheckerFamily, ClaimId, ComputeQuoteId, JobId, NodeCredentialId, NodeId,
    ObservationVerdict, OperatorClusterId, OperatorCredentialId, OperatorId, PolicyId, ProofId,
    ReceiptId, ResearcherId, TheoryId, UserCredentialId, VerifiedUserId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    pub expected_astra_units: f64,
    pub expected_lean_units: f64,
    pub expected_review_units: f64,
    pub expected_storage_units: f64,
    pub quoted_amount: Amount,
    pub maximum_authorization: Amount,
    pub success_probability: f64,
    pub novelty_clearance_probability: f64,
    pub risk_premium_bps: u16,
    pub valid_until: DateTime<Utc>,
    pub provider_offer_roots: Vec<String>,
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

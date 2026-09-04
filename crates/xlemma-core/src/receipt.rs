use crate::{
    Amount, ArtifactId, CheckerFamily, ClaimId, ComputeQuoteId, JobId, NodeId,
    ObservationVerdict, OperatorClusterId, PolicyId, ProofId, ReceiptId, ResearcherId, TheoryId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    pub operator_cluster_id: OperatorClusterId,
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

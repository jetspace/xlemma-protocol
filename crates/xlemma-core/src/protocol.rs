//! Native XLMP protocol records.
//!
//! These records describe research state independently of any model provider,
//! formal checker, payment rail, chain, transport, or storage implementation.
//! Historical records are immutable; corrections are represented by new
//! records that reference the object they supersede.

use crate::{
    Amount, ArtifactId, AssuranceLevel, CertificateId, ChallengeId, CheckerFamily, ClaimId,
    ComputeQuoteId, CreditId, DividendId, FormalStatus, JobId, LicenseId, MessageId, NodeId,
    ObservationVerdict, OperatorClusterId, PolicyId, ProofId, PublicationId, QuarantineId,
    ReceiptId, ResearcherId, RevenueEventId, TheoryId, VaultId, VerificationState,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationJob {
    pub job_id: JobId,
    pub researcher_id: ResearcherId,
    pub claim_id: ClaimId,
    pub theory_id: TheoryId,
    pub candidate_proof_id: Option<ProofId>,
    pub artifact_id: ArtifactId,
    pub verification_policy_id: PolicyId,
    pub maximum_budget: Amount,
    pub state: VerificationState,
    pub observation_receipt_ids: Vec<ReceiptId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoIRCertificate {
    pub certificate_id: CertificateId,
    pub job_id: JobId,
    pub theory_id: TheoryId,
    pub claim_id: ClaimId,
    pub proof_id: ProofId,
    pub artifact_id: ArtifactId,
    pub verification_policy_id: PolicyId,
    pub observation_receipt_ids: Vec<ReceiptId>,
    pub checker_families: Vec<CheckerFamily>,
    pub operator_cluster_ids: Vec<OperatorClusterId>,
    pub artifact_root: String,
    pub environment_root: String,
    pub dependency_root: String,
    pub axiom_set_root: String,
    pub formal_status: FormalStatus,
    pub assurance_level: AssuranceLevel,
    pub issued_at: DateTime<Utc>,
    pub challenge_window_ends_at: DateTime<Utc>,
    pub aggregate_signature: String,
}

impl PoIRCertificate {
    /// This is a structural guard, not a substitute for evaluating the policy
    /// and exact checker receipts used to issue the certificate.
    pub fn has_independent_reproduction(&self) -> bool {
        let operators = self.operator_cluster_ids.iter().collect::<BTreeSet<_>>();
        self.formal_status == FormalStatus::Certified
            && self.observation_receipt_ids.len() >= 2
            && operators.len() >= 2
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeKind {
    ArtifactMismatch,
    ReproductionFailure,
    CheckerCompromise,
    PolicyViolation,
    ProvenanceFraud,
    RightsDispute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeStatus {
    Open,
    EvidenceRequested,
    Upheld,
    Dismissed,
    Superseded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Challenge {
    pub challenge_id: ChallengeId,
    pub certificate_id: CertificateId,
    pub challenger: String,
    pub kind: ChallengeKind,
    pub evidence_root: String,
    pub bond: Amount,
    pub status: ChallengeStatus,
    pub opened_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_evidence_root: Option<String>,
    pub supersedes: Option<ChallengeId>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuarantineRecord {
    pub quarantine_id: QuarantineId,
    pub certificate_id: CertificateId,
    pub challenge_id: Option<ChallengeId>,
    pub affected_claim_id: ClaimId,
    pub reason: String,
    pub evidence_roots: Vec<String>,
    pub quarantined_at: DateTime<Utc>,
    pub supersedes: Option<QuarantineId>,
    pub authority_signature: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputeService {
    Formalization,
    ProofSearch,
    ProofRepair,
    FormalVerification,
    IndependentReproduction,
    NoveltyReview,
    Storage,
    Revalidation,
    Explanation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeReceipt {
    pub receipt_id: ReceiptId,
    pub job_id: JobId,
    pub quote_id: Option<ComputeQuoteId>,
    pub service: ComputeService,
    pub provider: String,
    pub implementation_id: String,
    pub implementation_snapshot: Option<String>,
    pub execution_parameters: BTreeMap<String, String>,
    pub request_hash: String,
    pub context_root: String,
    pub metering: BTreeMap<String, u64>,
    pub charged_amount: Amount,
    pub output_artifact_roots: Vec<String>,
    pub completed_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchCredit {
    pub credit_id: CreditId,
    pub researcher_id: ResearcherId,
    pub credit_amount: Amount,
    pub backing_asset_amount: Amount,
    /// Conservative backing value expressed in the credit's smallest unit.
    pub backing_value_in_credit_units: u128,
    pub valuation_policy_id: PolicyId,
    pub backing_reference: String,
    pub issued_at: DateTime<Utc>,
    pub signature: String,
}

impl ResearchCredit {
    pub fn is_fully_backed(&self) -> bool {
        self.backing_value_in_credit_units >= self.credit_amount.units
            && self.credit_amount.units > 0
            && self.backing_asset_amount.units > 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchVault {
    pub vault_id: VaultId,
    pub researcher_id: ResearcherId,
    pub credit_asset: String,
    pub backing_assets: BTreeMap<String, Amount>,
    pub backing_value_in_credit_units: u128,
    pub outstanding_credit_units: u128,
    pub valuation_policy_id: PolicyId,
    pub state_root: String,
    pub observed_at: DateTime<Utc>,
    pub signature: String,
}

impl ResearchVault {
    pub fn is_solvent(&self) -> bool {
        self.backing_value_in_credit_units >= self.outstanding_credit_units
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueEvent {
    pub revenue_event_id: RevenueEventId,
    pub claim_id: ClaimId,
    pub source: String,
    pub settlement_receipt_id: ReceiptId,
    pub gross_collected: Amount,
    pub refunds: Amount,
    pub service_costs: Amount,
    pub reserves: Amount,
    pub realized_at: DateTime<Utc>,
    pub evidence_root: String,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyDividend {
    pub dividend_id: DividendId,
    pub revenue_event_id: RevenueEventId,
    pub downstream_claim_id: ClaimId,
    pub upstream_claim_id: ClaimId,
    pub used_in_final_proof: bool,
    pub final_dependency_root: String,
    pub compute_savings_evidence_root: String,
    pub downstream_net_revenue: Amount,
    pub payout: Amount,
    pub cap_bps: u16,
    pub finalized_at: DateTime<Utc>,
    pub signature: String,
}

impl DependencyDividend {
    pub fn respects_protocol_cap(&self) -> bool {
        if !self.used_in_final_proof || self.cap_bps > 10_000 {
            return false;
        }
        if self
            .downstream_net_revenue
            .ensure_compatible(&self.payout)
            .is_err()
        {
            return false;
        }
        self.downstream_net_revenue
            .mul_bps(self.cap_bps)
            .map(|cap| self.payout.units <= cap.units)
            .unwrap_or(false)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct License {
    pub license_id: LicenseId,
    pub rights_manifest_hash: String,
    pub licensor: String,
    pub licensee: String,
    pub scope: Vec<String>,
    pub consideration_receipt_id: Option<ReceiptId>,
    pub effective_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub supersedes: Option<LicenseId>,
    pub signatures: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationRecord {
    pub publication_id: PublicationId,
    pub claim_id: ClaimId,
    pub proof_id: ProofId,
    pub certificate_id: CertificateId,
    pub artifact_id: ArtifactId,
    pub rights_manifest_hash: String,
    pub license_ids: Vec<LicenseId>,
    pub locations: Vec<String>,
    pub published_at: DateTime<Utc>,
    pub supersedes: Option<PublicationId>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportReceipt {
    pub receipt_id: ReceiptId,
    pub message_id: MessageId,
    pub transport: String,
    pub destination: String,
    pub delivered_at: DateTime<Utc>,
    pub transport_reference: String,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationObservationSummary {
    pub node_id: NodeId,
    pub operator_cluster_id: OperatorClusterId,
    pub checker_family: Option<CheckerFamily>,
    pub verdict: ObservationVerdict,
    pub receipt_id: ReceiptId,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(units: u128) -> Amount {
        Amount::new(units, "USDC", 6)
    }

    #[test]
    fn credit_cannot_claim_full_backing_from_insufficient_value() {
        let credit = ResearchCredit {
            credit_id: CreditId::derive(&"credit").unwrap(),
            researcher_id: ResearcherId::derive(&"researcher").unwrap(),
            credit_amount: Amount::new(101, "R-TEST", 6),
            backing_asset_amount: amount(100),
            backing_value_in_credit_units: 100,
            valuation_policy_id: PolicyId::derive(&"valuation").unwrap(),
            backing_reference: "settlement:1".into(),
            issued_at: Utc::now(),
            signature: "signature".into(),
        };
        assert!(!credit.is_fully_backed());
    }

    #[test]
    fn unused_dependency_never_qualifies_for_a_dividend() {
        let dividend = DependencyDividend {
            dividend_id: DividendId::derive(&"dividend").unwrap(),
            revenue_event_id: RevenueEventId::derive(&"revenue").unwrap(),
            downstream_claim_id: ClaimId::from_canonical_elaborated_type(
                &TheoryId::derive(&"theory").unwrap(),
                "downstream",
            )
            .unwrap(),
            upstream_claim_id: ClaimId::from_canonical_elaborated_type(
                &TheoryId::derive(&"theory").unwrap(),
                "upstream",
            )
            .unwrap(),
            used_in_final_proof: false,
            final_dependency_root: "blake3:final-dependencies".into(),
            compute_savings_evidence_root: "blake3:savings".into(),
            downstream_net_revenue: amount(1_000),
            payout: amount(10),
            cap_bps: 1_000,
            finalized_at: Utc::now(),
            signature: "signature".into(),
        };
        assert!(!dividend.respects_protocol_cap());
    }
}

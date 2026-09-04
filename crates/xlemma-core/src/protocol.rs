//! Native XLMP protocol records.
//!
//! These records describe research state independently of any model provider,
//! formal checker, payment rail, chain, transport, or storage implementation.
//! Historical records are immutable; corrections are represented by new
//! records that reference the object they supersede.

use crate::{
    Amount, ArtifactId, AssuranceLevel, CapsuleEconomicMode, CertificateId, ChallengeId,
    CheckerFamily, ClaimId, ComputeQuoteId, CreditId, DividendId, FormalStatus, JobId, LicenseId,
    MessageId, NodeCredentialId, NodeId, ObservationVerdict, OperatorClusterId,
    OperatorCredentialId, OperatorId, PolicyId, ProofId, PublicationId, QuarantineId, ReceiptId,
    ResearchCertificateId, ResearcherId, RevenueEventId, SovereigntyError, TheoryId,
    UserCredentialId, VaultId, VerificationEvidenceKind, VerificationProfile,
    VerificationProfileClass, VerificationState, VerifiedUserId,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationJob {
    pub job_id: JobId,
    pub researcher_id: ResearcherId,
    pub claim_id: ClaimId,
    pub theory_id: TheoryId,
    pub candidate_proof_id: Option<ProofId>,
    pub artifact_id: ArtifactId,
    pub verification_policy_id: PolicyId,
    #[serde(default)]
    pub verification_profile_class: VerificationProfileClass,
    #[serde(default)]
    pub evidence_roots: BTreeMap<VerificationEvidenceKind, String>,
    #[serde(default)]
    pub producer_operator_cluster_id: Option<OperatorClusterId>,
    pub maximum_budget: Amount,
    pub state: VerificationState,
    pub observation_receipt_ids: Vec<ReceiptId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VerificationJob {
    pub fn validate_against_profile(
        &self,
        profile: &VerificationProfile,
    ) -> Result<(), ResearchVerificationError> {
        profile.validate()?;
        self.job_id.validate()?;
        self.researcher_id.validate()?;
        self.claim_id.validate()?;
        self.theory_id.validate()?;
        self.artifact_id.validate()?;
        self.verification_policy_id.validate()?;
        if let Some(proof_id) = &self.candidate_proof_id {
            proof_id.validate()?;
        }
        if let Some(cluster_id) = &self.producer_operator_cluster_id {
            cluster_id.validate()?;
        }
        if self.verification_policy_id != profile.policy_id
            || self.verification_profile_class != profile.class
        {
            return Err(ResearchVerificationError::ProfileMismatch);
        }
        if self.maximum_budget.units == 0
            || self.maximum_budget.asset.trim().is_empty()
            || self.created_at > self.updated_at
        {
            return Err(ResearchVerificationError::InvalidJob);
        }
        if profile.required_evidence.iter().any(|kind| {
            self.evidence_roots
                .get(kind)
                .is_none_or(|root| root.trim().is_empty())
        }) {
            return Err(ResearchVerificationError::MissingRequiredEvidence);
        }
        Ok(())
    }
}

impl Default for VerificationProfileClass {
    fn default() -> Self {
        Self::Formal
    }
}

/// A verifier-neutral observation over an exact evidence bundle. Formal Lean
/// observations continue to use `ObservationReceipt`; this record gives the
/// other XLMP verification profiles the same independent-reproduction shape.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproductionObservation {
    pub receipt_id: ReceiptId,
    pub job_id: JobId,
    pub claim_id: ClaimId,
    pub verification_profile_class: VerificationProfileClass,
    pub verifier_node_id: NodeId,
    pub verified_user_id: VerifiedUserId,
    pub operator_id: OperatorId,
    pub operator_cluster_id: OperatorClusterId,
    pub user_credential_id: UserCredentialId,
    pub operator_credential_id: OperatorCredentialId,
    pub node_credential_id: NodeCredentialId,
    pub credential_chain_root: String,
    pub verifier_implementation: String,
    pub infrastructure_provider: String,
    pub region: String,
    pub input_artifact_id: ArtifactId,
    pub exact_input_root: String,
    pub evidence_roots: BTreeMap<VerificationEvidenceKind, String>,
    pub verdict: ObservationVerdict,
    pub execution_trace_root: String,
    pub observation_root: String,
    pub commitment: String,
    pub reveal_salt: String,
    pub committed_at: DateTime<Utc>,
    pub reproduced_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Serialize)]
struct ReproductionObservationIdentity<'a> {
    job_id: &'a JobId,
    claim_id: &'a ClaimId,
    verification_profile_class: VerificationProfileClass,
    verifier_node_id: &'a NodeId,
    verified_user_id: &'a VerifiedUserId,
    operator_id: &'a OperatorId,
    operator_cluster_id: &'a OperatorClusterId,
    user_credential_id: &'a UserCredentialId,
    operator_credential_id: &'a OperatorCredentialId,
    node_credential_id: &'a NodeCredentialId,
    credential_chain_root: &'a str,
    verifier_implementation: &'a str,
    infrastructure_provider: &'a str,
    region: &'a str,
    input_artifact_id: &'a ArtifactId,
    exact_input_root: &'a str,
    evidence_roots: &'a BTreeMap<VerificationEvidenceKind, String>,
    verdict: ObservationVerdict,
    execution_trace_root: &'a str,
    observation_root: &'a str,
    commitment: &'a str,
    reveal_salt: &'a str,
    committed_at: DateTime<Utc>,
    reproduced_at: DateTime<Utc>,
}

impl ReproductionObservation {
    pub fn derive_receipt_id(&self) -> Result<ReceiptId, crate::IdError> {
        ReceiptId::derive(&ReproductionObservationIdentity {
            job_id: &self.job_id,
            claim_id: &self.claim_id,
            verification_profile_class: self.verification_profile_class,
            verifier_node_id: &self.verifier_node_id,
            verified_user_id: &self.verified_user_id,
            operator_id: &self.operator_id,
            operator_cluster_id: &self.operator_cluster_id,
            user_credential_id: &self.user_credential_id,
            operator_credential_id: &self.operator_credential_id,
            node_credential_id: &self.node_credential_id,
            credential_chain_root: &self.credential_chain_root,
            verifier_implementation: &self.verifier_implementation,
            infrastructure_provider: &self.infrastructure_provider,
            region: &self.region,
            input_artifact_id: &self.input_artifact_id,
            exact_input_root: &self.exact_input_root,
            evidence_roots: &self.evidence_roots,
            verdict: self.verdict,
            execution_trace_root: &self.execution_trace_root,
            observation_root: &self.observation_root,
            commitment: &self.commitment,
            reveal_salt: &self.reveal_salt,
            committed_at: self.committed_at,
            reproduced_at: self.reproduced_at,
        })
    }

    pub fn expected_observation_root(&self) -> Result<String, crate::CanonicalizationError> {
        let mut value = serde_json::to_value(self).map_err(crate::CanonicalizationError::from)?;
        if let Some(object) = value.as_object_mut() {
            for metadata in [
                "receipt_id",
                "observation_root",
                "commitment",
                "reveal_salt",
                "committed_at",
                "reproduced_at",
                "signature",
            ] {
                object.remove(metadata);
            }
        }
        let hash = crate::canonical_json_hash("research-reproduction-evidence-v1", &value)?;
        Ok(format!("blake3:{}", hex::encode(hash)))
    }

    pub fn signing_bytes(&self) -> Result<Vec<u8>, crate::CanonicalizationError> {
        let mut value = serde_json::to_value(self).map_err(crate::CanonicalizationError::from)?;
        if let Some(object) = value.as_object_mut() {
            object.remove("signature");
        }
        crate::canonical_json_bytes(&value)
    }

    pub fn validate_against(
        &self,
        job: &VerificationJob,
        profile: &VerificationProfile,
    ) -> Result<(), ResearchVerificationError> {
        job.validate_against_profile(profile)?;
        self.receipt_id.validate()?;
        self.verifier_node_id.validate()?;
        self.verified_user_id.validate()?;
        self.operator_id.validate()?;
        self.operator_cluster_id.validate()?;
        self.user_credential_id.validate()?;
        self.operator_credential_id.validate()?;
        self.node_credential_id.validate()?;
        if self.receipt_id != self.derive_receipt_id()? {
            return Err(ResearchVerificationError::IdentityMismatch);
        }
        if self.job_id != job.job_id
            || self.claim_id != job.claim_id
            || self.verification_profile_class != profile.class
            || self.input_artifact_id != job.artifact_id
        {
            return Err(ResearchVerificationError::JobMismatch);
        }
        if job
            .producer_operator_cluster_id
            .as_ref()
            .is_some_and(|producer| producer == &self.operator_cluster_id)
        {
            return Err(ResearchVerificationError::ProducerSelfCertification);
        }
        if !profile
            .verifier_implementations
            .contains(&self.verifier_implementation)
            || self.verifier_implementation.trim().is_empty()
            || self.credential_chain_root.trim().is_empty()
            || self.infrastructure_provider.trim().is_empty()
            || self.region.trim().is_empty()
            || self.exact_input_root.trim().is_empty()
            || self.execution_trace_root.trim().is_empty()
            || self.observation_root.trim().is_empty()
            || self.commitment.trim().is_empty()
            || self.reveal_salt.trim().is_empty()
            || self.signature.trim().is_empty()
            || self.committed_at > self.reproduced_at
        {
            return Err(ResearchVerificationError::InvalidObservation);
        }
        if self.observation_root != self.expected_observation_root()?
            || self.commitment
                != crate::observation_commitment(
                    &self.job_id,
                    self.verdict,
                    &self.observation_root,
                    self.reveal_salt.as_bytes(),
                )
        {
            return Err(ResearchVerificationError::CommitmentMismatch);
        }
        if profile.required_evidence.iter().any(|kind| {
            let expected = job.evidence_roots.get(kind);
            let observed = self.evidence_roots.get(kind);
            expected != observed || observed.is_none_or(|root| root.trim().is_empty())
        }) {
            return Err(ResearchVerificationError::MissingRequiredEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchCertificationStatus {
    Certified,
    Failed,
    Divergent,
    Inconclusive,
}

/// A generalized PoIR certificate. It records the outcome of independent
/// reproduction without pretending non-formal evidence is a Lean proof.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchVerificationCertificate {
    pub certificate_id: ResearchCertificateId,
    pub job_id: JobId,
    pub claim_id: ClaimId,
    pub candidate_proof_id: Option<ProofId>,
    pub verification_policy_id: PolicyId,
    pub verification_profile_class: VerificationProfileClass,
    pub observation_receipt_ids: Vec<ReceiptId>,
    pub operator_cluster_ids: Vec<OperatorClusterId>,
    pub verifier_implementations: Vec<String>,
    pub evidence_bundle_root: String,
    pub status: ResearchCertificationStatus,
    pub issued_at: DateTime<Utc>,
    pub challenge_window_ends_at: DateTime<Utc>,
    pub aggregate_signature: String,
}

#[derive(Serialize)]
struct ResearchCertificateIdentity<'a> {
    job_id: &'a JobId,
    claim_id: &'a ClaimId,
    candidate_proof_id: &'a Option<ProofId>,
    verification_policy_id: &'a PolicyId,
    verification_profile_class: VerificationProfileClass,
    observation_receipt_ids: &'a [ReceiptId],
    operator_cluster_ids: &'a [OperatorClusterId],
    verifier_implementations: &'a [String],
    evidence_bundle_root: &'a str,
    status: ResearchCertificationStatus,
    issued_at: DateTime<Utc>,
    challenge_window_ends_at: DateTime<Utc>,
}

impl ResearchVerificationCertificate {
    pub fn issue(
        job: &VerificationJob,
        profile: &VerificationProfile,
        observations: &[ReproductionObservation],
        evidence_bundle_root: String,
        issued_at: DateTime<Utc>,
        aggregate_signature: String,
    ) -> Result<Self, ResearchVerificationError> {
        let status = evaluate_reproduction(job, profile, observations)?;
        let challenge_seconds = i64::try_from(profile.challenge_window_seconds)
            .map_err(|_| ResearchVerificationError::InvalidChallengeWindow)?;
        let challenge_window_ends_at = issued_at
            .checked_add_signed(Duration::seconds(challenge_seconds))
            .ok_or(ResearchVerificationError::InvalidChallengeWindow)?;
        let mut certificate = Self {
            certificate_id: ResearchCertificateId::derive(&"placeholder")?,
            job_id: job.job_id.clone(),
            claim_id: job.claim_id.clone(),
            candidate_proof_id: job.candidate_proof_id.clone(),
            verification_policy_id: job.verification_policy_id.clone(),
            verification_profile_class: job.verification_profile_class,
            observation_receipt_ids: observations
                .iter()
                .map(|observation| observation.receipt_id.clone())
                .collect(),
            operator_cluster_ids: observations
                .iter()
                .map(|observation| observation.operator_cluster_id.clone())
                .collect(),
            verifier_implementations: observations
                .iter()
                .map(|observation| observation.verifier_implementation.clone())
                .collect(),
            evidence_bundle_root,
            status,
            issued_at,
            challenge_window_ends_at,
            aggregate_signature,
        };
        certificate.certificate_id = certificate.derive_certificate_id()?;
        certificate.validate_against(job, profile, observations)?;
        Ok(certificate)
    }

    pub fn derive_certificate_id(&self) -> Result<ResearchCertificateId, crate::IdError> {
        ResearchCertificateId::derive(&ResearchCertificateIdentity {
            job_id: &self.job_id,
            claim_id: &self.claim_id,
            candidate_proof_id: &self.candidate_proof_id,
            verification_policy_id: &self.verification_policy_id,
            verification_profile_class: self.verification_profile_class,
            observation_receipt_ids: &self.observation_receipt_ids,
            operator_cluster_ids: &self.operator_cluster_ids,
            verifier_implementations: &self.verifier_implementations,
            evidence_bundle_root: &self.evidence_bundle_root,
            status: self.status,
            issued_at: self.issued_at,
            challenge_window_ends_at: self.challenge_window_ends_at,
        })
    }

    pub fn validate_against(
        &self,
        job: &VerificationJob,
        profile: &VerificationProfile,
        observations: &[ReproductionObservation],
    ) -> Result<(), ResearchVerificationError> {
        let expected_status = evaluate_reproduction(job, profile, observations)?;
        self.certificate_id.validate()?;
        if self.certificate_id != self.derive_certificate_id()? {
            return Err(ResearchVerificationError::IdentityMismatch);
        }
        let expected_receipts = observations
            .iter()
            .map(|observation| observation.receipt_id.clone())
            .collect::<Vec<_>>();
        let expected_clusters = observations
            .iter()
            .map(|observation| observation.operator_cluster_id.clone())
            .collect::<Vec<_>>();
        let expected_implementations = observations
            .iter()
            .map(|observation| observation.verifier_implementation.clone())
            .collect::<Vec<_>>();
        if self.job_id != job.job_id
            || self.claim_id != job.claim_id
            || self.candidate_proof_id != job.candidate_proof_id
            || self.verification_policy_id != profile.policy_id
            || self.verification_profile_class != profile.class
            || self.status != expected_status
            || self.observation_receipt_ids != expected_receipts
            || self.operator_cluster_ids != expected_clusters
            || self.verifier_implementations != expected_implementations
            || self.evidence_bundle_root.trim().is_empty()
            || self.aggregate_signature.trim().is_empty()
            || self.challenge_window_ends_at <= self.issued_at
        {
            return Err(ResearchVerificationError::InvalidCertificate);
        }
        let actual_window = self.challenge_window_ends_at - self.issued_at;
        let required_window = i64::try_from(profile.challenge_window_seconds)
            .map(Duration::seconds)
            .map_err(|_| ResearchVerificationError::InvalidChallengeWindow)?;
        if actual_window < required_window {
            return Err(ResearchVerificationError::InvalidChallengeWindow);
        }
        Ok(())
    }
}

pub fn evaluate_reproduction(
    job: &VerificationJob,
    profile: &VerificationProfile,
    observations: &[ReproductionObservation],
) -> Result<ResearchCertificationStatus, ResearchVerificationError> {
    job.validate_against_profile(profile)?;
    let mut receipt_ids = BTreeSet::new();
    let mut node_ids = BTreeSet::new();
    let mut users = BTreeSet::new();
    let mut operators = BTreeSet::new();
    let mut clusters = BTreeSet::new();
    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut unresolved = 0usize;
    for observation in observations {
        observation.validate_against(job, profile)?;
        if !receipt_ids.insert(&observation.receipt_id)
            || !node_ids.insert(&observation.verifier_node_id)
            || !users.insert(&observation.verified_user_id)
            || !operators.insert(&observation.operator_id)
            || !clusters.insert(&observation.operator_cluster_id)
        {
            return Err(ResearchVerificationError::DuplicateIndependenceDomain);
        }
        match observation.verdict {
            ObservationVerdict::Pass => pass += 1,
            ObservationVerdict::Fail => fail += 1,
            ObservationVerdict::Error | ObservationVerdict::Abstain => unresolved += 1,
        }
    }
    if pass > 0 && fail > 0 {
        return Ok(ResearchCertificationStatus::Divergent);
    }
    if fail > 0 {
        return Ok(ResearchCertificationStatus::Failed);
    }
    if unresolved > 0
        || pass < usize::from(profile.minimum_reproductions)
        || clusters.len() < usize::from(profile.minimum_independent_operators)
    {
        return Ok(ResearchCertificationStatus::Inconclusive);
    }
    Ok(ResearchCertificationStatus::Certified)
}

#[derive(Debug, Error)]
pub enum ResearchVerificationError {
    #[error(transparent)]
    Id(#[from] crate::IdError),
    #[error(transparent)]
    Canonical(#[from] crate::CanonicalizationError),
    #[error(transparent)]
    Profile(#[from] SovereigntyError),
    #[error("verification job does not match its profile")]
    ProfileMismatch,
    #[error("verification job has invalid bounds or time ordering")]
    InvalidJob,
    #[error("required verification evidence is missing or changed")]
    MissingRequiredEvidence,
    #[error("reproduction observation content identity does not match")]
    IdentityMismatch,
    #[error("reproduction observation does not match the exact job")]
    JobMismatch,
    #[error("the research producer cannot independently reproduce its own output")]
    ProducerSelfCertification,
    #[error("reproduction observation is incomplete or uses an unauthorized verifier")]
    InvalidObservation,
    #[error("reproduction observation root or commit-reveal binding does not match")]
    CommitmentMismatch,
    #[error("reproduction receipts reuse a receipt, node, or operator-control domain")]
    DuplicateIndependenceDomain,
    #[error("research verification certificate is inconsistent with its observations")]
    InvalidCertificate,
    #[error("research verification challenge window is invalid")]
    InvalidChallengeWindow,
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
    /// Related-party settlement is disclosed and is not arm's-length demand.
    pub related_party: bool,
    pub settlement_receipt_id: ReceiptId,
    pub gross_collected: Amount,
    pub refunds: Amount,
    pub service_costs: Amount,
    pub reserves: Amount,
    pub realized_at: DateTime<Utc>,
    pub evidence_root: String,
    pub signature: String,
}

#[derive(Serialize)]
struct RevenueEventIdentity<'a> {
    claim_id: &'a ClaimId,
    source: &'a str,
    related_party: bool,
    settlement_receipt_id: &'a ReceiptId,
    gross_collected: &'a Amount,
    refunds: &'a Amount,
    service_costs: &'a Amount,
    reserves: &'a Amount,
    realized_at: DateTime<Utc>,
    evidence_root: &'a str,
}

impl RevenueEvent {
    pub fn derive_revenue_event_id(&self) -> Result<RevenueEventId, crate::IdError> {
        RevenueEventId::derive(&RevenueEventIdentity {
            claim_id: &self.claim_id,
            source: &self.source,
            related_party: self.related_party,
            settlement_receipt_id: &self.settlement_receipt_id,
            gross_collected: &self.gross_collected,
            refunds: &self.refunds,
            service_costs: &self.service_costs,
            reserves: &self.reserves,
            realized_at: self.realized_at,
            evidence_root: &self.evidence_root,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), RevenueEventError> {
        self.revenue_event_id.validate()?;
        self.claim_id.validate()?;
        self.settlement_receipt_id.validate()?;
        if self.revenue_event_id != self.derive_revenue_event_id()? {
            return Err(RevenueEventError::IdentityMismatch);
        }
        if self.source.trim().is_empty()
            || self.gross_collected.units == 0
            || self.gross_collected.asset.trim().is_empty()
            || self.evidence_root.trim().is_empty()
            || self.signature.trim().is_empty()
        {
            return Err(RevenueEventError::MissingSettlementEvidence);
        }
        let deductions = [&self.refunds, &self.service_costs, &self.reserves];
        let deduction_total = deductions.iter().try_fold(0u128, |total, deduction| {
            self.gross_collected.ensure_compatible(deduction)?;
            total
                .checked_add(deduction.units)
                .ok_or(RevenueEventError::Overflow)
        })?;
        if deduction_total > self.gross_collected.units {
            return Err(RevenueEventError::DeductionsExceedGross);
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RevenueEventError {
    #[error(transparent)]
    Id(#[from] crate::IdError),
    #[error(transparent)]
    Money(#[from] crate::MoneyError),
    #[error("revenue-event content identity does not match")]
    IdentityMismatch,
    #[error("revenue event lacks realized settlement evidence")]
    MissingSettlementEvidence,
    #[error("revenue deductions exceed gross collection")]
    DeductionsExceedGross,
    #[error("checked revenue arithmetic overflow")]
    Overflow,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyDividend {
    pub dividend_id: DividendId,
    pub revenue_event_id: RevenueEventId,
    pub downstream_claim_id: ClaimId,
    pub upstream_claim_id: ClaimId,
    /// A formal dependency is evidence of use, not payment authorization.
    pub used_in_final_proof: bool,
    pub final_dependency_root: String,
    /// Prescriptive economic edge and policy agreed before allocation.
    pub eligible_economic_edge_root: String,
    pub economic_policy_root: String,
    pub settlement_receipt_id: ReceiptId,
    pub compute_savings_evidence_root: String,
    pub downstream_net_revenue: Amount,
    pub upstream_pool: Amount,
    pub payout: Amount,
    pub cap_bps: u16,
    pub non_recursive: bool,
    pub finalized_at: DateTime<Utc>,
    pub signature: String,
}

impl DependencyDividend {
    pub fn respects_protocol_cap(&self) -> bool {
        if !self.used_in_final_proof
            || !self.non_recursive
            || self.cap_bps > 10_000
            || self.eligible_economic_edge_root.trim().is_empty()
            || self.economic_policy_root.trim().is_empty()
            || self.settlement_receipt_id.validate().is_err()
        {
            return false;
        }
        if self
            .downstream_net_revenue
            .ensure_compatible(&self.payout)
            .is_err()
            || self.upstream_pool.ensure_compatible(&self.payout).is_err()
        {
            return false;
        }
        let Ok(revenue_cap) = self.downstream_net_revenue.mul_bps(self.cap_bps) else {
            return false;
        };
        self.payout.units <= revenue_cap.units && self.payout.units <= self.upstream_pool.units
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomicParticipationTerms {
    pub revenue_source: String,
    pub payer: String,
    pub calculation_base: String,
    pub exclusions: Vec<String>,
    pub share_bps: u16,
    pub payout_cap: Amount,
    pub transferable: bool,
    pub term_starts_at: DateTime<Utc>,
    pub term_ends_at: DateTime<Utc>,
    pub dispute_process: String,
    pub economic_policy_root: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct License {
    pub license_id: LicenseId,
    pub rights_manifest_hash: String,
    pub licensor: String,
    pub licensee: String,
    pub mode: CapsuleEconomicMode,
    pub scope: Vec<String>,
    pub economic_terms: Option<EconomicParticipationTerms>,
    pub consideration_receipt_id: Option<ReceiptId>,
    pub effective_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub supersedes: Option<LicenseId>,
    pub signatures: Vec<String>,
}

impl License {
    pub fn has_bounded_economic_scope(&self) -> bool {
        if self.licensor.trim().is_empty()
            || self.licensee.trim().is_empty()
            || self.scope.is_empty()
            || self.signatures.is_empty()
            || self.expires_at.is_some_and(|end| end <= self.effective_at)
        {
            return false;
        }
        match (&self.mode, &self.economic_terms) {
            (CapsuleEconomicMode::Commons, None) => true,
            (CapsuleEconomicMode::Commons, Some(_)) => false,
            (_, Some(terms)) => {
                !terms.revenue_source.trim().is_empty()
                    && !terms.payer.trim().is_empty()
                    && !terms.calculation_base.trim().is_empty()
                    && !terms.dispute_process.trim().is_empty()
                    && !terms.economic_policy_root.trim().is_empty()
                    && terms.share_bps <= 10_000
                    && terms.payout_cap.units > 0
                    && terms.term_starts_at >= self.effective_at
                    && terms.term_ends_at > terms.term_starts_at
                    && self
                        .expires_at
                        .is_none_or(|license_end| terms.term_ends_at <= license_end)
            }
            (_, None) => false,
        }
    }
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

    fn computational_profile() -> VerificationProfile {
        VerificationProfile {
            policy_id: PolicyId::derive(&"computational-profile").unwrap(),
            class: VerificationProfileClass::Computational,
            required_evidence: [
                VerificationEvidenceKind::ReproducibleSource,
                VerificationEvidenceKind::ExecutionEnvironment,
                VerificationEvidenceKind::DependencyLock,
                VerificationEvidenceKind::DeterministicRerun,
            ]
            .into_iter()
            .collect(),
            verifier_implementations: ["container-runner-v1".to_owned()].into_iter().collect(),
            minimum_reproductions: 2,
            minimum_independent_operators: 2,
            challenge_window_seconds: 86_400,
            policy_root: "blake3:computational-profile".into(),
        }
    }

    fn computational_job(profile: &VerificationProfile) -> VerificationJob {
        let theory_id = TheoryId::derive(&"computational-methodology").unwrap();
        let now = Utc::now();
        VerificationJob {
            job_id: JobId::derive(&"computational-job").unwrap(),
            researcher_id: ResearcherId::derive(&"researcher").unwrap(),
            claim_id: ClaimId::from_canonical_elaborated_type(
                &theory_id,
                "benchmark output is reproducible under the declared environment",
            )
            .unwrap(),
            theory_id,
            candidate_proof_id: None,
            artifact_id: ArtifactId::derive(&"computational-bundle").unwrap(),
            verification_policy_id: profile.policy_id.clone(),
            verification_profile_class: profile.class,
            evidence_roots: [
                (
                    VerificationEvidenceKind::ReproducibleSource,
                    "blake3:source".to_owned(),
                ),
                (
                    VerificationEvidenceKind::ExecutionEnvironment,
                    "blake3:container".to_owned(),
                ),
                (
                    VerificationEvidenceKind::DependencyLock,
                    "blake3:lock".to_owned(),
                ),
                (
                    VerificationEvidenceKind::DeterministicRerun,
                    "blake3:rerun-protocol".to_owned(),
                ),
            ]
            .into_iter()
            .collect(),
            producer_operator_cluster_id: Some(
                OperatorClusterId::derive(&"producer-cluster").unwrap(),
            ),
            maximum_budget: amount(1_000),
            state: VerificationState::CheckersRevealed,
            observation_receipt_ids: vec![],
            created_at: now,
            updated_at: now,
        }
    }

    fn reproduction(
        job: &VerificationJob,
        node: &str,
        cluster: &str,
        verdict: ObservationVerdict,
    ) -> ReproductionObservation {
        let mut observation = ReproductionObservation {
            receipt_id: ReceiptId::derive(&"placeholder").unwrap(),
            job_id: job.job_id.clone(),
            claim_id: job.claim_id.clone(),
            verification_profile_class: job.verification_profile_class,
            verifier_node_id: NodeId::derive(&node).unwrap(),
            verified_user_id: VerifiedUserId::derive(&format!("user-{node}")).unwrap(),
            operator_id: OperatorId::derive(&format!("operator-{node}")).unwrap(),
            operator_cluster_id: OperatorClusterId::derive(&cluster).unwrap(),
            user_credential_id: UserCredentialId::derive(&format!("user-credential-{node}"))
                .unwrap(),
            operator_credential_id: OperatorCredentialId::derive(&format!(
                "operator-credential-{node}"
            ))
            .unwrap(),
            node_credential_id: NodeCredentialId::derive(&format!("node-credential-{node}"))
                .unwrap(),
            credential_chain_root: format!("blake3:credential-chain-{node}"),
            verifier_implementation: "container-runner-v1".into(),
            infrastructure_provider: format!("provider-{node}"),
            region: format!("region-{node}"),
            input_artifact_id: job.artifact_id.clone(),
            exact_input_root: "blake3:exact-computational-input".into(),
            evidence_roots: job.evidence_roots.clone(),
            verdict,
            execution_trace_root: format!("blake3:trace-{node}"),
            observation_root: String::new(),
            commitment: String::new(),
            reveal_salt: format!("salt-{node}"),
            committed_at: job.updated_at,
            reproduced_at: job.updated_at + Duration::minutes(1),
            signature: format!("signature-{node}"),
        };
        observation.observation_root = observation.expected_observation_root().unwrap();
        observation.commitment = crate::observation_commitment(
            &observation.job_id,
            observation.verdict,
            &observation.observation_root,
            observation.reveal_salt.as_bytes(),
        );
        observation.receipt_id = observation.derive_receipt_id().unwrap();
        observation
    }

    #[test]
    fn non_formal_profiles_receive_independent_reproduction_certificates() {
        let profile = computational_profile();
        let job = computational_job(&profile);
        let observations = vec![
            reproduction(&job, "node-a", "cluster-a", ObservationVerdict::Pass),
            reproduction(&job, "node-b", "cluster-b", ObservationVerdict::Pass),
        ];
        let certificate = ResearchVerificationCertificate::issue(
            &job,
            &profile,
            &observations,
            "blake3:reproduction-evidence".into(),
            job.updated_at + Duration::hours(1),
            "aggregate-signature".into(),
        )
        .unwrap();
        assert_eq!(certificate.status, ResearchCertificationStatus::Certified);
        assert!(certificate
            .validate_against(&job, &profile, &observations)
            .is_ok());
    }

    #[test]
    fn published_computational_vectors_share_exact_content_identities() {
        let profile: VerificationProfile = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/computational-verification-profile.json"
        ))
        .unwrap();
        let job: VerificationJob = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/computational-verification-job.json"
        ))
        .unwrap();
        let observations: Vec<ReproductionObservation> = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/computational-observations.json"
        ))
        .unwrap();
        let certificate: ResearchVerificationCertificate = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/computational-research-certificate.json"
        ))
        .unwrap();
        assert!(certificate
            .validate_against(&job, &profile, &observations)
            .is_ok());
    }

    #[test]
    fn one_dissenting_reproduction_causes_divergence_not_majority_certification() {
        let profile = computational_profile();
        let job = computational_job(&profile);
        let observations = vec![
            reproduction(&job, "node-a", "cluster-a", ObservationVerdict::Pass),
            reproduction(&job, "node-b", "cluster-b", ObservationVerdict::Pass),
            reproduction(&job, "node-c", "cluster-c", ObservationVerdict::Fail),
        ];
        assert_eq!(
            evaluate_reproduction(&job, &profile, &observations).unwrap(),
            ResearchCertificationStatus::Divergent
        );
    }

    #[test]
    fn producer_cannot_count_as_an_independent_reproducer() {
        let profile = computational_profile();
        let job = computational_job(&profile);
        let observation = reproduction(
            &job,
            "producer-node",
            "producer-cluster",
            ObservationVerdict::Pass,
        );
        assert!(matches!(
            observation.validate_against(&job, &profile),
            Err(ResearchVerificationError::ProducerSelfCertification)
        ));
    }

    #[test]
    fn reproduction_commitment_binds_identity_evidence_and_verdict() {
        let profile = computational_profile();
        let job = computational_job(&profile);
        let observation = reproduction(&job, "node-a", "cluster-a", ObservationVerdict::Pass);

        let mut substituted_operator = observation.clone();
        substituted_operator.operator_id = OperatorId::derive(&"attacker-operator").unwrap();
        assert!(matches!(
            substituted_operator.validate_against(&job, &profile),
            Err(ResearchVerificationError::IdentityMismatch)
        ));

        let mut changed_evidence = observation.clone();
        changed_evidence.execution_trace_root = "blake3:substituted-trace".into();
        assert!(matches!(
            changed_evidence.validate_against(&job, &profile),
            Err(ResearchVerificationError::IdentityMismatch)
        ));

        let mut changed_verdict = observation;
        changed_verdict.verdict = ObservationVerdict::Fail;
        changed_verdict.observation_root = changed_verdict.expected_observation_root().unwrap();
        changed_verdict.receipt_id = changed_verdict.derive_receipt_id().unwrap();
        assert!(matches!(
            changed_verdict.validate_against(&job, &profile),
            Err(ResearchVerificationError::CommitmentMismatch)
        ));
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
    fn revenue_event_identity_binds_related_party_and_settlement_amounts() {
        let theory_id = TheoryId::derive(&"revenue-theory").unwrap();
        let mut event = RevenueEvent {
            revenue_event_id: RevenueEventId::derive(&"placeholder").unwrap(),
            claim_id: ClaimId::from_canonical_elaborated_type(&theory_id, "revenue claim").unwrap(),
            source: "commercial_license".into(),
            related_party: false,
            settlement_receipt_id: ReceiptId::derive(&"settlement").unwrap(),
            gross_collected: amount(1_000),
            refunds: amount(100),
            service_costs: amount(100),
            reserves: amount(100),
            realized_at: Utc::now(),
            evidence_root: "blake3:settlement-evidence".into(),
            signature: "signature".into(),
        };
        event.revenue_event_id = event.derive_revenue_event_id().unwrap();
        assert!(event.validate_integrity().is_ok());
        event.related_party = true;
        assert!(matches!(
            event.validate_integrity(),
            Err(RevenueEventError::IdentityMismatch)
        ));
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
            eligible_economic_edge_root: "blake3:economic-edge".into(),
            economic_policy_root: "blake3:economic-policy".into(),
            settlement_receipt_id: ReceiptId::derive(&"settlement").unwrap(),
            compute_savings_evidence_root: "blake3:savings".into(),
            downstream_net_revenue: amount(1_000),
            upstream_pool: amount(100),
            payout: amount(10),
            cap_bps: 1_000,
            non_recursive: true,
            finalized_at: Utc::now(),
            signature: "signature".into(),
        };
        assert!(!dividend.respects_protocol_cap());
    }

    #[test]
    fn formal_dependency_without_an_economic_edge_never_creates_payment() {
        let claim = |name| {
            ClaimId::from_canonical_elaborated_type(&TheoryId::derive(&"theory").unwrap(), name)
                .unwrap()
        };
        let dividend = DependencyDividend {
            dividend_id: DividendId::derive(&"dividend").unwrap(),
            revenue_event_id: RevenueEventId::derive(&"revenue").unwrap(),
            downstream_claim_id: claim("downstream"),
            upstream_claim_id: claim("upstream"),
            used_in_final_proof: true,
            final_dependency_root: "blake3:final-dependencies".into(),
            eligible_economic_edge_root: String::new(),
            economic_policy_root: "blake3:economic-policy".into(),
            settlement_receipt_id: ReceiptId::derive(&"settlement").unwrap(),
            compute_savings_evidence_root: "blake3:impact-signal".into(),
            downstream_net_revenue: amount(1_000),
            upstream_pool: amount(100),
            payout: amount(10),
            cap_bps: 1_000,
            non_recursive: true,
            finalized_at: Utc::now(),
            signature: "signature".into(),
        };
        assert!(!dividend.respects_protocol_cap());
    }

    #[test]
    fn open_commons_license_cannot_hide_revenue_participation_terms() {
        let license = License {
            license_id: LicenseId::derive(&"license").unwrap(),
            rights_manifest_hash: "blake3:rights".into(),
            licensor: "licensor".into(),
            licensee: "public".into(),
            mode: CapsuleEconomicMode::Commons,
            scope: vec!["formal artifact".into()],
            economic_terms: Some(EconomicParticipationTerms {
                revenue_source: "future use".into(),
                payer: "unknown".into(),
                calculation_base: "undefined".into(),
                exclusions: vec![],
                share_bps: 100,
                payout_cap: amount(1),
                transferable: false,
                term_starts_at: Utc::now(),
                term_ends_at: Utc::now() + chrono::Duration::days(30),
                dispute_process: "review".into(),
                economic_policy_root: "blake3:policy".into(),
            }),
            consideration_receipt_id: None,
            effective_at: Utc::now(),
            expires_at: None,
            supersedes: None,
            signatures: vec!["signature".into()],
        };
        assert!(!license.has_bounded_economic_scope());
    }

    #[test]
    fn commercial_economic_participation_requires_a_finite_term() {
        let now = Utc::now();
        let mut license = License {
            license_id: LicenseId::derive(&"commercial-license").unwrap(),
            rights_manifest_hash: "blake3:rights".into(),
            licensor: "licensor".into(),
            licensee: "commercial-buyer".into(),
            mode: CapsuleEconomicMode::CommercialArtifact,
            scope: vec!["certified implementation".into()],
            economic_terms: Some(EconomicParticipationTerms {
                revenue_source: "license receipts".into(),
                payer: "commercial-buyer".into(),
                calculation_base: "net settled receipts".into(),
                exclusions: vec!["refunds".into()],
                share_bps: 1_000,
                payout_cap: amount(100),
                transferable: false,
                term_starts_at: now,
                term_ends_at: now + chrono::Duration::days(30),
                dispute_process: "contract arbitration".into(),
                economic_policy_root: "blake3:commercial-policy".into(),
            }),
            consideration_receipt_id: None,
            effective_at: now,
            expires_at: Some(now + chrono::Duration::days(90)),
            supersedes: None,
            signatures: vec!["signature".into()],
        };
        assert!(license.has_bounded_economic_scope());

        let terms = license.economic_terms.as_mut().unwrap();
        terms.term_ends_at = terms.term_starts_at;
        assert!(!license.has_bounded_economic_scope());
    }
}

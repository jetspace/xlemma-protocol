//! Native XLMP node-network, marketplace, reputation, bond, and sortition records.
//!
//! These objects describe discoverable services and auditable selection without
//! allowing price, collateral, or reputation to become mathematical voting
//! weight. Historical advertisements, reputation snapshots, bonds, orders,
//! matches, and committee selections are append-only protocol evidence.

use crate::{
    canonical_json_hash, AdvertisementId, Amount, BondId, CanonicalizationError, CheckerFamily,
    CredentialTier, DiscoveryId, IdError, JobId, NodeCredentialChain, NodeCredentialId, NodeId,
    NodeRole, OperatorClusterId, OperatorCredentialId, OperatorId, PolicyId, ReputationId,
    ServiceMatchId, ServiceOrderId, SortitionId, TheoryId, UserCredentialId, VerifiedUserId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeServiceKind {
    Formalization,
    ProofSearch,
    ProofRepair,
    Explanation,
    FormalBuild,
    OfficialVerification,
    IndependentVerification,
    NoveltyReview,
    SignificanceReview,
    HumanExpertReview,
    Storage,
    Indexing,
    ChallengeMonitoring,
    CertificateFinalization,
    Revalidation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingModel {
    Fixed,
    Metered,
    UpTo,
    BatchSettlement,
    InstitutionalInvoice,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServicePrice {
    pub pricing_model: PricingModel,
    pub unit_name: String,
    pub quantity_scale: u64,
    pub amount: Amount,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub class: String,
    pub architecture: String,
    pub accelerator: Option<String>,
    pub memory_mib: Option<u64>,
    pub trusted_execution_attestation: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub transport: String,
    pub uri: String,
    pub authentication_profile: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceCapability {
    pub service: NodeServiceKind,
    pub implementation_id: Option<String>,
    pub checker_families: BTreeSet<CheckerFamily>,
    pub supported_theory_ids: BTreeSet<TheoryId>,
    pub domains: BTreeSet<String>,
    pub hardware: Option<HardwareProfile>,
    pub maximum_parallel_jobs: u32,
    pub capacity_units: u64,
    pub available_units: u64,
    pub p50_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub price: ServicePrice,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReputationMetric {
    pub score_bps: u16,
    pub sample_size: u64,
    pub evidence_root: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReputationRequirement {
    pub minimum_score_bps: u16,
    pub minimum_sample_size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeReputationVector {
    pub formal_accuracy: ReputationMetric,
    pub availability: ReputationMetric,
    pub latency: ReputationMetric,
    pub novelty_calibration: ReputationMetric,
    pub challenge_quality: ReputationMetric,
    pub independence: ReputationMetric,
    pub storage_quality: ReputationMetric,
    pub integrity: ReputationMetric,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReputationRequirements {
    pub formal_accuracy: Option<ReputationRequirement>,
    pub availability: Option<ReputationRequirement>,
    pub latency: Option<ReputationRequirement>,
    pub novelty_calibration: Option<ReputationRequirement>,
    pub challenge_quality: Option<ReputationRequirement>,
    pub independence: Option<ReputationRequirement>,
    pub storage_quality: Option<ReputationRequirement>,
    pub integrity: Option<ReputationRequirement>,
}

impl NodeReputationVector {
    pub fn is_valid(&self) -> bool {
        self.metrics()
            .into_iter()
            .all(|metric| metric.score_bps <= 10_000 && !metric.evidence_root.trim().is_empty())
    }

    /// Requirements are evaluated dimension by dimension. No scalar score is
    /// produced, so excellence in one role cannot hide weakness in another.
    pub fn meets(&self, requirements: &ReputationRequirements) -> bool {
        self.is_valid()
            && metric_meets(&self.formal_accuracy, requirements.formal_accuracy.as_ref())
            && metric_meets(&self.availability, requirements.availability.as_ref())
            && metric_meets(&self.latency, requirements.latency.as_ref())
            && metric_meets(
                &self.novelty_calibration,
                requirements.novelty_calibration.as_ref(),
            )
            && metric_meets(
                &self.challenge_quality,
                requirements.challenge_quality.as_ref(),
            )
            && metric_meets(&self.independence, requirements.independence.as_ref())
            && metric_meets(&self.storage_quality, requirements.storage_quality.as_ref())
            && metric_meets(&self.integrity, requirements.integrity.as_ref())
    }

    fn metrics(&self) -> [&ReputationMetric; 8] {
        [
            &self.formal_accuracy,
            &self.availability,
            &self.latency,
            &self.novelty_calibration,
            &self.challenge_quality,
            &self.independence,
            &self.storage_quality,
            &self.integrity,
        ]
    }
}

fn metric_meets(metric: &ReputationMetric, requirement: Option<&ReputationRequirement>) -> bool {
    requirement.is_none_or(|required| {
        required.minimum_score_bps <= 10_000
            && metric.score_bps >= required.minimum_score_bps
            && metric.sample_size >= required.minimum_sample_size
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeReputationSnapshot {
    pub reputation_id: ReputationId,
    pub operator_id: OperatorId,
    pub node_id: NodeId,
    pub operator_cluster_id: OperatorClusterId,
    pub vector: NodeReputationVector,
    pub policy_id: PolicyId,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub evidence_root: String,
    pub assessed_at: DateTime<Utc>,
    pub supersedes: Option<ReputationId>,
    pub assessor_signature: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeBondStatus {
    Active,
    Released,
    PartiallySlashed,
    Slashed,
    Quarantined,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeBond {
    pub bond_id: BondId,
    pub node_id: NodeId,
    pub operator_id: OperatorId,
    pub operator_cluster_id: OperatorClusterId,
    pub amount: Amount,
    pub eligible_roles: BTreeSet<NodeRole>,
    pub slashing_policy_id: PolicyId,
    pub escrow_reference: String,
    pub status: NodeBondStatus,
    pub locked_until: DateTime<Utc>,
    pub evidence_root: String,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeServiceAdvertisement {
    pub advertisement_id: AdvertisementId,
    pub node_id: NodeId,
    pub operator_id: OperatorId,
    pub operator_cluster_id: OperatorClusterId,
    pub user_credential_id: UserCredentialId,
    pub operator_credential_id: OperatorCredentialId,
    pub node_credential_id: NodeCredentialId,
    pub credential_chain_root: String,
    pub jurisdiction_class: String,
    pub sequence: u64,
    pub roles: BTreeSet<NodeRole>,
    pub endpoints: Vec<ServiceEndpoint>,
    pub capabilities: Vec<ServiceCapability>,
    pub reputation_snapshot_id: ReputationId,
    pub bond_id: BondId,
    pub terms_root: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub supersedes: Option<AdvertisementId>,
    pub delegation_signature: String,
    pub signature: String,
}

#[derive(Serialize)]
struct AdvertisementIdentity<'a> {
    node_id: &'a NodeId,
    operator_id: &'a OperatorId,
    operator_cluster_id: &'a OperatorClusterId,
    user_credential_id: &'a UserCredentialId,
    operator_credential_id: &'a OperatorCredentialId,
    node_credential_id: &'a NodeCredentialId,
    credential_chain_root: &'a str,
    jurisdiction_class: &'a str,
    sequence: u64,
    roles: &'a BTreeSet<NodeRole>,
    endpoints: &'a [ServiceEndpoint],
    capabilities: &'a [ServiceCapability],
    reputation_snapshot_id: &'a ReputationId,
    bond_id: &'a BondId,
    terms_root: &'a str,
    valid_from: DateTime<Utc>,
    valid_until: DateTime<Utc>,
    supersedes: &'a Option<AdvertisementId>,
}

impl NodeServiceAdvertisement {
    pub fn derive_advertisement_id(&self) -> Result<AdvertisementId, IdError> {
        AdvertisementId::derive(&AdvertisementIdentity {
            node_id: &self.node_id,
            operator_id: &self.operator_id,
            operator_cluster_id: &self.operator_cluster_id,
            user_credential_id: &self.user_credential_id,
            operator_credential_id: &self.operator_credential_id,
            node_credential_id: &self.node_credential_id,
            credential_chain_root: &self.credential_chain_root,
            jurisdiction_class: &self.jurisdiction_class,
            sequence: self.sequence,
            roles: &self.roles,
            endpoints: &self.endpoints,
            capabilities: &self.capabilities,
            reputation_snapshot_id: &self.reputation_snapshot_id,
            bond_id: &self.bond_id,
            terms_root: &self.terms_root,
            valid_from: self.valid_from,
            valid_until: self.valid_until,
            supersedes: &self.supersedes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeDiscoveryRequest {
    pub discovery_id: DiscoveryId,
    pub requester: String,
    pub services: BTreeSet<NodeServiceKind>,
    pub required_roles: BTreeSet<NodeRole>,
    pub required_checker_families: BTreeSet<CheckerFamily>,
    pub theory_id: Option<TheoryId>,
    pub domains: BTreeSet<String>,
    pub minimum_available_units: u64,
    pub maximum_p95_latency_ms: Option<u64>,
    pub maximum_unit_price: Option<ServicePrice>,
    pub reputation_requirements: ReputationRequirements,
    pub excluded_operator_clusters: BTreeSet<OperatorClusterId>,
    pub requested_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeDiscoveryResult {
    pub discovery_id: DiscoveryId,
    pub advertisement_ids: Vec<AdvertisementId>,
    pub advertisement_set_root: String,
    pub generated_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceOrder {
    pub order_id: ServiceOrderId,
    pub job_id: JobId,
    pub requester: String,
    pub service: NodeServiceKind,
    pub required_role: NodeRole,
    pub required_checker_families: BTreeSet<CheckerFamily>,
    pub theory_id: Option<TheoryId>,
    pub domains: BTreeSet<String>,
    pub quantity_units: u64,
    pub maximum_total_price: Amount,
    pub maximum_p95_latency_ms: Option<u64>,
    pub reputation_requirements: ReputationRequirements,
    pub excluded_operator_clusters: BTreeSet<OperatorClusterId>,
    pub delivery_deadline: DateTime<Utc>,
    pub terms_root: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceMatch {
    pub match_id: ServiceMatchId,
    pub order_id: ServiceOrderId,
    pub advertisement_id: AdvertisementId,
    pub advertisement_sequence: u64,
    pub node_id: NodeId,
    pub operator_id: OperatorId,
    pub operator_cluster_id: OperatorClusterId,
    pub service: NodeServiceKind,
    pub reserved_units: u64,
    pub agreed_price: Amount,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: DateTime<Utc>,
    pub matched_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Serialize)]
struct ServiceMatchIdentity<'a> {
    order_id: &'a ServiceOrderId,
    advertisement_id: &'a AdvertisementId,
    advertisement_sequence: u64,
    node_id: &'a NodeId,
    operator_id: &'a OperatorId,
    operator_cluster_id: &'a OperatorClusterId,
    service: NodeServiceKind,
    reserved_units: u64,
    agreed_price: &'a Amount,
    scheduled_start: DateTime<Utc>,
    scheduled_end: DateTime<Utc>,
    matched_at: DateTime<Utc>,
}

impl ServiceMatch {
    pub fn derive_match_id(&self) -> Result<ServiceMatchId, IdError> {
        ServiceMatchId::derive(&ServiceMatchIdentity {
            order_id: &self.order_id,
            advertisement_id: &self.advertisement_id,
            advertisement_sequence: self.advertisement_sequence,
            node_id: &self.node_id,
            operator_id: &self.operator_id,
            operator_cluster_id: &self.operator_cluster_id,
            service: self.service,
            reserved_units: self.reserved_units,
            agreed_price: &self.agreed_price,
            scheduled_start: self.scheduled_start,
            scheduled_end: self.scheduled_end,
            matched_at: self.matched_at,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EligibleNode {
    pub node_id: NodeId,
    pub operator_id: OperatorId,
    pub operator_cluster_id: OperatorClusterId,
    pub credential_chain: NodeCredentialChain,
    pub advertisement_id: AdvertisementId,
    pub roles: BTreeSet<NodeRole>,
    pub checker_families: BTreeSet<CheckerFamily>,
    pub bond_id: BondId,
    pub active_bond: Amount,
    pub reputation_snapshot_id: ReputationId,
    pub reputation: NodeReputationVector,
    pub infrastructure_provider: String,
    pub region: String,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeRequirement {
    pub role: NodeRole,
    pub count: u16,
    pub minimum_bond: Amount,
    pub reputation_requirements: ReputationRequirements,
    pub required_checker_families: BTreeSet<CheckerFamily>,
    pub minimum_credential_tier: CredentialTier,
    pub maximum_status_age_seconds: u64,
    pub required_qualifications: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RandomnessBeacon {
    pub source: String,
    pub round: u64,
    pub seed_commitment: String,
    pub proof_reference: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeSortitionRequest {
    pub sortition_id: SortitionId,
    pub job_id: JobId,
    pub policy_id: PolicyId,
    pub epoch: u64,
    pub eligible_set_root: String,
    pub randomness: RandomnessBeacon,
    pub requirements: Vec<CommitteeRequirement>,
    pub minimum_distinct_providers: u16,
    pub minimum_distinct_regions: u16,
    pub excluded_operator_clusters: BTreeSet<OperatorClusterId>,
    pub requested_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortitionMember {
    pub role: NodeRole,
    pub slot: u16,
    pub node_id: NodeId,
    pub verified_user_id: VerifiedUserId,
    pub operator_id: OperatorId,
    pub operator_cluster_id: OperatorClusterId,
    pub user_credential_id: UserCredentialId,
    pub operator_credential_id: OperatorCredentialId,
    pub node_credential_id: NodeCredentialId,
    pub credential_tier: CredentialTier,
    pub credential_chain_root: String,
    pub advertisement_id: AdvertisementId,
    pub bond_id: BondId,
    pub reputation_snapshot_id: ReputationId,
    pub infrastructure_provider: String,
    pub region: String,
    pub rank_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeSelection {
    pub sortition_id: SortitionId,
    pub job_id: JobId,
    pub policy_id: PolicyId,
    pub randomness_commitment: String,
    pub eligible_set_root: String,
    pub members: Vec<SortitionMember>,
    pub selection_root: String,
    pub selected_at: DateTime<Utc>,
}

/// Canonical commitment to the complete records made eligible for sortition.
pub fn derive_eligible_set_root(nodes: &[EligibleNode]) -> Result<String, CanonicalizationError> {
    let mut ordered = nodes.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then_with(|| left.operator_id.cmp(&right.operator_id))
            .then_with(|| left.advertisement_id.cmp(&right.advertisement_id))
            .then_with(|| left.operator_cluster_id.cmp(&right.operator_cluster_id))
            .then_with(|| left.bond_id.cmp(&right.bond_id))
            .then_with(|| {
                left.reputation_snapshot_id
                    .cmp(&right.reputation_snapshot_id)
            })
    });
    let digest = canonical_json_hash("xlemma-sortition-eligible-set-v1", &ordered)?;
    Ok(format!(
        "blake3:{}",
        blake3::Hash::from_bytes(digest).to_hex()
    ))
}

pub fn service_role(service: NodeServiceKind) -> NodeRole {
    match service {
        NodeServiceKind::Formalization
        | NodeServiceKind::ProofSearch
        | NodeServiceKind::ProofRepair
        | NodeServiceKind::Explanation => NodeRole::ResearchProver,
        NodeServiceKind::FormalBuild => NodeRole::LeanBuilder,
        NodeServiceKind::OfficialVerification => NodeRole::OfficialKernelChecker,
        NodeServiceKind::IndependentVerification => NodeRole::IndependentChecker,
        NodeServiceKind::NoveltyReview | NodeServiceKind::HumanExpertReview => {
            NodeRole::NoveltyReviewer
        }
        NodeServiceKind::SignificanceReview => NodeRole::SignificanceReviewer,
        NodeServiceKind::Storage => NodeRole::StorageProvider,
        NodeServiceKind::Indexing => NodeRole::Indexer,
        NodeServiceKind::ChallengeMonitoring => NodeRole::Challenger,
        NodeServiceKind::CertificateFinalization | NodeServiceKind::Revalidation => {
            NodeRole::CertificateFinalizer
        }
    }
}

pub fn capability_index(
    advertisement: &NodeServiceAdvertisement,
) -> BTreeMap<NodeServiceKind, Vec<&ServiceCapability>> {
    let mut index: BTreeMap<NodeServiceKind, Vec<&ServiceCapability>> = BTreeMap::new();
    for capability in &advertisement.capabilities {
        index
            .entry(capability.service)
            .or_default()
            .push(capability);
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metric(score_bps: u16) -> ReputationMetric {
        ReputationMetric {
            score_bps,
            sample_size: 100,
            evidence_root: "blake3:evidence".into(),
        }
    }

    #[test]
    fn reputation_dimensions_do_not_compensate_for_each_other() {
        let vector = NodeReputationVector {
            formal_accuracy: metric(7_000),
            availability: metric(9_900),
            latency: metric(10_000),
            novelty_calibration: metric(9_900),
            challenge_quality: metric(9_900),
            independence: metric(9_900),
            storage_quality: metric(9_900),
            integrity: metric(9_900),
        };
        let requirements = ReputationRequirements {
            formal_accuracy: Some(ReputationRequirement {
                minimum_score_bps: 9_000,
                minimum_sample_size: 10,
            }),
            latency: Some(ReputationRequirement {
                minimum_score_bps: 8_000,
                minimum_sample_size: 10,
            }),
            ..ReputationRequirements::default()
        };
        assert!(!vector.meets(&requirements));
    }
}

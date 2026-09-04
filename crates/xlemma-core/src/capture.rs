//! Cooperative compute ownership, work-backed node revenue, objective
//! misconduct, and protocol capture-resistance measurements.

use crate::{
    Amount, BondId, CaptureDashboardId, ComputeCooperativeId, IdError, JobId, MisconductRecordId,
    NodeId, NodeServiceKind, NodeWorkReceiptId, OperatorClusterId, OperatorId, PolicyId, ReceiptId,
    VerifiedUserId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchComputeCooperative {
    pub cooperative_id: ComputeCooperativeId,
    /// Ownership or governance participation in basis points. Membership does
    /// not create extra committee independence inside this one cluster.
    pub members: BTreeMap<VerifiedUserId, u16>,
    pub operator_cluster_id: OperatorClusterId,
    pub node_ids: BTreeSet<NodeId>,
    pub capability_classes: BTreeSet<String>,
    pub shared_treasury: String,
    pub governance_policy_id: PolicyId,
    pub beneficial_control_evidence_root: String,
    pub created_at: DateTime<Utc>,
    pub supersedes: Option<ComputeCooperativeId>,
    pub signatures: Vec<String>,
}

#[derive(Serialize)]
struct CooperativeIdentity<'a> {
    members: &'a BTreeMap<VerifiedUserId, u16>,
    operator_cluster_id: &'a OperatorClusterId,
    node_ids: &'a BTreeSet<NodeId>,
    capability_classes: &'a BTreeSet<String>,
    shared_treasury: &'a str,
    governance_policy_id: &'a PolicyId,
    beneficial_control_evidence_root: &'a str,
    created_at: DateTime<Utc>,
    supersedes: &'a Option<ComputeCooperativeId>,
}

impl ResearchComputeCooperative {
    pub fn derive_cooperative_id(&self) -> Result<ComputeCooperativeId, IdError> {
        ComputeCooperativeId::derive(&CooperativeIdentity {
            members: &self.members,
            operator_cluster_id: &self.operator_cluster_id,
            node_ids: &self.node_ids,
            capability_classes: &self.capability_classes,
            shared_treasury: &self.shared_treasury,
            governance_policy_id: &self.governance_policy_id,
            beneficial_control_evidence_root: &self.beneficial_control_evidence_root,
            created_at: self.created_at,
            supersedes: &self.supersedes,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), CaptureError> {
        self.cooperative_id.validate()?;
        self.operator_cluster_id.validate()?;
        self.governance_policy_id.validate()?;
        if self.cooperative_id != self.derive_cooperative_id()? {
            return Err(CaptureError::IdentityMismatch);
        }
        if self.supersedes.as_ref() == Some(&self.cooperative_id) {
            return Err(CaptureError::SelfSupersession);
        }
        if self.members.len() < 2
            || self.members.values().any(|share| *share == 0)
            || self
                .members
                .values()
                .try_fold(0u16, |total, share| total.checked_add(*share))
                != Some(10_000)
        {
            return Err(CaptureError::InvalidCooperativeOwnership);
        }
        for member in self.members.keys() {
            member.validate()?;
        }
        for node in &self.node_ids {
            node.validate()?;
        }
        if self.node_ids.is_empty()
            || self.capability_classes.is_empty()
            || self
                .capability_classes
                .iter()
                .any(|value| value.trim().is_empty())
            || self.shared_treasury.trim().is_empty()
            || self.beneficial_control_evidence_root.trim().is_empty()
            || self.signatures.len() < 2
            || self.signatures.iter().any(|value| value.trim().is_empty())
        {
            return Err(CaptureError::MissingEvidence);
        }
        Ok(())
    }
}

/// Returns the overlapping beneficial ownership in basis points using the
/// conservative sum of the smaller share held by every common member.
pub fn cooperative_ownership_overlap_bps(
    left: &ResearchComputeCooperative,
    right: &ResearchComputeCooperative,
) -> Result<u16, CaptureError> {
    left.validate_integrity()?;
    right.validate_integrity()?;
    Ok(left
        .members
        .iter()
        .filter_map(|(member, left_share)| {
            right
                .members
                .get(member)
                .map(|right_share| (*left_share).min(*right_share))
        })
        .sum())
}

pub fn cooperative_independence_credit_bps(
    left: &ResearchComputeCooperative,
    right: &ResearchComputeCooperative,
) -> Result<u16, CaptureError> {
    Ok(10_000 - cooperative_ownership_overlap_bps(left, right)?)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureLayer {
    Identity,
    Compute,
    Models,
    Verification,
    Storage,
    Settlement,
    Discovery,
    Governance,
}

impl CaptureLayer {
    fn all() -> [Self; 8] {
        [
            Self::Identity,
            Self::Compute,
            Self::Models,
            Self::Verification,
            Self::Storage,
            Self::Settlement,
            Self::Discovery,
            Self::Governance,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureLayerAssessment {
    pub largest_operator_share_bps: u16,
    pub largest_beneficial_owner_share_bps: u16,
    pub provider_concentration_bps: Option<u16>,
    pub region_concentration_bps: Option<u16>,
    pub software_family_concentration_bps: Option<u16>,
    pub credential_issuer_concentration_bps: Option<u16>,
    pub frontend_concentration_bps: Option<u16>,
    pub independent_control_domains: u16,
    pub minimum_censorship_coalition: u16,
    pub minimum_corruption_coalition: u16,
    pub evidence_root: String,
}

impl CaptureLayerAssessment {
    pub fn independence_score_bps(&self) -> Result<u16, CaptureError> {
        self.validate()?;
        let concentration_score = 10_000u16
            .checked_sub(self.largest_beneficial_owner_share_bps)
            .ok_or(CaptureError::InvalidAssessment)?;
        let censorship_score = coalition_score(
            self.minimum_censorship_coalition,
            self.independent_control_domains,
        );
        let corruption_score = coalition_score(
            self.minimum_corruption_coalition,
            self.independent_control_domains,
        );
        Ok(concentration_score
            .min(censorship_score)
            .min(corruption_score))
    }

    pub fn validate(&self) -> Result<(), CaptureError> {
        let optional_concentrations = [
            self.provider_concentration_bps,
            self.region_concentration_bps,
            self.software_family_concentration_bps,
            self.credential_issuer_concentration_bps,
            self.frontend_concentration_bps,
        ];
        if self.largest_operator_share_bps > 10_000
            || self.largest_beneficial_owner_share_bps > 10_000
            || self.largest_beneficial_owner_share_bps < self.largest_operator_share_bps
            || optional_concentrations
                .into_iter()
                .flatten()
                .any(|score| score > 10_000)
            || self.independent_control_domains == 0
            || self.minimum_censorship_coalition == 0
            || self.minimum_corruption_coalition == 0
            || self.minimum_censorship_coalition > self.independent_control_domains
            || self.minimum_corruption_coalition > self.independent_control_domains
            || self.evidence_root.trim().is_empty()
        {
            return Err(CaptureError::InvalidAssessment);
        }
        Ok(())
    }
}

fn coalition_score(coalition: u16, domains: u16) -> u16 {
    (u32::from(coalition) * 10_000 / u32::from(domains)) as u16
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureResistanceDashboard {
    pub dashboard_id: CaptureDashboardId,
    pub policy_id: PolicyId,
    pub layers: BTreeMap<CaptureLayer, CaptureLayerAssessment>,
    pub observed_at: DateTime<Utc>,
    pub evidence_root: String,
    pub assessor_signatures: Vec<String>,
}

#[derive(Serialize)]
struct DashboardIdentity<'a> {
    policy_id: &'a PolicyId,
    layers: &'a BTreeMap<CaptureLayer, CaptureLayerAssessment>,
    observed_at: DateTime<Utc>,
    evidence_root: &'a str,
}

impl CaptureResistanceDashboard {
    pub fn derive_dashboard_id(&self) -> Result<CaptureDashboardId, IdError> {
        CaptureDashboardId::derive(&DashboardIdentity {
            policy_id: &self.policy_id,
            layers: &self.layers,
            observed_at: self.observed_at,
            evidence_root: &self.evidence_root,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), CaptureError> {
        self.dashboard_id.validate()?;
        self.policy_id.validate()?;
        if self.dashboard_id != self.derive_dashboard_id()? {
            return Err(CaptureError::IdentityMismatch);
        }
        if self.layers.len() != CaptureLayer::all().len()
            || self.evidence_root.trim().is_empty()
            || self.assessor_signatures.is_empty()
            || self
                .assessor_signatures
                .iter()
                .any(|signature| signature.trim().is_empty())
        {
            return Err(CaptureError::MissingLayerOrEvidence);
        }
        for layer in CaptureLayer::all() {
            self.layers
                .get(&layer)
                .ok_or(CaptureError::MissingLayerOrEvidence)?
                .validate()?;
        }
        Ok(())
    }

    /// Effective decentralization is constrained by the weakest layer.
    pub fn effective_independence_bps(&self) -> Result<u16, CaptureError> {
        self.validate_integrity()?;
        self.layers
            .values()
            .map(CaptureLayerAssessment::independence_score_bps)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .min()
            .ok_or(CaptureError::MissingLayerOrEvidence)
    }

    pub fn limiting_layers(&self) -> Result<BTreeSet<CaptureLayer>, CaptureError> {
        let effective = self.effective_independence_bps()?;
        self.layers
            .iter()
            .filter_map(|(layer, assessment)| {
                assessment
                    .independence_score_bps()
                    .ok()
                    .filter(|score| *score == effective)
                    .map(|_| *layer)
            })
            .collect::<BTreeSet<_>>()
            .pipe(Ok)
    }
}

trait Pipe: Sized {
    fn pipe<T>(self, function: impl FnOnce(Self) -> T) -> T {
        function(self)
    }
}
impl<T> Pipe for T {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeRevenueKind {
    Execution,
    ReservedCapacity,
    Availability,
    Specialization,
    SuccessfulChallenge,
    Maintenance,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeWorkReceipt {
    pub receipt_id: NodeWorkReceiptId,
    pub node_id: NodeId,
    pub operator_id: OperatorId,
    pub job_id: Option<JobId>,
    pub service: NodeServiceKind,
    pub revenue_kind: NodeRevenueKind,
    pub settled_amount: Amount,
    pub settlement_receipt_id: ReceiptId,
    pub work_evidence_root: String,
    pub external_value_evidence_root: String,
    pub completed_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Serialize)]
struct NodeWorkIdentity<'a> {
    node_id: &'a NodeId,
    operator_id: &'a OperatorId,
    job_id: &'a Option<JobId>,
    service: NodeServiceKind,
    revenue_kind: NodeRevenueKind,
    settled_amount: &'a Amount,
    settlement_receipt_id: &'a ReceiptId,
    work_evidence_root: &'a str,
    external_value_evidence_root: &'a str,
    completed_at: DateTime<Utc>,
}

impl NodeWorkReceipt {
    pub fn derive_receipt_id(&self) -> Result<NodeWorkReceiptId, IdError> {
        NodeWorkReceiptId::derive(&NodeWorkIdentity {
            node_id: &self.node_id,
            operator_id: &self.operator_id,
            job_id: &self.job_id,
            service: self.service,
            revenue_kind: self.revenue_kind,
            settled_amount: &self.settled_amount,
            settlement_receipt_id: &self.settlement_receipt_id,
            work_evidence_root: &self.work_evidence_root,
            external_value_evidence_root: &self.external_value_evidence_root,
            completed_at: self.completed_at,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), CaptureError> {
        self.receipt_id.validate()?;
        self.node_id.validate()?;
        self.operator_id.validate()?;
        self.settlement_receipt_id.validate()?;
        if let Some(job_id) = &self.job_id {
            job_id.validate()?;
        }
        if self.receipt_id != self.derive_receipt_id()? {
            return Err(CaptureError::IdentityMismatch);
        }
        if self.settled_amount.units == 0
            || self.settled_amount.asset.trim().is_empty()
            || self.work_evidence_root.trim().is_empty()
            || self.external_value_evidence_root.trim().is_empty()
            || self.signature.trim().is_empty()
        {
            return Err(CaptureError::UnbackedNodeRevenue);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeExposureLimit {
    pub bond_id: BondId,
    pub bond_amount: Amount,
    /// Coverage multiplier where 10_000 bps means exposure cannot exceed bond.
    pub coverage_multiplier_bps: u16,
    pub maximum_certificate_exposure: Amount,
    pub policy_id: PolicyId,
}

impl NodeExposureLimit {
    pub fn is_covered(&self) -> bool {
        if self.bond_id.validate().is_err()
            || self.policy_id.validate().is_err()
            || self.coverage_multiplier_bps == 0
            || self.bond_amount.units == 0
            || self
                .bond_amount
                .ensure_compatible(&self.maximum_certificate_exposure)
                .is_err()
        {
            return false;
        }
        self.bond_amount
            .mul_bps(self.coverage_multiplier_bps)
            .is_ok_and(|coverage| self.maximum_certificate_exposure.units <= coverage.units)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveMisconductKind {
    SignedUncheckedArtifact,
    Equivocation,
    FalsifiedExecutionEvidence,
    ConcealedBeneficialControl,
    FailedPromisedRetention,
    UnauthorizedArtifactSubstitution,
    ReceiptForgery,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveMisconductRecord {
    pub record_id: MisconductRecordId,
    pub node_id: NodeId,
    pub operator_id: OperatorId,
    pub bond_id: BondId,
    pub kind: ObjectiveMisconductKind,
    pub evidence_roots: BTreeSet<String>,
    pub adjudication_policy_id: PolicyId,
    pub slash_amount: Amount,
    pub observed_at: DateTime<Utc>,
    pub signatures: Vec<String>,
}

#[derive(Serialize)]
struct MisconductIdentity<'a> {
    node_id: &'a NodeId,
    operator_id: &'a OperatorId,
    bond_id: &'a BondId,
    kind: ObjectiveMisconductKind,
    evidence_roots: &'a BTreeSet<String>,
    adjudication_policy_id: &'a PolicyId,
    slash_amount: &'a Amount,
    observed_at: DateTime<Utc>,
}

impl ObjectiveMisconductRecord {
    pub fn derive_record_id(&self) -> Result<MisconductRecordId, IdError> {
        MisconductRecordId::derive(&MisconductIdentity {
            node_id: &self.node_id,
            operator_id: &self.operator_id,
            bond_id: &self.bond_id,
            kind: self.kind,
            evidence_roots: &self.evidence_roots,
            adjudication_policy_id: &self.adjudication_policy_id,
            slash_amount: &self.slash_amount,
            observed_at: self.observed_at,
        })
    }

    pub fn validate(&self, active_bond: &Amount) -> Result<(), CaptureError> {
        self.record_id.validate()?;
        self.node_id.validate()?;
        self.operator_id.validate()?;
        self.bond_id.validate()?;
        self.adjudication_policy_id.validate()?;
        if self.record_id != self.derive_record_id()? {
            return Err(CaptureError::IdentityMismatch);
        }
        active_bond.ensure_compatible(&self.slash_amount)?;
        if self.slash_amount.units == 0
            || self.slash_amount.units > active_bond.units
            || self.evidence_roots.is_empty()
            || self
                .evidence_roots
                .iter()
                .any(|root| root.trim().is_empty())
            || self.signatures.is_empty()
            || self.signatures.iter().any(|value| value.trim().is_empty())
        {
            return Err(CaptureError::UnprovenMisconduct);
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    Money(#[from] crate::MoneyError),
    #[error("content-derived identity does not match the record")]
    IdentityMismatch,
    #[error("an append-only record cannot supersede itself")]
    SelfSupersession,
    #[error("cooperative ownership must contain multiple members and total 10,000 bps")]
    InvalidCooperativeOwnership,
    #[error("required control, capability, or signature evidence is missing")]
    MissingEvidence,
    #[error("capture-layer assessment is outside valid bounds")]
    InvalidAssessment,
    #[error("capture dashboard is missing a layer, evidence, or assessor signature")]
    MissingLayerOrEvidence,
    #[error("node revenue lacks completed-work and externally settled value evidence")]
    UnbackedNodeRevenue,
    #[error("objective misconduct lacks bounded, independently reviewable evidence")]
    UnprovenMisconduct,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cooperative(name: &str, members: &[(&str, u16)]) -> ResearchComputeCooperative {
        let mut cooperative = ResearchComputeCooperative {
            cooperative_id: ComputeCooperativeId::derive(&"placeholder").unwrap(),
            members: members
                .iter()
                .map(|(member, share)| (VerifiedUserId::derive(member).unwrap(), *share))
                .collect(),
            operator_cluster_id: OperatorClusterId::derive(&name).unwrap(),
            node_ids: BTreeSet::from([NodeId::derive(&name).unwrap()]),
            capability_classes: BTreeSet::from(["formal_verification".into()]),
            shared_treasury: format!("vault:{name}"),
            governance_policy_id: PolicyId::derive(&name).unwrap(),
            beneficial_control_evidence_root: format!("blake3:{name}-owners"),
            created_at: Utc::now(),
            supersedes: None,
            signatures: vec!["member-one".into(), "member-two".into()],
        };
        cooperative.cooperative_id = cooperative.derive_cooperative_id().unwrap();
        cooperative
    }

    fn assessment(beneficial_owner_share_bps: u16) -> CaptureLayerAssessment {
        CaptureLayerAssessment {
            largest_operator_share_bps: beneficial_owner_share_bps / 2,
            largest_beneficial_owner_share_bps: beneficial_owner_share_bps,
            provider_concentration_bps: Some(3_000),
            region_concentration_bps: Some(4_000),
            software_family_concentration_bps: Some(5_000),
            credential_issuer_concentration_bps: None,
            frontend_concentration_bps: None,
            independent_control_domains: 10,
            minimum_censorship_coalition: 4,
            minimum_corruption_coalition: 5,
            evidence_root: "blake3:assessment".into(),
        }
    }

    #[test]
    fn shared_cooperative_owners_reduce_independence_credit() {
        let left = cooperative("left", &[("alice", 6_000), ("bob", 4_000)]);
        let right = cooperative("right", &[("alice", 2_500), ("carol", 7_500)]);
        assert_eq!(
            cooperative_ownership_overlap_bps(&left, &right).unwrap(),
            2_500
        );
        assert_eq!(
            cooperative_independence_credit_bps(&left, &right).unwrap(),
            7_500
        );
    }

    #[test]
    fn effective_decentralization_is_the_weakest_layer() {
        let mut layers = CaptureLayer::all()
            .into_iter()
            .map(|layer| (layer, assessment(2_000)))
            .collect::<BTreeMap<_, _>>();
        layers.insert(CaptureLayer::Models, assessment(8_500));
        let mut dashboard = CaptureResistanceDashboard {
            dashboard_id: CaptureDashboardId::derive(&"placeholder").unwrap(),
            policy_id: PolicyId::derive(&"capture-policy").unwrap(),
            layers,
            observed_at: Utc::now(),
            evidence_root: "blake3:dashboard".into(),
            assessor_signatures: vec!["assessor".into()],
        };
        dashboard.dashboard_id = dashboard.derive_dashboard_id().unwrap();
        assert_eq!(dashboard.effective_independence_bps().unwrap(), 1_500);
        assert_eq!(
            dashboard.limiting_layers().unwrap(),
            BTreeSet::from([CaptureLayer::Models])
        );
    }

    #[test]
    fn node_exposure_cannot_exceed_bond_coverage() {
        let limit = NodeExposureLimit {
            bond_id: BondId::derive(&"bond").unwrap(),
            bond_amount: Amount::new(100, "USDC", 6),
            coverage_multiplier_bps: 15_000,
            maximum_certificate_exposure: Amount::new(151, "USDC", 6),
            policy_id: PolicyId::derive(&"coverage").unwrap(),
        };
        assert!(!limit.is_covered());
    }

    #[test]
    fn honest_divergence_is_not_a_slashable_offense() {
        let encoded = serde_json::to_string(&[
            ObjectiveMisconductKind::SignedUncheckedArtifact,
            ObjectiveMisconductKind::Equivocation,
            ObjectiveMisconductKind::FalsifiedExecutionEvidence,
            ObjectiveMisconductKind::ConcealedBeneficialControl,
            ObjectiveMisconductKind::FailedPromisedRetention,
            ObjectiveMisconductKind::UnauthorizedArtifactSubstitution,
            ObjectiveMisconductKind::ReceiptForgery,
        ])
        .unwrap();
        assert!(!encoded.contains("divergence"));
        assert!(!encoded.contains("fail_verdict"));
    }
}

//! Researcher sovereignty, portable exit, economic-graph, and verification-profile records.
//!
//! These records keep origin and research custody durable without turning
//! mathematical truth or descriptive dependency edges into private property.

use crate::{
    Amount, CapsuleEconomicMode, ClaimId, EconomicComplianceCertificateId, IdError,
    OperatorClusterId, PolicyId, PortabilityManifestId, ReceiptId, ResearcherId, ResidualRightId,
    RevenueEventId, SovereigntyBundleId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SovereigntyRightKind {
    Origin,
    Attribution,
    ArtifactControl,
    EconomicParticipation,
    LicenseControl,
    GovernanceConsent,
    PortabilityExit,
}

impl SovereigntyRightKind {
    fn all() -> [Self; 7] {
        [
            Self::Origin,
            Self::Attribution,
            Self::ArtifactControl,
            Self::EconomicParticipation,
            Self::LicenseControl,
            Self::GovernanceConsent,
            Self::PortabilityExit,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SovereigntyRightRecord {
    pub evidence_root: String,
    pub transferable: bool,
    pub revocable_by_protocol: bool,
    pub challengeable: bool,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearcherSovereigntyBundle {
    pub bundle_id: SovereigntyBundleId,
    pub researcher_id: ResearcherId,
    pub claim_id: ClaimId,
    pub origin_receipt_id: ReceiptId,
    /// Credential commitments link origin to an accountable holder while
    /// permitting selective disclosure of private identity attributes.
    pub holder_credential_roots: BTreeSet<String>,
    pub rights: BTreeMap<SovereigntyRightKind, SovereigntyRightRecord>,
    pub controlled_artifact_roots: BTreeSet<String>,
    pub contribution_manifest_root: String,
    pub rights_manifest_root: String,
    pub economic_policy_root: String,
    pub direct_custody_vaults: BTreeSet<String>,
    pub portability_manifest_id: PortabilityManifestId,
    pub supersedes: Option<SovereigntyBundleId>,
    pub issued_at: DateTime<Utc>,
    pub signatures: Vec<String>,
}

#[derive(Serialize)]
struct SovereigntyBundleIdentity<'a> {
    researcher_id: &'a ResearcherId,
    claim_id: &'a ClaimId,
    origin_receipt_id: &'a ReceiptId,
    holder_credential_roots: &'a BTreeSet<String>,
    rights: &'a BTreeMap<SovereigntyRightKind, SovereigntyRightRecord>,
    controlled_artifact_roots: &'a BTreeSet<String>,
    contribution_manifest_root: &'a str,
    rights_manifest_root: &'a str,
    economic_policy_root: &'a str,
    direct_custody_vaults: &'a BTreeSet<String>,
    portability_manifest_id: &'a PortabilityManifestId,
    supersedes: &'a Option<SovereigntyBundleId>,
    issued_at: DateTime<Utc>,
}

impl ResearcherSovereigntyBundle {
    pub fn derive_bundle_id(&self) -> Result<SovereigntyBundleId, IdError> {
        SovereigntyBundleId::derive(&SovereigntyBundleIdentity {
            researcher_id: &self.researcher_id,
            claim_id: &self.claim_id,
            origin_receipt_id: &self.origin_receipt_id,
            holder_credential_roots: &self.holder_credential_roots,
            rights: &self.rights,
            controlled_artifact_roots: &self.controlled_artifact_roots,
            contribution_manifest_root: &self.contribution_manifest_root,
            rights_manifest_root: &self.rights_manifest_root,
            economic_policy_root: &self.economic_policy_root,
            direct_custody_vaults: &self.direct_custody_vaults,
            portability_manifest_id: &self.portability_manifest_id,
            supersedes: &self.supersedes,
            issued_at: self.issued_at,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), SovereigntyError> {
        self.bundle_id.validate()?;
        self.researcher_id.validate()?;
        self.claim_id.validate()?;
        self.origin_receipt_id.validate()?;
        self.portability_manifest_id.validate()?;
        if self.bundle_id != self.derive_bundle_id()? {
            return Err(SovereigntyError::IdentityMismatch);
        }
        if self.supersedes.as_ref() == Some(&self.bundle_id) {
            return Err(SovereigntyError::SelfSupersession);
        }
        for right in SovereigntyRightKind::all() {
            let record = self
                .rights
                .get(&right)
                .ok_or(SovereigntyError::MissingRight(right))?;
            require_root(&record.evidence_root, "sovereignty right evidence")?;
        }
        if self.rights.len() != SovereigntyRightKind::all().len() {
            return Err(SovereigntyError::UnexpectedRightSet);
        }

        for durable in [
            SovereigntyRightKind::Origin,
            SovereigntyRightKind::Attribution,
            SovereigntyRightKind::PortabilityExit,
        ] {
            let record = &self.rights[&durable];
            if record.transferable || record.revocable_by_protocol {
                return Err(SovereigntyError::DurableRightCanBeTaken(durable));
            }
        }
        if !self.rights[&SovereigntyRightKind::Origin].challengeable {
            return Err(SovereigntyError::OriginMustBeChallengeable);
        }
        require_nonempty_roots(&self.holder_credential_roots, "holder credentials")?;
        require_nonempty_roots(&self.controlled_artifact_roots, "controlled artifacts")?;
        require_nonempty_roots(&self.direct_custody_vaults, "direct-custody vaults")?;
        require_root(&self.contribution_manifest_root, "contribution manifest")?;
        require_root(&self.rights_manifest_root, "rights manifest")?;
        require_root(&self.economic_policy_root, "economic policy")?;
        if self.signatures.is_empty() || self.signatures.iter().any(|value| value.trim().is_empty())
        {
            return Err(SovereigntyError::MissingEvidence("bundle signatures"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearcherPortabilityManifest {
    pub manifest_id: PortabilityManifestId,
    pub researcher_id: ResearcherId,
    pub protocol_version: String,
    pub identity_credential_roots: BTreeSet<String>,
    pub artifact_roots: BTreeSet<String>,
    pub contribution_manifest_roots: BTreeSet<String>,
    pub verification_receipt_roots: BTreeSet<String>,
    pub economic_policy_roots: BTreeSet<String>,
    pub settlement_commitments: BTreeSet<String>,
    pub event_log_checkpoints: BTreeSet<String>,
    /// Every artifact root maps to independent, reconstructable providers.
    pub storage_locations: BTreeMap<String, BTreeSet<PortableStorageLocation>>,
    pub event_log_locations: BTreeSet<PortableStorageLocation>,
    pub reconstruction_client: String,
    pub reconstruction_client_source_root: String,
    pub funds_exit_instructions_root: String,
    pub created_at: DateTime<Utc>,
    pub supersedes: Option<PortabilityManifestId>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PortableStorageLocation {
    pub provider_id: String,
    pub location: String,
    pub retrieval_evidence_root: String,
}

#[derive(Serialize)]
struct PortabilityIdentity<'a> {
    researcher_id: &'a ResearcherId,
    protocol_version: &'a str,
    identity_credential_roots: &'a BTreeSet<String>,
    artifact_roots: &'a BTreeSet<String>,
    contribution_manifest_roots: &'a BTreeSet<String>,
    verification_receipt_roots: &'a BTreeSet<String>,
    economic_policy_roots: &'a BTreeSet<String>,
    settlement_commitments: &'a BTreeSet<String>,
    event_log_checkpoints: &'a BTreeSet<String>,
    storage_locations: &'a BTreeMap<String, BTreeSet<PortableStorageLocation>>,
    event_log_locations: &'a BTreeSet<PortableStorageLocation>,
    reconstruction_client: &'a str,
    reconstruction_client_source_root: &'a str,
    funds_exit_instructions_root: &'a str,
    created_at: DateTime<Utc>,
    supersedes: &'a Option<PortabilityManifestId>,
}

impl ResearcherPortabilityManifest {
    pub fn derive_manifest_id(&self) -> Result<PortabilityManifestId, IdError> {
        PortabilityManifestId::derive(&PortabilityIdentity {
            researcher_id: &self.researcher_id,
            protocol_version: &self.protocol_version,
            identity_credential_roots: &self.identity_credential_roots,
            artifact_roots: &self.artifact_roots,
            contribution_manifest_roots: &self.contribution_manifest_roots,
            verification_receipt_roots: &self.verification_receipt_roots,
            economic_policy_roots: &self.economic_policy_roots,
            settlement_commitments: &self.settlement_commitments,
            event_log_checkpoints: &self.event_log_checkpoints,
            storage_locations: &self.storage_locations,
            event_log_locations: &self.event_log_locations,
            reconstruction_client: &self.reconstruction_client,
            reconstruction_client_source_root: &self.reconstruction_client_source_root,
            funds_exit_instructions_root: &self.funds_exit_instructions_root,
            created_at: self.created_at,
            supersedes: &self.supersedes,
        })
    }

    pub fn validate_reconstructable(&self) -> Result<(), SovereigntyError> {
        self.manifest_id.validate()?;
        self.researcher_id.validate()?;
        if self.manifest_id != self.derive_manifest_id()? {
            return Err(SovereigntyError::IdentityMismatch);
        }
        if self.supersedes.as_ref() == Some(&self.manifest_id) {
            return Err(SovereigntyError::SelfSupersession);
        }
        if self.protocol_version != crate::XLMP_VERSION
            || self.reconstruction_client.trim().is_empty()
            || self.reconstruction_client_source_root.trim().is_empty()
            || self.funds_exit_instructions_root.trim().is_empty()
            || self.signature.trim().is_empty()
        {
            return Err(SovereigntyError::MissingEvidence("portability metadata"));
        }
        for (roots, label) in [
            (&self.identity_credential_roots, "identity credentials"),
            (&self.artifact_roots, "artifacts"),
            (&self.contribution_manifest_roots, "contribution manifests"),
            (&self.verification_receipt_roots, "verification receipts"),
            (&self.economic_policy_roots, "economic policies"),
            (&self.settlement_commitments, "settlement commitments"),
            (&self.event_log_checkpoints, "event-log checkpoints"),
        ] {
            require_nonempty_roots(roots, label)?;
        }
        if self.storage_locations.keys().collect::<BTreeSet<_>>()
            != self.artifact_roots.iter().collect::<BTreeSet<_>>()
        {
            return Err(SovereigntyError::StorageMapMismatch);
        }
        if self
            .storage_locations
            .values()
            .any(|locations| !portable_locations_are_independent(locations))
            || !portable_locations_are_independent(&self.event_log_locations)
        {
            return Err(SovereigntyError::InsufficientStorageReplication);
        }
        Ok(())
    }
}

fn portable_locations_are_independent(locations: &BTreeSet<PortableStorageLocation>) -> bool {
    if locations.len() < 2
        || locations.iter().any(|location| {
            location.provider_id.trim().is_empty()
                || location.location.trim().is_empty()
                || location.retrieval_evidence_root.trim().is_empty()
        })
    {
        return false;
    }
    locations
        .iter()
        .map(|location| location.provider_id.as_str())
        .collect::<BTreeSet<_>>()
        .len()
        >= 2
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidualRightAssignment {
    pub from_beneficiary: String,
    pub to_beneficiary: String,
    pub signed_agreement_root: String,
    pub effective_at: DateTime<Utc>,
    pub signatures: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearcherResidualRight {
    pub right_id: ResidualRightId,
    pub origin_researcher_id: ResearcherId,
    pub claim_id: ClaimId,
    pub current_beneficiary: String,
    pub payee_vault: String,
    pub economic_policy_root: String,
    pub qualifying_revenue_sources: BTreeSet<String>,
    pub share_bps: u16,
    pub per_event_cap: Amount,
    pub lifetime_cap: Amount,
    pub max_dependency_depth: u16,
    pub depth_decay_bps: u16,
    pub per_ancestor_cap_bps: u16,
    pub nonexclusive: bool,
    pub no_downstream_veto: bool,
    pub non_recursive: bool,
    pub single_charge_per_revenue_event: bool,
    pub equivalent_claim_clustering: bool,
    pub assignment: Option<ResidualRightAssignment>,
    pub issued_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Serialize)]
struct ResidualRightIdentity<'a> {
    origin_researcher_id: &'a ResearcherId,
    claim_id: &'a ClaimId,
    current_beneficiary: &'a str,
    payee_vault: &'a str,
    economic_policy_root: &'a str,
    qualifying_revenue_sources: &'a BTreeSet<String>,
    share_bps: u16,
    per_event_cap: &'a Amount,
    lifetime_cap: &'a Amount,
    max_dependency_depth: u16,
    depth_decay_bps: u16,
    per_ancestor_cap_bps: u16,
    nonexclusive: bool,
    no_downstream_veto: bool,
    non_recursive: bool,
    single_charge_per_revenue_event: bool,
    equivalent_claim_clustering: bool,
    assignment: &'a Option<ResidualRightAssignment>,
    issued_at: DateTime<Utc>,
}

impl ResearcherResidualRight {
    pub fn derive_right_id(&self) -> Result<ResidualRightId, IdError> {
        ResidualRightId::derive(&ResidualRightIdentity {
            origin_researcher_id: &self.origin_researcher_id,
            claim_id: &self.claim_id,
            current_beneficiary: &self.current_beneficiary,
            payee_vault: &self.payee_vault,
            economic_policy_root: &self.economic_policy_root,
            qualifying_revenue_sources: &self.qualifying_revenue_sources,
            share_bps: self.share_bps,
            per_event_cap: &self.per_event_cap,
            lifetime_cap: &self.lifetime_cap,
            max_dependency_depth: self.max_dependency_depth,
            depth_decay_bps: self.depth_decay_bps,
            per_ancestor_cap_bps: self.per_ancestor_cap_bps,
            nonexclusive: self.nonexclusive,
            no_downstream_veto: self.no_downstream_veto,
            non_recursive: self.non_recursive,
            single_charge_per_revenue_event: self.single_charge_per_revenue_event,
            equivalent_claim_clustering: self.equivalent_claim_clustering,
            assignment: &self.assignment,
            issued_at: self.issued_at,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), SovereigntyError> {
        self.right_id.validate()?;
        self.origin_researcher_id.validate()?;
        self.claim_id.validate()?;
        if self.right_id != self.derive_right_id()? {
            return Err(SovereigntyError::IdentityMismatch);
        }
        if !self.nonexclusive
            || !self.no_downstream_veto
            || !self.non_recursive
            || !self.single_charge_per_revenue_event
            || !self.equivalent_claim_clustering
        {
            return Err(SovereigntyError::ResidualRightCreatesControlOrRecursion);
        }
        if self.share_bps == 0
            || self.share_bps > 10_000
            || self.per_ancestor_cap_bps == 0
            || self.per_ancestor_cap_bps > self.share_bps
            || self.depth_decay_bps > 10_000
            || self.max_dependency_depth == 0
        {
            return Err(SovereigntyError::UnboundedEconomicTerms);
        }
        self.per_event_cap.ensure_compatible(&self.lifetime_cap)?;
        if self.per_event_cap.units == 0
            || self.per_event_cap.units > self.lifetime_cap.units
            || self.per_event_cap.asset.trim().is_empty()
        {
            return Err(SovereigntyError::UnboundedEconomicTerms);
        }
        require_root(&self.economic_policy_root, "economic policy")?;
        require_nonempty_roots(
            &self.qualifying_revenue_sources,
            "qualifying revenue sources",
        )?;
        if self.current_beneficiary.trim().is_empty()
            || self.payee_vault.trim().is_empty()
            || self.signature.trim().is_empty()
        {
            return Err(SovereigntyError::MissingEvidence("residual-right custody"));
        }
        match &self.assignment {
            None if self.current_beneficiary != self.origin_researcher_id.to_string() => {
                Err(SovereigntyError::UnsignedAssignment)
            }
            Some(assignment)
                if assignment.from_beneficiary.trim().is_empty()
                    || assignment.to_beneficiary != self.current_beneficiary
                    || assignment.signed_agreement_root.trim().is_empty()
                    || assignment.signatures.len() < 2
                    || assignment
                        .signatures
                        .iter()
                        .any(|signature| signature.trim().is_empty()) =>
            {
                Err(SovereigntyError::UnsignedAssignment)
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomicConstitution {
    pub policy_id: PolicyId,
    pub mode: CapsuleEconomicMode,
    pub qualifying_revenue_sources: BTreeSet<String>,
    pub upstream_pool_bps: u16,
    pub per_ancestor_cap_bps: u16,
    pub max_ancestors: u16,
    pub max_dependency_depth: u16,
    pub depth_decay_bps: u16,
    /// Asset-minor-unit floor applied after proportional weighting. Values
    /// below this floor remain in the unallocated remainder.
    pub minimum_payout_units: u128,
    pub bare_citations_eligible: bool,
    pub externally_independent_claims_eligible: bool,
    pub downstream_veto: bool,
    pub recursive_charging: bool,
    pub single_charge_per_revenue_event: bool,
    pub equivalent_claim_clustering: bool,
    pub policy_root: String,
    pub signatures: Vec<String>,
}

impl EconomicConstitution {
    pub fn validate(&self) -> Result<(), SovereigntyError> {
        self.policy_id.validate()?;
        require_root(&self.policy_root, "economic constitution")?;
        if self.signatures.is_empty() || self.signatures.iter().any(|value| value.trim().is_empty())
        {
            return Err(SovereigntyError::MissingEvidence(
                "economic constitution signatures",
            ));
        }
        if self.upstream_pool_bps > 10_000
            || self.per_ancestor_cap_bps > self.upstream_pool_bps
            || self.depth_decay_bps > 10_000
            || (self.upstream_pool_bps > 0 && self.minimum_payout_units == 0)
            || self.bare_citations_eligible
            || self.externally_independent_claims_eligible
            || self.downstream_veto
            || self.recursive_charging
            || !self.single_charge_per_revenue_event
            || !self.equivalent_claim_clustering
        {
            return Err(SovereigntyError::UnboundedEconomicTerms);
        }
        match self.mode {
            CapsuleEconomicMode::Commons if self.upstream_pool_bps != 0 => {
                Err(SovereigntyError::CommonsHasMandatoryPayment)
            }
            CapsuleEconomicMode::Reciprocal
                if self.upstream_pool_bps == 0
                    || self.per_ancestor_cap_bps == 0
                    || self.max_ancestors == 0
                    || self.max_dependency_depth == 0
                    || self.minimum_payout_units == 0
                    || self.qualifying_revenue_sources.is_empty() =>
            {
                Err(SovereigntyError::UnboundedEconomicTerms)
            }
            CapsuleEconomicMode::CommercialArtifact | CapsuleEconomicMode::SponsoredChallenge
                if self.qualifying_revenue_sources.is_empty() =>
            {
                Err(SovereigntyError::MissingEvidence(
                    "qualifying revenue sources",
                ))
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EconomicComplianceStatus {
    Compliant,
    Noncompliant,
    Disputed,
}

/// Certifies performance of explicit protocol economic obligations. It does
/// not certify a proof, research claim, origin record, or legal right.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomicComplianceCertificate {
    pub certificate_id: EconomicComplianceCertificateId,
    pub claim_id: ClaimId,
    pub economic_policy_id: PolicyId,
    pub required_obligation_roots: BTreeSet<String>,
    pub satisfied_obligation_roots: BTreeSet<String>,
    pub revenue_event_ids: BTreeSet<RevenueEventId>,
    pub settlement_receipt_ids: BTreeSet<ReceiptId>,
    pub status: EconomicComplianceStatus,
    pub evaluation_evidence_root: String,
    pub evaluator_operator_cluster_id: OperatorClusterId,
    pub evaluated_at: DateTime<Utc>,
    pub supersedes: Option<EconomicComplianceCertificateId>,
    pub signature: String,
}

#[derive(Serialize)]
struct EconomicComplianceIdentity<'a> {
    claim_id: &'a ClaimId,
    economic_policy_id: &'a PolicyId,
    required_obligation_roots: &'a BTreeSet<String>,
    satisfied_obligation_roots: &'a BTreeSet<String>,
    revenue_event_ids: &'a BTreeSet<RevenueEventId>,
    settlement_receipt_ids: &'a BTreeSet<ReceiptId>,
    status: EconomicComplianceStatus,
    evaluation_evidence_root: &'a str,
    evaluator_operator_cluster_id: &'a OperatorClusterId,
    evaluated_at: DateTime<Utc>,
    supersedes: &'a Option<EconomicComplianceCertificateId>,
}

impl EconomicComplianceCertificate {
    pub fn derive_certificate_id(&self) -> Result<EconomicComplianceCertificateId, IdError> {
        EconomicComplianceCertificateId::derive(&EconomicComplianceIdentity {
            claim_id: &self.claim_id,
            economic_policy_id: &self.economic_policy_id,
            required_obligation_roots: &self.required_obligation_roots,
            satisfied_obligation_roots: &self.satisfied_obligation_roots,
            revenue_event_ids: &self.revenue_event_ids,
            settlement_receipt_ids: &self.settlement_receipt_ids,
            status: self.status,
            evaluation_evidence_root: &self.evaluation_evidence_root,
            evaluator_operator_cluster_id: &self.evaluator_operator_cluster_id,
            evaluated_at: self.evaluated_at,
            supersedes: &self.supersedes,
        })
    }

    pub fn validate_against(
        &self,
        constitution: &EconomicConstitution,
    ) -> Result<(), SovereigntyError> {
        constitution.validate()?;
        self.certificate_id.validate()?;
        self.claim_id.validate()?;
        self.economic_policy_id.validate()?;
        self.evaluator_operator_cluster_id.validate()?;
        if let Some(previous) = &self.supersedes {
            previous.validate()?;
        }
        for event_id in &self.revenue_event_ids {
            event_id.validate()?;
        }
        for receipt_id in &self.settlement_receipt_ids {
            receipt_id.validate()?;
        }
        if self.certificate_id != self.derive_certificate_id()? {
            return Err(SovereigntyError::IdentityMismatch);
        }
        if self.economic_policy_id != constitution.policy_id
            || self.supersedes.as_ref() == Some(&self.certificate_id)
            || self.evaluation_evidence_root.trim().is_empty()
            || self.signature.trim().is_empty()
            || self
                .required_obligation_roots
                .iter()
                .chain(self.satisfied_obligation_roots.iter())
                .any(|root| root.trim().is_empty())
            || !self
                .satisfied_obligation_roots
                .is_subset(&self.required_obligation_roots)
        {
            return Err(SovereigntyError::InvalidEconomicCompliance);
        }
        match self.status {
            EconomicComplianceStatus::Compliant => {
                if self.required_obligation_roots != self.satisfied_obligation_roots {
                    return Err(SovereigntyError::InvalidEconomicCompliance);
                }
                if constitution.mode == CapsuleEconomicMode::Commons {
                    if !self.required_obligation_roots.is_empty() {
                        return Err(SovereigntyError::InvalidEconomicCompliance);
                    }
                } else if !self.required_obligation_roots.is_empty()
                    && (self.revenue_event_ids.is_empty() || self.settlement_receipt_ids.is_empty())
                {
                    return Err(SovereigntyError::InvalidEconomicCompliance);
                }
            }
            EconomicComplianceStatus::Noncompliant
                if self.required_obligation_roots == self.satisfied_obligation_roots =>
            {
                return Err(SovereigntyError::InvalidEconomicCompliance);
            }
            EconomicComplianceStatus::Disputed => {}
            EconomicComplianceStatus::Noncompliant => {}
        }
        Ok(())
    }

    pub const fn affects_research_validity(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceEdgeKind {
    FormallyDependsOn,
    Extends,
    UsesDataset,
    UsesLibrary,
    Cites,
    EquivalentTo,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceGraphEdge {
    pub downstream_claim_id: ClaimId,
    pub upstream_claim_id: ClaimId,
    pub kind: EvidenceEdgeKind,
    pub used_in_final_artifact: bool,
    pub evidence_root: String,
}

impl EvidenceGraphEdge {
    /// A descriptive edge is never payment authorization by itself.
    pub const fn authorizes_payment(&self) -> bool {
        false
    }

    pub fn validate(&self) -> Result<(), SovereigntyError> {
        self.downstream_claim_id.validate()?;
        self.upstream_claim_id.validate()?;
        require_root(&self.evidence_root, "evidence graph edge")?;
        if self.downstream_claim_id == self.upstream_claim_id {
            return Err(SovereigntyError::SelfDependency);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EconomicEdgeKind {
    PaysTo,
    ContributesToUpstreamPool,
    AllocatesBountyTo,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomicGraphEdge {
    pub downstream_claim_id: ClaimId,
    pub upstream_claim_id: ClaimId,
    pub kind: EconomicEdgeKind,
    pub policy_id: PolicyId,
    pub qualifying_revenue_source: String,
    pub cap_bps: u16,
    pub authorization_root: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub signatures: Vec<String>,
}

impl EconomicGraphEdge {
    pub fn validate(&self) -> Result<(), SovereigntyError> {
        self.downstream_claim_id.validate()?;
        self.upstream_claim_id.validate()?;
        self.policy_id.validate()?;
        if self.downstream_claim_id == self.upstream_claim_id {
            return Err(SovereigntyError::SelfDependency);
        }
        if self.cap_bps == 0
            || self.cap_bps > 10_000
            || self.qualifying_revenue_source.trim().is_empty()
            || self.authorization_root.trim().is_empty()
            || self.valid_from >= self.valid_until
            || self.signatures.is_empty()
            || self
                .signatures
                .iter()
                .any(|signature| signature.trim().is_empty())
        {
            return Err(SovereigntyError::MissingEconomicAuthorization);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationProfileClass {
    Formal,
    Computational,
    Statistical,
    Simulation,
    Empirical,
    Hybrid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationEvidenceKind {
    FormalStatement,
    ProofObject,
    PinnedToolchain,
    AxiomInventory,
    ReproducibleSource,
    ExecutionEnvironment,
    DependencyLock,
    DeterministicRerun,
    DatasetProvenance,
    AnalysisPlan,
    UncertaintyReport,
    RobustnessChecks,
    ModelDefinition,
    RandomSeeds,
    ParameterRanges,
    ConvergenceAnalysis,
    SensitivityAnalysis,
    ProtocolRegistration,
    InstrumentManifest,
    DataLineage,
    IndependentReplication,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationProfile {
    pub policy_id: PolicyId,
    pub class: VerificationProfileClass,
    pub required_evidence: BTreeSet<VerificationEvidenceKind>,
    pub verifier_implementations: BTreeSet<String>,
    pub minimum_reproductions: u16,
    pub minimum_independent_operators: u16,
    pub challenge_window_seconds: u64,
    pub policy_root: String,
}

impl VerificationProfile {
    pub fn validate(&self) -> Result<(), SovereigntyError> {
        self.policy_id.validate()?;
        require_nonempty_roots(&self.verifier_implementations, "verifier implementations")?;
        require_root(&self.policy_root, "verification profile")?;
        if self.minimum_reproductions < 2
            || self.minimum_independent_operators < 2
            || self.minimum_independent_operators > self.minimum_reproductions
            || self.challenge_window_seconds == 0
        {
            return Err(SovereigntyError::InsufficientIndependentReproduction);
        }
        let required = required_evidence_for(self.class);
        if !required.is_subset(&self.required_evidence) {
            return Err(SovereigntyError::IncompleteVerificationProfile(self.class));
        }
        Ok(())
    }
}

fn required_evidence_for(class: VerificationProfileClass) -> BTreeSet<VerificationEvidenceKind> {
    use VerificationEvidenceKind as E;
    let required: &[E] = match class {
        VerificationProfileClass::Formal => &[
            E::FormalStatement,
            E::ProofObject,
            E::PinnedToolchain,
            E::AxiomInventory,
        ],
        VerificationProfileClass::Computational => &[
            E::ReproducibleSource,
            E::ExecutionEnvironment,
            E::DependencyLock,
            E::DeterministicRerun,
        ],
        VerificationProfileClass::Statistical => &[
            E::DatasetProvenance,
            E::AnalysisPlan,
            E::ReproducibleSource,
            E::UncertaintyReport,
            E::RobustnessChecks,
        ],
        VerificationProfileClass::Simulation => &[
            E::ModelDefinition,
            E::RandomSeeds,
            E::ParameterRanges,
            E::ConvergenceAnalysis,
            E::SensitivityAnalysis,
        ],
        VerificationProfileClass::Empirical => &[
            E::ProtocolRegistration,
            E::InstrumentManifest,
            E::DataLineage,
            E::IndependentReplication,
        ],
        VerificationProfileClass::Hybrid => &[
            E::FormalStatement,
            E::ProofObject,
            E::ReproducibleSource,
            E::ExecutionEnvironment,
            E::DataLineage,
            E::IndependentReplication,
        ],
    };
    required.iter().copied().collect()
}

fn require_root(value: &str, label: &'static str) -> Result<(), SovereigntyError> {
    if value.trim().is_empty() {
        return Err(SovereigntyError::MissingEvidence(label));
    }
    Ok(())
}

fn require_nonempty_roots(
    values: &BTreeSet<String>,
    label: &'static str,
) -> Result<(), SovereigntyError> {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        return Err(SovereigntyError::MissingEvidence(label));
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum SovereigntyError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    Money(#[from] crate::MoneyError),
    #[error("content-derived identity does not match the object")]
    IdentityMismatch,
    #[error("missing required sovereignty right: {0:?}")]
    MissingRight(SovereigntyRightKind),
    #[error("sovereignty bundle contains an unexpected right set")]
    UnexpectedRightSet,
    #[error("durable sovereignty right can be transferred or revoked: {0:?}")]
    DurableRightCanBeTaken(SovereigntyRightKind),
    #[error("origin evidence must remain challengeable")]
    OriginMustBeChallengeable,
    #[error("required evidence is missing: {0}")]
    MissingEvidence(&'static str),
    #[error("an immutable record cannot supersede itself")]
    SelfSupersession,
    #[error("portability storage map does not cover exactly the artifact roots")]
    StorageMapMismatch,
    #[error("portable artifacts require at least two independent storage locations")]
    InsufficientStorageReplication,
    #[error("residual right creates exclusivity, veto power, or recursive charging")]
    ResidualRightCreatesControlOrRecursion,
    #[error("economic terms are zero, uncapped, recursive, or structurally invalid")]
    UnboundedEconomicTerms,
    #[error("residual-right assignment lacks bilateral signed agreement evidence")]
    UnsignedAssignment,
    #[error("Commons mode cannot carry a mandatory upstream payment pool")]
    CommonsHasMandatoryPayment,
    #[error("a descriptive dependency cannot depend on itself")]
    SelfDependency,
    #[error("economic edge lacks explicit bounded authorization")]
    MissingEconomicAuthorization,
    #[error("verification profile does not require independent reproduction")]
    InsufficientIndependentReproduction,
    #[error("verification profile is missing required evidence for {0:?}")]
    IncompleteVerificationProfile(VerificationProfileClass),
    #[error("economic-compliance status is inconsistent with explicit obligations or settlement")]
    InvalidEconomicCompliance,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn claim(name: &str) -> ClaimId {
        ClaimId::from_canonical_elaborated_type(
            &crate::TheoryId::derive(&"sovereignty-theory").unwrap(),
            name,
        )
        .unwrap()
    }

    fn rights() -> BTreeMap<SovereigntyRightKind, SovereigntyRightRecord> {
        SovereigntyRightKind::all()
            .into_iter()
            .map(|kind| {
                (
                    kind,
                    SovereigntyRightRecord {
                        evidence_root: format!("blake3:{kind:?}"),
                        transferable: !matches!(
                            kind,
                            SovereigntyRightKind::Origin
                                | SovereigntyRightKind::Attribution
                                | SovereigntyRightKind::PortabilityExit
                        ),
                        revocable_by_protocol: false,
                        challengeable: kind == SovereigntyRightKind::Origin,
                        limitations: vec![],
                    },
                )
            })
            .collect()
    }

    fn residual_right() -> ResearcherResidualRight {
        let researcher_id = ResearcherId::derive(&"researcher").unwrap();
        let mut right = ResearcherResidualRight {
            right_id: ResidualRightId::derive(&"placeholder").unwrap(),
            origin_researcher_id: researcher_id.clone(),
            claim_id: claim("residual"),
            current_beneficiary: researcher_id.to_string(),
            payee_vault: "vault:researcher".into(),
            economic_policy_root: "blake3:economic-policy".into(),
            qualifying_revenue_sources: set(&["settled_license_revenue"]),
            share_bps: 500,
            per_event_cap: Amount::new(100, "USDC", 6),
            lifetime_cap: Amount::new(10_000, "USDC", 6),
            max_dependency_depth: 4,
            depth_decay_bps: 5_000,
            per_ancestor_cap_bps: 100,
            nonexclusive: true,
            no_downstream_veto: true,
            non_recursive: true,
            single_charge_per_revenue_event: true,
            equivalent_claim_clustering: true,
            assignment: None,
            issued_at: Utc::now(),
            signature: "signature".into(),
        };
        right.right_id = right.derive_right_id().unwrap();
        right
    }

    #[test]
    fn sovereignty_bundle_requires_every_durable_right() {
        let researcher_id = ResearcherId::derive(&"researcher").unwrap();
        let mut bundle = ResearcherSovereigntyBundle {
            bundle_id: SovereigntyBundleId::derive(&"placeholder").unwrap(),
            researcher_id,
            claim_id: claim("bundle"),
            origin_receipt_id: ReceiptId::derive(&"origin").unwrap(),
            holder_credential_roots: set(&["blake3:credential"]),
            rights: rights(),
            controlled_artifact_roots: set(&["blake3:artifact"]),
            contribution_manifest_root: "blake3:contribution".into(),
            rights_manifest_root: "blake3:rights".into(),
            economic_policy_root: "blake3:economic".into(),
            direct_custody_vaults: set(&["vault:researcher"]),
            portability_manifest_id: PortabilityManifestId::derive(&"portable").unwrap(),
            supersedes: None,
            issued_at: Utc::now(),
            signatures: vec!["signature".into()],
        };
        bundle.bundle_id = bundle.derive_bundle_id().unwrap();
        assert!(bundle.validate_integrity().is_ok());

        bundle.rights.remove(&SovereigntyRightKind::PortabilityExit);
        bundle.bundle_id = bundle.derive_bundle_id().unwrap();
        assert!(matches!(
            bundle.validate_integrity(),
            Err(SovereigntyError::MissingRight(
                SovereigntyRightKind::PortabilityExit
            ))
        ));
    }

    #[test]
    fn residual_right_cannot_create_a_downstream_veto() {
        let mut right = residual_right();
        assert!(right.validate_integrity().is_ok());
        right.no_downstream_veto = false;
        right.right_id = right.derive_right_id().unwrap();
        assert!(matches!(
            right.validate_integrity(),
            Err(SovereigntyError::ResidualRightCreatesControlOrRecursion)
        ));
    }

    #[test]
    fn residual_assignment_requires_bilateral_signatures() {
        let mut right = residual_right();
        right.current_beneficiary = "cooperative:one".into();
        right.assignment = Some(ResidualRightAssignment {
            from_beneficiary: right.origin_researcher_id.to_string(),
            to_beneficiary: "cooperative:one".into(),
            signed_agreement_root: "blake3:agreement".into(),
            effective_at: Utc::now(),
            signatures: vec!["researcher-signature".into()],
        });
        right.right_id = right.derive_right_id().unwrap();
        assert!(matches!(
            right.validate_integrity(),
            Err(SovereigntyError::UnsignedAssignment)
        ));
    }

    #[test]
    fn evidence_edge_never_authorizes_payment() {
        let edge = EvidenceGraphEdge {
            downstream_claim_id: claim("downstream"),
            upstream_claim_id: claim("upstream"),
            kind: EvidenceEdgeKind::FormallyDependsOn,
            used_in_final_artifact: true,
            evidence_root: "blake3:proof-dependencies".into(),
        };
        assert!(edge.validate().is_ok());
        assert!(!edge.authorizes_payment());
    }

    #[test]
    fn reciprocal_constitution_is_bounded_and_nonblocking() {
        let mut constitution = EconomicConstitution {
            policy_id: PolicyId::derive(&"reciprocal").unwrap(),
            mode: CapsuleEconomicMode::Reciprocal,
            qualifying_revenue_sources: set(&["settled_xlmp_license"]),
            upstream_pool_bps: 500,
            per_ancestor_cap_bps: 100,
            max_ancestors: 16,
            max_dependency_depth: 4,
            depth_decay_bps: 5_000,
            minimum_payout_units: 1,
            bare_citations_eligible: false,
            externally_independent_claims_eligible: false,
            downstream_veto: false,
            recursive_charging: false,
            single_charge_per_revenue_event: true,
            equivalent_claim_clustering: true,
            policy_root: "blake3:reciprocal-policy".into(),
            signatures: vec!["signature".into()],
        };
        assert!(constitution.validate().is_ok());
        constitution.recursive_charging = true;
        assert!(matches!(
            constitution.validate(),
            Err(SovereigntyError::UnboundedEconomicTerms)
        ));
    }

    #[test]
    fn economic_compliance_is_certified_without_changing_research_validity() {
        let constitution = EconomicConstitution {
            policy_id: PolicyId::derive(&"compliance-policy").unwrap(),
            mode: CapsuleEconomicMode::Reciprocal,
            qualifying_revenue_sources: set(&["settled_license"]),
            upstream_pool_bps: 500,
            per_ancestor_cap_bps: 100,
            max_ancestors: 8,
            max_dependency_depth: 4,
            depth_decay_bps: 5_000,
            minimum_payout_units: 1,
            bare_citations_eligible: false,
            externally_independent_claims_eligible: false,
            downstream_veto: false,
            recursive_charging: false,
            single_charge_per_revenue_event: true,
            equivalent_claim_clustering: true,
            policy_root: "blake3:compliance-policy".into(),
            signatures: vec!["policy-signature".into()],
        };
        let obligations = set(&["blake3:upstream-pool-settlement"]);
        let mut certificate = EconomicComplianceCertificate {
            certificate_id: EconomicComplianceCertificateId::derive(&"placeholder").unwrap(),
            claim_id: claim("economically-compliant"),
            economic_policy_id: constitution.policy_id.clone(),
            required_obligation_roots: obligations.clone(),
            satisfied_obligation_roots: obligations,
            revenue_event_ids: BTreeSet::from(
                [RevenueEventId::derive(&"settled-revenue").unwrap()],
            ),
            settlement_receipt_ids: BTreeSet::from([ReceiptId::derive(&"settlement").unwrap()]),
            status: EconomicComplianceStatus::Compliant,
            evaluation_evidence_root: "blake3:economic-compliance-evidence".into(),
            evaluator_operator_cluster_id: OperatorClusterId::derive(&"economic-auditor").unwrap(),
            evaluated_at: Utc::now(),
            supersedes: None,
            signature: "auditor-signature".into(),
        };
        certificate.certificate_id = certificate.derive_certificate_id().unwrap();
        assert!(certificate.validate_against(&constitution).is_ok());
        assert!(!certificate.affects_research_validity());

        certificate.status = EconomicComplianceStatus::Noncompliant;
        certificate.certificate_id = certificate.derive_certificate_id().unwrap();
        assert!(matches!(
            certificate.validate_against(&constitution),
            Err(SovereigntyError::InvalidEconomicCompliance)
        ));
    }

    #[test]
    fn published_economic_compliance_vector_is_content_derived() {
        let constitution: EconomicConstitution = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/economic-constitution.json"
        ))
        .unwrap();
        let certificate: EconomicComplianceCertificate = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/economic-compliance-certificate.json"
        ))
        .unwrap();
        assert!(certificate.validate_against(&constitution).is_ok());
        assert!(!certificate.affects_research_validity());
    }

    #[test]
    fn portability_requires_redundant_artifact_locations() {
        let artifact = "blake3:artifact".to_owned();
        let independent_locations = BTreeSet::from([
            PortableStorageLocation {
                provider_id: "ipfs-provider".into(),
                location: "ipfs:cid".into(),
                retrieval_evidence_root: "blake3:ipfs-retrieval".into(),
            },
            PortableStorageLocation {
                provider_id: "arweave-provider".into(),
                location: "ar:transaction".into(),
                retrieval_evidence_root: "blake3:arweave-retrieval".into(),
            },
        ]);
        let mut locations = BTreeMap::new();
        locations.insert(artifact.clone(), independent_locations.clone());
        let mut manifest = ResearcherPortabilityManifest {
            manifest_id: PortabilityManifestId::derive(&"placeholder").unwrap(),
            researcher_id: ResearcherId::derive(&"researcher").unwrap(),
            protocol_version: "XLMP/1".into(),
            identity_credential_roots: set(&["blake3:identity"]),
            artifact_roots: BTreeSet::from([artifact.clone()]),
            contribution_manifest_roots: set(&["blake3:contribution"]),
            verification_receipt_roots: set(&["blake3:verification"]),
            economic_policy_roots: set(&["blake3:economic"]),
            settlement_commitments: set(&["chain:commitment"]),
            event_log_checkpoints: set(&["blake3:event-log"]),
            storage_locations: locations,
            event_log_locations: independent_locations,
            reconstruction_client: "xlemma-cli/0.2".into(),
            reconstruction_client_source_root: "blake3:open-client-source".into(),
            funds_exit_instructions_root: "blake3:funds-exit-instructions".into(),
            created_at: Utc::now(),
            supersedes: None,
            signature: "signature".into(),
        };
        manifest.manifest_id = manifest.derive_manifest_id().unwrap();
        assert!(manifest.validate_reconstructable().is_ok());
        manifest.storage_locations.insert(
            artifact,
            BTreeSet::from([PortableStorageLocation {
                provider_id: "ipfs-provider".into(),
                location: "ipfs:cid".into(),
                retrieval_evidence_root: "blake3:ipfs-retrieval".into(),
            }]),
        );
        manifest.manifest_id = manifest.derive_manifest_id().unwrap();
        assert!(matches!(
            manifest.validate_reconstructable(),
            Err(SovereigntyError::InsufficientStorageReplication)
        ));
    }

    #[test]
    fn published_portability_vector_survives_company_disappearance() {
        let manifest: ResearcherPortabilityManifest = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/portability-manifest.json"
        ))
        .unwrap();
        assert!(manifest.validate_reconstructable().is_ok());
        assert_eq!(manifest.event_log_locations.len(), 2);
        assert!(!manifest.reconstruction_client_source_root.is_empty());
        assert!(!manifest.funds_exit_instructions_root.is_empty());
    }

    #[test]
    fn every_verification_class_has_mandatory_evidence() {
        for class in [
            VerificationProfileClass::Formal,
            VerificationProfileClass::Computational,
            VerificationProfileClass::Statistical,
            VerificationProfileClass::Simulation,
            VerificationProfileClass::Empirical,
            VerificationProfileClass::Hybrid,
        ] {
            let mut profile = VerificationProfile {
                policy_id: PolicyId::derive(&format!("{class:?}")).unwrap(),
                class,
                required_evidence: required_evidence_for(class),
                verifier_implementations: set(&["adapter:one", "adapter:two"]),
                minimum_reproductions: 2,
                minimum_independent_operators: 2,
                challenge_window_seconds: 86_400,
                policy_root: "blake3:verification-profile".into(),
            };
            assert!(profile.validate().is_ok());
            profile.required_evidence.clear();
            assert!(matches!(
                profile.validate(),
                Err(SovereigntyError::IncompleteVerificationProfile(_))
            ));
        }
    }
}

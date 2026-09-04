use crate::{
    Amount, ArtifactId, ClaimId, FormalStatus, IdError, LemmaId, NoveltyDecision, PolicyId,
    ProofId, ReceiptId, ResearcherId, TheoryId, XLMP_VERSION,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TheoryManifest {
    pub protocol_version: String,
    pub lean_toolchain: String,
    pub dependency_merkle_root: String,
    pub trust_policy_id: PolicyId,
    pub checker_policy_id: PolicyId,
    pub permitted_axioms: Vec<String>,
    pub canonical_encoding: String,
}

impl TheoryManifest {
    pub fn derive_theory_id(&self) -> Result<TheoryId, IdError> {
        TheoryId::derive(self)
    }

    pub fn validate_integrity(&self) -> Result<(), ManifestIntegrityError> {
        self.trust_policy_id.validate()?;
        self.checker_policy_id.validate()?;
        let unique_axioms = self.permitted_axioms.iter().collect::<BTreeSet<_>>();
        if self.protocol_version != XLMP_VERSION
            || self.lean_toolchain.trim().is_empty()
            || self.dependency_merkle_root.trim().is_empty()
            || self.canonical_encoding.trim().is_empty()
            || unique_axioms.len() != self.permitted_axioms.len()
            || self
                .permitted_axioms
                .iter()
                .any(|axiom| axiom.trim().is_empty())
        {
            return Err(ManifestIntegrityError::InvalidTheory);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimManifest {
    pub protocol_version: String,
    pub theory_id: TheoryId,
    /// Canonical serialization of the elaborated Lean type, not source text.
    pub canonical_elaborated_type: String,
    pub declaration_name: String,
    pub source_artifact: Option<ArtifactId>,
    pub created_at: DateTime<Utc>,
}

impl ClaimManifest {
    /// Derive formal identity only from the elaborated type and its theory.
    /// Presentation names, source locations, and timestamps are deliberately
    /// excluded so source text can never become the final ClaimID.
    pub fn derive_claim_id(&self) -> Result<ClaimId, IdError> {
        ClaimId::from_canonical_elaborated_type(&self.theory_id, &self.canonical_elaborated_type)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofManifest {
    pub protocol_version: String,
    pub claim_id: ClaimId,
    /// Canonical serialized proof term or exported checker object.
    pub canonical_proof_object: String,
    pub artifact_id: ArtifactId,
    pub direct_dependencies: Vec<ClaimId>,
    pub dependency_root: String,
    pub observed_axioms: Vec<String>,
}

impl ProofManifest {
    /// Bind the exact checker-consumable proof object to its formal claim.
    pub fn derive_proof_id(&self) -> Result<ProofId, IdError> {
        ProofId::from_canonical_proof_object(&self.claim_id, &self.canonical_proof_object)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactEntry {
    pub path: String,
    pub media_type: String,
    pub content_hash: String,
    pub byte_length: u64,
    pub encrypted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub protocol_version: String,
    pub entries: Vec<ArtifactEntry>,
    pub root: String,
    pub source_commit: Option<String>,
    pub lean_toolchain: String,
    pub dependency_lock_hash: String,
    pub build_image_digest: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct ArtifactIdentityMaterial<'a> {
    protocol_version: &'a str,
    entries: &'a [ArtifactEntry],
    root: &'a str,
    lean_toolchain: &'a str,
    dependency_lock_hash: &'a str,
    build_image_digest: &'a Option<String>,
}

impl ArtifactManifest {
    pub fn expected_root(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"xlemma-artifact-root-v1\0");
        for entry in &self.entries {
            hasher.update(entry.path.as_bytes());
            hasher.update(b"\0");
            hasher.update(entry.media_type.as_bytes());
            hasher.update(b"\0");
            hasher.update(entry.content_hash.as_bytes());
            hasher.update(b"\0");
            hasher.update(&entry.byte_length.to_le_bytes());
            hasher.update(&[u8::from(entry.encrypted)]);
        }
        format!("blake3:{}", hasher.finalize().to_hex())
    }

    pub fn validate_for_identity(&self) -> Result<(), IdError> {
        if self.protocol_version != XLMP_VERSION {
            return Err(IdError::InvalidArtifactManifest("protocol version"));
        }
        if self.entries.is_empty() {
            return Err(IdError::InvalidArtifactManifest("empty entry set"));
        }
        if !self
            .entries
            .windows(2)
            .all(|pair| pair[0].path < pair[1].path)
        {
            return Err(IdError::InvalidArtifactManifest(
                "entries are not strictly path-sorted",
            ));
        }
        for entry in &self.entries {
            if !is_safe_canonical_artifact_path(&entry.path)
                || entry.media_type.trim().is_empty()
                || entry.content_hash.trim().is_empty()
            {
                return Err(IdError::InvalidArtifactManifest("invalid entry"));
            }
        }
        if self.lean_toolchain.trim().is_empty()
            || self.dependency_lock_hash.trim().is_empty()
            || self.root != self.expected_root()
        {
            return Err(IdError::InvalidArtifactManifest(
                "environment or artifact root mismatch",
            ));
        }
        Ok(())
    }

    /// Derive artifact identity from reproducibility-critical content. Source
    /// labels and creation time remain provenance metadata and cannot change
    /// the identity of an otherwise identical bundle.
    pub fn derive_artifact_id(&self) -> Result<ArtifactId, IdError> {
        self.validate_for_identity()?;
        ArtifactId::derive(&ArtifactIdentityMaterial {
            protocol_version: &self.protocol_version,
            entries: &self.entries,
            root: &self.root,
            lean_toolchain: &self.lean_toolchain,
            dependency_lock_hash: &self.dependency_lock_hash,
            build_image_digest: &self.build_image_digest,
        })
    }
}

fn is_safe_canonical_artifact_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains('\0')
        && !path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
        && !path
            .split('/')
            .next()
            .is_some_and(|component| component.ends_with(':'))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionRole {
    QuestionOriginator,
    FormulaOriginator,
    ConjectureAuthor,
    ProofDiscoverer,
    LeanFormalizer,
    TacticAuthor,
    ToolDeveloper,
    LibraryAuthor,
    DatasetCreator,
    ExperimentalContributor,
    StatementAlignmentReviewer,
    IndependentVerifier,
    ApplicationDeveloper,
    Maintainer,
    Reviewer,
    ExpositionAuthor,
    Sponsor,
    ComputeProvider,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContributionShare {
    pub contributor: ResearcherId,
    pub roles: Vec<ContributionRole>,
    /// Share of the creator pool, in basis points. Total MUST equal 10_000.
    pub share_bps: u16,
    pub evidence_root: String,
    pub signed_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContributionManifest {
    pub claim_id: ClaimId,
    pub contributors: Vec<ContributionShare>,
    pub machine_contributions: Vec<MachineContributionRecord>,
    pub amendment_parent: Option<String>,
    pub dispute_status: String,
}

impl ContributionManifest {
    pub fn validate_integrity(&self) -> Result<(), ManifestIntegrityError> {
        self.claim_id.validate()?;
        let unique_contributors = self
            .contributors
            .iter()
            .map(|share| &share.contributor)
            .collect::<BTreeSet<_>>();
        let total = self.contributors.iter().try_fold(0_u32, |sum, share| {
            sum.checked_add(u32::from(share.share_bps))
                .ok_or(ManifestIntegrityError::InvalidContribution)
        })?;
        if self.contributors.is_empty()
            || unique_contributors.len() != self.contributors.len()
            || total != 10_000
            || self.dispute_status.trim().is_empty()
        {
            return Err(ManifestIntegrityError::InvalidContribution);
        }
        for share in &self.contributors {
            share.contributor.validate()?;
            let unique_roles = share.roles.iter().map(|role| format!("{role:?}"));
            if share.roles.is_empty()
                || unique_roles.collect::<BTreeSet<_>>().len() != share.roles.len()
                || share.share_bps == 0
                || share.evidence_root.trim().is_empty()
                || share.signature.trim().is_empty()
            {
                return Err(ManifestIntegrityError::InvalidContribution);
            }
        }
        if self.machine_contributions.iter().any(|record| {
            record.provider.trim().is_empty()
                || record.model.trim().is_empty()
                || record.request_hash.trim().is_empty()
                || record.context_root.trim().is_empty()
                || record.output_artifact_roots.is_empty()
                || record
                    .output_artifact_roots
                    .iter()
                    .any(|root| root.trim().is_empty())
                || record.disclosure.trim().is_empty()
        }) {
            return Err(ManifestIntegrityError::InvalidContribution);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineContributionRecord {
    pub provider: String,
    pub model: String,
    pub model_snapshot: Option<String>,
    pub request_hash: String,
    pub context_root: String,
    pub output_artifact_roots: Vec<String>,
    pub human_selection_and_edits_root: Option<String>,
    pub disclosure: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsKind {
    Attribution,
    ManuscriptCopyright,
    SourceCodeCopyright,
    DatasetRights,
    PatentOrApplication,
    TradeSecret,
    ContractualLicense,
    CommercialImplementation,
    AccessRight,
    NoExclusiveRightClaimed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RightsClaim {
    pub kind: RightsKind,
    pub controller: String,
    pub jurisdiction: Option<String>,
    pub source_agreement_hash: Option<String>,
    pub transferable: bool,
    pub sublicensable: bool,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RightsManifest {
    pub claim_id: ClaimId,
    pub originator_attribution_nontransferable: bool,
    pub claims: Vec<RightsClaim>,
    pub employer_university_grant_clearance: String,
    pub clearance_evidence_root: Option<String>,
    pub legal_wrapper: Option<String>,
    pub signed_by: Vec<String>,
    pub signed_at: DateTime<Utc>,
}

impl RightsManifest {
    pub fn validate_integrity(&self) -> Result<(), ManifestIntegrityError> {
        self.claim_id.validate()?;
        let unique_signers = self.signed_by.iter().collect::<BTreeSet<_>>();
        if !self.originator_attribution_nontransferable
            || self.employer_university_grant_clearance.trim().is_empty()
            || self.signed_by.is_empty()
            || unique_signers.len() != self.signed_by.len()
            || self.signed_by.iter().any(|signer| signer.trim().is_empty())
            || self.claims.iter().any(|claim| {
                claim.controller.trim().is_empty()
                    || claim
                        .limitations
                        .iter()
                        .any(|limitation| limitation.trim().is_empty())
            })
        {
            return Err(ManifestIntegrityError::InvalidRights);
        }
        Ok(())
    }
}

/// Economic treatment selected for a capsule. This never changes origin,
/// formal validity, or the scope of legal rights actually held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleEconomicMode {
    /// Public artifact use carries no mandatory per-use protocol payment.
    #[serde(alias = "open_commons")]
    Commons,
    /// Monetized participating descendants route one bounded, non-recursive
    /// pool upstream without creating a veto over publication or use.
    Reciprocal,
    /// Defined commercial artifacts may carry explicit, bounded license terms.
    #[serde(alias = "commercial_research")]
    CommercialArtifact,
    /// A sponsor precommits bounty, allocation, acceptance, and result terms.
    SponsoredChallenge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueWaterfall {
    pub creator_pool_bps: u16,
    pub upstream_dependency_pool_bps: u16,
    pub reverification_security_pool_bps: u16,
    pub open_research_pool_bps: u16,
    pub dispute_insurance_pool_bps: u16,
    pub protocol_operations_bps: u16,
}

impl RevenueWaterfall {
    pub fn total_bps(&self) -> u32 {
        u32::from(self.creator_pool_bps)
            + u32::from(self.upstream_dependency_pool_bps)
            + u32::from(self.reverification_security_pool_bps)
            + u32::from(self.open_research_pool_bps)
            + u32::from(self.dispute_insurance_pool_bps)
            + u32::from(self.protocol_operations_bps)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueRoute {
    pub settlement_asset: String,
    pub researcher_vault: String,
    pub waterfall: RevenueWaterfall,
    pub contributor_manifest_hash: String,
    /// Root of the prescriptive economic graph. The formal dependency root is
    /// evidence only and cannot substitute for this authorization.
    pub economic_policy_root: String,
    pub dependency_reward_cap_bps: u16,
    pub auto_compound_bps_by_researcher: BTreeMap<ResearcherId, u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LemmaCapsule {
    pub lemma_id: LemmaId,
    pub theory_id: TheoryId,
    pub claim_id: ClaimId,
    pub proof_id: Option<ProofId>,
    pub artifact_id: ArtifactId,
    pub presentation_ids: Vec<String>,
    pub origin_certificate_id: ReceiptId,
    pub contribution_manifest_hash: String,
    pub rights_manifest_hash: String,
    pub dependency_root: String,
    pub verification_receipt_ids: Vec<ReceiptId>,
    pub novelty_receipt_ids: Vec<ReceiptId>,
    pub statement_alignment_receipt_ids: Vec<ReceiptId>,
    pub economic_mode: CapsuleEconomicMode,
    pub revenue_route: RevenueRoute,
    pub formal_status: FormalStatus,
    pub novelty_status: NoveltyDecision,
    pub parent_capsule: Option<LemmaId>,
    pub supersedes: Option<LemmaId>,
    pub created_at: DateTime<Utc>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct LemmaCapsuleIdentity<'a> {
    theory_id: &'a TheoryId,
    claim_id: &'a ClaimId,
    proof_id: &'a Option<ProofId>,
    artifact_id: &'a ArtifactId,
    presentation_ids: &'a [String],
    origin_certificate_id: &'a ReceiptId,
    contribution_manifest_hash: &'a str,
    rights_manifest_hash: &'a str,
    dependency_root: &'a str,
    verification_receipt_ids: &'a [ReceiptId],
    novelty_receipt_ids: &'a [ReceiptId],
    statement_alignment_receipt_ids: &'a [ReceiptId],
    economic_mode: CapsuleEconomicMode,
    revenue_route: &'a RevenueRoute,
    formal_status: FormalStatus,
    novelty_status: NoveltyDecision,
    parent_capsule: &'a Option<LemmaId>,
    supersedes: &'a Option<LemmaId>,
    created_at: &'a DateTime<Utc>,
    metadata: &'a BTreeMap<String, String>,
}

impl LemmaCapsule {
    pub fn derive_lemma_id(&self) -> Result<LemmaId, IdError> {
        LemmaId::derive(&LemmaCapsuleIdentity {
            theory_id: &self.theory_id,
            claim_id: &self.claim_id,
            proof_id: &self.proof_id,
            artifact_id: &self.artifact_id,
            presentation_ids: &self.presentation_ids,
            origin_certificate_id: &self.origin_certificate_id,
            contribution_manifest_hash: &self.contribution_manifest_hash,
            rights_manifest_hash: &self.rights_manifest_hash,
            dependency_root: &self.dependency_root,
            verification_receipt_ids: &self.verification_receipt_ids,
            novelty_receipt_ids: &self.novelty_receipt_ids,
            statement_alignment_receipt_ids: &self.statement_alignment_receipt_ids,
            economic_mode: self.economic_mode,
            revenue_route: &self.revenue_route,
            formal_status: self.formal_status,
            novelty_status: self.novelty_status,
            parent_capsule: &self.parent_capsule,
            supersedes: &self.supersedes,
            created_at: &self.created_at,
            metadata: &self.metadata,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), ManifestIntegrityError> {
        self.lemma_id.validate()?;
        self.theory_id.validate()?;
        self.claim_id.validate()?;
        self.artifact_id.validate()?;
        self.origin_certificate_id.validate()?;
        if let Some(proof_id) = &self.proof_id {
            proof_id.validate()?;
        }
        if let Some(parent) = &self.parent_capsule {
            parent.validate()?;
        }
        if let Some(parent) = &self.supersedes {
            parent.validate()?;
        }
        for receipt_id in self
            .verification_receipt_ids
            .iter()
            .chain(&self.novelty_receipt_ids)
            .chain(&self.statement_alignment_receipt_ids)
        {
            receipt_id.validate()?;
        }
        for researcher_id in self.revenue_route.auto_compound_bps_by_researcher.keys() {
            researcher_id.validate()?;
        }
        let certified_without_proof = matches!(
            self.formal_status,
            FormalStatus::Reproduced | FormalStatus::Certified
        ) && (self.proof_id.is_none()
            || self.verification_receipt_ids.is_empty());
        let commons_charges_dependencies = self.economic_mode == CapsuleEconomicMode::Commons
            && (self.revenue_route.waterfall.upstream_dependency_pool_bps != 0
                || self.revenue_route.dependency_reward_cap_bps != 0);
        if self.lemma_id != self.derive_lemma_id()?
            || self.contribution_manifest_hash.trim().is_empty()
            || self.rights_manifest_hash.trim().is_empty()
            || self.dependency_root.trim().is_empty()
            || self.revenue_route.settlement_asset.trim().is_empty()
            || self.revenue_route.researcher_vault.trim().is_empty()
            || self
                .revenue_route
                .contributor_manifest_hash
                .trim()
                .is_empty()
            || self.revenue_route.economic_policy_root.trim().is_empty()
            || self.revenue_route.waterfall.total_bps() != 10_000
            || self.revenue_route.dependency_reward_cap_bps > 10_000
            || self
                .revenue_route
                .auto_compound_bps_by_researcher
                .values()
                .any(|bps| *bps > 10_000)
            || certified_without_proof
            || commons_charges_dependencies
            || self.parent_capsule.as_ref() == Some(&self.lemma_id)
            || self.supersedes.as_ref() == Some(&self.lemma_id)
        {
            return Err(ManifestIntegrityError::InvalidCapsule);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearcherNodeManifest {
    pub researcher_id: ResearcherId,
    #[serde(default)]
    pub verified_user_id: Option<crate::VerifiedUserId>,
    #[serde(default)]
    pub user_credential_id: Option<crate::UserCredentialId>,
    pub display_name: Option<String>,
    pub identity_keys: Vec<String>,
    pub research_credit_asset: String,
    pub research_vault: String,
    pub governance_policy_id: PolicyId,
    pub contribution_identity_root: String,
    pub reputation_root: String,
    pub supported_domains: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl ResearcherNodeManifest {
    pub fn validate_integrity(&self) -> Result<(), ManifestIntegrityError> {
        self.researcher_id.validate()?;
        self.governance_policy_id.validate()?;
        if let Some(verified_user_id) = &self.verified_user_id {
            verified_user_id.validate()?;
        }
        if let Some(credential_id) = &self.user_credential_id {
            credential_id.validate()?;
        }
        let unique_keys = self.identity_keys.iter().collect::<BTreeSet<_>>();
        let unique_domains = self.supported_domains.iter().collect::<BTreeSet<_>>();
        if self.verified_user_id.is_some() != self.user_credential_id.is_some()
            || self.identity_keys.is_empty()
            || unique_keys.len() != self.identity_keys.len()
            || self.identity_keys.iter().any(|key| key.trim().is_empty())
            || unique_domains.len() != self.supported_domains.len()
            || self
                .supported_domains
                .iter()
                .any(|domain| domain.trim().is_empty())
            || self.research_credit_asset.trim().is_empty()
            || self.research_vault.trim().is_empty()
            || self.contribution_identity_root.trim().is_empty()
            || self.reputation_root.trim().is_empty()
        {
            return Err(ManifestIntegrityError::InvalidResearcher);
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ManifestIntegrityError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error("theory manifest is incomplete, duplicated, or uses the wrong protocol")]
    InvalidTheory,
    #[error("contribution manifest does not conserve shares or lacks evidence")]
    InvalidContribution,
    #[error("rights manifest permits mutable origin or lacks clearance evidence")]
    InvalidRights,
    #[error("lemma capsule identity, assurance, economics, or lineage is invalid")]
    InvalidCapsule,
    #[error("researcher manifest identity, credential binding, or endpoints are invalid")]
    InvalidResearcher,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BountyManifest {
    pub bounty_id: String,
    pub sponsor: String,
    pub theory_id: TheoryId,
    pub claim_id: ClaimId,
    pub exact_trusted_challenge_hash: String,
    pub allowed_dependency_root: String,
    pub axiom_policy_id: PolicyId,
    pub required_verification_policy_id: PolicyId,
    pub reward: Amount,
    pub deadline: DateTime<Utc>,
    pub commit_reveal_required: bool,
    pub challenge_period_seconds: u64,
}

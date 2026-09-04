use crate::{
    Amount, ArtifactId, ClaimId, FormalStatus, IdError, LemmaId, NoveltyDecision, PolicyId,
    ProofId, ReceiptId, ResearcherId, TheoryId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionRole {
    FormulaOriginator,
    ConjectureAuthor,
    ProofDiscoverer,
    LeanFormalizer,
    TacticAuthor,
    LibraryAuthor,
    DatasetCreator,
    ExperimentalContributor,
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

/// Economic treatment selected for a capsule. This never changes origin,
/// formal validity, or the scope of legal rights actually held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleEconomicMode {
    /// Public artifact use carries no mandatory per-use protocol payment.
    OpenCommons,
    /// Defined commercial artifacts may carry explicit, bounded license terms.
    CommercialResearch,
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

//! Authenticated discovery command processing. Persistence and certificate
//! resolution are supplied by the API; money moves only in the funded escrow.
use crate::{
    ContributorShare, DeclaredEvidenceStatus, DiscoveryEvent, DiscoveryFunding, DiscoveryRemedy,
    DiscoveryRoundPolicy, DiscoverySimulation, DiscoverySubmission, FundingReceipt, RewardCategory,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xlemma_core::{
    canonical_json_bytes, ArtifactId, ClaimId, ContributionGroupId, DiscoveryRoundId, MessageId,
    OperatorClusterId, PolicyId, ReceiptId, ResearcherId, VerificationEvidenceKind,
    VerificationProfileClass,
};
use xlemma_xlmp::{validate_ed25519_signer, verify_ed25519_detached};

const SAFE: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryRole {
    Administrator,
    Researcher,
    FundingObserver,
    Verifier,
    Assessor,
    AppealReviewer,
    SettlementObserver,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryPrincipal {
    pub researcher_id: ResearcherId,
    pub cluster_id: OperatorClusterId,
    pub payout_address: String,
    pub credential_root: ArtifactId,
    pub roles: BTreeSet<DiscoveryRole>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryTrust {
    pub network: String,
    pub chain_id: u64,
    pub escrow_address: String,
    pub usdc_asset: String,
    pub principals: BTreeMap<String, DiscoveryPrincipal>,
}

impl DiscoveryTrust {
    pub fn root(&self) -> Result<PolicyId, ServiceError> {
        Ok(PolicyId::derive(self)?)
    }
    pub fn validate(&self) -> Result<(), ServiceError> {
        check(
            !self.network.is_empty()
                && self.chain_id > 0
                && self.chain_id <= SAFE
                && address(&self.escrow_address)
                && !self.usdc_asset.is_empty(),
            "invalid settlement trust",
        )?;
        let mut identities = BTreeSet::new();
        for (key, principal) in &self.principals {
            validate_ed25519_signer(key)
                .map_err(|_| ServiceError::Invalid("invalid principal key"))?;
            principal.researcher_id.validate()?;
            principal.cluster_id.validate()?;
            principal.credential_root.validate()?;
            check(
                address(&principal.payout_address)
                    && !principal
                        .payout_address
                        .eq_ignore_ascii_case(&self.escrow_address)
                    && identities.insert(&principal.researcher_id),
                "invalid or repeated principal",
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceRoundPolicy {
    pub economics: DiscoveryRoundPolicy,
    pub domain: String,
    pub settlement_expires_at: u64,
    pub simultaneous_window_seconds: u64,
    pub reproduction_fee_units: u64,
    pub maximum_verifier_clusters: u16,
    pub formal_policy_bindings: BTreeMap<PolicyId, PolicyId>,
    pub minimum_foundational_bps: u16,
    /// Independent reference-cost tiers committed before participation.
    pub calibration: BTreeMap<String, CalibrationTier>,
    pub calibration_root: ArtifactId,
    pub prior_art_cutoff: DateTime<Utc>,
    /// Admission reserves for applicants without prepaid compute.
    pub assisted_submission_slots: u32,
    pub per_researcher_submission_cap: u32,
    pub max_contributors_per_submission: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationTier {
    pub reference_cost_units: u64,
    pub attempts: u32,
    pub accepted_results: u32,
    pub uncertainty_bps: u16,
    pub outcomes_root: ArtifactId,
}

impl ServiceRoundPolicy {
    pub fn id(&self) -> Result<DiscoveryRoundId, ServiceError> {
        Ok(DiscoveryRoundId::derive(self)?)
    }
    fn validate(&self, trust: &DiscoveryTrust) -> Result<(), ServiceError> {
        check(
            !self.domain.is_empty() && self.economics.usdc_asset == trust.usdc_asset,
            "round domain or asset mismatch",
        )?;
        check(
            self.minimum_foundational_bps > 0 && self.minimum_foundational_bps <= 10_000,
            "foundational protection required",
        )?;
        check(
            self.settlement_expires_at > self.economics.review_deadline.saturating_add(86_400)
                && self.settlement_expires_at <= SAFE,
            "settlement grace must exceed one day after review deadline",
        )?;
        check(
            self.simultaneous_window_seconds > 0
                && self.simultaneous_window_seconds
                    <= self
                        .economics
                        .submissions_close_at
                        .saturating_sub(self.economics.opens_at),
            "invalid simultaneous discovery window",
        )?;
        check(
            self.reproduction_fee_units >= u64::from(self.maximum_verifier_clusters)
                && self.economics.verification_fee_units >= 2
                && self.economics.appeal_fee_units >= 2
                && self.reproduction_fee_units <= SAFE
                && (2..=8).contains(&self.maximum_verifier_clusters),
            "invalid funded reproduction capacity",
        )?;
        for profile in &self.economics.profiles {
            check(
                profile.minimum_independent_operators <= self.maximum_verifier_clusters,
                "profile exceeds funded reproduction capacity",
            )?;
            if profile.class == VerificationProfileClass::Formal {
                self.formal_policy_bindings
                    .get(&profile.policy_id)
                    .ok_or(ServiceError::Invalid(
                        "formal profile requires an exact PoIR policy binding",
                    ))?
                    .validate()?;
            }
        }
        self.calibration_root.validate()?;
        let total: u128 = self
            .economics
            .budgets
            .values()
            .map(|b| u128::from(b.solver_units))
            .sum();
        let foundation = self
            .economics
            .budgets
            .get(&RewardCategory::FoundationalResearch)
            .map(|b| u128::from(b.solver_units))
            .unwrap_or(0);
        check(
            total > 0 && foundation * 10_000 >= total * u128::from(self.minimum_foundational_bps),
            "foundational allocation below protected floor",
        )?;
        check(
            self.assisted_submission_slots > 0
                && self.assisted_submission_slots <= self.economics.maximum_submissions
                && self.per_researcher_submission_cap > 0,
            "invalid fair admission limits",
        )?;
        check(
            self.max_contributors_per_submission > 0
                && u64::from(self.economics.maximum_submissions)
                    * (u64::from(self.max_contributors_per_submission)
                        + 2
                        + u64::from(self.maximum_verifier_clusters))
                    + u64::from(self.economics.maximum_appeals) * 2
                    <= 256,
            "round exceeds atomic escrow batch capacity",
        )?;
        check(
            !self.calibration.is_empty() && self.calibration.len() <= 32,
            "missing calibration",
        )?;
        for tier in self.calibration.values() {
            tier.outcomes_root.validate()?;
            check(
                tier.reference_cost_units > 0
                    && tier.reference_cost_units <= SAFE
                    && tier.attempts >= 10
                    && tier.accepted_results > 0
                    && tier.accepted_results <= tier.attempts
                    && tier.uncertainty_bps <= 10_000,
                "insufficient calibration evidence",
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchSubmission {
    pub claim_id: ClaimId,
    pub artifact_id: ArtifactId,
    pub profile_id: PolicyId,
    pub category: RewardCategory,
    pub contributors: Vec<ContributorShare>,
    pub manifest_root: ArtifactId,
    pub evidence_roots: BTreeMap<VerificationEvidenceKind, ArtifactId>,
    pub registered_study: Option<crate::RegisteredStudy>,
    pub assisted: bool,
    /// Disclosure/provenance commitment; never enters reward weighting.
    pub research_context_root: ArtifactId,
}
impl ResearchSubmission {
    pub fn id(&self, round: &DiscoveryRoundId) -> Result<ReceiptId, ServiceError> {
        Ok(ReceiptId::derive(&(
            "discovery-submission-v1",
            round,
            self,
        ))?)
    }
    pub fn commitment(
        &self,
        round: &DiscoveryRoundId,
        salt: &str,
    ) -> Result<ReceiptId, ServiceError> {
        check(
            salt.len() >= 32 && salt.len() <= 256,
            "commit salt must contain at least 32 characters",
        )?;
        Ok(ReceiptId::derive(&(
            "discovery-commit-v1",
            round,
            self,
            salt,
        ))?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RewardAssessment {
    pub submission_id: ReceiptId,
    pub group_id: ContributionGroupId,
    pub eligible: bool,
    pub calibration_tier: String,
    pub prior_art_root: ArtifactId,
    pub additional_contribution_root: ArtifactId,
    pub independent_discovery_root: Option<ArtifactId>,
    pub reasons_root: ArtifactId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DiscoveryCommand {
    CreateRound {
        policy: ServiceRoundPolicy,
    },
    ApproveCalibration {
        round_id: DiscoveryRoundId,
    },
    ObserveFunding {
        round_id: DiscoveryRoundId,
        #[serde(deserialize_with = "deserialize_funding")]
        receipt: FundingReceipt,
        category: RewardCategory,
        donor_cluster: OperatorClusterId,
        administrator_cluster: OperatorClusterId,
    },
    OpenRound {
        round_id: DiscoveryRoundId,
    },
    Commit {
        round_id: DiscoveryRoundId,
        commitment: ReceiptId,
    },
    Reveal {
        round_id: DiscoveryRoundId,
        submission: ResearchSubmission,
        salt: String,
    },
    Consent {
        round_id: DiscoveryRoundId,
        submission_id: ReceiptId,
    },
    AttachEvidence {
        round_id: DiscoveryRoundId,
        submission_id: ReceiptId,
        certificate_message_id: MessageId,
    },
    Assess {
        round_id: DiscoveryRoundId,
        assessment: RewardAssessment,
    },
    Appeal {
        round_id: DiscoveryRoundId,
        submission_id: ReceiptId,
        grounds: crate::AppealGround,
        evidence_root: ArtifactId,
    },
    ResolveAppeal {
        round_id: DiscoveryRoundId,
        appeal_id: ReceiptId,
        reasons_root: ArtifactId,
        remedy: DiscoveryRemedy,
    },
    Finalize {
        round_id: DiscoveryRoundId,
    },
    Expire {
        round_id: DiscoveryRoundId,
    },
    ObserveExpiry {
        round_id: DiscoveryRoundId,
        transaction_hash: String,
        block_hash: String,
    },
    ObserveSettlement {
        round_id: DiscoveryRoundId,
        plan_id: ReceiptId,
        transaction_hash: String,
        block_hash: String,
    },
}

// Internally tagged serde enums buffer their contents in a deserializer that
// cannot visit u128. Preserve the canonical numeric Amount wire format through
// serde_json::Value instead of changing money into floats or strings.
fn deserialize_funding<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<FundingReceipt, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    serde_json::from_value(value).map_err(serde::de::Error::custom)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryEnvelope {
    pub command_id: ReceiptId,
    pub trust_root: PolicyId,
    pub nonce: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub signer: String,
    pub command: DiscoveryCommand,
    pub signature: String,
}
impl DiscoveryEnvelope {
    pub fn expected_id(&self) -> Result<ReceiptId, ServiceError> {
        Ok(ReceiptId::derive(&(
            "discovery-command-v1",
            &self.trust_root,
            &self.nonce,
            self.issued_at,
            self.expires_at,
            &self.signer,
            &self.command,
        ))?)
    }
    pub fn signing_bytes(&self) -> Result<Vec<u8>, ServiceError> {
        Ok(canonical_json_bytes(&(
            "discovery-command-signature-v1",
            self.expected_id()?,
        ))?)
    }
    pub fn authenticate(
        &self,
        trust: &DiscoveryTrust,
        at: DateTime<Utc>,
    ) -> Result<(), ServiceError> {
        check(
            self.trust_root == trust.root()? && self.command_id == self.expected_id()?,
            "command identity mismatch",
        )?;
        check(
            !self.nonce.is_empty()
                && self.nonce.len() <= 128
                && self.issued_at <= at
                && at <= self.expires_at
                && self
                    .expires_at
                    .signed_duration_since(self.issued_at)
                    .num_seconds()
                    <= 3600,
            "command expired or invalid nonce",
        )?;
        verify_ed25519_detached(&self.signer, &self.signature, &self.signing_bytes()?)
            .map_err(|_| ServiceError::Invalid("invalid command signature"))?;
        check(
            trust.principals.contains_key(&self.signer),
            "unregistered discovery principal",
        )
    }
}

/// Constructed by the authenticated XLMP resolver, never deserialized from a discovery request.
#[derive(Clone, Debug)]
pub struct ResolvedDiscoveryEvidence {
    pub job_id: xlemma_core::JobId,
    pub claim_id: ClaimId,
    pub artifact_id: ArtifactId,
    pub profile_id: PolicyId,
    pub class: VerificationProfileClass,
    pub evidence_roots: BTreeMap<VerificationEvidenceKind, ArtifactId>,
    pub status: DeclaredEvidenceStatus,
    pub final_after: DateTime<Utc>,
    pub verifier_clusters: BTreeSet<OperatorClusterId>,
    pub certificate_digest: String,
    pub observation_digest: String,
}

pub trait DiscoveryEvidenceSource {
    fn resolve(
        &self,
        id: &MessageId,
        at: DateTime<Utc>,
    ) -> Result<ResolvedDiscoveryEvidence, ServiceError>;
}

#[derive(Clone, Debug, Serialize)]
pub struct SettlementItem {
    pub completed_review: bool,
    pub work_root: String,
    pub category: RewardCategory,
    pub recipient: String,
    pub amount_units: u64,
    pub certificate_digest: String,
    pub claim_digest: String,
    pub artifact_digest: String,
    pub policy_digest: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct DiscoverySettlementPlan {
    pub plan_id: ReceiptId,
    pub round_id: DiscoveryRoundId,
    pub chain_id: u64,
    pub escrow_address: String,
    pub usdc_asset: String,
    pub items: Vec<SettlementItem>,
    pub total_units: u64,
    pub allocation_evidence_root: ReceiptId,
}

#[derive(Clone, Debug, Serialize)]
pub struct QueueEntry {
    pub submission_id: ReceiptId,
    pub researcher_id: ResearcherId,
    pub assisted: bool,
    pub committed_at: DateTime<Utc>,
    pub revealed_at: DateTime<Utc>,
    pub consent_count: usize,
    pub assessment_count: usize,
    pub evidence_attached: bool,
}

#[derive(Clone)]
struct Admitted {
    submission: ResearchSubmission,
    owner: ResearcherId,
    committed_at: DateTime<Utc>,
    revealed_at: DateTime<Utc>,
    consents: BTreeSet<ResearcherId>,
    evidence: Option<MessageId>,
    completed_reproduction: Option<ResolvedDiscoveryEvidence>,
    assessments: BTreeMap<OperatorClusterId, RewardAssessment>,
    assessor_signers: BTreeMap<OperatorClusterId, String>,
    materialized: bool,
    requires_revalidation: bool,
}

#[derive(Clone)]
struct LiveRound {
    creator_cluster: OperatorClusterId,
    calibration_approvals: BTreeSet<OperatorClusterId>,
    policy: ServiceRoundPolicy,
    funding: BTreeMap<ReceiptId, (DiscoveryFunding, BTreeSet<OperatorClusterId>)>,
    opened: bool,
    commitments: BTreeMap<(ResearcherId, ReceiptId), DateTime<Utc>>,
    submissions: BTreeMap<ReceiptId, Admitted>,
    events: Vec<DiscoveryEvent>,
    resolutions: BTreeMap<ReceiptId, (DiscoveryRemedy, ArtifactId, BTreeSet<OperatorClusterId>)>,
    resolution_signers: BTreeMap<ReceiptId, BTreeMap<OperatorClusterId, String>>,
    process_appeals: BTreeMap<ReceiptId, ProcessAppeal>,
    appeal_dissent: BTreeSet<ReceiptId>,
    plan: Option<DiscoverySettlementPlan>,
    expired: bool,
    settlement: Option<(String, String, BTreeSet<OperatorClusterId>)>,
    chain_expiry: Option<(String, String, BTreeSet<OperatorClusterId>)>,
    reserved_groups: BTreeSet<ContributionGroupId>,
    reserved_reproduction_work: BTreeSet<ReceiptId>,
    reserved_claims: BTreeMap<RewardCategory, BTreeSet<ClaimId>>,
    reservations_released: bool,
}

#[derive(Clone)]
struct ProcessAppeal {
    submission_id: ReceiptId,
    appellant_cluster: OperatorClusterId,
    resolved: bool,
}

#[derive(Clone, Default)]
pub struct DiscoveryLedger {
    trust: Option<DiscoveryTrust>,
    rounds: BTreeMap<DiscoveryRoundId, LiveRound>,
    consumed_commands: BTreeSet<ReceiptId>,
    last_received_at: Option<DateTime<Utc>>,
    nonces: BTreeSet<(String, String)>,
    history: BTreeMap<DiscoveryRoundId, Vec<DiscoveryEnvelope>>,
    settlement_rounds: BTreeMap<ReceiptId, DiscoveryRoundId>,
    rewarded_groups: BTreeSet<ContributionGroupId>,
    reproduction_work: BTreeSet<ReceiptId>,
    rewarded_claims: BTreeMap<RewardCategory, BTreeSet<ClaimId>>,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("{0}")]
    Invalid(&'static str),
    #[error(transparent)]
    Identity(#[from] xlemma_core::IdError),
    #[error(transparent)]
    Canonical(#[from] xlemma_core::CanonicalizationError),
    #[error(transparent)]
    Economics(#[from] crate::DiscoveryError),
}
fn check(ok: bool, why: &'static str) -> Result<(), ServiceError> {
    if ok {
        Ok(())
    } else {
        Err(ServiceError::Invalid(why))
    }
}
fn address(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|b| b.is_ascii_hexdigit())
        && value[2..].bytes().any(|b| b != b'0')
}
fn seconds(at: DateTime<Utc>) -> Result<u64, ServiceError> {
    u64::try_from(at.timestamp()).map_err(|_| ServiceError::Invalid("negative time"))
}
fn digest(id: &str) -> Result<String, ServiceError> {
    let hex = id.rsplit(':').next().unwrap_or("");
    check(
        hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid digest",
    )?;
    Ok(format!("0x{hex}"))
}

impl DiscoveryLedger {
    pub fn new(trust: DiscoveryTrust) -> Result<Self, ServiceError> {
        trust.validate()?;
        Ok(Self {
            trust: Some(trust),
            ..Self::default()
        })
    }
    pub fn history(&self, id: &DiscoveryRoundId) -> Result<&[DiscoveryEnvelope], ServiceError> {
        check(self.rounds.contains_key(id), "unknown round")?;
        Ok(self.history.get(id).map(Vec::as_slice).unwrap_or_default())
    }
    pub fn evidence_publication(
        &self,
        round_id: &DiscoveryRoundId,
        submission_id: &ReceiptId,
        source: &impl DiscoveryEvidenceSource,
        at: DateTime<Utc>,
    ) -> Result<serde_json::Value, ServiceError> {
        let round = self
            .rounds
            .get(round_id)
            .ok_or(ServiceError::Invalid("unknown round"))?;
        let s = round
            .submissions
            .get(submission_id)
            .ok_or(ServiceError::Invalid("unknown submission"))?;
        let e = source.resolve(
            s.evidence
                .as_ref()
                .ok_or(ServiceError::Invalid("no authenticated evidence"))?,
            at,
        )?;
        check(!round.process_appeals.values().any(|a|&a.submission_id==submission_id && !a.resolved)
            && !round.events.iter().any(|event|match event {DiscoveryEvent::Appeal{appeal_id,submission_id:id,grounds:crate::AppealGround::Evidence,..} if id==submission_id=>
                !round.events.iter().any(|r|matches!(r,DiscoveryEvent::Resolve{appeal_id:resolved,..} if resolved==appeal_id)),_=>false}),"pending evidence or process appeal holds publication")?;
        validate_evidence(s, &e, &round.policy)?;
        check(
            !e.verifier_clusters.contains(&round.creator_cluster)
                && round.funding.values().all(|(f, _)| {
                    !e.verifier_clusters.contains(&f.donor_cluster)
                        && !e.verifier_clusters.contains(&f.administrator_cluster)
                }),
            "conflicted verification publication",
        )?;
        check(
            e.status == DeclaredEvidenceStatus::Supported && !s.requires_revalidation,
            "evidence publication held",
        )?;
        let producers = s
            .submission
            .contributors
            .iter()
            .map(|c| digest(c.operator_cluster_id.as_str()))
            .collect::<Result<BTreeSet<_>, _>>()?;
        Ok(
            serde_json::json!({"certificate":e.certificate_digest,"claim":digest(e.claim_id.as_str())?,"artifact":digest(e.artifact_id.as_str())?,
            "policy":digest(e.profile_id.as_str())?,"observation_root":e.observation_digest,"producer_clusters":producers,
            "challenge_ends_at":seconds(e.final_after)?,"verification_class":e.class,"certificate_message_id":s.evidence}),
        )
    }
    pub fn contains(&self, id: &ReceiptId) -> bool {
        self.consumed_commands.contains(id)
    }
    pub fn plan(&self, id: &DiscoveryRoundId) -> Option<&DiscoverySettlementPlan> {
        self.rounds.get(id)?.plan.as_ref()
    }
    pub fn queue(&self, id: &DiscoveryRoundId) -> Result<Vec<QueueEntry>, ServiceError> {
        let round = self
            .rounds
            .get(id)
            .ok_or(ServiceError::Invalid("unknown round"))?;
        let mut result: Vec<_> = round
            .submissions
            .iter()
            .map(|(id, s)| QueueEntry {
                submission_id: id.clone(),
                researcher_id: s.owner.clone(),
                assisted: s.submission.assisted,
                committed_at: s.committed_at,
                revealed_at: s.revealed_at,
                consent_count: s.consents.len(),
                assessment_count: s.assessments.len(),
                evidence_attached: s.evidence.is_some(),
            })
            .collect();
        result.sort_by(|a, b| {
            (a.committed_at, &a.submission_id).cmp(&(b.committed_at, &b.submission_id))
        });
        Ok(result)
    }
    pub fn overview(&self) -> Result<serde_json::Value, ServiceError> {
        let rounds: Vec<_> = self.rounds.iter().map(|(id,r)| {
            let confirmed: u128 = r.funding.values().filter(|(_, observers)| observers.len() >= 2)
                .map(|(f,_)| u128::from(f.gross_units)).sum();
            let work_commitments:u128=r.submissions.values().map(|s|u128::from(if s.completed_reproduction.is_some(){r.policy.reproduction_fee_units}else{0})
                +s.assessor_signers.len() as u128*u128::from(r.policy.economics.verification_fee_units/2)).sum::<u128>()
                +r.resolution_signers.values().map(|v|v.len() as u128*u128::from(r.policy.economics.appeal_fee_units/2)).sum::<u128>();
            let review_capacity:u128=r.policy.economics.budgets.values().map(|b|u128::from(b.verification_units)+u128::from(b.appeal_units)).sum();
            serde_json::json!({"maximum_work_commitments_units":work_commitments,"reserved_review_capacity_units":review_capacity,"funded_review_horizon":r.policy.economics.review_deadline,"round_id":id,"domain":r.policy.domain,"opened":r.opened,"expired":r.expired,
                "confirmed_funding_units":confirmed,"policy":r.policy,"budgets":r.policy.economics.budgets,
                "funding":r.funding.values().map(|(f,o)|serde_json::json!({"funding":f,"observer_clusters":o})).collect::<Vec<_>>(),
                "queue_size":r.submissions.len(),"maximum_submissions":r.policy.economics.maximum_submissions,
                "assisted_slots":r.policy.assisted_submission_slots,"verification_fee_units":r.policy.economics.verification_fee_units,
                "plan":r.plan,"settlement":r.settlement,"settlement_confirmed":r.settlement.as_ref().is_some_and(|(_,_,o)|o.len()>=2),
                "allocated_commitments_units":r.plan.as_ref().map(|p|p.total_units).unwrap_or(0),
                "retained_after_plan_units":confirmed.saturating_sub(u128::from(r.plan.as_ref().map(|p|p.total_units).unwrap_or(0))),
                "provisional":r.plan.is_none(),"unpaid_expiry":r.chain_expiry,"reservations_released":r.reservations_released,"submission_deadline":r.policy.economics.submissions_close_at,"appeal_deadline":r.policy.economics.appeals_close_at,"review_deadline":r.policy.economics.review_deadline})
        }).collect();
        Ok(
            serde_json::json!({"rounds":rounds,"rewarded_groups":self.rewarded_groups,
            "rewarded_claims":self.rewarded_claims,"reserved_reproduction_work":self.reproduction_work,"accepted_commands":self.consumed_commands.len()}),
        )
    }

    /// Atomic in-memory transaction; the API fsyncs the command before publishing this state.
    pub fn apply(
        &mut self,
        envelope: &DiscoveryEnvelope,
        at: DateTime<Utc>,
        evidence: &impl DiscoveryEvidenceSource,
    ) -> Result<(), ServiceError> {
        let mut next = self.clone();
        next.apply_inner(envelope, at, evidence)?;
        *self = next;
        Ok(())
    }
    fn apply_inner(
        &mut self,
        e: &DiscoveryEnvelope,
        at: DateTime<Utc>,
        source: &impl DiscoveryEvidenceSource,
    ) -> Result<(), ServiceError> {
        let trust = self
            .trust
            .clone()
            .ok_or(ServiceError::Invalid("discovery service disabled"))?;
        e.authenticate(&trust, at)?;
        check(
            self.last_received_at.is_none_or(|prior| at >= prior),
            "discovery clock moved backwards",
        )?;
        check(
            !self.contains(&e.command_id)
                && self.nonces.insert((e.signer.clone(), e.nonce.clone())),
            "command replay",
        )?;
        let principal = &trust.principals[&e.signer];
        let role = |r| {
            check(
                principal.roles.contains(&r),
                "discovery role not authorized",
            )
        };
        if let DiscoveryCommand::CreateRound { policy } = &e.command {
            role(DiscoveryRole::Administrator)?;
            policy.validate(&trust)?;
            check(
                seconds(at)? < policy.economics.opens_at,
                "policy must be committed before round opens",
            )?;
            let id = policy.id()?;
            check(!self.rounds.contains_key(&id), "round already exists")?;
            self.history.entry(id.clone()).or_default().push(e.clone());
            self.rounds.insert(
                id,
                LiveRound {
                    creator_cluster: principal.cluster_id.clone(),
                    calibration_approvals: BTreeSet::new(),
                    policy: policy.clone(),
                    funding: BTreeMap::new(),
                    opened: false,
                    commitments: BTreeMap::new(),
                    submissions: BTreeMap::new(),
                    events: vec![],
                    resolutions: BTreeMap::new(),
                    resolution_signers: BTreeMap::new(),
                    process_appeals: BTreeMap::new(),
                    appeal_dissent: BTreeSet::new(),
                    plan: None,
                    expired: false,
                    settlement: None,
                    chain_expiry: None,
                    reserved_groups: BTreeSet::new(),
                    reserved_reproduction_work: BTreeSet::new(),
                    reserved_claims: BTreeMap::new(),
                    reservations_released: false,
                },
            );
        } else {
            let round_id = match &e.command {
                DiscoveryCommand::ApproveCalibration { round_id }
                | DiscoveryCommand::ObserveFunding { round_id, .. }
                | DiscoveryCommand::OpenRound { round_id }
                | DiscoveryCommand::Commit { round_id, .. }
                | DiscoveryCommand::Reveal { round_id, .. }
                | DiscoveryCommand::Consent { round_id, .. }
                | DiscoveryCommand::AttachEvidence { round_id, .. }
                | DiscoveryCommand::Assess { round_id, .. }
                | DiscoveryCommand::Appeal { round_id, .. }
                | DiscoveryCommand::ResolveAppeal { round_id, .. }
                | DiscoveryCommand::Finalize { round_id }
                | DiscoveryCommand::Expire { round_id }
                | DiscoveryCommand::ObserveSettlement { round_id, .. }
                | DiscoveryCommand::ObserveExpiry { round_id, .. } => round_id,
                DiscoveryCommand::CreateRound { .. } => unreachable!(),
            };
            self.history
                .entry(round_id.clone())
                .or_default()
                .push(e.clone());
            let round = self
                .rounds
                .get_mut(round_id)
                .ok_or(ServiceError::Invalid("unknown round"))?;
            check(
                (!round.expired && round.plan.is_none())
                    || matches!(
                        e.command,
                        DiscoveryCommand::ObserveSettlement { .. }
                            | DiscoveryCommand::ObserveExpiry { .. }
                    ),
                "round closed",
            )?;
            let now = seconds(at)?;
            match &e.command {
                DiscoveryCommand::ApproveCalibration { .. } => {
                    role(DiscoveryRole::Assessor)?;
                    check(
                        principal.cluster_id != round.creator_cluster,
                        "round administrator cannot assess",
                    )?;
                    check(
                        !round.opened
                            && now < round.policy.economics.opens_at
                            && principal.cluster_id != round.creator_cluster
                            && round
                                .policy
                                .economics
                                .assessors
                                .contains(&principal.cluster_id),
                        "calibration approval conflict or closed policy",
                    )?;
                    check(
                        round
                            .calibration_approvals
                            .insert(principal.cluster_id.clone()),
                        "calibration approval replay",
                    )?;
                }
                DiscoveryCommand::ObserveFunding {
                    receipt,
                    category,
                    donor_cluster,
                    administrator_cluster,
                    ..
                } => {
                    role(DiscoveryRole::FundingObserver)?;
                    check(
                        !round.opened && now < round.policy.economics.opens_at,
                        "funding fixed before opening",
                    )?;
                    check(
                        round.funding.contains_key(&receipt.settlement_receipt_id)
                            || round.funding.len() < 1000,
                        "funding receipt capacity exhausted",
                    )?;
                    receipt
                        .validate_integrity()
                        .map_err(|_| ServiceError::Invalid("invalid funding receipt"))?;
                    check(
                        receipt.settled_at <= at
                            && receipt.destination_vault == trust.escrow_address
                            && receipt.policy_id == PolicyId::derive(&round.policy)?
                            && receipt.settled_amount.asset == trust.usdc_asset
                            && receipt.settled_amount.decimals == 6
                            && receipt.settled_amount.units <= u128::from(SAFE)
                            && !receipt.related_party,
                        "funding settlement binding mismatch",
                    )?;
                    donor_cluster.validate()?;
                    administrator_cluster.validate()?;
                    check(
                        &principal.cluster_id != donor_cluster
                            && &principal.cluster_id != administrator_cluster,
                        "funding observer conflict",
                    )?;
                    check(
                        self.settlement_rounds
                            .get(&receipt.settlement_receipt_id)
                            .is_none_or(|r| r == round_id),
                        "settlement already funds another round",
                    )?;
                    let f = DiscoveryFunding {
                        settlement_receipt_id: receipt.settlement_receipt_id.clone(),
                        donor_cluster: donor_cluster.clone(),
                        administrator_cluster: administrator_cluster.clone(),
                        category: *category,
                        gross_units: receipt.settled_amount.units as u64,
                        administrator_fee_units: 0,
                        mandate_root: ArtifactId::derive(receipt)?,
                    };
                    let entry = round
                        .funding
                        .entry(receipt.settlement_receipt_id.clone())
                        .or_insert_with(|| (f.clone(), BTreeSet::new()));
                    check(
                        canonical_json_bytes(&entry.0)? == canonical_json_bytes(&f)?,
                        "conflicting funding observations",
                    )?;
                    check(
                        entry.1.insert(principal.cluster_id.clone()),
                        "repeated funding observer",
                    )?;
                    self.settlement_rounds
                        .insert(receipt.settlement_receipt_id.clone(), round_id.clone());
                }
                DiscoveryCommand::OpenRound { .. } => {
                    role(DiscoveryRole::Administrator)?;
                    check(
                        !round.opened
                            && now >= round.policy.economics.opens_at
                            && now < round.policy.economics.submissions_close_at,
                        "outside opening window",
                    )?;
                    check(
                        round.calibration_approvals.len() >= 2
                            && round.funding.values().all(|(f, _)| {
                                !round.calibration_approvals.contains(&f.donor_cluster)
                                    && !round
                                        .calibration_approvals
                                        .contains(&f.administrator_cluster)
                            }),
                        "calibration lacks independent approval",
                    )?;
                    calculate(round, &self.rewarded_groups, &self.rewarded_claims)?;
                    round.opened = true;
                }
                DiscoveryCommand::Commit { commitment, .. } => {
                    role(DiscoveryRole::Researcher)?;
                    commitment.validate()?;
                    check(
                        round.opened && now < round.policy.economics.submissions_close_at,
                        "commit window closed",
                    )?;
                    check(
                        round
                            .commitments
                            .keys()
                            .filter(|(r, _)| r == &principal.researcher_id)
                            .count()
                            < round.policy.per_researcher_submission_cap as usize,
                        "researcher commitment capacity exhausted",
                    )?;
                    check(
                        round
                            .commitments
                            .insert((principal.researcher_id.clone(), commitment.clone()), at)
                            .is_none(),
                        "commitment replay",
                    )?;
                }
                DiscoveryCommand::Reveal {
                    submission, salt, ..
                } => {
                    role(DiscoveryRole::Researcher)?;
                    check(
                        round.opened && now < round.policy.economics.submissions_close_at,
                        "reveal window closed",
                    )?;
                    submission.claim_id.validate()?;
                    submission.artifact_id.validate()?;
                    submission.profile_id.validate()?;
                    submission.manifest_root.validate()?;
                    submission.research_context_root.validate()?;
                    let committed_at = *round
                        .commitments
                        .get(&(
                            principal.researcher_id.clone(),
                            submission.commitment(round_id, salt)?,
                        ))
                        .ok_or(ServiceError::Invalid(
                            "reveal has no matching owner commitment",
                        ))?;
                    check(committed_at < at, "commit must precede reveal")?;
                    let id = submission.id(round_id)?;
                    check(
                        !round.submissions.contains_key(&id)
                            && round.submissions.len()
                                < round.policy.economics.maximum_submissions as usize,
                        "submission replay or full queue",
                    )?;
                    if !submission.assisted {
                        check(
                            round
                                .submissions
                                .values()
                                .filter(|s| !s.submission.assisted)
                                .count()
                                < (round.policy.economics.maximum_submissions
                                    - round.policy.assisted_submission_slots)
                                    as usize,
                            "remaining capacity reserved for assisted applicants",
                        )?;
                    }
                    let mut shares = 0u32;
                    let mut seen = BTreeSet::new();
                    check(
                        submission.contributors.len()
                            <= usize::from(round.policy.max_contributors_per_submission),
                        "contributor capacity exceeded",
                    )?;
                    for c in &submission.contributors {
                        let identity = trust
                            .principals
                            .values()
                            .find(|p| p.researcher_id == c.researcher_id)
                            .ok_or(ServiceError::Invalid("unregistered contributor"))?;
                        check(
                            identity.cluster_id == c.operator_cluster_id
                                && c.share_bps > 0
                                && seen.insert(&c.researcher_id),
                            "contribution identity mismatch",
                        )?;
                        check(
                            !round.calibration_approvals.contains(&c.operator_cluster_id),
                            "producer cannot calibrate its own reward",
                        )?;
                        shares += u32::from(c.share_bps);
                    }
                    check(
                        shares == 10_000 && seen.contains(&principal.researcher_id),
                        "invalid contributor agreement",
                    )?;
                    check(
                        round
                            .policy
                            .economics
                            .budgets
                            .contains_key(&submission.category)
                            && round
                                .policy
                                .economics
                                .profiles
                                .iter()
                                .any(|p| p.policy_id == submission.profile_id),
                        "unsupported category or profile",
                    )?;
                    let admitted_in_category = round
                        .submissions
                        .values()
                        .filter(|s| s.submission.category == submission.category)
                        .count() as u128;
                    check(
                        (admitted_in_category + 1)
                            * (u128::from(round.policy.economics.verification_fee_units)
                                + u128::from(round.policy.reproduction_fee_units))
                            <= u128::from(
                                round.policy.economics.budgets[&submission.category]
                                    .verification_units,
                            ),
                        "category has no funded review capacity",
                    )?;
                    round.submissions.insert(
                        id,
                        Admitted {
                            submission: submission.clone(),
                            owner: principal.researcher_id.clone(),
                            committed_at,
                            revealed_at: at,
                            consents: BTreeSet::from([principal.researcher_id.clone()]),
                            evidence: None,
                            completed_reproduction: None,
                            assessments: BTreeMap::new(),
                            assessor_signers: BTreeMap::new(),
                            materialized: false,
                            requires_revalidation: false,
                        },
                    );
                }
                DiscoveryCommand::Consent { submission_id, .. } => {
                    let s = round
                        .submissions
                        .get_mut(submission_id)
                        .ok_or(ServiceError::Invalid("unknown submission"))?;
                    check(
                        s.submission
                            .contributors
                            .iter()
                            .any(|c| c.researcher_id == principal.researcher_id),
                        "not a contributor",
                    )?;
                    check(
                        !s.materialized && s.consents.insert(principal.researcher_id.clone()),
                        "consent replay or assessment locked",
                    )?;
                }
                DiscoveryCommand::AttachEvidence {
                    submission_id,
                    certificate_message_id,
                    ..
                } => {
                    let s = round
                        .submissions
                        .get_mut(submission_id)
                        .ok_or(ServiceError::Invalid("unknown submission"))?;
                    check(
                        s.owner == principal.researcher_id
                            && !s.materialized
                            && s.evidence.is_none(),
                        "evidence binding already fixed or wrong owner",
                    )?;
                    let resolved = source.resolve(certificate_message_id, at)?;
                    validate_evidence(s, &resolved, &round.policy)?;
                    check(
                        !resolved.verifier_clusters.contains(&round.creator_cluster)
                            && round.funding.values().all(|(f, _)| {
                                !resolved.verifier_clusters.contains(&f.donor_cluster)
                                    && !resolved
                                        .verifier_clusters
                                        .contains(&f.administrator_cluster)
                            }),
                        "funded verifier conflict",
                    )?;
                    check(
                        resolved.verifier_clusters.len()
                            <= usize::from(round.policy.maximum_verifier_clusters),
                        "reproduction exceeds funded operator capacity",
                    )?;
                    for cluster in &resolved.verifier_clusters {
                        check(
                            trust
                                .principals
                                .values()
                                .filter(|p| {
                                    p.cluster_id == *cluster
                                        && p.roles.contains(&DiscoveryRole::Verifier)
                                })
                                .count()
                                == 1,
                            "missing unique registered verifier payee",
                        )?;
                    }
                    s.completed_reproduction = Some(resolved);
                    s.evidence = Some(certificate_message_id.clone());
                }
                DiscoveryCommand::Assess { assessment, .. } => {
                    role(DiscoveryRole::Assessor)?;
                    check(
                        principal.cluster_id != round.creator_cluster,
                        "round administrator cannot assess",
                    )?;
                    let s = round
                        .submissions
                        .get_mut(&assessment.submission_id)
                        .ok_or(ServiceError::Invalid("unknown submission"))?;
                    check(
                        !s.materialized
                            && !s.requires_revalidation
                            && s.assessments.len() < 2
                            && now < round.policy.economics.submissions_close_at,
                        "assessment window closed",
                    )?;
                    check(
                        s.consents.len() == s.submission.contributors.len(),
                        "missing contributor signatures",
                    )?;
                    assessment.group_id.validate()?;
                    assessment.prior_art_root.validate()?;
                    assessment.additional_contribution_root.validate()?;
                    if let Some(root) = &assessment.independent_discovery_root {
                        root.validate()?;
                    }
                    assessment.reasons_root.validate()?;
                    check(
                        round
                            .policy
                            .economics
                            .assessors
                            .contains(&principal.cluster_id)
                            && !s
                                .submission
                                .contributors
                                .iter()
                                .any(|c| c.operator_cluster_id == principal.cluster_id),
                        "assessor conflict",
                    )?;
                    check(
                        round
                            .policy
                            .calibration
                            .contains_key(&assessment.calibration_tier),
                        "unknown calibration tier",
                    )?;
                    check(
                        round.funding.values().all(|(f, _)| {
                            f.donor_cluster != principal.cluster_id
                                && f.administrator_cluster != principal.cluster_id
                        }),
                        "funded assessor conflict",
                    )?;
                    let resolved = source.resolve(
                        s.evidence
                            .as_ref()
                            .ok_or(ServiceError::Invalid("missing authenticated evidence"))?,
                        at,
                    )?;
                    validate_evidence(s, &resolved, &round.policy)?;
                    check(
                        !resolved.verifier_clusters.contains(&round.creator_cluster)
                            && round.funding.values().all(|(f, _)| {
                                !resolved.verifier_clusters.contains(&f.donor_cluster)
                                    && !resolved
                                        .verifier_clusters
                                        .contains(&f.administrator_cluster)
                            }),
                        "funded verifier conflict",
                    )?;
                    check(
                        s.assessments
                            .insert(principal.cluster_id.clone(), assessment.clone())
                            .is_none(),
                        "assessor replay",
                    )?;
                    s.assessor_signers
                        .insert(principal.cluster_id.clone(), e.signer.clone());
                    if s.assessments.len() >= 2 && s.assessments.values().all(|a| a == assessment) {
                        let resolved = source.resolve(
                            s.evidence
                                .as_ref()
                                .ok_or(ServiceError::Invalid("missing authenticated evidence"))?,
                            at,
                        )?;
                        validate_evidence(s, &resolved, &round.policy)?;
                        check(
                            !resolved.verifier_clusters.contains(&round.creator_cluster)
                                && round.funding.values().all(|(f, _)| {
                                    !resolved.verifier_clusters.contains(&f.donor_cluster)
                                        && !resolved
                                            .verifier_clusters
                                            .contains(&f.administrator_cluster)
                                }),
                            "funded verifier conflict",
                        )?;
                        let tier = round
                            .policy
                            .calibration
                            .get(&assessment.calibration_tier)
                            .ok_or(ServiceError::Invalid("unknown calibration tier"))?;
                        let weight = (u128::from(tier.reference_cost_units)
                            * u128::from(10_000 - tier.uncertainty_bps)
                            / 10_000)
                            .min(u128::from(
                                round.policy.economics.budgets[&s.submission.category]
                                    .maximum_weight,
                            )) as u64;
                        round.events.push(DiscoveryEvent::Submit {
                            at: now,
                            submission: DiscoverySubmission {
                                submission_id: assessment.submission_id.clone(),
                                group_id: assessment.group_id.clone(),
                                category: s.submission.category,
                                claims: BTreeSet::from([s.submission.claim_id.clone()]),
                                contributors: s.submission.contributors.clone(),
                                contribution_manifest_root: s.submission.manifest_root.clone(),
                                profile_id: s.submission.profile_id.clone(),
                                evidence_roots: s.submission.evidence_roots.clone(),
                                declared_evidence_status: resolved.status,
                                reward_eligible: assessment.eligible,
                                assessed_weight: weight,
                                assessment_root: ArtifactId::derive(assessment)?,
                                assessors: s.assessments.keys().cloned().collect(),
                                reported_tokens: 0,
                                reported_compute_units: 0,
                                registered_study: s.submission.registered_study.clone(),
                            },
                        });
                        s.materialized = true;
                        calculate(round, &self.rewarded_groups, &self.rewarded_claims)?;
                    }
                }
                DiscoveryCommand::Appeal {
                    submission_id,
                    grounds,
                    evidence_root,
                    ..
                } => {
                    let s = round
                        .submissions
                        .get(submission_id)
                        .ok_or(ServiceError::Invalid("unknown appealed submission"))?;
                    evidence_root.validate()?;
                    let category = s.submission.category;
                    let native_appeals = round
                        .events
                        .iter()
                        .filter(|event| matches!(event, DiscoveryEvent::Appeal { .. }))
                        .count();
                    check(
                        now < round.policy.economics.appeals_close_at
                            && native_appeals + round.process_appeals.len()
                                < round.policy.economics.maximum_appeals as usize,
                        "appeal window or capacity exhausted",
                    )?;
                    let category_appeals = round
                        .events
                        .iter()
                        .filter(|event| match event {
                            DiscoveryEvent::Appeal { submission_id, .. } => {
                                round.submissions[submission_id].submission.category == category
                            }
                            _ => false,
                        })
                        .count()
                        + round
                            .process_appeals
                            .values()
                            .filter(|a| {
                                round.submissions[&a.submission_id].submission.category == category
                            })
                            .count();
                    check(
                        (category_appeals as u128 + 1)
                            * u128::from(round.policy.economics.appeal_fee_units)
                            <= u128::from(round.policy.economics.budgets[&category].appeal_units),
                        "appeal reserve exhausted",
                    )?;
                    if s.materialized {
                        round.events.push(DiscoveryEvent::Appeal {
                            at: now,
                            appeal_id: e.command_id.clone(),
                            submission_id: submission_id.clone(),
                            appellant: principal.researcher_id.clone(),
                            appellant_cluster: principal.cluster_id.clone(),
                            grounds: *grounds,
                            evidence_root: evidence_root.clone(),
                        });
                        calculate(round, &self.rewarded_groups, &self.rewarded_claims)?;
                    } else {
                        check(
                            matches!(
                                grounds,
                                crate::AppealGround::Process | crate::AppealGround::Evidence
                            ),
                            "awaiting assessment: use process or evidence appeal",
                        )?;
                        check(
                            !round.process_appeals.values().any(|a| {
                                a.submission_id == *submission_id
                                    && a.appellant_cluster == principal.cluster_id
                            }),
                            "repeated process appeal",
                        )?;
                        round.process_appeals.insert(
                            e.command_id.clone(),
                            ProcessAppeal {
                                submission_id: submission_id.clone(),
                                appellant_cluster: principal.cluster_id.clone(),
                                resolved: false,
                            },
                        );
                    }
                }
                DiscoveryCommand::ResolveAppeal {
                    appeal_id,
                    reasons_root,
                    remedy,
                    ..
                } => {
                    role(DiscoveryRole::AppealReviewer)?;
                    check(
                        principal.cluster_id != round.creator_cluster,
                        "round administrator cannot review appeals",
                    )?;
                    reasons_root.validate()?;
                    validate_review(round, appeal_id, principal, remedy, &trust)?;
                    check(
                        now <= round.policy.economics.review_deadline
                            && round
                                .policy
                                .economics
                                .appeal_reviewers
                                .contains(&principal.cluster_id),
                        "appeal review unavailable",
                    )?;
                    if let Some(appeal) = round.process_appeals.get(appeal_id) {
                        let s = &round.submissions[&appeal.submission_id];
                        check(
                            !appeal.resolved
                                && appeal.appellant_cluster != principal.cluster_id
                                && !s.assessments.contains_key(&principal.cluster_id)
                                && !s
                                    .submission
                                    .contributors
                                    .iter()
                                    .any(|c| c.operator_cluster_id == principal.cluster_id)
                                && round.funding.values().all(|(f, _)| {
                                    f.donor_cluster != principal.cluster_id
                                        && f.administrator_cluster != principal.cluster_id
                                }),
                            "process appeal reviewer conflict",
                        )?;
                        check(
                            matches!(
                                remedy,
                                DiscoveryRemedy::Uphold | DiscoveryRemedy::RequireRevalidation
                            ),
                            "process appeal cannot manufacture a reward or proof",
                        )?;
                    }
                    let entry = round
                        .resolutions
                        .entry(appeal_id.clone())
                        .or_insert_with(|| (remedy.clone(), reasons_root.clone(), BTreeSet::new()));
                    check(entry.2.len() < 2, "appeal panel capacity exhausted")?;
                    if canonical_json_bytes(&entry.0)? != canonical_json_bytes(remedy)?
                        || &entry.1 != reasons_root
                    {
                        round.appeal_dissent.insert(appeal_id.clone());
                    }
                    check(
                        entry.2.insert(principal.cluster_id.clone()),
                        "appeal reviewer replay",
                    )?;
                    round
                        .resolution_signers
                        .entry(appeal_id.clone())
                        .or_default()
                        .insert(principal.cluster_id.clone(), e.signer.clone());
                    if entry.2.len() >= 2 && !round.appeal_dissent.contains(appeal_id) {
                        if let Some(appeal) = round.process_appeals.get_mut(appeal_id) {
                            appeal.resolved = true;
                            if matches!(remedy, DiscoveryRemedy::RequireRevalidation) {
                                let s = round.submissions.get_mut(&appeal.submission_id).unwrap();
                                s.requires_revalidation = true;
                            }
                        } else {
                            round.events.push(DiscoveryEvent::Resolve {
                                at: now,
                                appeal_id: appeal_id.clone(),
                                reviewers: entry.2.clone(),
                                reasons_root: reasons_root.clone(),
                                remedy: remedy.clone(),
                            });
                            calculate(round, &self.rewarded_groups, &self.rewarded_claims)?;
                            if matches!(remedy, DiscoveryRemedy::RequireRevalidation) {
                                let id = round
                                    .events
                                    .iter()
                                    .find_map(|e| match e {
                                        DiscoveryEvent::Appeal {
                                            appeal_id: a,
                                            submission_id,
                                            ..
                                        } if a == appeal_id => Some(submission_id.clone()),
                                        _ => None,
                                    })
                                    .ok_or(ServiceError::Invalid("missing appealed submission"))?;
                                round
                                    .submissions
                                    .get_mut(&id)
                                    .unwrap()
                                    .requires_revalidation = true;
                            }
                        }
                    }
                }
                DiscoveryCommand::Finalize { .. } => {
                    role(DiscoveryRole::Administrator)?;
                    check(round.opened, "round not open")?;
                    check(
                        round.process_appeals.values().all(|a| a.resolved),
                        "unresolved process appeal holds allocation",
                    )?;
                    round.events.push(DiscoveryEvent::Finalize { at: now });
                    let report = calculate(round, &self.rewarded_groups, &self.rewarded_claims)?;
                    let mut items = vec![];
                    let allocations = simultaneous_allocations(round, &report)?;
                    for allocation in &allocations {
                        let s = &round.submissions[&allocation.submission_id];
                        let resolved = source.resolve(s.evidence.as_ref().unwrap(), at)?;
                        validate_evidence(s, &resolved, &round.policy)?;
                        check(
                            !resolved.verifier_clusters.contains(&round.creator_cluster)
                                && round.funding.values().all(|(f, _)| {
                                    !resolved.verifier_clusters.contains(&f.donor_cluster)
                                        && !resolved
                                            .verifier_clusters
                                            .contains(&f.administrator_cluster)
                                }),
                            "funded verifier conflict",
                        )?;
                        check(
                            resolved.final_after <= at
                                && resolved.status == DeclaredEvidenceStatus::Supported,
                            "evidence not final or quarantined",
                        )?;
                        for (researcher, amount) in &allocation.contributor_units {
                            if *amount == 0 {
                                continue;
                            }
                            let recipient_identity = trust
                                .principals
                                .values()
                                .find(|p| &p.researcher_id == researcher)
                                .ok_or(ServiceError::Invalid("missing payout identity"))?;
                            check(
                                !resolved
                                    .verifier_clusters
                                    .contains(&recipient_identity.cluster_id),
                                "corrected contributor cannot self-certify",
                            )?;
                            let recipient = &recipient_identity.payout_address;
                            items.push(SettlementItem {
                                completed_review: false,
                                work_root: digest(
                                    ArtifactId::derive(&(
                                        "awarded-contributors-v1",
                                        &s.submission.manifest_root,
                                        &allocation.contributor_units,
                                        &report.event_receipts,
                                    ))?
                                    .as_str(),
                                )?,
                                category: allocation.category,
                                recipient: recipient.clone(),
                                amount_units: *amount,
                                certificate_digest: resolved.certificate_digest.clone(),
                                claim_digest: digest(s.submission.claim_id.as_str())?,
                                artifact_digest: digest(s.submission.artifact_id.as_str())?,
                                policy_digest: digest(resolved.profile_id.as_str())?,
                            });
                        }
                        round.reserved_groups.insert(allocation.group_id.clone());
                        round
                            .reserved_claims
                            .entry(allocation.category)
                            .or_default()
                            .insert(s.submission.claim_id.clone());
                        self.rewarded_groups.insert(allocation.group_id.clone());
                        self.rewarded_claims
                            .entry(allocation.category)
                            .or_default()
                            .insert(s.submission.claim_id.clone());
                    }
                    finish_plan(
                        round,
                        round_id,
                        &trust,
                        &report.event_receipts,
                        items,
                        &mut self.reproduction_work,
                    )?;
                }
                DiscoveryCommand::Expire { .. } => {
                    check(
                        now.saturating_add(3600) < round.policy.settlement_expires_at,
                        "onchain settlement grace exhausted",
                    )?;
                    round.events.push(DiscoveryEvent::Expire { at: now });
                    let report = calculate(round, &self.rewarded_groups, &self.rewarded_claims)?;
                    finish_plan(
                        round,
                        round_id,
                        &trust,
                        &report.event_receipts,
                        vec![],
                        &mut self.reproduction_work,
                    )?;
                    round.expired = true;
                }
                DiscoveryCommand::ObserveExpiry {
                    transaction_hash,
                    block_hash,
                    ..
                } => {
                    role(DiscoveryRole::SettlementObserver)?;
                    check(
                        now >= round.policy.settlement_expires_at
                            && !round.reservations_released
                            && !round
                                .settlement
                                .as_ref()
                                .is_some_and(|(_, _, o)| o.len() >= 2),
                        "round not eligible for confirmed unpaid expiry",
                    )?;
                    check(
                        chain_hash(transaction_hash) && chain_hash(block_hash),
                        "invalid expiry receipt",
                    )?;
                    let observed = round.chain_expiry.get_or_insert_with(|| {
                        (
                            transaction_hash.clone(),
                            block_hash.clone(),
                            BTreeSet::new(),
                        )
                    });
                    check(
                        &observed.0 == transaction_hash
                            && &observed.1 == block_hash
                            && observed.2.insert(principal.cluster_id.clone()),
                        "conflicting or repeated expiry observation",
                    )?;
                    if observed.2.len() >= 2 {
                        round.reservations_released = true;
                        round.expired = true;
                    }
                }
                DiscoveryCommand::ObserveSettlement {
                    plan_id,
                    transaction_hash,
                    block_hash,
                    ..
                } => {
                    role(DiscoveryRole::SettlementObserver)?;
                    check(
                        !round.reservations_released,
                        "round expired without settlement",
                    )?;
                    check(
                        round.plan.as_ref().is_some_and(|p| p.total_units > 0),
                        "empty plan has no settlement",
                    )?;
                    check(
                        round.plan.as_ref().is_some_and(|p| &p.plan_id == plan_id),
                        "settlement plan mismatch",
                    )?;
                    check(
                        transaction_hash.len() == 66
                            && block_hash.len() == 66
                            && transaction_hash.starts_with("0x")
                            && block_hash.starts_with("0x")
                            && transaction_hash[2..].bytes().all(|b| b.is_ascii_hexdigit())
                            && block_hash[2..].bytes().all(|b| b.is_ascii_hexdigit()),
                        "invalid chain receipt",
                    )?;
                    let observed = round.settlement.get_or_insert_with(|| {
                        (
                            transaction_hash.clone(),
                            block_hash.clone(),
                            BTreeSet::new(),
                        )
                    });
                    check(
                        &observed.0 == transaction_hash
                            && &observed.1 == block_hash
                            && observed.2.insert(principal.cluster_id.clone()),
                        "conflicting or repeated settlement observation",
                    )?;
                }
                DiscoveryCommand::CreateRound { .. } => unreachable!(),
            }
        }
        if matches!(e.command, DiscoveryCommand::ObserveExpiry { .. }) {
            self.reproduction_work.clear();
            self.rewarded_groups.clear();
            self.rewarded_claims.clear();
            for r in self.rounds.values().filter(|r| !r.reservations_released) {
                self.rewarded_groups
                    .extend(r.reserved_groups.iter().cloned());
                self.reproduction_work
                    .extend(r.reserved_reproduction_work.iter().cloned());
                for (category, claims) in &r.reserved_claims {
                    self.rewarded_claims
                        .entry(*category)
                        .or_default()
                        .extend(claims.iter().cloned());
                }
            }
        }
        self.last_received_at = Some(at);
        self.consumed_commands.insert(e.command_id.clone());
        Ok(())
    }
}

fn work_item(
    category: RewardCategory,
    recipient: &str,
    amount_units: u64,
    root: &ArtifactId,
) -> Result<SettlementItem, ServiceError> {
    let zero = format!("0x{}", "0".repeat(64));
    Ok(SettlementItem {
        completed_review: true,
        work_root: digest(root.as_str())?,
        category,
        recipient: recipient.to_owned(),
        amount_units,
        certificate_digest: zero.clone(),
        claim_digest: zero.clone(),
        artifact_digest: zero.clone(),
        policy_digest: zero,
    })
}

fn validate_evidence(
    s: &Admitted,
    e: &ResolvedDiscoveryEvidence,
    policy: &ServiceRoundPolicy,
) -> Result<(), ServiceError> {
    e.job_id.validate()?;
    check(
        e.claim_id == s.submission.claim_id
            && e.artifact_id == s.submission.artifact_id
            && (if e.class == VerificationProfileClass::Formal {
                policy.formal_policy_bindings.get(&s.submission.profile_id) == Some(&e.profile_id)
            } else {
                e.profile_id == s.submission.profile_id
            })
            && e.evidence_roots == s.submission.evidence_roots
            && policy.economics.profiles.iter().any(|p| {
                p.policy_id == s.submission.profile_id
                    && p.class == e.class
                    && usize::from(p.minimum_independent_operators) <= e.verifier_clusters.len()
            }),
        "certificate does not bind exact submission",
    )?;
    check(
        e.verifier_clusters.len() >= 2
            && s.submission
                .contributors
                .iter()
                .all(|c| !e.verifier_clusters.contains(&c.operator_cluster_id)),
        "producer cannot independently certify its contribution",
    )
}
fn calculate(
    round: &LiveRound,
    groups: &BTreeSet<ContributionGroupId>,
    claims: &BTreeMap<RewardCategory, BTreeSet<ClaimId>>,
) -> Result<crate::DiscoverySimulationReport, ServiceError> {
    let priority = round
        .submissions
        .iter()
        .map(|(id, s)| (id.clone(), (s.committed_at, id.clone())))
        .collect::<BTreeMap<_, _>>();
    Ok(crate::discovery::simulate_discovery_ordered(
        &DiscoverySimulation {
            policy: round.policy.economics.clone(),
            funding: round
                .funding
                .values()
                .filter(|(_, o)| o.len() >= 2)
                .map(|(f, _)| f.clone())
                .collect(),
            pledged_units: 0,
            previously_rewarded_groups: groups.clone(),
            previously_rewarded_claims: claims.clone(),
            previously_consumed_settlements: BTreeSet::new(),
            events: round.events.clone(),
        },
        Some(&priority),
    )?)
}

fn finish_plan(
    round: &mut LiveRound,
    round_id: &DiscoveryRoundId,
    trust: &DiscoveryTrust,
    receipts: &[ReceiptId],
    mut items: Vec<SettlementItem>,
    reserved_work: &mut BTreeSet<ReceiptId>,
) -> Result<(), ServiceError> {
    for s in round.submissions.values() {
        if let Some(completed) = &s.completed_reproduction {
            let fee =
                round.policy.reproduction_fee_units / completed.verifier_clusters.len() as u64;
            for cluster in &completed.verifier_clusters {
                let work_id = ReceiptId::derive(&(
                    "reproduction-operator-work-v1",
                    &completed.job_id,
                    cluster,
                ))?;
                if !reserved_work.insert(work_id.clone()) {
                    continue;
                }
                round.reserved_reproduction_work.insert(work_id.clone());
                let work = ArtifactId::derive(&work_id)?;
                let payee = trust
                    .principals
                    .values()
                    .find(|p| {
                        p.cluster_id == *cluster && p.roles.contains(&DiscoveryRole::Verifier)
                    })
                    .ok_or(ServiceError::Invalid("missing verifier payee"))?;
                if fee > 0 {
                    items.push(work_item(
                        s.submission.category,
                        &payee.payout_address,
                        fee,
                        &work,
                    )?);
                }
            }
        }
    }
    for (id, s) in round
        .submissions
        .iter()
        .filter(|(_, s)| !s.assessor_signers.is_empty())
    {
        let fee = round.policy.economics.verification_fee_units / 2;
        for signer in s.assessor_signers.values() {
            if fee == 0 {
                continue;
            }
            items.push(work_item(
                s.submission.category,
                &trust.principals[signer].payout_address,
                fee,
                &ArtifactId::derive(&(
                    "completed-assessment",
                    round_id,
                    id,
                    &s.assessments,
                    signer,
                ))?,
            )?);
        }
    }
    for (appeal_id, (_, root, _reviewers)) in &round.resolutions {
        let category = round
            .events
            .iter()
            .find_map(|event| match event {
                DiscoveryEvent::Appeal {
                    appeal_id: id,
                    submission_id,
                    ..
                } if id == appeal_id => Some(round.submissions[submission_id].submission.category),
                _ => None,
            })
            .or_else(|| {
                round
                    .process_appeals
                    .get(appeal_id)
                    .map(|a| round.submissions[&a.submission_id].submission.category)
            })
            .ok_or(ServiceError::Invalid("missing appeal service record"))?;
        let fee = round.policy.economics.appeal_fee_units / 2;
        for signer in round.resolution_signers[appeal_id].values() {
            if fee > 0 {
                items.push(work_item(
                    category,
                    &trust.principals[signer].payout_address,
                    fee,
                    &ArtifactId::derive(&(
                        "completed-appeal-review",
                        round_id,
                        appeal_id,
                        signer,
                        root,
                    ))?,
                )?);
            }
        }
    }
    let total_units = items
        .iter()
        .try_fold(0u64, |total, item| total.checked_add(item.amount_units))
        .filter(|total| *total <= SAFE)
        .ok_or(ServiceError::Invalid("settlement amount overflow"))?;
    let root = ReceiptId::derive(&("discovery-allocation-v1", round_id, receipts, &items))?;
    let plan_id = ReceiptId::derive(&(
        "discovery-settlement-plan-v1",
        trust.chain_id,
        &trust.escrow_address,
        round_id,
        &items,
        &root,
    ))?;
    round.plan = Some(DiscoverySettlementPlan {
        plan_id,
        round_id: round_id.clone(),
        chain_id: trust.chain_id,
        escrow_address: trust.escrow_address.clone(),
        usdc_asset: trust.usdc_asset.clone(),
        items,
        total_units,
        allocation_evidence_root: root,
    });
    Ok(())
}

fn validate_review(
    round: &LiveRound,
    id: &ReceiptId,
    reviewer: &DiscoveryPrincipal,
    remedy: &DiscoveryRemedy,
    trust: &DiscoveryTrust,
) -> Result<(), ServiceError> {
    let (submission_id, appellant, grounds) = if let Some(a) = round.process_appeals.get(id) {
        check(!a.resolved, "appeal already resolved")?;
        (
            &a.submission_id,
            &a.appellant_cluster,
            crate::AppealGround::Process,
        )
    } else {
        check(
            !round
                .events
                .iter()
                .any(|e| matches!(e,DiscoveryEvent::Resolve{appeal_id,..} if appeal_id==id)),
            "appeal already resolved",
        )?;
        round
            .events
            .iter()
            .find_map(|e| match e {
                DiscoveryEvent::Appeal {
                    appeal_id,
                    submission_id,
                    appellant_cluster,
                    grounds,
                    ..
                } if appeal_id == id => Some((submission_id, appellant_cluster, *grounds)),
                _ => None,
            })
            .ok_or(ServiceError::Invalid("unknown appeal"))?
    };
    let s = &round.submissions[submission_id];
    check(
        !s.completed_reproduction
            .as_ref()
            .is_some_and(|e| e.verifier_clusters.contains(&reviewer.cluster_id)),
        "original verifier cannot review its own appeal",
    )?;
    check(
        appellant != &reviewer.cluster_id
            && !s.assessments.contains_key(&reviewer.cluster_id)
            && !s
                .submission
                .contributors
                .iter()
                .any(|c| c.operator_cluster_id == reviewer.cluster_id)
            && round.funding.values().all(|(f, _)| {
                f.donor_cluster != reviewer.cluster_id
                    && f.administrator_cluster != reviewer.cluster_id
            }),
        "appeal reviewer conflict",
    )?;
    match remedy {
        DiscoveryRemedy::CorrectReward {
            group_id, weight, ..
        } => {
            group_id.validate()?;
            check(
                grounds != crate::AppealGround::Evidence
                    && grounds != crate::AppealGround::Attribution
                    && *weight
                        <= round.policy.economics.budgets[&s.submission.category].maximum_weight,
                "incompatible reward remedy",
            )?;
        }
        DiscoveryRemedy::CorrectContributors {
            contributors,
            manifest_root,
        } => {
            manifest_root.validate()?;
            check(
                grounds == crate::AppealGround::Attribution
                    && contributors.len()
                        <= usize::from(round.policy.max_contributors_per_submission),
                "incompatible attribution remedy",
            )?;
            let mut seen = BTreeSet::new();
            let mut total = 0u32;
            for c in contributors {
                check(
                    trust.principals.values().any(|p| {
                        p.researcher_id == c.researcher_id && p.cluster_id == c.operator_cluster_id
                    }) && c.share_bps > 0
                        && seen.insert(&c.researcher_id)
                        && !s.assessments.contains_key(&c.operator_cluster_id)
                        && c.operator_cluster_id != reviewer.cluster_id,
                    "invalid corrected contributor",
                )?;
                total += u32::from(c.share_bps);
            }
            check(total == 10_000, "invalid corrected shares")?;
        }
        _ => {}
    }
    Ok(())
}

fn final_submission(
    round: &LiveRound,
    id: &ReceiptId,
) -> Result<DiscoverySubmission, ServiceError> {
    let mut submission = round
        .events
        .iter()
        .find_map(|e| match e {
            DiscoveryEvent::Submit { submission, .. } if &submission.submission_id == id => {
                Some(submission.clone())
            }
            _ => None,
        })
        .ok_or(ServiceError::Invalid("missing completed assessment"))?;
    for event in &round.events {
        if let DiscoveryEvent::Resolve {
            appeal_id, remedy, ..
        } = event
        {
            if !round.events.iter().any(|e|matches!(e,DiscoveryEvent::Appeal{appeal_id:a,submission_id,..} if a==appeal_id && submission_id==id)) {continue;}
            match remedy {
                DiscoveryRemedy::CorrectReward {
                    group_id,
                    eligible,
                    weight,
                } => {
                    submission.group_id = group_id.clone();
                    submission.reward_eligible = *eligible;
                    submission.assessed_weight = *weight;
                }
                DiscoveryRemedy::CorrectContributors {
                    contributors,
                    manifest_root,
                } => {
                    submission.contributors = contributors.clone();
                    submission.contribution_manifest_root = manifest_root.clone();
                }
                DiscoveryRemedy::RequireRevalidation => {
                    submission.declared_evidence_status = DeclaredEvidenceStatus::Inconclusive
                }
                DiscoveryRemedy::Uphold => {}
            }
        }
    }
    Ok(submission)
}

fn simultaneous_allocations(
    round: &LiveRound,
    report: &crate::DiscoverySimulationReport,
) -> Result<Vec<crate::DiscoveryAllocation>, ServiceError> {
    let mut result = vec![];
    for allocation in &report.allocations {
        let primary = &round.submissions[&allocation.submission_id];
        let mut independent = vec![final_submission(round, &allocation.submission_id)?];
        let mut clusters: BTreeSet<_> = independent[0]
            .contributors
            .iter()
            .map(|c| c.operator_cluster_id.clone())
            .collect();
        let mut candidates: Vec<_> = round.submissions.iter().collect();
        candidates.sort_by_key(|(id, s)| (s.committed_at, *id));
        for (id, s) in candidates {
            if id == &allocation.submission_id
                || !s.materialized
                || s.requires_revalidation
                || report.excluded_submissions.get(id).map(String::as_str)
                    != Some("already_accounted_contribution")
                || s.committed_at >= primary.revealed_at
                || seconds(s.committed_at)?.saturating_sub(seconds(primary.committed_at)?)
                    > round.policy.simultaneous_window_seconds
                || !s
                    .assessments
                    .values()
                    .all(|a| a.independent_discovery_root.is_some())
            {
                continue;
            }
            let candidate = final_submission(round, id)?;
            if candidate.group_id != allocation.group_id
                || candidate.category != allocation.category
                || !candidate.reward_eligible
                || candidate.declared_evidence_status != DeclaredEvidenceStatus::Supported
                || candidate
                    .contributors
                    .iter()
                    .any(|c| clusters.contains(&c.operator_cluster_id))
            {
                continue;
            }
            clusters.extend(
                candidate
                    .contributors
                    .iter()
                    .map(|c| c.operator_cluster_id.clone()),
            );
            independent.push(candidate);
        }
        let share = allocation.gross_units / independent.len() as u64;
        for s in independent {
            let contributor_units: BTreeMap<_, _> = s
                .contributors
                .iter()
                .map(|c| {
                    (
                        c.researcher_id.clone(),
                        (u128::from(share) * u128::from(c.share_bps) / 10_000) as u64,
                    )
                })
                .collect();
            let paid: u64 = contributor_units.values().sum();
            result.push(crate::DiscoveryAllocation {
                group_id: allocation.group_id.clone(),
                submission_id: s.submission_id,
                category: allocation.category,
                gross_units: share,
                contributor_units,
                split_remainder_units: share - paid,
            });
        }
    }
    Ok(result)
}

fn chain_hash(value: &str) -> bool {
    value.len() == 66
        && value.starts_with("0x")
        && value[2..].bytes().all(|b| b.is_ascii_hexdigit())
}

/// Canonical exact-object commitments for preparing a formal discovery submission.
/// This helper does not authenticate a certificate or perform proof checking.

pub fn formal_discovery_roots(
    claim: &xlemma_core::ClaimId,
    proof: &xlemma_core::ProofId,
    artifact: &str,
    environment: &str,
    dependencies: &str,
    axioms: &str,
) -> Result<BTreeMap<xlemma_core::VerificationEvidenceKind, xlemma_core::ArtifactId>, ServiceError>
{
    use xlemma_core::{ArtifactId, VerificationEvidenceKind as E};
    Ok(BTreeMap::from([
        (E::FormalStatement, ArtifactId::derive(claim)?),
        (E::ProofObject, ArtifactId::derive(&(proof, artifact))?),
        (
            E::PinnedToolchain,
            ArtifactId::derive(&(environment, dependencies))?,
        ),
        (E::AxiomInventory, ArtifactId::derive(&axioms)?),
    ]))
}

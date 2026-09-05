//! Offline discovery-round economics. All funding, grouping, qualifications,
//! and evidence verdicts are DECLARED simulation inputs, not authenticated facts.
//! This module neither certifies research nor issues payment authorizations.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xlemma_core::{
    ArtifactId, ClaimId, ContributionGroupId, DiscoveryRoundId, IdError, OperatorClusterId,
    ReceiptId, ResearcherId, VerificationEvidenceKind, VerificationProfile,
    VerificationProfileClass,
};

const MAX_SAFE: u64 = 9_007_199_254_740_991;
const MAX_EVENTS: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewardCategory {
    Discovery,
    Formalization,
    ProofImprovement,
    Replication,
    ResearchTools,
    NegativeResult,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CategoryBudget {
    pub solver_units: u64,
    pub verification_units: u64,
    pub appeal_units: u64,
    pub maximum_weight: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryRoundPolicy {
    pub name: String,
    /// Exact deployment asset identifier; amounts are six-decimal USDC units.
    pub usdc_asset: String,
    pub opens_at: u64,
    pub submissions_close_at: u64,
    pub appeals_close_at: u64,
    pub review_deadline: u64,
    pub maximum_submissions: u32,
    pub maximum_appeals: u32,
    /// Authorized review fees are outcome-independent and pool-funded.
    pub verification_fee_units: u64,
    pub appeal_fee_units: u64,
    pub assessors: BTreeSet<OperatorClusterId>,
    pub appeal_reviewers: BTreeSet<OperatorClusterId>,
    pub budgets: BTreeMap<RewardCategory, CategoryBudget>,
    pub profiles: Vec<VerificationProfile>,
    /// Commits calibration, category eligibility, priority, and collaboration rules.
    pub assessment_policy_root: ArtifactId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryFunding {
    pub settlement_receipt_id: ReceiptId,
    pub donor_cluster: OperatorClusterId,
    pub administrator_cluster: OperatorClusterId,
    /// This restricted category cannot subsidize another category.
    pub category: RewardCategory,
    pub gross_units: u64,
    pub administrator_fee_units: u64,
    pub mandate_root: ArtifactId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContributorShare {
    pub researcher_id: ResearcherId,
    pub operator_cluster_id: OperatorClusterId,
    pub share_bps: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeclaredEvidenceStatus {
    Supported,
    Rejected,
    Divergent,
    Inconclusive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyOutcome {
    Positive,
    Null,
    Counterexample,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisteredStudy {
    pub registration_root: ArtifactId,
    pub registered_at: u64,
    pub data_collection_started_at: u64,
    pub outcome: StudyOutcome,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoverySubmission {
    pub submission_id: ReceiptId,
    pub group_id: ContributionGroupId,
    pub category: RewardCategory,
    pub claims: BTreeSet<ClaimId>,
    pub contributors: Vec<ContributorShare>,
    pub contribution_manifest_root: ArtifactId,
    pub profile_id: xlemma_core::PolicyId,
    pub evidence_roots: BTreeMap<VerificationEvidenceKind, ArtifactId>,
    pub declared_evidence_status: DeclaredEvidenceStatus,
    pub reward_eligible: bool,
    pub assessed_weight: u64,
    pub assessment_root: ArtifactId,
    pub assessors: BTreeSet<OperatorClusterId>,
    /// Telemetry only: these numbers never enter the allocator.
    pub reported_tokens: u64,
    pub reported_compute_units: u64,
    /// Optional for exploration; required by this pilot's empirical profile.
    pub registered_study: Option<RegisteredStudy>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppealGround {
    Evidence,
    Grouping,
    Eligibility,
    Allocation,
    Attribution,
    Process,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DiscoveryRemedy {
    Uphold,
    CorrectReward {
        group_id: ContributionGroupId,
        eligible: bool,
        weight: u64,
    },
    CorrectContributors {
        contributors: Vec<ContributorShare>,
        manifest_root: ArtifactId,
    },
    RequireRevalidation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DiscoveryEvent {
    Submit {
        at: u64,
        submission: DiscoverySubmission,
    },
    Appeal {
        at: u64,
        appeal_id: ReceiptId,
        submission_id: ReceiptId,
        appellant: ResearcherId,
        appellant_cluster: OperatorClusterId,
        grounds: AppealGround,
        evidence_root: ArtifactId,
    },
    Resolve {
        at: u64,
        appeal_id: ReceiptId,
        reviewers: BTreeSet<OperatorClusterId>,
        reasons_root: ArtifactId,
        remedy: DiscoveryRemedy,
    },
    Finalize {
        at: u64,
    },
    Expire {
        at: u64,
    },
}

impl DiscoveryEvent {
    fn at(&self) -> u64 {
        match self {
            Self::Submit { at, .. }
            | Self::Appeal { at, .. }
            | Self::Resolve { at, .. }
            | Self::Finalize { at }
            | Self::Expire { at } => *at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoverySimulation {
    pub policy: DiscoveryRoundPolicy,
    pub funding: Vec<DiscoveryFunding>,
    /// Informational; promises never contribute to available funds.
    pub pledged_units: u64,
    pub previously_rewarded_groups: BTreeSet<ContributionGroupId>,
    pub previously_rewarded_claims: BTreeMap<RewardCategory, BTreeSet<ClaimId>>,
    pub previously_consumed_settlements: BTreeSet<ReceiptId>,
    pub events: Vec<DiscoveryEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryRoundState {
    Reviewing,
    Finalized,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveryAllocation {
    pub group_id: ContributionGroupId,
    pub submission_id: ReceiptId,
    pub category: RewardCategory,
    pub gross_units: u64,
    pub contributor_units: BTreeMap<ResearcherId, u64>,
    pub split_remainder_units: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoverySimulationReport {
    pub simulation_only: bool,
    pub authenticates_evidence: bool,
    pub executes_payments: bool,
    pub round_id: DiscoveryRoundId,
    pub state: DiscoveryRoundState,
    pub declared_funding_units: u64,
    pub ignored_pledged_units: u64,
    pub administrator_fees_units: u64,
    pub largest_donor_share_bps: u64,
    pub verification_spent_units: u64,
    pub appeal_spent_units: u64,
    pub pending_appeal_reserved_units: u64,
    pub allocated_units: u64,
    /// All non-spent/non-allocated funds, including reserves and rounding dust.
    pub retained_units: u64,
    pub admitted_submissions: usize,
    pub remaining_submission_capacity: usize,
    pub unresolved_appeals: usize,
    pub excluded_submissions: BTreeMap<ReceiptId, String>,
    pub allocations: Vec<DiscoveryAllocation>,
    /// Replaying the same ordered input yields the same append-only receipt chain.
    pub event_receipts: Vec<ReceiptId>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error(transparent)]
    Identity(#[from] IdError),
    #[error("invalid discovery input: {0}")]
    Invalid(&'static str),
}

fn require(value: bool, reason: &'static str) -> Result<(), DiscoveryError> {
    if value {
        Ok(())
    } else {
        Err(DiscoveryError::Invalid(reason))
    }
}

fn sum(values: impl IntoIterator<Item = u64>) -> Result<u64, DiscoveryError> {
    values.into_iter().try_fold(0u64, |a, b| {
        a.checked_add(b)
            .filter(|n| *n <= MAX_SAFE)
            .ok_or(DiscoveryError::Invalid("unsafe or overflowing amount"))
    })
}

fn portion(amount: u64, numerator: u64, denominator: u64) -> u64 {
    // All callers check nonzero denominator and numerator <= denominator.
    (u128::from(amount) * u128::from(numerator) / u128::from(denominator)) as u64
}

fn validate_shares(shares: &[ContributorShare]) -> Result<(), DiscoveryError> {
    require(
        !shares.is_empty() && shares.len() <= 100,
        "invalid contributor count",
    )?;
    let mut identities = BTreeSet::new();
    for share in shares {
        share.researcher_id.validate()?;
        share.operator_cluster_id.validate()?;
        require(
            share.share_bps > 0 && identities.insert(&share.researcher_id),
            "invalid contributor share",
        )?;
    }
    require(
        sum(shares.iter().map(|s| u64::from(s.share_bps)))? == 10_000,
        "shares must sum to 10000",
    )
}

fn independent_panel(
    panel: &BTreeSet<OperatorClusterId>,
    allowed: &BTreeSet<OperatorClusterId>,
    excluded: &BTreeSet<OperatorClusterId>,
) -> Result<(), DiscoveryError> {
    require(
        panel.len() >= 2 && panel.is_subset(allowed) && panel.is_disjoint(excluded),
        "unqualified or conflicted review panel",
    )
}

struct PendingAppeal {
    submission_id: ReceiptId,
    appellant_cluster: OperatorClusterId,
    grounds: AppealGround,
    resolved: bool,
}

/// Replays synthetic declared facts through a bounded economic state machine.
/// A report is never a certificate, signed settlement instruction, or payment receipt.
pub fn simulate_discovery(
    input: &DiscoverySimulation,
) -> Result<DiscoverySimulationReport, DiscoveryError> {
    let p = &input.policy;
    require(
        input.pledged_units <= MAX_SAFE
            && input.previously_rewarded_groups.len() <= MAX_EVENTS
            && input.previously_consumed_settlements.len() <= MAX_EVENTS
            && input
                .previously_rewarded_claims
                .values()
                .all(|claims| claims.len() <= MAX_EVENTS),
        "history or pledged amount exceeds simulation bounds",
    )?;
    require(
        !p.name.trim().is_empty() && !p.usdc_asset.trim().is_empty(),
        "missing round name or asset",
    )?;
    p.assessment_policy_root.validate()?;
    require(
        p.opens_at < p.submissions_close_at
            && p.submissions_close_at < p.appeals_close_at
            && p.appeals_close_at < p.review_deadline
            && p.review_deadline <= MAX_SAFE,
        "invalid round deadlines",
    )?;
    require(
        p.maximum_submissions > 0
            && p.maximum_submissions <= 1000
            && p.maximum_appeals > 0
            && p.maximum_appeals <= 1000
            && input.events.len() <= MAX_EVENTS
            && input.funding.len() <= 1000,
        "round capacity out of bounds",
    )?;
    require(
        p.assessors.len() >= 2
            && p.appeal_reviewers.len() >= 2
            && p.assessors.is_disjoint(&p.appeal_reviewers),
        "review panels must be separate",
    )?;
    for cluster in p.assessors.iter().chain(&p.appeal_reviewers) {
        cluster.validate()?;
    }
    require(
        p.verification_fee_units > 0 && p.appeal_fee_units > 0,
        "review work must be funded",
    )?;
    require(
        !p.budgets.is_empty() && !p.profiles.is_empty() && p.profiles.len() <= 32,
        "missing categories or profiles",
    )?;
    let mut profiles = BTreeMap::new();
    for profile in &p.profiles {
        profile
            .validate()
            .map_err(|_| DiscoveryError::Invalid("invalid verification profile"))?;
        require(
            sum([p.submissions_close_at, profile.challenge_window_seconds])? <= p.appeals_close_at,
            "round truncates a required profile challenge window",
        )?;
        require(
            profiles.insert(&profile.policy_id, profile).is_none(),
            "duplicate profile",
        )?;
        if profile.class == VerificationProfileClass::Empirical {
            use VerificationEvidenceKind as E;
            require(
                [
                    E::DatasetProvenance,
                    E::AnalysisPlan,
                    E::UncertaintyReport,
                    E::RobustnessChecks,
                    E::IndependentReplication,
                ]
                .iter()
                .all(|e| profile.required_evidence.contains(e)),
                "empirical pilot requires uncertainty, methods, and replication",
            )?;
        }
    }
    let mut category_funds = BTreeMap::new();
    let mut settlements = input.previously_consumed_settlements.clone();
    for receipt in &settlements {
        receipt.validate()?;
    }
    let mut excluded_clusters = BTreeSet::new();
    let mut donors = BTreeMap::new();
    for f in &input.funding {
        f.settlement_receipt_id.validate()?;
        f.donor_cluster.validate()?;
        f.administrator_cluster.validate()?;
        f.mandate_root.validate()?;
        require(
            settlements.insert(f.settlement_receipt_id.clone()),
            "reused funding settlement",
        )?;
        require(
            p.budgets.contains_key(&f.category)
                && f.gross_units > 0
                && f.administrator_fee_units <= f.gross_units,
            "invalid funding restriction or fee",
        )?;
        let net = f.gross_units - f.administrator_fee_units;
        let amount = category_funds.entry(f.category).or_insert(0);
        *amount = sum([*amount, net])?;
        let donor = donors.entry(f.donor_cluster.clone()).or_insert(0);
        *donor = sum([*donor, f.gross_units])?;
        excluded_clusters.extend([f.donor_cluster.clone(), f.administrator_cluster.clone()]);
    }
    for (category, b) in &p.budgets {
        require(
            b.maximum_weight > 0 && b.maximum_weight <= 1_000_000,
            "invalid weight cap",
        )?;
        require(
            b.verification_units >= p.verification_fee_units
                && b.appeal_units >= p.appeal_fee_units,
            "unfunded review access",
        )?;
        require(
            sum([b.solver_units, b.verification_units, b.appeal_units])?
                <= *category_funds.get(category).unwrap_or(&0),
            "category budget exceeds restricted funding",
        )?;
    }
    for group in &input.previously_rewarded_groups {
        group.validate()?;
    }
    for claims in input.previously_rewarded_claims.values() {
        for claim in claims {
            claim.validate()?;
        }
    }
    let round_id = DiscoveryRoundId::derive(p)?;
    let mut receipt_chain = Vec::new();
    // The initial commitment includes funding and cross-round history, not just events.
    let mut previous = ReceiptId::derive(&(
        "discovery-simulation-input-v1",
        p,
        &input.funding,
        &input.previously_rewarded_groups,
        &input.previously_rewarded_claims,
        &input.previously_consumed_settlements,
    ))?;
    let mut submissions: BTreeMap<ReceiptId, DiscoverySubmission> = BTreeMap::new();
    let mut submission_order = Vec::new();
    let mut appeals = BTreeMap::new();
    let mut appealed_submissions = BTreeSet::new();
    let mut verification_spent: BTreeMap<RewardCategory, u64> = BTreeMap::new();
    let mut appeal_spent: BTreeMap<RewardCategory, u64> = BTreeMap::new();
    let mut state = DiscoveryRoundState::Reviewing;
    let mut last_time = p.opens_at;
    for event in &input.events {
        require(
            state == DiscoveryRoundState::Reviewing,
            "round already closed",
        )?;
        let now = event.at();
        require(
            now >= last_time && now <= MAX_SAFE,
            "nonmonotonic event time",
        )?;
        last_time = now;
        match event {
            DiscoveryEvent::Submit { submission: s, .. } => {
                require(
                    now < p.submissions_close_at
                        && submissions.len() < p.maximum_submissions as usize,
                    "submission window or capacity exhausted",
                )?;
                s.submission_id.validate()?;
                s.group_id.validate()?;
                s.contribution_manifest_root.validate()?;
                s.assessment_root.validate()?;
                require(
                    s.reported_tokens <= MAX_SAFE && s.reported_compute_units <= MAX_SAFE,
                    "telemetry exceeds canonical integer bounds",
                )?;
                require(
                    !submissions.contains_key(&s.submission_id),
                    "submission replay",
                )?;
                require(
                    !s.claims.is_empty() && s.claims.len() <= 100,
                    "invalid claim set",
                )?;
                for claim in &s.claims {
                    claim.validate()?;
                }
                validate_shares(&s.contributors)?;
                let mut conflicts = excluded_clusters.clone();
                conflicts.extend(s.contributors.iter().map(|s| s.operator_cluster_id.clone()));
                independent_panel(&s.assessors, &p.assessors, &conflicts)?;
                let profile = profiles
                    .get(&s.profile_id)
                    .ok_or(DiscoveryError::Invalid("unsupported verification profile"))?;
                require(
                    profile
                        .required_evidence
                        .iter()
                        .all(|e| s.evidence_roots.contains_key(e)),
                    "missing required evidence",
                )?;
                for root in s.evidence_roots.values() {
                    root.validate()?;
                }
                if let Some(study) = &s.registered_study {
                    study.registration_root.validate()?;
                    require(
                        study.registered_at <= study.data_collection_started_at
                            && study.data_collection_started_at <= now,
                        "registration must precede data collection and submission",
                    )?;
                    require(
                        s.evidence_roots
                            .get(&VerificationEvidenceKind::ProtocolRegistration)
                            == Some(&study.registration_root),
                        "study registration must bind the evidence packet",
                    )?;
                }
                if profile.class == VerificationProfileClass::Empirical {
                    require(
                        s.registered_study.is_some(),
                        "empirical pilot requires a registered method",
                    )?;
                }
                if s.category == RewardCategory::NegativeResult {
                    require(s.registered_study.as_ref().is_some_and(|study|
                        matches!(study.outcome, StudyOutcome::Null | StudyOutcome::Counterexample)),
                        "negative-result reward requires an informative registered null or counterexample")?;
                }
                if matches!(
                    s.category,
                    RewardCategory::Formalization | RewardCategory::ProofImprovement
                ) {
                    require(
                        profile.class == VerificationProfileClass::Formal,
                        "formal reward requires formal profile",
                    )?;
                }
                let budget = p
                    .budgets
                    .get(&s.category)
                    .ok_or(DiscoveryError::Invalid("unsupported reward category"))?;
                require(
                    s.assessed_weight <= budget.maximum_weight,
                    "weight exceeds category cap",
                )?;
                let spent = verification_spent.entry(s.category).or_insert(0);
                *spent = sum([*spent, p.verification_fee_units])?;
                require(
                    *spent <= budget.verification_units,
                    "verification reserve exhausted",
                )?;
                submission_order.push(s.submission_id.clone());
                submissions.insert(s.submission_id.clone(), s.clone());
            }
            DiscoveryEvent::Appeal {
                appeal_id,
                submission_id,
                appellant,
                appellant_cluster,
                grounds,
                evidence_root,
                ..
            } => {
                appeal_id.validate()?;
                appellant.validate()?;
                appellant_cluster.validate()?;
                evidence_root.validate()?;
                require(
                    now < p.appeals_close_at && appeals.len() < p.maximum_appeals as usize,
                    "appeal window or capacity exhausted",
                )?;
                require(
                    !appeals.contains_key(appeal_id)
                        && appealed_submissions.insert((submission_id.clone(), appellant.clone())),
                    "appeal replay or repeated filing",
                )?;
                let s = submissions
                    .get(submission_id)
                    .ok_or(DiscoveryError::Invalid("unknown appealed submission"))?;
                require(
                    s.contributors
                        .iter()
                        .filter(|c| &c.researcher_id == appellant)
                        .all(|c| &c.operator_cluster_id == appellant_cluster),
                    "appellant control identity mismatch",
                )?;
                let spent = appeal_spent.entry(s.category).or_insert(0);
                *spent = sum([*spent, p.appeal_fee_units])?;
                require(
                    *spent <= p.budgets[&s.category].appeal_units,
                    "appeal reserve exhausted",
                )?;
                appeals.insert(
                    appeal_id.clone(),
                    PendingAppeal {
                        submission_id: submission_id.clone(),
                        appellant_cluster: appellant_cluster.clone(),
                        grounds: *grounds,
                        resolved: false,
                    },
                );
            }
            DiscoveryEvent::Resolve {
                appeal_id,
                reviewers,
                reasons_root,
                remedy,
                ..
            } => {
                reasons_root.validate()?;
                require(now <= p.review_deadline, "review deadline expired")?;
                let appeal = appeals
                    .get_mut(appeal_id)
                    .ok_or(DiscoveryError::Invalid("unknown appeal"))?;
                require(!appeal.resolved, "appeal already resolved")?;
                let s = submissions
                    .get_mut(&appeal.submission_id)
                    .expect("appeals bind admitted submissions");
                let mut conflicts = excluded_clusters.clone();
                conflicts.extend(s.assessors.iter().cloned());
                conflicts.insert(appeal.appellant_cluster.clone());
                conflicts.extend(s.contributors.iter().map(|c| c.operator_cluster_id.clone()));
                if let DiscoveryRemedy::CorrectContributors { contributors, .. } = remedy {
                    conflicts.extend(contributors.iter().map(|c| c.operator_cluster_id.clone()));
                }
                independent_panel(reviewers, &p.appeal_reviewers, &conflicts)?;
                match remedy {
                    DiscoveryRemedy::Uphold => {}
                    DiscoveryRemedy::CorrectReward {
                        group_id,
                        eligible,
                        weight,
                    } => {
                        require(
                            appeal.grounds != AppealGround::Evidence
                                && appeal.grounds != AppealGround::Attribution,
                            "wrong remedy for appeal grounds",
                        )?;
                        group_id.validate()?;
                        require(
                            *weight <= p.budgets[&s.category].maximum_weight,
                            "appeal weight exceeds cap",
                        )?;
                        s.group_id = group_id.clone();
                        s.reward_eligible = *eligible;
                        s.assessed_weight = *weight;
                    }
                    DiscoveryRemedy::CorrectContributors {
                        contributors,
                        manifest_root,
                    } => {
                        require(
                            appeal.grounds == AppealGround::Attribution,
                            "wrong remedy for appeal grounds",
                        )?;
                        validate_shares(contributors)?;
                        manifest_root.validate()?;
                        // New claimants must also be independent of the original assessors.
                        require(
                            contributors
                                .iter()
                                .all(|c| !s.assessors.contains(&c.operator_cluster_id)),
                            "contributor correction reveals assessor conflict",
                        )?;
                        s.contributors = contributors.clone();
                        s.contribution_manifest_root = manifest_root.clone();
                    }
                    DiscoveryRemedy::RequireRevalidation => {
                        s.declared_evidence_status = DeclaredEvidenceStatus::Inconclusive;
                    }
                }
                // A reward correction never modifies the evidence verdict.
                appeal.resolved = true;
            }
            DiscoveryEvent::Finalize { .. } => {
                require(
                    now >= p.appeals_close_at && now <= p.review_deadline,
                    "outside finalization window",
                )?;
                require(
                    appeals.values().all(|a| a.resolved),
                    "unresolved appeal holds entire allocation batch",
                )?;
                state = DiscoveryRoundState::Finalized;
            }
            DiscoveryEvent::Expire { .. } => {
                require(now > p.review_deadline, "review period has not expired")?;
                state = DiscoveryRoundState::Expired;
            }
        }
        previous =
            ReceiptId::derive(&("discovery-simulation-event-v1", &round_id, &previous, event))?;
        receipt_chain.push(previous.clone());
    }
    let mut seen_groups = input.previously_rewarded_groups.clone();
    let mut seen_claims = input.previously_rewarded_claims.clone();
    let mut selected = Vec::new();
    let mut excluded = BTreeMap::new();
    // The original assessment order is deterministic. Duplicate arrivals cannot
    // replace a group's payees or increase its weight; corrections require appeal.
    for id in &submission_order {
        let s = &submissions[id];
        let reason = if s.declared_evidence_status != DeclaredEvidenceStatus::Supported {
            Some("evidence_not_supported")
        } else if !s.reward_eligible || s.assessed_weight == 0 {
            Some("not_reward_eligible")
        } else if seen_groups.contains(&s.group_id)
            || (matches!(
                s.category,
                RewardCategory::Discovery | RewardCategory::Formalization
            ) && seen_claims
                .get(&s.category)
                .is_some_and(|claims| !claims.is_disjoint(&s.claims)))
        {
            Some("already_accounted_contribution")
        } else {
            None
        };
        if let Some(reason) = reason {
            excluded.insert(id.clone(), reason.to_owned());
        } else {
            seen_groups.insert(s.group_id.clone());
            seen_claims
                .entry(s.category)
                .or_default()
                .extend(s.claims.iter().cloned());
            selected.push(s);
        }
    }
    let mut allocations = Vec::new();
    if state == DiscoveryRoundState::Finalized {
        for (category, budget) in &p.budgets {
            let lane: Vec<_> = selected
                .iter()
                .filter(|s| &s.category == category)
                .collect();
            let denominator = sum(lane.iter().map(|s| s.assessed_weight))?;
            for s in lane {
                let gross_units = portion(budget.solver_units, s.assessed_weight, denominator);
                let contributor_units: BTreeMap<_, _> = s
                    .contributors
                    .iter()
                    .map(|c| {
                        (
                            c.researcher_id.clone(),
                            portion(gross_units, u64::from(c.share_bps), 10_000),
                        )
                    })
                    .collect();
                let distributed = sum(contributor_units.values().copied())?;
                allocations.push(DiscoveryAllocation {
                    group_id: s.group_id.clone(),
                    submission_id: s.submission_id.clone(),
                    category: *category,
                    gross_units,
                    contributor_units,
                    split_remainder_units: gross_units - distributed,
                });
            }
        }
    }
    let funding = sum(input.funding.iter().map(|f| f.gross_units))?;
    let fees = sum(input.funding.iter().map(|f| f.administrator_fee_units))?;
    let verification = sum(verification_spent.values().copied())?;
    let appeal_cost = sum(appeals
        .values()
        .filter(|a| a.resolved)
        .map(|_| p.appeal_fee_units))?;
    let pending_appeal_reserved_units = if state == DiscoveryRoundState::Reviewing {
        sum(appeals
            .values()
            .filter(|a| !a.resolved)
            .map(|_| p.appeal_fee_units))?
    } else {
        0
    };
    let allocated = sum(allocations
        .iter()
        .flat_map(|a| a.contributor_units.values().copied()))?;
    let accounted = sum([fees, verification, appeal_cost, allocated])?;
    require(accounted <= funding, "economic conservation failure")?;
    Ok(DiscoverySimulationReport {
        simulation_only: true, authenticates_evidence: false, executes_payments: false,
        round_id, state, declared_funding_units: funding, ignored_pledged_units: input.pledged_units,
        administrator_fees_units: fees,
        largest_donor_share_bps: if funding == 0 { 0 } else { portion(10_000, donors.values().copied().max().unwrap_or(0), funding) },
        verification_spent_units: verification, appeal_spent_units: appeal_cost, pending_appeal_reserved_units,
        allocated_units: allocated, retained_units: funding - accounted,
        admitted_submissions: submissions.len(), remaining_submission_capacity: p.maximum_submissions as usize - submissions.len(),
        unresolved_appeals: appeals.values().filter(|a| !a.resolved).count(), excluded_submissions: excluded,
        allocations, event_receipts: receipt_chain,
        limitations: vec![
            "Synthetic funding, verdicts, grouping, qualifications, mandates and histories are not authenticated.".into(),
            "Fresh group IDs or altered claims can evade grouping; no semantic novelty/difficulty oracle is implemented.".into(),
            "Empirical evidence presence does not verify an instrument, experiment, uncertainty analysis or law of nature.".into(),
            "All review access is pool-funded and bounded; expiry retains funds and unresolved cases without declaring them false.".into(),
            "Reports cannot authorize USDC transfers; production requires signed admission, verification, independent assessment and durable atomic settlement.".into(),
        ],
    })
}

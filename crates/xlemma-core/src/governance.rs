//! Constitutional, multi-constituency, and forkable governance records.

use crate::{ConstitutionalCommitmentId, ForkExitPlanId, GovernanceProposalId, IdError, PolicyId};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const MATERIAL_CHANGE_MIN_TIMELOCK_SECONDS: i64 = 7 * 24 * 60 * 60;
pub const MAX_SINGLE_OPERATOR_INFLUENCE_BPS: u16 = 1_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceChamber {
    Researcher,
    Operator,
    Commons,
}

impl GovernanceChamber {
    fn all() -> [Self; 3] {
        [Self::Researcher, Self::Operator, Self::Commons]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceImpact {
    Routine,
    Material,
    Constitutional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceSubject {
    CredentialPolicy,
    EconomicParameterRange,
    TreasuryAllocation,
    SettlementAdapter,
    SoftwareVersionMigration,
    ChallengeWindow,
    InsuranceRequirement,
    FormalValidity,
    HistoricalOrigin,
    HistoricalEvidence,
    TheoremAcceptability,
    MandatoryModelProvider,
}

impl GovernanceSubject {
    pub const fn is_constitutionally_permitted(self) -> bool {
        matches!(
            self,
            Self::CredentialPolicy
                | Self::EconomicParameterRange
                | Self::TreasuryAllocation
                | Self::SettlementAdapter
                | Self::SoftwareVersionMigration
                | Self::ChallengeWindow
                | Self::InsuranceRequirement
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstitutionalInvariant {
    DeterministicValidity,
    IndependentReproduction,
    ProverCannotSelfCertify,
    BackedResearchCredit,
    WorkNeutralVerifierCompensation,
    CheckerDivergenceQuarantine,
    FormalClaimIdentity,
    ImmutableOriginAndHistory,
    RightsDoNotOwnTruth,
    EvidenceEconomicGraphSeparation,
    BoundedNonrecursiveParticipation,
    AdapterNeutrality,
    PortableExit,
}

impl ConstitutionalInvariant {
    fn all() -> [Self; 13] {
        [
            Self::DeterministicValidity,
            Self::IndependentReproduction,
            Self::ProverCannotSelfCertify,
            Self::BackedResearchCredit,
            Self::WorkNeutralVerifierCompensation,
            Self::CheckerDivergenceQuarantine,
            Self::FormalClaimIdentity,
            Self::ImmutableOriginAndHistory,
            Self::RightsDoNotOwnTruth,
            Self::EvidenceEconomicGraphSeparation,
            Self::BoundedNonrecursiveParticipation,
            Self::AdapterNeutrality,
            Self::PortableExit,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionalCommitment {
    pub commitment_id: ConstitutionalCommitmentId,
    pub protocol_version: String,
    pub invariants: BTreeSet<ConstitutionalInvariant>,
    pub immutable: bool,
    pub specification_root: String,
    pub effective_at: DateTime<Utc>,
    pub signatures: Vec<String>,
}

#[derive(Serialize)]
struct ConstitutionalIdentity<'a> {
    protocol_version: &'a str,
    invariants: &'a BTreeSet<ConstitutionalInvariant>,
    immutable: bool,
    specification_root: &'a str,
    effective_at: DateTime<Utc>,
}

impl ConstitutionalCommitment {
    pub fn derive_commitment_id(&self) -> Result<ConstitutionalCommitmentId, IdError> {
        ConstitutionalCommitmentId::derive(&ConstitutionalIdentity {
            protocol_version: &self.protocol_version,
            invariants: &self.invariants,
            immutable: self.immutable,
            specification_root: &self.specification_root,
            effective_at: self.effective_at,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), GovernanceError> {
        self.commitment_id.validate()?;
        if self.commitment_id != self.derive_commitment_id()? {
            return Err(GovernanceError::IdentityMismatch);
        }
        if !self.immutable
            || self.invariants != ConstitutionalInvariant::all().into_iter().collect()
            || self.protocol_version.trim().is_empty()
            || self.specification_root.trim().is_empty()
            || self.signatures.is_empty()
            || self.signatures.iter().any(|value| value.trim().is_empty())
        {
            return Err(GovernanceError::IncompleteConstitution);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForkExitPlan {
    pub plan_id: ForkExitPlanId,
    pub identity_export_root: String,
    pub artifact_export_root: String,
    pub history_export_root: String,
    pub funds_exit_root: String,
    pub open_client_source_root: String,
    pub destination_protocol: String,
    pub available_from: DateTime<Utc>,
    pub available_until: DateTime<Utc>,
    pub signatures: Vec<String>,
}

#[derive(Serialize)]
struct ForkExitIdentity<'a> {
    identity_export_root: &'a str,
    artifact_export_root: &'a str,
    history_export_root: &'a str,
    funds_exit_root: &'a str,
    open_client_source_root: &'a str,
    destination_protocol: &'a str,
    available_from: DateTime<Utc>,
    available_until: DateTime<Utc>,
}

impl ForkExitPlan {
    pub fn derive_plan_id(&self) -> Result<ForkExitPlanId, IdError> {
        ForkExitPlanId::derive(&ForkExitIdentity {
            identity_export_root: &self.identity_export_root,
            artifact_export_root: &self.artifact_export_root,
            history_export_root: &self.history_export_root,
            funds_exit_root: &self.funds_exit_root,
            open_client_source_root: &self.open_client_source_root,
            destination_protocol: &self.destination_protocol,
            available_from: self.available_from,
            available_until: self.available_until,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), GovernanceError> {
        self.plan_id.validate()?;
        if self.plan_id != self.derive_plan_id()? {
            return Err(GovernanceError::IdentityMismatch);
        }
        let roots = [
            self.identity_export_root.as_str(),
            self.artifact_export_root.as_str(),
            self.history_export_root.as_str(),
            self.funds_exit_root.as_str(),
            self.open_client_source_root.as_str(),
        ];
        if roots.iter().any(|root| root.trim().is_empty())
            || self.destination_protocol.trim().is_empty()
            || self.available_from >= self.available_until
            || self.signatures.is_empty()
            || self.signatures.iter().any(|value| value.trim().is_empty())
        {
            return Err(GovernanceError::InvalidExitPlan);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChamberApproval {
    pub chamber: GovernanceChamber,
    pub eligible_independent_domains: u32,
    pub approving_independent_domains: u32,
    pub approval_threshold_bps: u16,
    pub maximum_single_operator_influence_bps: u16,
    pub vote_record_root: String,
    pub finalized_at: DateTime<Utc>,
    pub signature: String,
}

impl ChamberApproval {
    pub fn passed(&self) -> Result<bool, GovernanceError> {
        if self.eligible_independent_domains == 0
            || self.approving_independent_domains > self.eligible_independent_domains
            || self.approval_threshold_bps == 0
            || self.approval_threshold_bps > 10_000
            || self.maximum_single_operator_influence_bps == 0
            || self.maximum_single_operator_influence_bps > MAX_SINGLE_OPERATOR_INFLUENCE_BPS
            || self.vote_record_root.trim().is_empty()
            || self.signature.trim().is_empty()
        {
            return Err(GovernanceError::InvalidChamberApproval);
        }
        let actual_bps = u64::from(self.approving_independent_domains)
            .checked_mul(10_000)
            .ok_or(GovernanceError::Overflow)?
            / u64::from(self.eligible_independent_domains);
        Ok(actual_bps >= u64::from(self.approval_threshold_bps))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub proposal_id: GovernanceProposalId,
    pub policy_id: PolicyId,
    pub constitutional_commitment_id: ConstitutionalCommitmentId,
    pub subject: GovernanceSubject,
    pub impact: GovernanceImpact,
    pub change_root: String,
    pub public_simulation_root: String,
    pub fork_exit_plan_id: ForkExitPlanId,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub timelock_ends_at: DateTime<Utc>,
    pub activation_at: DateTime<Utc>,
    pub approvals: BTreeMap<GovernanceChamber, ChamberApproval>,
    pub signature: String,
}

#[derive(Serialize)]
struct ProposalIdentity<'a> {
    policy_id: &'a PolicyId,
    constitutional_commitment_id: &'a ConstitutionalCommitmentId,
    subject: GovernanceSubject,
    impact: GovernanceImpact,
    change_root: &'a str,
    public_simulation_root: &'a str,
    fork_exit_plan_id: &'a ForkExitPlanId,
    proposer: &'a str,
    created_at: DateTime<Utc>,
    voting_ends_at: DateTime<Utc>,
    timelock_ends_at: DateTime<Utc>,
    activation_at: DateTime<Utc>,
    approvals: &'a BTreeMap<GovernanceChamber, ChamberApproval>,
}

impl GovernanceProposal {
    pub fn derive_proposal_id(&self) -> Result<GovernanceProposalId, IdError> {
        GovernanceProposalId::derive(&ProposalIdentity {
            policy_id: &self.policy_id,
            constitutional_commitment_id: &self.constitutional_commitment_id,
            subject: self.subject,
            impact: self.impact,
            change_root: &self.change_root,
            public_simulation_root: &self.public_simulation_root,
            fork_exit_plan_id: &self.fork_exit_plan_id,
            proposer: &self.proposer,
            created_at: self.created_at,
            voting_ends_at: self.voting_ends_at,
            timelock_ends_at: self.timelock_ends_at,
            activation_at: self.activation_at,
            approvals: &self.approvals,
        })
    }

    pub fn validate_for_activation(
        &self,
        constitution: &ConstitutionalCommitment,
        exit_plan: &ForkExitPlan,
        now: DateTime<Utc>,
    ) -> Result<(), GovernanceError> {
        self.proposal_id.validate()?;
        self.policy_id.validate()?;
        self.constitutional_commitment_id.validate()?;
        self.fork_exit_plan_id.validate()?;
        constitution.validate_integrity()?;
        exit_plan.validate_integrity()?;
        if self.proposal_id != self.derive_proposal_id()? {
            return Err(GovernanceError::IdentityMismatch);
        }
        if self.constitutional_commitment_id != constitution.commitment_id
            || self.fork_exit_plan_id != exit_plan.plan_id
        {
            return Err(GovernanceError::MissingConstitutionOrExit);
        }
        if !self.subject.is_constitutionally_permitted() {
            return Err(GovernanceError::ForbiddenSubject);
        }
        if self.change_root.trim().is_empty()
            || self.public_simulation_root.trim().is_empty()
            || self.proposer.trim().is_empty()
            || self.signature.trim().is_empty()
            || self.created_at >= self.voting_ends_at
            || self.voting_ends_at > self.timelock_ends_at
            || self.timelock_ends_at > self.activation_at
            || now < self.activation_at
            || exit_plan.available_from > self.created_at
            || exit_plan.available_until < self.activation_at
        {
            return Err(GovernanceError::InvalidTimelineOrEvidence);
        }
        let required_timelock = match self.impact {
            GovernanceImpact::Routine => Duration::zero(),
            GovernanceImpact::Material | GovernanceImpact::Constitutional => {
                Duration::seconds(MATERIAL_CHANGE_MIN_TIMELOCK_SECONDS)
            }
        };
        if self.timelock_ends_at - self.voting_ends_at < required_timelock {
            return Err(GovernanceError::TimelockTooShort);
        }
        if self.approvals.len() != GovernanceChamber::all().len() {
            return Err(GovernanceError::MissingChamberApproval);
        }
        for chamber in GovernanceChamber::all() {
            let approval = self
                .approvals
                .get(&chamber)
                .ok_or(GovernanceError::MissingChamberApproval)?;
            if approval.chamber != chamber
                || approval.finalized_at > self.voting_ends_at
                || !approval.passed()?
            {
                return Err(GovernanceError::MissingChamberApproval);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error("content-derived governance identity does not match")]
    IdentityMismatch,
    #[error("constitutional commitment does not contain every immutable invariant")]
    IncompleteConstitution,
    #[error(
        "fork/exit plan does not preserve identity, artifacts, history, funds, and client access"
    )]
    InvalidExitPlan,
    #[error("governance subject is constitutionally forbidden")]
    ForbiddenSubject,
    #[error("governance chamber approval is invalid or uncapped")]
    InvalidChamberApproval,
    #[error("all three independent chambers must approve")]
    MissingChamberApproval,
    #[error("proposal does not bind the supplied constitution and fork/exit plan")]
    MissingConstitutionOrExit,
    #[error("governance timeline, simulation, change, proposer, or exit evidence is invalid")]
    InvalidTimelineOrEvidence,
    #[error("material governance change has an insufficient public timelock")]
    TimelockTooShort,
    #[error("checked governance arithmetic overflow")]
    Overflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn constitution(now: DateTime<Utc>) -> ConstitutionalCommitment {
        let mut constitution = ConstitutionalCommitment {
            commitment_id: ConstitutionalCommitmentId::derive(&"placeholder").unwrap(),
            protocol_version: "XLMP/1".into(),
            invariants: ConstitutionalInvariant::all().into_iter().collect(),
            immutable: true,
            specification_root: "blake3:constitution".into(),
            effective_at: now,
            signatures: vec!["researcher".into(), "operator".into(), "commons".into()],
        };
        constitution.commitment_id = constitution.derive_commitment_id().unwrap();
        constitution
    }

    fn exit_plan(now: DateTime<Utc>) -> ForkExitPlan {
        let mut plan = ForkExitPlan {
            plan_id: ForkExitPlanId::derive(&"placeholder").unwrap(),
            identity_export_root: "blake3:identity".into(),
            artifact_export_root: "blake3:artifacts".into(),
            history_export_root: "blake3:history".into(),
            funds_exit_root: "blake3:funds".into(),
            open_client_source_root: "blake3:client".into(),
            destination_protocol: "XLMP/1-compatible-fork".into(),
            available_from: now,
            available_until: now + Duration::days(60),
            signatures: vec!["signature".into()],
        };
        plan.plan_id = plan.derive_plan_id().unwrap();
        plan
    }

    fn approval(chamber: GovernanceChamber, finalized_at: DateTime<Utc>) -> ChamberApproval {
        ChamberApproval {
            chamber,
            eligible_independent_domains: 100,
            approving_independent_domains: 75,
            approval_threshold_bps: 6_667,
            maximum_single_operator_influence_bps: 500,
            vote_record_root: format!("blake3:{chamber:?}"),
            finalized_at,
            signature: "signature".into(),
        }
    }

    fn proposal(
        now: DateTime<Utc>,
        subject: GovernanceSubject,
    ) -> (GovernanceProposal, ConstitutionalCommitment, ForkExitPlan) {
        let constitution = constitution(now);
        let exit_plan = exit_plan(now);
        let voting_ends = now + Duration::days(3);
        let mut proposal = GovernanceProposal {
            proposal_id: GovernanceProposalId::derive(&"placeholder").unwrap(),
            policy_id: PolicyId::derive(&"governance-policy").unwrap(),
            constitutional_commitment_id: constitution.commitment_id.clone(),
            subject,
            impact: GovernanceImpact::Material,
            change_root: "blake3:change".into(),
            public_simulation_root: "blake3:simulation".into(),
            fork_exit_plan_id: exit_plan.plan_id.clone(),
            proposer: "did:key:proposer".into(),
            created_at: now,
            voting_ends_at: voting_ends,
            timelock_ends_at: voting_ends + Duration::days(7),
            activation_at: voting_ends + Duration::days(8),
            approvals: GovernanceChamber::all()
                .into_iter()
                .map(|chamber| (chamber, approval(chamber, voting_ends)))
                .collect(),
            signature: "signature".into(),
        };
        proposal.proposal_id = proposal.derive_proposal_id().unwrap();
        (proposal, constitution, exit_plan)
    }

    #[test]
    fn material_change_requires_all_chambers_timelock_simulation_and_exit() {
        let now = Utc::now();
        let (proposal, constitution, exit_plan) =
            proposal(now, GovernanceSubject::SettlementAdapter);
        assert!(proposal
            .validate_for_activation(&constitution, &exit_plan, proposal.activation_at)
            .is_ok());
    }

    #[test]
    fn governance_can_never_vote_a_proof_valid() {
        let now = Utc::now();
        let (proposal, constitution, exit_plan) = proposal(now, GovernanceSubject::FormalValidity);
        assert!(matches!(
            proposal.validate_for_activation(&constitution, &exit_plan, proposal.activation_at),
            Err(GovernanceError::ForbiddenSubject)
        ));
    }

    #[test]
    fn wealth_cannot_create_uncapped_chamber_influence() {
        let now = Utc::now();
        let (mut proposal, constitution, exit_plan) =
            proposal(now, GovernanceSubject::TreasuryAllocation);
        proposal
            .approvals
            .get_mut(&GovernanceChamber::Operator)
            .unwrap()
            .maximum_single_operator_influence_bps = 5_000;
        proposal.proposal_id = proposal.derive_proposal_id().unwrap();
        assert!(matches!(
            proposal.validate_for_activation(&constitution, &exit_plan, proposal.activation_at),
            Err(GovernanceError::InvalidChamberApproval)
                | Err(GovernanceError::MissingChamberApproval)
        ));
    }
}

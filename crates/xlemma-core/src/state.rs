use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationState {
    Draft,
    ClaimCommitted,
    Quoted,
    Funded,
    Assigned,
    Formalizing,
    CandidateReady,
    Building,
    CheckersCommitted,
    CheckersRevealed,
    Passed,
    Failed,
    Divergent,
    Challenged,
    Finalized,
    Rejected,
    Quarantined,
    Published,
    Revalidated,
    Superseded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormalStatus {
    Unchecked,
    Reproduced,
    Certified,
    Rejected,
    Divergent,
    Quarantined,
    Revoked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssuranceLevel {
    Committed,
    LocallyChecked,
    IndependentlyReproduced,
    FormallyCertified,
    ResearchCertified,
    EconomicallyFinalized,
    Mature,
    Quarantined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeRole {
    Researcher,
    #[serde(alias = "astra_prover")]
    ResearchProver,
    LeanBuilder,
    OfficialKernelChecker,
    IndependentChecker,
    NoveltyReviewer,
    SignificanceReviewer,
    Challenger,
    StorageProvider,
    Indexer,
    PaymentFacilitator,
    CertificateFinalizer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusDomain {
    FormalValidity,
    Provenance,
    Novelty,
    Significance,
    EconomicState,
    Availability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckerFamily {
    LeanKernel,
    Nanoda,
    OtherIndependent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationVerdict {
    Pass,
    Fail,
    Error,
    Abstain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoveltyDecision {
    Unreviewed,
    KnownEquivalent,
    Incremental,
    MateriallyNovel,
    Inconclusive,
    Disputed,
}

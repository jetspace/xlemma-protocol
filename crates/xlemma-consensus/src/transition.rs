use thiserror::Error;
use xlemma_core::VerificationState;

#[derive(Debug, Error)]
#[error("invalid verification transition: {from:?} -> {to:?}")]
pub struct TransitionError {
    pub from: VerificationState,
    pub to: VerificationState,
}

pub fn ensure_transition(
    from: VerificationState,
    to: VerificationState,
) -> Result<(), TransitionError> {
    use VerificationState::*;

    let valid = matches!(
        (from, to),
        (Draft, ClaimCommitted)
            | (ClaimCommitted, Quoted)
            | (Quoted, Funded)
            | (Funded, Assigned)
            | (Assigned, Formalizing)
            | (Assigned, Building)
            | (Formalizing, CandidateReady)
            | (CandidateReady, Building)
            | (Building, CheckersCommitted)
            | (CheckersCommitted, CheckersRevealed)
            | (CheckersRevealed, Passed)
            | (CheckersRevealed, Failed)
            | (CheckersRevealed, Divergent)
            | (Passed, Challenged)
            | (Passed, Finalized)
            | (Failed, Rejected)
            | (Divergent, Quarantined)
            | (Challenged, Finalized)
            | (Challenged, Rejected)
            | (Challenged, Quarantined)
            | (Finalized, Published)
            | (Published, Revalidated)
            | (Published, Quarantined)
            | (Revalidated, Published)
            | (Published, Superseded)
            | (Quarantined, Revalidated)
            | (Quarantined, Rejected)
    );

    valid.then_some(()).ok_or(TransitionError { from, to })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_lifecycle_transition_is_allowed() {
        assert!(ensure_transition(
            VerificationState::CheckersRevealed,
            VerificationState::Divergent
        )
        .is_ok());
        assert!(ensure_transition(
            VerificationState::Divergent,
            VerificationState::Quarantined
        )
        .is_ok());
    }

    #[test]
    fn invalid_shortcut_to_finality_is_rejected() {
        assert!(ensure_transition(
            VerificationState::ClaimCommitted,
            VerificationState::Finalized
        )
        .is_err());
    }
}

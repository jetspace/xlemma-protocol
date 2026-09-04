use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Canonical research lifecycle. More granular implementation state may be
/// tracked locally, but it cannot bypass these protocol gates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResearchLifecycleState {
    Claim,
    Commit,
    Formalize,
    Prove,
    Reproduce,
    Certify,
    Challenge,
    Finalize,
    Publish,
    Reuse,
    Reward,
    Revalidate,
    Quarantined,
    Rejected,
    Superseded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("invalid XLMP lifecycle transition: {from:?} -> {to:?}")]
pub struct LifecycleTransitionError {
    pub from: ResearchLifecycleState,
    pub to: ResearchLifecycleState,
}

pub fn ensure_lifecycle_transition(
    from: ResearchLifecycleState,
    to: ResearchLifecycleState,
) -> Result<(), LifecycleTransitionError> {
    use ResearchLifecycleState::*;

    let valid = matches!(
        (from, to),
        (Claim, Commit)
            | (Commit, Formalize)
            | (Formalize, Prove)
            | (Prove, Formalize)
            | (Prove, Reproduce)
            | (Reproduce, Certify)
            | (Reproduce, Quarantined)
            | (Reproduce, Rejected)
            | (Certify, Challenge)
            | (Challenge, Finalize)
            | (Challenge, Quarantined)
            | (Challenge, Rejected)
            | (Finalize, Publish)
            | (Publish, Reuse)
            | (Publish, Revalidate)
            | (Publish, Quarantined)
            | (Reuse, Reward)
            | (Reuse, Revalidate)
            | (Reward, Revalidate)
            | (Revalidate, Reproduce)
            | (Revalidate, Formalize)
            | (Revalidate, Quarantined)
            | (Revalidate, Rejected)
            | (Publish, Superseded)
            | (Quarantined, Revalidate)
            | (Quarantined, Rejected)
    );

    valid
        .then_some(())
        .ok_or(LifecycleTransitionError { from, to })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_happy_path_is_complete() {
        use ResearchLifecycleState::*;
        let states = [
            Claim, Commit, Formalize, Prove, Reproduce, Certify, Challenge, Finalize, Publish,
            Reuse, Reward, Revalidate,
        ];
        for pair in states.windows(2) {
            assert!(ensure_lifecycle_transition(pair[0], pair[1]).is_ok());
        }
    }

    #[test]
    fn claim_cannot_skip_independent_reproduction() {
        assert!(ensure_lifecycle_transition(
            ResearchLifecycleState::Claim,
            ResearchLifecycleState::Finalize
        )
        .is_err());
    }

    #[test]
    fn divergent_reproduction_can_fail_closed() {
        assert!(ensure_lifecycle_transition(
            ResearchLifecycleState::Reproduce,
            ResearchLifecycleState::Quarantined
        )
        .is_ok());
    }

    #[test]
    fn revalidation_cannot_republish_without_fresh_reproduction() {
        assert!(ensure_lifecycle_transition(
            ResearchLifecycleState::Revalidate,
            ResearchLifecycleState::Publish
        )
        .is_err());
        assert!(ensure_lifecycle_transition(
            ResearchLifecycleState::Revalidate,
            ResearchLifecycleState::Reproduce
        )
        .is_ok());
    }

    #[test]
    fn supersession_is_append_only_and_cannot_restore_old_publication_state() {
        assert!(ensure_lifecycle_transition(
            ResearchLifecycleState::Publish,
            ResearchLifecycleState::Superseded
        )
        .is_ok());
        assert!(ensure_lifecycle_transition(
            ResearchLifecycleState::Superseded,
            ResearchLifecycleState::Publish
        )
        .is_err());
    }
}

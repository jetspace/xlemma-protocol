//! Proof of Independent Reproduction (PoIR).
//!
//! Formal consensus uses exact reproduction and role-qualified generalized
//! quorums. Subjective novelty assessment is deliberately isolated in a
//! calibrated evidence aggregator.

pub mod commit_reveal;
pub mod committee;
pub mod formal;
pub mod novelty;
pub mod transition;

pub use commit_reveal::*;
pub use committee::*;
pub use formal::*;
pub use novelty::*;
pub use transition::*;

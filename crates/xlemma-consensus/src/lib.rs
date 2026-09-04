//! Proof of Independent Reproduction (PoIR).
//!
//! Formal consensus uses exact reproduction and role-qualified generalized
//! quorums. Subjective novelty assessment is deliberately isolated in a
//! calibrated evidence aggregator.

pub mod committee;
pub mod commit_reveal;
pub mod formal;
pub mod novelty;
pub mod transition;

pub use committee::*;
pub use commit_reveal::*;
pub use formal::*;
pub use novelty::*;
pub use transition::*;

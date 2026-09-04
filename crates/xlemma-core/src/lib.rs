//! Core xLemma protocol types.
//!
//! This crate is intentionally free of network, database, model-provider and
//! blockchain dependencies. Protocol-facing services should exchange these
//! content-addressed objects rather than mutable database identifiers.

pub mod canonical;
pub mod id;
pub mod manifest;
pub mod money;
pub mod receipt;
pub mod state;

pub use canonical::{canonical_json_bytes, canonical_json_hash, CanonicalizationError};
pub use id::*;
pub use manifest::*;
pub use money::*;
pub use receipt::*;
pub use state::*;

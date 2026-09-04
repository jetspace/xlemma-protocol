//! Core xLemma protocol types.
//!
//! This crate is intentionally free of network, database, model-provider and
//! blockchain dependencies. Protocol-facing services should exchange these
//! content-addressed objects rather than mutable database identifiers.

pub mod canonical;
pub mod id;
pub mod manifest;
pub mod money;
pub mod network;
pub mod protocol;
pub mod receipt;
pub mod state;

pub use canonical::{canonical_json_bytes, canonical_json_hash, CanonicalizationError};
pub use id::*;
pub use manifest::*;
pub use money::*;
pub use network::*;
pub use protocol::*;
pub use receipt::*;
pub use state::*;

/// Canonical name and version of the xLemma wire protocol.
pub const XLMP_PROTOCOL: &str = "XLMP";
pub const XLMP_MAJOR_VERSION: u16 = 1;
pub const XLMP_VERSION: &str = "XLMP/1";

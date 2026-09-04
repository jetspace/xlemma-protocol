//! XLMP/1: the canonical, provider-neutral xLemma protocol and wire layer.
//!
//! XLMP defines research objects, message meaning, integrity, and lifecycle
//! transitions. HTTP, libp2p, WebSocket, x402, chains, model providers,
//! checkers, and storage networks are adapters and cannot redefine XLMP
//! research state or consensus.

pub mod adapter;
pub mod framing;
pub mod state;
pub mod wire;

pub use adapter::*;
pub use framing::*;
pub use state::*;
pub use wire::*;
pub use xlemma_core::{XLMP_MAJOR_VERSION, XLMP_PROTOCOL, XLMP_VERSION};

pub const XLMP_MEDIA_TYPE: &str = "application/x-xlmp+json;version=1";
pub const XLMP_SIGNATURE_DOMAIN: &str = "xlmp-envelope-signature-v1";

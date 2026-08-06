//! Compatibility facade for the independently versioned host-control contract.
//!
//! Semantic ownership lives in `host-control-core`. Animus retains only this
//! re-export surface while product policy, transport, and host authority remain
//! consumer-owned.

pub use host_control_core::*;

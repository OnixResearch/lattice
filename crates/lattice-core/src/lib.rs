//! Pure Lattice workflow decisions.
//!
//! This crate owns content-addressed values, expression mapping, control-flow
//! decisions, and Lattice schedule identity framing. It performs no filesystem,
//! process, network, persistence, clock, provider, or terminal effects.

#![forbid(unsafe_code)]

// r[impl lattice.architecture.core_crate]
// r[impl lattice.architecture.core_crate.pure]
pub mod content_identity;
#[path = "control/flow.rs"]
pub mod control_flow;
pub mod expressions;
pub mod value;

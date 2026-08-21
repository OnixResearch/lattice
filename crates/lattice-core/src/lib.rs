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
#[path = "expression/dependencies.rs"]
pub mod mapping;
#[allow(
    unknown_lints,
    renamed_imports,
    reason = "the compatibility name keeps the existing expression dependency API while the file uses a non-repeating module owner"
)]
pub use mapping as expression_dependencies;
pub mod expressions;
#[path = "frontier/planner.rs"]
pub mod scheduling;
pub mod value;

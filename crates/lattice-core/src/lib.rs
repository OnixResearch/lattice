//! Lattice core: host-capability-free decisions shared across the Lattice shell.
//!
//! This crate owns Lattice decision logic and DTOs that must run without store,
//! clock, filesystem, process, network, provider, async-runtime, CLI, or UI
//! authority. The imperative `lattice` package depends on it; it never depends
//! on the shell.

pub mod workflow;

pub use workflow::TemporalError;
pub use workflow::WorkflowMeta;

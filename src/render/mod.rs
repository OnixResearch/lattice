//! Artifact-bound consumer of Aspen's canonical render and batch contracts.
//! Remote placement, retries, tile execution and assembly remain producer-owned.
//!
//! r[impl lattice.render_binding.producer]
//! r[impl lattice.render_binding.admission]
//! r[impl lattice.render_binding.acceptance]
//! r[impl lattice.render_binding.recovery]
//! r[impl lattice.render_binding.truthfulness]
//! r[impl lattice.render_binding.qualification]

pub mod cli;
pub mod model;
pub mod operation;
pub mod shell;

pub use model::AcceptedManifest;
pub use model::Authority;
pub use model::Budget;
pub use model::Outcome;
pub use model::Request;

pub const PRODUCER_REVISION: &str = "afc8308bff9a8c50b2315cb4a17c9454252d7c45";
pub const SCHEMA: &str = "lattice.render-binding.v1";

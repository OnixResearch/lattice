//! Immutable, versioned operation data with a pure core and capability-confined shell.
//!
//! r[impl lattice.operation_data.records]
//! r[impl lattice.operation_data.versions]
//! r[impl lattice.operation_data.query]
//! r[impl lattice.operation_data.retention]
//! r[impl lattice.operation_data.confidentiality]
//! r[impl lattice.operation_data.valence_projection]

pub mod cli;
pub mod core;
pub mod lifecycle;
pub mod model;
pub mod selection;
pub mod shell;

pub use core::AdmissionPolicy;
pub use core::Error;
pub use core::admit;
pub use core::canonical_json_bytes;
pub use core::publication_decision;
pub use core::validate_metadata;

pub use lifecycle::orphan_prune_plan;
pub use lifecycle::project_recorded_only;
pub use lifecycle::retention_plan;
pub use lifecycle::validate_deletion_plan;
pub use lifecycle::validate_recorded_only_projection;
pub use model::*;
pub use selection::bind_resolved_inputs;
pub use selection::recheck;
pub use selection::reduce_query;
pub use selection::resolve_exact;
pub use selection::resolve_latest;
pub use selection::validate_bound_invocation_plan;
pub use shell::DeletionReceipt;
pub use shell::Plane;
pub use shell::PublicationReceipt;
pub use shell::publish_projection_create_only;

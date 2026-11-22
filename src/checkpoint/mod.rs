//! Application checkpoints and restart for the workflow exchange route (Lattice #12).
//!
//! Profile `workflow-node-boundary-v1`: completed, non-sensitive node outcomes plus workspace
//! objects, published as a `lattice.application-checkpoint-manifest.v1` (C8) at completed-node
//! boundaries and before every effect reservation. A committed manifest is restored into a fresh
//! store under an opaque Kiln admission reference and yields a
//! `lattice.application-restart-receipt.v1`. See `docs/application-checkpoint.md`.

pub mod cli;
mod core;
mod model;
mod publish;
mod restore;

pub use core::Admission;
pub use core::AdmissionInput;
pub use core::AdmittedCheckpoint;
pub use core::ManifestInput;
pub use core::admit;
pub use core::admit_route;
pub use core::admit_runtime_facts;
pub use core::adoption_record;
pub use core::build_manifest;
pub use core::canonical;
pub use core::checkpoint_offer;
pub use core::completed_node;
pub use core::content_hex;
pub use core::identity_of_bytes;
pub use core::is_workspace_path;
pub use core::node_outputs_object;
pub use core::restart_kind;
pub use core::validate_admission_reference;
pub use core::validate_binding;
pub use core::validate_manifest;
pub use core::verify_carry_forward;

pub use model::*;
pub use publish::CheckpointRoot;
pub use publish::HookSeed;
pub use publish::NodeBoundaryHook;
pub use publish::local_runtime;
pub use publish::snapshot_workspace;
pub use restore::Adopted;
pub use restore::AdoptionTarget;
pub use restore::admit_from_root;
pub use restore::adopt;
pub use restore::adopted_steps;
pub use restore::now_unix_ms;

#[cfg(test)]
mod tests;

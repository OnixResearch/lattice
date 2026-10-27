//! Pure dependency-maintenance adapter contracts.
//!
//! This crate adapts Remora's authority-free dependency inventory, candidate,
//! advisory, grouping, update-plan, service-port, and receipt patterns. It
//! performs no host effect and discovers no implementation. Hosts own manifest
//! parsing, registries, advisory databases, workspace mutation, validation,
//! VCS, change-request publication, clocks, scheduling, and persistence.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::too_many_arguments
)]
#![forbid(unsafe_code)]

extern crate alloc;

// r[impl agent_harness_runtime.remora_adapters.maintenance]
// r[impl agent_harness_runtime.remora_adapters.composition]
// r[impl agent_harness_runtime.remora_adapters.provenance]
mod model;
mod planning;
mod ports;
mod receipt;

pub use animus_adapter_change_request as change_request;
pub use model::*;
pub use planning::*;
pub use ports::*;
pub use receipt::*;

/// Adapter contract schema.
pub const MAINTENANCE_ADAPTER_SCHEMA: &str = "animus.maintenance-adapter.v1";
/// Reviewed Remora repository.
pub const REMORA_SOURCE_REPOSITORY: &str = animus_adapter_change_request::REMORA_SOURCE_REPOSITORY;
/// Reviewed Remora revision.
pub const REMORA_SOURCE_REVISION: &str = animus_adapter_change_request::REMORA_SOURCE_REVISION;
/// BLAKE3 identity of the reviewed Remora source cohort.
pub const REMORA_SOURCE_COHORT_IDENTITY: &str =
    animus_adapter_change_request::REMORA_SOURCE_COHORT_IDENTITY;
/// Reviewed Remora source cohort.
pub const REMORA_SOURCE_COHORT: &[&str] = animus_adapter_change_request::REMORA_SOURCE_COHORT;
/// Declared license for the reviewed Remora workspace.
pub const REMORA_DECLARED_LICENSE: &str = animus_adapter_change_request::REMORA_DECLARED_LICENSE;
/// Owner-authorized imported scope.
pub const REMORA_IMPORTED_SCOPE: &str =
    "authority-free dependency inventory, candidate, grouping, plan, port, and receipt contracts";
/// Basis for the owner-authorized adaptation.
pub const REMORA_OWNER_GRANT_BASIS: &str = animus_adapter_change_request::REMORA_OWNER_GRANT_BASIS;
/// Maximum bytes in one dependency or version value.
pub const MAX_DEPENDENCY_TEXT_BYTES: usize = 256;
/// Maximum observed manifests in one run.
pub const DEFAULT_MAX_MANIFESTS: u32 = 64;
/// Maximum dependencies in one observed manifest.
pub const DEFAULT_MAX_DEPENDENCIES_PER_MANIFEST: u32 = 512;
/// Maximum update candidates in one run.
pub const DEFAULT_MAX_CANDIDATES: u32 = 128;
/// Maximum updates in one routine group.
pub const DEFAULT_MAX_GROUP_SIZE: u32 = 4;
/// Maximum update groups in one run.
pub const DEFAULT_MAX_GROUPS: u32 = 16;

#[cfg(test)]
mod tests;

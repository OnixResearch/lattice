//! Pure forge-neutral change-request adapter contracts.
//!
//! This crate adapts the authority-free repository, change-request, review,
//! CI-failure, publication-plan, and receipt patterns from Remora. It performs
//! no host effect and discovers no implementation. Hosts own VCS access, diff
//! loading, model calls, network transport, credentials, publication, clocks,
//! scheduling, and persistence.

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

// r[impl agent_harness_runtime.remora_adapters.change_request]
// r[impl agent_harness_runtime.remora_adapters.composition]
// r[impl agent_harness_runtime.remora_adapters.provenance]
mod model;
mod planning;
mod ports;
mod receipt;

pub use model::*;
pub use planning::*;
pub use ports::*;
pub use receipt::*;

/// Adapter contract schema.
pub const CHANGE_REQUEST_ADAPTER_SCHEMA: &str = "animus.change-request-adapter.v1";
/// Reviewed Remora repository.
pub const REMORA_SOURCE_REPOSITORY: &str = "git@github.com:OnixResearch/remora.git";
/// Reviewed Remora revision.
pub const REMORA_SOURCE_REVISION: &str = "e3c857c857f4aa170d8a5b6286aad5855db541be";
/// BLAKE3 identity of the reviewed Remora source cohort.
pub const REMORA_SOURCE_COHORT_IDENTITY: &str =
    "b3:a084a8a06727b1fae59377d34754f224065351e550caadf4e12c0bbcdcc26e42";
/// Reviewed Remora source cohort.
pub const REMORA_SOURCE_COHORT: &[&str] = &[
    "crates/remora-core/src/domain.rs",
    "crates/remora-core/src/effect.rs",
    "crates/remora-core/src/planning.rs",
    "crates/remora-core/src/review.rs",
];
/// Declared license for the reviewed Remora workspace.
pub const REMORA_DECLARED_LICENSE: &str = "MIT OR Apache-2.0";
/// Owner-authorized imported scope.
pub const REMORA_IMPORTED_SCOPE: &str =
    "authority-free change-request, review, CI-triage, binding, port, and receipt contracts";
/// Basis for the owner-authorized adaptation.
pub const REMORA_OWNER_GRANT_BASIS: &str = "user-requested OnixResearch owner adaptation";
/// Maximum bytes in one safe identity.
pub const MAX_IDENTITY_BYTES: usize = 256;
/// Maximum bytes in one bounded title.
pub const MAX_TITLE_BYTES: usize = 512;
/// Maximum bytes in one bounded review body.
pub const MAX_BODY_BYTES: usize = 16 * 1024;
/// Maximum files in one review observation.
pub const DEFAULT_MAX_REVIEW_FILES: u32 = 64;
/// Maximum hunks in one review observation.
pub const DEFAULT_MAX_REVIEW_HUNKS: u32 = 256;
/// Maximum changed lines in one review observation.
pub const DEFAULT_MAX_REVIEW_LINES: u32 = 4_096;
/// Maximum review findings in one plan.
pub const DEFAULT_MAX_REVIEW_FINDINGS: u32 = 32;
/// Maximum CI failures in one review plan.
pub const DEFAULT_MAX_CI_FAILURES: u32 = 32;
/// Maximum repair attempts before escalation.
pub const DEFAULT_MAX_REPAIR_ATTEMPTS: u32 = 1;
/// Maximum accepted review confidence.
pub const MAX_REVIEW_CONFIDENCE: u8 = 100;
/// Default minimum confidence for automated comments.
pub const DEFAULT_MINIMUM_COMMENT_CONFIDENCE: u8 = 80;

#[cfg(test)]
mod tests;

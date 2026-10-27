//! Pure native evaluation adapter contracts.
//!
//! This crate owns typed service catalogs, requests, observations, model-call
//! metadata projection, compatibility bindings, and metadata-only receipts. It
//! performs no host effect and discovers no implementation. Hosts own task
//! loading, agent and verifier execution, artifact transfer, model traffic,
//! cancellation, deadlines, clocks, and result storage.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::struct_field_names,
    clippy::too_many_arguments
)]
#![forbid(unsafe_code)]

extern crate alloc;

mod binding;
mod catalog;
mod gateway;
mod ports;
mod receipt;

pub use binding::*;
pub use catalog::*;
pub use gateway::*;
pub use ports::*;
pub use receipt::*;

/// Adapter contract schema.
pub const EVALUATION_ADAPTER_SCHEMA: &str = "animus.native-evaluation-adapter.v1";
/// Reviewed Verifiers revision.
pub const VERIFIERS_REVISION: &str = "1a997814e85a2e4f409add960aa852737388dbd8";
/// Reviewed Verifiers license.
pub const VERIFIERS_LICENSE: &str = "MIT";
/// Canonical BLAKE3 identity of the reviewed repository, revision, and source-path cohort.
pub const VERIFIERS_SOURCE_COHORT_IDENTITY: &str =
    "b3:facd1eab5ae08491dbab9ae51a85aac73b90415e61030b551d1f7f3f3ffc5918";
/// Bounded reviewed source cohort.
pub const VERIFIERS_SOURCE_COHORT: &[&str] = &[
    "verifiers/v1/dialects/base.py",
    "verifiers/v1/dialects/chat.py",
    "verifiers/v1/dialects/responses.py",
    "verifiers/v1/dialects/anthropic.py",
    "verifiers/v1/interception/base.py",
    "verifiers/v1/interception/pool.py",
    "verifiers/v1/interception/server.py",
    "verifiers/v1/state.py",
];
/// Maximum safe public identity bytes.
pub const MAX_ADAPTER_IDENTITY_BYTES: usize = animus_eval_core::MAX_IDENTITY_BYTES;
/// Maximum service entries in one catalog.
pub const MAX_SERVICE_ENTRIES: usize = 32;
/// Maximum capabilities in one service descriptor.
pub const MAX_SERVICE_CAPABILITIES: usize = 8;
/// Maximum safe metadata labels in one receipt.
pub const MAX_ADAPTER_RECEIPT_LABELS: usize = 64;

#[cfg(test)]
mod tests;

//! Reusable agent-loop runtime for `animus`.
//!
//! This crate owns typed orchestration over loop-fabric decisions and injected
//! model/tool adapters. It does not construct concrete providers, tools,
//! filesystem handles, processes, network clients, terminal UI, persistence, or
//! Syndicate dataspaces. The deterministic transition core is separated from the
//! thin imperative shell that calls fabrics and adapters.

#![cfg_attr(animus_octet, feature(register_tool))]
#![cfg_attr(animus_octet, register_tool(tigerstyle))]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![allow(unknown_lints)]
#![allow(module_file_count)]
// Octet-only suppression keeps archived compatibility facades reviewable until
// their public APIs are split by owner without requiring nightly-only lint names
// in normal stable cargo builds.
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::excessive_file_length,
        tigerstyle::platform_dependent_cast,
        tigerstyle::unbounded_loop,
        tigerstyle::unjustified_allow
    )
)]
#![forbid(unsafe_code)]

/// Pure active-turn durability, transaction, recovery, and receipt contracts.
pub mod active_turn_recovery;
mod admission;
#[allow(excessive_file_length)]
#[path = "aeneasproofcampaign.rs"]
pub mod aeneas_proof_campaign;
#[path = "agentroles/mod.rs"]
pub mod agent_roles;
#[allow(excessive_file_length)]
#[path = "authoritativecompletion/mod.rs"]
pub mod authoritative_completion;
mod background;
#[allow(excessive_file_length)]
mod batch;
mod boundary;
/// Pure bounded session branch projection, control planning, comparison, and evidence.
pub mod branch_explorer;
mod branching;
mod capmap;
mod caps;
mod checkpoint;
/// Pure canonical conversation projection and cache reconciliation.
#[path = "conversationprojection.rs"]
pub mod conversation_projection;
#[path = "codingsession/mod.rs"]
mod sessionhost;
/// Pure coding-session lifecycle facade and effect-request state machine.
pub mod coding_session_host {
    pub use super::sessionhost::*;
}
#[path = "authorityprofile/mod.rs"]
mod sessionauthority;
/// Pure metadata-only runtime authority profiles, durable bindings, and receipts.
pub mod session_authority_profile {
    pub use super::sessionauthority::*;
}
#[path = "composition/conformance.rs"]
mod composition_conformance;
mod context_layering;
mod contract;
#[allow(excessive_file_length)]
#[path = "continuation/mod.rs"]
mod controller;
mod dataset_export;
#[path = "dogfoodshell.rs"]
pub mod dogfood_shell;
#[path = "portfolio_search/mod.rs"]
mod engine;
mod eval_feedback;
#[path = "toolexecutorkit.rs"]
mod executionkit;
mod exports;
mod fabric;
mod fresh_context_tasks;
/// Pure optional Harbor-compatible evaluation planning and evidence projection.
#[path = "harbor/eval.rs"]
pub mod harbor_eval;
mod headless;
mod hooks;
#[path = "control.rs"]
mod runstate;
#[path = "interrupted_tool_repair/mod.rs"]
mod session_recovery;
/// Pure active-branch diagnosis and append-only interrupted tool-call repair.
pub mod interrupted_tool_repair {
    pub use super::session_recovery::*;
}
pub mod kernel;
mod mailbox;
mod modelprofiles;
#[path = "nestedsubagentgraphs.rs"]
pub mod nested_subagent_graphs;
#[allow(excessive_file_length)]
#[path = "kanban.rs"]
mod orchestration;
mod pi_compaction;
#[cfg(test)]
mod policyprojectiontests;
mod prelude;
#[path = "productionmemorypolicy.rs"]
pub mod production_memory_policy;
mod provenance;
mod provider_hardening;
#[path = "resilience/mod.rs"]
mod resilience;
/// Pure bounded same-plan provider retry contracts.
pub mod provider_retry {
    pub use super::resilience::*;
}
mod quality_records;
#[path = "sessionexport/mod.rs"]
mod redactedview;
mod researchspec;
#[path = "researchspecpromotiondag.rs"]
pub mod researchspec_promotion_dag;
#[cfg(feature = "rlm-kernel")]
#[path = "rlmkernel.rs"]
pub mod rlm_kernel;
mod runtime_dag;
mod scaffold;
/// Pure redaction-first session-tree export contracts.
pub mod session_tree_export {
    pub use super::redactedview::*;
}
#[path = "sessionstore/mod.rs"]
mod durablelog;
#[path = "sessionpersistence.rs"]
pub mod session_persistence;
/// Pure durable local session-store framing, recovery, and receipt contracts.
pub mod durable_session_store {
    pub use super::durablelog::*;
}
mod shell;
mod status;
pub mod subagents;
pub use subagents::TypedSubagentCapability;
pub use subagents::subagent_capability_receipt_label;
mod telemetry;
#[path = "tooladmission.rs"]
mod tool_admission;
/// Coherent tool catalog, admission, injected execution, and safe receipt kit.
pub mod tool_executor_kit {
    pub use crate::executionkit::*;
}
#[path = "toolexecutorpacks.rs"]
pub mod tool_executor_packs;
mod transition;
mod trust_boundary;
mod turns;
#[path = "turnstoneeval.rs"]
pub mod turnstone_eval;
mod workcmd;
mod worker_team;
mod zerostack_midturn;

pub use exports::*;
pub use prelude::*;

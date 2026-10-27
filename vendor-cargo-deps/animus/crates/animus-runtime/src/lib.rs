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

extern crate alloc;

/// Pure autonomous continuation, quality-gate, fingerprint, and receipt contracts.
#[cfg(feature = "autonomous-gates")]
pub mod autonomous_gates;

/// Pure persistent goal lifecycle, budget, continuation, ownership, and receipt contracts.
#[cfg(feature = "persistent-goals")]
#[path = "persistentgoals.rs"]
pub mod persistent_goals;

/// Pure schedule ownership, claim, coalescing, recovery, and receipt contracts.
#[cfg(feature = "schedule-reliability")]
#[path = "schedulereliability/mod.rs"]
pub mod schedule_reliability;

// r[agent_harness_runtime.session_core_crate.compatibility]
/// Pure bounded reconnect replay over admitted session facts.
#[cfg(feature = "session-event-replay")]
pub use animus_session_core::session_event_replay;

/// Pure weakest-admitted execution backend selection.
#[cfg(feature = "execution-backend-planner")]
#[path = "executionbackend.rs"]
pub mod execution_backend_planner;

/// Pure active-turn durability, transaction, recovery, and receipt contracts.
pub use animus_session_core::active_turn_recovery;
mod admission;
#[allow(excessive_file_length)]
#[path = "aeneasproofcampaign.rs"]
pub mod aeneas_proof_campaign;
#[path = "agentroles/mod.rs"]
pub mod agent_roles;
/// Authority-free application-host binding projection for one fake-backed pilot.
#[path = "applicationbinding/mod.rs"]
pub mod application_host_binding;
#[allow(excessive_file_length)]
#[path = "authoritativecompletion/mod.rs"]
pub mod authoritative_completion;
mod background;
#[allow(excessive_file_length)]
mod batch;
mod boundary;
#[path = "capabilityconditional.rs"]
mod capabilityconditional;
/// Pure capability-conditional prompt composition invariant.
pub mod capability_conditional {
    pub use super::capabilityconditional::*;
}
/// Pure bounded session branch projection, control planning, comparison, and evidence.
pub use animus_session_core::branch_explorer;
mod capmap;
#[path = "caps/core.rs"]
mod caps;
mod checkpoint;
#[path = "childsession.rs"]
mod childsession;
/// Pure concrete child-session admission, lifecycle, result, and persistence contracts.
pub mod child_session_service {
    pub use super::childsession::*;
}
/// Pure generation-fenced component registration, resolution, and teardown.
pub use animus_session_core::component_scope;
/// Pure composition-activation graph validation, preflight, and safe inspection.
pub use animus_session_core::composition_activation_graph;
/// Pure canonical conversation projection and cache reconciliation.
pub use animus_session_core::conversation_projection;
/// Pure private model-request snapshots, dispatch binding, recovery, and safe evidence.
pub use animus_session_core::model_request_snapshot;
#[path = "codingsession/mod.rs"]
mod sessionhost;
/// Pure coding-session lifecycle facade and effect-request state machine.
pub mod coding_session_host {
    pub use super::sessionhost::*;
}
/// Pure per-dispatch runtime-authority seal contracts.
pub use animus_session_core::runtime_authority_seal;
/// Pure metadata-only runtime authority profiles, durable bindings, and receipts.
pub use animus_session_core::session_authority_profile;
#[path = "composition/conformance.rs"]
mod composition_conformance;
mod context_layering;
mod contract;
#[allow(excessive_file_length)]
#[path = "continuation/mod.rs"]
mod controller;
#[path = "daemonhandoff/mod.rs"]
mod handover;
/// Pure strict-idle, monotonic lease handoff, claim, rollback, and receipt contracts.
pub mod daemon_handoff {
    pub use crate::handover::*;
}
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
/// Focused authority-free native evaluation planning and reduction contracts.
#[cfg(feature = "native-evaluation")]
pub use animus_eval_core as native_evaluation;

/// Binding contracts for typed results, history, silence, cycles, and telemetry.
pub mod convergence;
/// Pure optional Harbor-compatible evaluation planning and evidence projection.
#[cfg(feature = "harbor-eval")]
#[path = "harbor/eval.rs"]
pub mod harbor_eval;
mod headless;
mod hooks;
/// Pure active-branch diagnosis and append-only interrupted tool-call repair.
pub use animus_session_core::interrupted_tool_repair;
pub mod kernel;
mod mailbox;
mod modelprofiles;
#[path = "nestedsubagentgraphs.rs"]
pub mod nested_subagent_graphs;
#[allow(excessive_file_length)]
#[path = "kanban.rs"]
mod orchestration;
/// Pure pending-entry recovery and injected host service contracts.
#[path = "pendinginputvault/mod.rs"]
pub mod pending_input_vault;
mod pi_compaction;
/// Pure validation and existing-owner projection for the optional Pijul CLI adapter.
#[cfg(feature = "pijul-vcs-adapter")]
pub mod pijul;
#[cfg(test)]
mod policyprojectiontests;
mod prelude;
#[path = "productionmemorypolicy.rs"]
pub mod production_memory_policy;
mod provenance;
mod provider_hardening;
/// Pure request-local context selection and injected skill-turn loading contracts.
#[path = "requestcontext/mod.rs"]
pub mod request_context;
#[path = "resilience/mod.rs"]
mod resilience;
/// Pure generation-scoped ownership for reusable response connections.
#[path = "responseownership.rs"]
pub mod response_ownership;
/// Pure validation and existing-owner projection for optional transcript normalization.
#[cfg(feature = "trajectory-transcript-adapter")]
pub mod trajectory;
/// Pure bounded same-plan provider retry contracts.
pub mod provider_retry {
    pub use super::resilience::*;
}
mod quality_records;
/// Pure VCS-neutral rejected-attempt admission, projection, and receipts.
pub mod rejected_attempt;
mod researchspec;
#[path = "researchspecpromotiondag.rs"]
pub mod researchspec_promotion_dag;
#[cfg(feature = "rlm-kernel")]
#[path = "rlmkernel.rs"]
pub mod rlm_kernel;
mod runtime_dag;
mod scaffold;
/// Pure redaction-first session-tree export contracts.
pub use animus_session_core::session_tree_export;
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
pub use subagents::{TypedSubagentCapability, subagent_capability_receipt_label};
mod telemetry;
#[path = "tooladmission.rs"]
mod tool_admission;
/// Coherent tool catalog, admission, injected execution, and safe receipt kit.
pub mod tool_executor_kit {
    pub use crate::executionkit::*;
}
#[path = "toolexecutionstages/mod.rs"]
mod toolexecutionstages;
/// Fixed monotonic stages around authoritative tool admission, execution, and commit owners.
pub mod tool_execution_stages {
    pub use super::toolexecutionstages::*;
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

//! Pure session state, projection, recovery, and evidence contracts for `animus`.
//!
//! This crate has no provider, filesystem, process, network, credential, clock,
//! terminal, concrete persistence, Steel, or host-shell authority.

#![no_std]
#![cfg_attr(animus_octet, feature(register_tool))]
#![cfg_attr(animus_octet, register_tool(tigerstyle))]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![allow(unknown_lints)]
#![allow(module_file_count)]
// These moved modules had the same focused Octet suppression in `animus-runtime`.
// Keep that reviewed lint posture while their large compatibility APIs are split.
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

// r[agent_harness_runtime.session_core_crate]
// r[agent_harness_runtime.session_core_crate.pure]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod prelude;

/// Pure active-turn durability, transaction, recovery, and receipt contracts.
pub mod active_turn_recovery;
/// Pure bounded session branch projection, control planning, comparison, and evidence.
pub mod branch_explorer;
#[path = "componentscope/mod.rs"]
mod componentscope;
/// Pure generation-fenced component registration, resolution, and teardown.
pub mod component_scope {
    pub use super::componentscope::*;
}
#[path = "compositionactivation.rs"]
mod compositionactivation;
/// Pure composition-activation graph validation, preflight, and safe inspection.
pub mod composition_activation_graph {
    pub use super::compositionactivation::*;
}
#[path = "conversationprojection.rs"]
/// Pure canonical conversation projection and cache reconciliation.
pub mod conversation_projection;
/// Pure persist-before-process gateway admission, ordering, recovery, and authority.
pub mod gateway;
#[path = "modelrequestsnapshot/mod.rs"]
mod modelrequestsnapshot;
/// Pure private model-request snapshots, dispatch binding, recovery, and safe evidence.
pub mod model_request_snapshot {
    pub use super::modelrequestsnapshot::*;
}
#[path = "interrupted_tool_repair/mod.rs"]
mod session_recovery;
/// Pure active-branch diagnosis and append-only interrupted tool-call repair.
pub mod interrupted_tool_repair {
    pub use super::session_recovery::*;
}
#[path = "authorityseal/mod.rs"]
mod authorityseal;
/// Pure per-dispatch runtime-authority seal contracts.
pub mod runtime_authority_seal {
    pub use super::authorityseal::*;
}
#[path = "authorityprofile/mod.rs"]
mod sessionauthority;
/// Pure metadata-only runtime authority profiles, durable bindings, and receipts.
pub mod session_authority_profile {
    pub use super::sessionauthority::*;
}
#[path = "sessionexport/mod.rs"]
mod redactedview;
#[cfg(feature = "session-event-replay")]
#[path = "sessioneventreplay.rs"]
/// Pure bounded reconnect replay over admitted session facts.
pub mod session_event_replay;
/// Pure redaction-first session-tree export contracts.
pub mod session_tree_export {
    pub use super::redactedview::*;
}

mod branching;
#[path = "control.rs"]
mod runstate;

pub use branching::{
    SessionBranchCheckpointStatus, SessionBranchContinuationTarget, SessionBranchDiagnostic,
    SessionBranchDiagnosticKind, SessionBranchError, SessionBranchErrorKind,
    SessionBranchOperation, SessionBranchPlan, SessionBranchPlanStatus, SessionBranchPlannerInput,
    SessionBranchPolicy, SessionBranchPolicyAdmission, SessionBranchPolicyDraft,
    SessionBranchReceipt, SessionBranchRequest, SessionBranchRequestDraft, SessionCopyDescriptor,
    SessionCopyDescriptorClass, SessionCopyDescriptorDraft, SessionLedgerEntry,
    SessionLedgerEntryDraft, plan_session_branch, validate_session_branch_receipt_text,
};
pub use runstate::{
    AgentControlCheckpointDecision, AgentControlCheckpointTransition,
    AgentControlCommandDisposition, AgentControlCommandTransition, AgentControlReductionError,
    AgentControlState, AgentControlTerminalTransition, reduce_agent_control_checkpoint,
    reduce_agent_control_command, reduce_agent_control_terminal,
};

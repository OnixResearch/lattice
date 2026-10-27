//! Fixed, monotonic tool-execution stages around existing authority owners.
//!
//! The module is a deterministic functional core. It does not discover
//! handlers, execute tools, persist results, call providers, or grant authority.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::too_many_parameters,
        tigerstyle::usize_in_public_api,
        reason = "published staged-execution contracts retain their typed stable signatures and collection limits"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::compound_condition,
        tigerstyle::function_length,
        reason = "stage reducers return typed errors for rejected observations instead of panicking assertions"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::non_trait_imports,
        reason = "the functional core uses explicit sibling DTO imports to keep stage ownership visible"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::bool_naming,
        tigerstyle::numeric_units,
        reason = "published execution vocabulary uses protocol field names fixed by its versioned contract"
    )
)]

mod core;
mod data;
#[path = "receipt.rs"]
mod redaction;

pub use core::{
    apply_tool_final_admission, apply_tool_output_narrowing, apply_tool_post_commit_candidate,
    apply_tool_pre_admission_candidate, apply_tool_pre_execution_candidate,
    build_tool_execution_stage_candidate, coding_session_tool_commit_owner_identity,
    exact_tool_execution_stage_candidate, observe_tool_final_validation,
    observe_tool_stage_durable_commit, observe_tool_stage_execution, plan_tool_execution_stages,
    start_tool_execution_stages, tool_execution_stage_handler_set_identity,
    validate_tool_execution_stage_handler,
};
pub use data::{
    TOOL_EXECUTION_STAGE_CONTRACT_IDENTITY, TOOL_EXECUTION_STAGE_NON_CLAIMS,
    TOOL_EXECUTION_STAGE_PHASES, TOOL_EXECUTION_STAGE_RECEIPT_SCHEMA,
    TOOL_EXECUTION_STAGE_REDACTION_MARKER, ToolExecutionStageCandidate,
    ToolExecutionStageCandidateDraft, ToolExecutionStageCommitObservation,
    ToolExecutionStageDisposition, ToolExecutionStageError, ToolExecutionStageFinalObservation,
    ToolExecutionStageHandler, ToolExecutionStageHandlerDraft, ToolExecutionStageIssue,
    ToolExecutionStageOutcomeClass, ToolExecutionStageOutputObservation, ToolExecutionStagePhase,
    ToolExecutionStagePlan, ToolExecutionStagePlanDraft, ToolExecutionStageRawObservation,
    ToolExecutionStageReceipt, ToolExecutionStageSensitiveDraft, ToolExecutionStageState,
    ToolExecutionStageStatus,
};
pub use redaction::{project_tool_execution_stage_receipt, validate_tool_execution_stage_receipt};

#[cfg(test)]
mod tests;

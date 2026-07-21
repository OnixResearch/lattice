//! Pure interrupted tool-call repair for already-loaded active branches.
//!
//! Hosts own session loading, storage, active-leaf mutation, provider dispatch,
//! cancellation observation, and durability. This module diagnoses typed values,
//! plans append-only synthetic terminal failures, validates host application
//! evidence, gates provider continuation, and projects metadata-only receipts.
//! No repair API can invoke or retry a tool.

mod core;
mod data;
#[path = "receipt.rs"]
mod evidence;

pub use core::{
    diagnose_interrupted_tool_branch, evaluate_interrupted_tool_provider_gate,
    evaluate_interrupted_tool_repair_application, interrupted_tool_repair_application_request,
    plan_interrupted_tool_repair, project_interrupted_tool_repair_results,
    reduce_interrupted_tool_reason,
};
pub use data::{
    INTERRUPTED_TOOL_ACTION_HASH_DOMAIN, INTERRUPTED_TOOL_LEAF_HASH_DOMAIN,
    INTERRUPTED_TOOL_PLAN_HASH_DOMAIN, INTERRUPTED_TOOL_RECEIPT_HASH_DOMAIN,
    INTERRUPTED_TOOL_REPAIR_SCHEMA, INTERRUPTED_TOOL_SNAPSHOT_HASH_DOMAIN,
    InterruptedToolBranchEntry, InterruptedToolBranchSnapshot, InterruptedToolCall,
    InterruptedToolDiagnosis, InterruptedToolDiagnosisStatus, InterruptedToolDurability,
    InterruptedToolEntryKind, InterruptedToolExecutorObservation, InterruptedToolGap,
    InterruptedToolObservation, InterruptedToolObservationKind,
    InterruptedToolProviderGateDecision, InterruptedToolProviderGateStatus, InterruptedToolReason,
    InterruptedToolRepairAction, InterruptedToolRepairApplicationDraft,
    InterruptedToolRepairApplicationOutcome, InterruptedToolRepairApplicationRequest,
    InterruptedToolRepairApplicationStatus, InterruptedToolRepairEffect,
    InterruptedToolRepairError, InterruptedToolRepairIssue, InterruptedToolRepairMode,
    InterruptedToolRepairPlan, InterruptedToolRepairReceipt, InterruptedToolRepairSensitiveDraft,
    InterruptedToolReplayState, InterruptedToolResult,
};
pub use evidence::project_interrupted_tool_repair_receipt;

#[cfg(test)]
mod tests;

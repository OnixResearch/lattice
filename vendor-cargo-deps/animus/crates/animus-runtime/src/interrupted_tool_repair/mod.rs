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

pub use core::diagnose_interrupted_tool_branch;
pub use core::evaluate_interrupted_tool_provider_gate;
pub use core::evaluate_interrupted_tool_repair_application;
pub use core::interrupted_tool_repair_application_request;
pub use core::plan_interrupted_tool_repair;
pub use core::project_interrupted_tool_repair_results;
pub use core::reduce_interrupted_tool_reason;

pub use data::INTERRUPTED_TOOL_ACTION_HASH_DOMAIN;
pub use data::INTERRUPTED_TOOL_LEAF_HASH_DOMAIN;
pub use data::INTERRUPTED_TOOL_PLAN_HASH_DOMAIN;
pub use data::INTERRUPTED_TOOL_RECEIPT_HASH_DOMAIN;
pub use data::INTERRUPTED_TOOL_REPAIR_SCHEMA;
pub use data::INTERRUPTED_TOOL_SNAPSHOT_HASH_DOMAIN;
pub use data::InterruptedToolBranchEntry;
pub use data::InterruptedToolBranchSnapshot;
pub use data::InterruptedToolCall;
pub use data::InterruptedToolDiagnosis;
pub use data::InterruptedToolDiagnosisStatus;
pub use data::InterruptedToolDurability;
pub use data::InterruptedToolEntryKind;
pub use data::InterruptedToolExecutorObservation;
pub use data::InterruptedToolGap;
pub use data::InterruptedToolObservation;
pub use data::InterruptedToolObservationKind;
pub use data::InterruptedToolProviderGateDecision;
pub use data::InterruptedToolProviderGateStatus;
pub use data::InterruptedToolReason;
pub use data::InterruptedToolRepairAction;
pub use data::InterruptedToolRepairApplicationDraft;
pub use data::InterruptedToolRepairApplicationOutcome;
pub use data::InterruptedToolRepairApplicationRequest;
pub use data::InterruptedToolRepairApplicationStatus;
pub use data::InterruptedToolRepairEffect;
pub use data::InterruptedToolRepairError;
pub use data::InterruptedToolRepairIssue;
pub use data::InterruptedToolRepairMode;
pub use data::InterruptedToolRepairPlan;
pub use data::InterruptedToolRepairReceipt;
pub use data::InterruptedToolRepairSensitiveDraft;
pub use data::InterruptedToolReplayState;
pub use data::InterruptedToolResult;
pub use evidence::project_interrupted_tool_repair_receipt;

#[cfg(test)]
mod tests;

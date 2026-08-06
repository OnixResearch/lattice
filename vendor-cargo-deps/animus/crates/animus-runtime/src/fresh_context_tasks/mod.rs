//! Pure fresh-context task and side-question contracts.

// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while
// drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(compound_condition, excessive_file_length, function_length, non_trait_imports)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
mod core;
mod data;

pub use core::blake3_fresh_task_fingerprint;
pub use core::evaluate_fresh_task_dispatch;
pub use core::evaluate_side_question_action;
pub use core::validate_fresh_task_descriptor;
pub use core::validate_fresh_task_result;

pub use data::FreshTaskActionKind;
pub use data::FreshTaskBudget;
pub use data::FreshTaskCancellationReason;
pub use data::FreshTaskDescriptor;
pub use data::FreshTaskDiagnostic;
pub use data::FreshTaskDiagnosticKind;
pub use data::FreshTaskDispatchDecision;
pub use data::FreshTaskFingerprint;
pub use data::FreshTaskInheritance;
pub use data::FreshTaskIsolation;
pub use data::FreshTaskMode;
pub use data::FreshTaskReceipt;
pub use data::FreshTaskResult;
pub use data::FreshTaskResultStatus;
pub use data::FreshTaskSchedulerState;
pub use data::FreshTaskValidationResult;

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const FRESH_TASK_HASH_DOMAIN: &str = "animus.fresh-context-task.v1";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const FIELD_ACTION: &str = "action";
pub(crate) const FIELD_DESCRIPTOR: &str = "descriptor";
pub(crate) const FIELD_RESULT: &str = "result";
pub(crate) const FIELD_SCHEDULER: &str = "scheduler";
pub(crate) const REQUIRED_COUNT: u32 = 1;

#[cfg(test)]
mod tests;

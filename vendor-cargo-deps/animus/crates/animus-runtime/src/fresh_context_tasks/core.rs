// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven
// validation compact while drained changes are archived.
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
use super::BLAKE3_ALGORITHM;
use super::FIELD_ACTION;
use super::FIELD_DESCRIPTOR;
use super::FIELD_RESULT;
use super::FIELD_SCHEDULER;
use super::FRESH_TASK_HASH_DOMAIN;
use super::HASH_PART_SEPARATOR;
use super::REQUIRED_COUNT;
use super::data::FreshTaskActionKind;
use super::data::FreshTaskDescriptor;
use super::data::FreshTaskDiagnostic;
use super::data::FreshTaskDiagnosticKind;
use super::data::FreshTaskDispatchDecision;
use super::data::FreshTaskFingerprint;
use super::data::FreshTaskInheritance;
use super::data::FreshTaskIsolation;
use super::data::FreshTaskMode;
use super::data::FreshTaskResult;
use super::data::FreshTaskResultStatus;
use super::data::FreshTaskSchedulerState;
use super::data::FreshTaskValidationResult;

const RAW_SECRET_MARKER: &str = "API_KEY";
const RAW_PATH_MARKER: &str = "/home/";
const RAW_OUTPUT_MARKER: &str = "tool_output:";

/// Computes a fresh-task BLAKE3 fingerprint.
#[must_use]
pub fn blake3_fresh_task_fingerprint(label: &str, bytes: &[u8]) -> FreshTaskFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FRESH_TASK_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    FreshTaskFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Validates a fresh-context task descriptor.
///
/// # Errors
///
/// Returns diagnostics for missing goal/context/tool/model/schema fields or
/// invalid budgets.
pub fn validate_fresh_task_descriptor(descriptor: &FreshTaskDescriptor) -> FreshTaskValidationResult {
    let mut diagnostics = Vec::new();
    require_text(&descriptor.descriptor_id, FIELD_DESCRIPTOR, &mut diagnostics);
    require_text(&descriptor.goal, FIELD_DESCRIPTOR, &mut diagnostics);
    require_text(&descriptor.model_policy, FIELD_DESCRIPTOR, &mut diagnostics);
    require_text(&descriptor.result_schema, FIELD_DESCRIPTOR, &mut diagnostics);
    require_collection(&descriptor.context_references, FIELD_DESCRIPTOR, &mut diagnostics);
    if descriptor.allowed_actions.is_empty()
        || descriptor.budget.tokens < REQUIRED_COUNT
        || descriptor.budget.tool_calls < REQUIRED_COUNT
        || descriptor.budget.timeout_label.trim().is_empty()
    {
        diagnostics.push(FreshTaskDiagnostic::new(
            FreshTaskDiagnosticKind::BudgetInvalid,
            FIELD_DESCRIPTOR,
            "fresh-context tasks require actions, positive budgets, and timeout policy",
        ));
    }
    to_validation_result(diagnostics)
}

/// Evaluates host scheduler dispatch readiness.
#[must_use]
pub fn evaluate_fresh_task_dispatch(
    descriptor: &FreshTaskDescriptor,
    isolation: &FreshTaskIsolation,
    scheduler: FreshTaskSchedulerState,
) -> FreshTaskDispatchDecision {
    let mut diagnostics = validate_fresh_task_descriptor(descriptor).err().unwrap_or_default();
    if scheduler != FreshTaskSchedulerState::Ready {
        diagnostics.push(FreshTaskDiagnostic::new(
            FreshTaskDiagnosticKind::MissingScheduler,
            FIELD_SCHEDULER,
            "fresh-context task scheduler is not ready",
        ));
    }
    if isolation.parent_allowlist == FreshTaskInheritance::Inherited
        || isolation.hidden_prompts == FreshTaskInheritance::Inherited
        || isolation.parent_secrets == FreshTaskInheritance::Inherited
        || isolation.process_globals == FreshTaskInheritance::Inherited
    {
        diagnostics.push(FreshTaskDiagnostic::new(
            FreshTaskDiagnosticKind::IsolationViolation,
            FIELD_DESCRIPTOR,
            "fresh-context task inherited parent-only authority or mutable global state",
        ));
    }
    FreshTaskDispatchDecision {
        dispatch: diagnostics.is_empty(),
        diagnostics,
    }
}

/// Evaluates read-only side-question action constraints.
#[must_use]
pub fn evaluate_side_question_action(
    descriptor: &FreshTaskDescriptor,
    action: FreshTaskActionKind,
) -> FreshTaskDispatchDecision {
    let denied = descriptor.mode == FreshTaskMode::SideQuestion
        && !matches!(action, FreshTaskActionKind::Read | FreshTaskActionKind::Search);
    if denied {
        FreshTaskDispatchDecision {
            dispatch: false,
            diagnostics: vec![FreshTaskDiagnostic::new(
                FreshTaskDiagnosticKind::ReadOnlyDenied,
                FIELD_ACTION,
                "read-only side question denied a non-read child action",
            )],
        }
    } else {
        FreshTaskDispatchDecision {
            dispatch: true,
            diagnostics: Vec::new(),
        }
    }
}

/// Validates an attributed child result before parent prompt influence.
///
/// # Errors
///
/// Returns diagnostics when required identity/status/schema fields are absent,
/// hashes are invalid, citations are missing, or evidence contains unsafe raw
/// material.
pub fn validate_fresh_task_result(result: &FreshTaskResult) -> FreshTaskValidationResult {
    let mut diagnostics = Vec::new();
    require_text(&result.child_run_id, FIELD_RESULT, &mut diagnostics);
    require_text(&result.descriptor_id, FIELD_RESULT, &mut diagnostics);
    require_collection(&result.citations, FIELD_RESULT, &mut diagnostics);
    require_collection(&result.evidence_summaries, FIELD_RESULT, &mut diagnostics);
    if result.status == FreshTaskResultStatus::Missing {
        diagnostics.push(FreshTaskDiagnostic::new(
            FreshTaskDiagnosticKind::MalformedResult,
            FIELD_RESULT,
            "fresh-context task result status is missing",
        ));
    }
    if result.output_hash.algorithm != BLAKE3_ALGORITHM || result.output_hash.value_hex.is_empty() {
        diagnostics.push(FreshTaskDiagnostic::new(
            FreshTaskDiagnosticKind::MalformedResult,
            FIELD_RESULT,
            "fresh-context task output hash must be BLAKE3",
        ));
    }
    for summary in &result.evidence_summaries {
        if contains_unsafe(summary) {
            diagnostics.push(FreshTaskDiagnostic::new(
                FreshTaskDiagnosticKind::UnsafeResult,
                FIELD_RESULT,
                "fresh-context task result contains unsafe raw material marker",
            ));
        }
    }
    to_validation_result(diagnostics)
}

fn contains_unsafe(text: &str) -> bool {
    text.contains(RAW_SECRET_MARKER) || text.contains(RAW_PATH_MARKER) || text.contains(RAW_OUTPUT_MARKER)
}

fn require_collection(values: &[String], field: &str, diagnostics: &mut Vec<FreshTaskDiagnostic>) {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        diagnostics.push(FreshTaskDiagnostic::new(
            FreshTaskDiagnosticKind::MissingField,
            field,
            "required fresh-context collection is empty or blank",
        ));
    }
}

fn require_text(text: &str, field: &str, diagnostics: &mut Vec<FreshTaskDiagnostic>) {
    if text.trim().is_empty() {
        diagnostics.push(FreshTaskDiagnostic::new(
            FreshTaskDiagnosticKind::MissingField,
            field,
            "required fresh-context field is empty",
        ));
    }
}

fn to_validation_result(diagnostics: Vec<FreshTaskDiagnostic>) -> FreshTaskValidationResult {
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

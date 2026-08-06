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
/// BLAKE3 fingerprint for fresh-context tasks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshTaskFingerprint {
    /// Hash algorithm.
    pub algorithm: String,
    /// Hex digest.
    pub value_hex: String,
}

impl FreshTaskFingerprint {
    /// Builds a fingerprint.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value_hex: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value_hex: value_hex.into(),
        }
    }
}

/// Child action kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreshTaskActionKind {
    /// Read file or context.
    Read,
    /// Search/index query.
    Search,
    /// Write file.
    Write,
    /// Edit state.
    Edit,
    /// Shell command.
    Shell,
    /// Network mutation.
    Network,
    /// Ledger mutation.
    LedgerMutation,
    /// Other non-read tool.
    OtherTool,
}

/// Budget inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshTaskBudget {
    /// Token budget.
    pub tokens: u32,
    /// Tool-call budget.
    pub tool_calls: u32,
    /// Timeout class label.
    pub timeout_label: String,
}

/// Task mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreshTaskMode {
    /// Fresh-context investigation.
    Investigation,
    /// Read-only side question.
    SideQuestion,
}

/// Inheritance state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreshTaskInheritance {
    /// Parent state is inherited.
    Inherited,
    /// Parent state is isolated.
    Isolated,
}

/// Fresh-context task descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshTaskDescriptor {
    /// Task descriptor ID.
    pub descriptor_id: String,
    /// Goal text.
    pub goal: String,
    /// Explicitly admitted context references.
    pub context_references: Vec<String>,
    /// Allowed actions.
    pub allowed_actions: Vec<FreshTaskActionKind>,
    /// Model/provider policy label.
    pub model_policy: String,
    /// Budgets.
    pub budget: FreshTaskBudget,
    /// Result schema label.
    pub result_schema: String,
    /// Task mode.
    pub mode: FreshTaskMode,
}

/// Child isolation proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshTaskIsolation {
    /// Parent allowlist inheritance state.
    pub parent_allowlist: FreshTaskInheritance,
    /// Hidden prompt inheritance state.
    pub hidden_prompts: FreshTaskInheritance,
    /// Parent secret inheritance state.
    pub parent_secrets: FreshTaskInheritance,
    /// Process-global mutable config inheritance state.
    pub process_globals: FreshTaskInheritance,
}

/// Scheduler availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreshTaskSchedulerState {
    /// Scheduler ready.
    Ready,
    /// Scheduler missing.
    Missing,
    /// Scheduler denied.
    Denied,
    /// Scheduler cancelled.
    Cancelled,
}

/// Dispatch decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshTaskDispatchDecision {
    /// Whether host scheduler may dispatch.
    pub dispatch: bool,
    /// Diagnostics.
    pub diagnostics: Vec<FreshTaskDiagnostic>,
}

/// Result status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreshTaskResultStatus {
    /// Completed.
    Completed,
    /// Cancelled.
    Cancelled,
    /// Failed.
    Failed,
    /// Malformed fixture.
    Missing,
}

/// Attributed task result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshTaskResult {
    /// Child run ID.
    pub child_run_id: String,
    /// Task descriptor ID.
    pub descriptor_id: String,
    /// Status.
    pub status: FreshTaskResultStatus,
    /// Budget used.
    pub budget_used: FreshTaskBudget,
    /// Cited context references.
    pub citations: Vec<String>,
    /// Uncertainty markers.
    pub uncertainty_markers: Vec<String>,
    /// Redacted evidence summaries.
    pub evidence_summaries: Vec<String>,
    /// Output hash.
    pub output_hash: FreshTaskFingerprint,
}

/// Cancellation reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreshTaskCancellationReason {
    /// Not cancelled.
    None,
    /// User cancellation.
    User,
    /// Timeout cancellation.
    Timeout,
    /// Policy cancellation.
    Policy,
    /// Scheduler failure.
    SchedulerFailure,
}

/// Safe task receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshTaskReceipt {
    /// Task ID.
    pub descriptor_id: String,
    /// Scheduler identity.
    pub scheduler_id: String,
    /// Status.
    pub status: FreshTaskResultStatus,
    /// Cancellation reason.
    pub cancellation_reason: FreshTaskCancellationReason,
    /// Budget used.
    pub budget_used: FreshTaskBudget,
    /// Output hash.
    pub output_hash: FreshTaskFingerprint,
    /// Full child transcript embedding state.
    pub transcript_embedding: FreshTaskInheritance,
}

/// Diagnostic class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreshTaskDiagnosticKind {
    /// Missing required field.
    MissingField,
    /// Missing scheduler.
    MissingScheduler,
    /// Read-only side question denied action.
    ReadOnlyDenied,
    /// Isolation violation.
    IsolationViolation,
    /// Budget invalid or exhausted.
    BudgetInvalid,
    /// Result malformed.
    MalformedResult,
    /// Unsafe result evidence.
    UnsafeResult,
}

/// Fresh task diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshTaskDiagnostic {
    /// Kind.
    pub kind: FreshTaskDiagnosticKind,
    /// Field.
    pub field: String,
    /// Detail.
    pub detail: String,
}

impl FreshTaskDiagnostic {
    /// Builds a diagnostic.
    #[must_use]
    pub fn new(kind: FreshTaskDiagnosticKind, field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Validation result.
pub type FreshTaskValidationResult = Result<(), Vec<FreshTaskDiagnostic>>;

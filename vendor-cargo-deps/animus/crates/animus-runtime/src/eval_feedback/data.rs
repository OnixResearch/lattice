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
/// BLAKE3 or explicitly declared interoperability fingerprint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalFeedbackFingerprint {
    /// Hash algorithm name.
    pub algorithm: String,
    /// Lowercase hexadecimal digest value.
    pub value_hex: String,
}

impl EvalFeedbackFingerprint {
    /// Builds an evaluation-feedback fingerprint from precomputed parts.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value_hex: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value_hex: value_hex.into(),
        }
    }
}

/// Task-set location class for offline evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalFeedbackTaskSetKind {
    /// Local checked fixture or local task set.
    Local,
    /// Remote task sets are outside this contract.
    Remote,
}

/// Task-set identity for an offline evaluation plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalFeedbackTaskSet {
    /// Safe task-set label.
    pub label: String,
    /// Task-set location class.
    pub kind: EvalFeedbackTaskSetKind,
}

/// Verifier identity used by an offline evaluation plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalFeedbackVerifierIdentity {
    /// Safe verifier label.
    pub label: String,
    /// BLAKE3 verifier hash.
    pub hash: EvalFeedbackFingerprint,
}

/// Supported output formats for local offline evaluation evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalFeedbackOutputFormat {
    /// JSON Lines output.
    Jsonl,
    /// Markdown summary output.
    Markdown,
    /// Compact summary output.
    Summary,
    /// Unsupported format fixture.
    Unsupported,
}

/// Explicit unsupported-scope request classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalFeedbackScopeRequest {
    /// Reinforcement-learning training request.
    RlTraining,
    /// Live runtime self-mutation request.
    LiveSelfMutation,
    /// Cloud benchmark runner request.
    CloudBenchmarkRunner,
    /// Hidden-test access request.
    HiddenTestAccess,
    /// Provider-specific evaluation API request.
    ProviderEvalApi,
    /// Public benchmark claim request.
    PublicBenchmarkClaim,
}

/// Offline scaffold evaluation plan DTO.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalFeedbackPlan {
    /// Task-set identity.
    pub task_set: EvalFeedbackTaskSet,
    /// Scaffold revision identifier.
    pub scaffold_revision_id: String,
    /// Scaffold revision hash.
    pub scaffold_hash: EvalFeedbackFingerprint,
    /// Model route label.
    pub model_route: String,
    /// Verifier identity and hash.
    pub verifier: EvalFeedbackVerifierIdentity,
    /// Trust-boundary policy hash.
    pub trust_boundary_hash: EvalFeedbackFingerprint,
    /// Tool-surface hash.
    pub tool_surface_hash: EvalFeedbackFingerprint,
    /// Sampling profile label.
    pub sampling_profile: String,
    /// Resource and timeout labels.
    pub resource_labels: Vec<String>,
    /// Number of local runs.
    pub run_count: u32,
    /// Output format.
    pub output_format: EvalFeedbackOutputFormat,
    /// Unsupported-scope requests found in the workflow.
    pub unsupported_scope_requests: Vec<EvalFeedbackScopeRequest>,
}

/// Aggregate status for local evaluation receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalFeedbackRunStatus {
    /// All required local runs passed.
    Passed,
    /// One or more required local runs failed.
    Failed,
    /// Evaluation was incomplete.
    Incomplete,
}

/// Receipt redaction state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalFeedbackRedactionState {
    /// Receipt contains safe metadata only.
    Redacted,
    /// Receipt contains unsafe material.
    Unsafe,
}

/// Sensitive receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalFeedbackReceiptLeak {
    /// Raw prompt text.
    RawPrompt,
    /// Hidden test content.
    HiddenTest,
    /// Provider payload.
    ProviderPayload,
    /// Credential-looking value.
    Credential,
    /// Full path dump.
    FullPath,
    /// Raw verifier body.
    VerifierBody,
    /// Unbounded tool output.
    UnboundedToolOutput,
}

/// Safe evaluation receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalFeedbackReceipt {
    /// Model identity label.
    pub model_id: String,
    /// Model route label.
    pub model_route: String,
    /// Scaffold hash.
    pub scaffold_hash: EvalFeedbackFingerprint,
    /// Trust-boundary hash.
    pub trust_boundary_hash: EvalFeedbackFingerprint,
    /// Tool-surface hash.
    pub tool_surface_hash: EvalFeedbackFingerprint,
    /// Verifier hash.
    pub verifier_hash: EvalFeedbackFingerprint,
    /// Runtime version label.
    pub runtime_version_label: String,
    /// Context-window class label.
    pub context_window_class: String,
    /// Sampling profile label.
    pub sampling_profile: String,
    /// Sampling labels safe for comparison.
    pub sampling_labels: Vec<String>,
    /// Timeout and resource labels.
    pub resource_labels: Vec<String>,
    /// Number of local runs.
    pub run_count: u32,
    /// Aggregate run status.
    pub status: EvalFeedbackRunStatus,
    /// Redaction state.
    pub redaction_state: EvalFeedbackRedactionState,
    /// Negative-test-only leak classes.
    pub leaks: Vec<EvalFeedbackReceiptLeak>,
}

/// Feedback candidate kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalFeedbackCandidateKind {
    /// Promote a scaffold revision.
    Promote,
    /// Demote a scaffold revision.
    Demote,
    /// Revise a scaffold revision.
    Revise,
    /// Investigate further.
    Investigate,
}

/// Host review outcome for a feedback candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalFeedbackReviewOutcome {
    /// Host explicitly approved the candidate.
    Approved,
    /// Host denied the candidate.
    Denied,
    /// Host review is unavailable.
    Unavailable,
    /// Evidence is stale.
    StaleEvidence,
    /// Review has not happened.
    Missing,
}

/// Offline feedback candidate record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalFeedbackCandidate {
    /// Candidate identifier.
    pub candidate_id: String,
    /// Candidate recommendation kind.
    pub kind: EvalFeedbackCandidateKind,
    /// Scaffold revision identifier under review.
    pub scaffold_revision_id: String,
    /// Evidence receipt hash.
    pub receipt_hash: EvalFeedbackFingerprint,
    /// Host review outcome.
    pub review_outcome: EvalFeedbackReviewOutcome,
}

/// Review decision for a feedback candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalFeedbackReviewDecision {
    /// Whether candidate can proceed to scaffold admission.
    pub can_request_scaffold_admission: bool,
    /// Whether candidate can activate a scaffold directly.
    pub activates_scaffold: bool,
    /// Whether candidate may alter runtime selection policy directly.
    pub changes_runtime_selection_policy: bool,
    /// Diagnostics for non-authoritative outcomes.
    pub diagnostics: Vec<EvalFeedbackDiagnostic>,
}

/// Current policy used to classify evidence freshness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalFeedbackFreshnessPolicy {
    /// Current model route label.
    pub model_route: String,
    /// Current scaffold hash.
    pub scaffold_hash: EvalFeedbackFingerprint,
    /// Current verifier hash.
    pub verifier_hash: EvalFeedbackFingerprint,
    /// Current trust-boundary hash.
    pub trust_boundary_hash: EvalFeedbackFingerprint,
    /// Current tool-surface hash.
    pub tool_surface_hash: EvalFeedbackFingerprint,
    /// Current runtime version label.
    pub runtime_version_label: String,
    /// Current sampling profile label.
    pub sampling_profile: String,
    /// Whether stale evidence should be downweighted instead of fully stale.
    pub reduce_weight_on_mismatch: bool,
}

/// Evidence freshness decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalFeedbackFreshnessDecision {
    /// Evidence matches current policy exactly.
    Current,
    /// Evidence mismatches current policy but may be reduced-weight.
    ReducedWeight,
    /// Evidence is stale and cannot justify promotion by itself.
    Stale,
}

/// Evaluation feedback diagnostic class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalFeedbackDiagnosticKind {
    /// Required text is missing.
    MissingField,
    /// Required collection is empty.
    EmptyCollection,
    /// Fingerprint is missing or not BLAKE3.
    InvalidHash,
    /// Remote task set is unsupported.
    RemoteTaskSet,
    /// Output format is unsupported.
    UnsupportedOutputFormat,
    /// Run count is not positive.
    NonPositiveRunCount,
    /// Scope request is unsupported.
    UnsupportedScope,
    /// Receipt contains unsafe material.
    ReceiptLeak,
    /// Feedback review is non-authoritative.
    ReviewNotAccepted,
    /// Evidence is stale or reduced-weight.
    FreshnessMismatch,
}

/// Evaluation feedback diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalFeedbackDiagnostic {
    /// Diagnostic class.
    pub kind: EvalFeedbackDiagnosticKind,
    /// Stable field label.
    pub field: String,
    /// Safe detail text.
    pub detail: String,
}

impl EvalFeedbackDiagnostic {
    /// Builds an evaluation feedback diagnostic.
    #[must_use]
    pub fn new(kind: EvalFeedbackDiagnosticKind, field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Evaluation feedback validation result.
pub type EvalFeedbackValidationResult = Result<(), Vec<EvalFeedbackDiagnostic>>;

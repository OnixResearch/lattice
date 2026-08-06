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
/// Fingerprint for compaction-sensitive content identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiCompactionFingerprint {
    /// Hash algorithm name.
    pub algorithm: String,
    /// Lowercase hexadecimal digest value.
    pub value_hex: String,
}

impl PiCompactionFingerprint {
    /// Builds a compaction fingerprint from precomputed parts.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value_hex: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value_hex: value_hex.into(),
        }
    }
}

/// Summary body format selected by policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PiCompactionSummaryFormat {
    /// Markdown bullet summary.
    Markdown,
    /// Structured JSON-compatible summary.
    Structured,
    /// Human-authored prose summary.
    HumanAuthored,
}

/// Summarizer selection class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PiCompactionSummarizerSelection {
    /// Deterministic local summarizer.
    Deterministic,
    /// Host-injected model summarizer.
    HostInjectedModel,
    /// Host-provided human summary.
    Human,
}

/// Redaction behavior for summary metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PiCompactionRedactionBehavior {
    /// Resource identities must be already redacted labels or hashes.
    RequireRedactedResourceIds,
    /// Only aggregate counts and hashes are stored.
    HashOnly,
}

/// Typed compaction policy inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiCompactionPolicy {
    /// Whether automatic compaction is enabled.
    pub enabled: bool,
    /// Total context budget units supplied by the host.
    pub context_budget_units: u32,
    /// Reserve units for the response.
    pub response_reserve_units: u32,
    /// Budget units to keep verbatim at the branch tail.
    pub kept_recent_units: u32,
    /// Requested summary format.
    pub summary_format: PiCompactionSummaryFormat,
    /// Requested summarizer selection.
    pub summarizer_selection: PiCompactionSummarizerSelection,
    /// Redaction behavior for metadata.
    pub redaction_behavior: PiCompactionRedactionBehavior,
}

/// Pure compaction trigger classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PiCompactionTrigger {
    /// Compaction is disabled by policy.
    Disabled,
    /// Host did not provide a context estimate.
    MissingEstimate,
    /// Context is within usable budget.
    UnderBudget,
    /// Context exceeds usable budget and compaction is required.
    Required,
    /// Policy is invalid.
    InvalidPolicy,
}

/// Replay entry kind for in-memory branch compaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PiCompactionReplayEntryKind {
    /// User-authored message.
    User,
    /// Assistant-authored message.
    Assistant,
    /// Assistant tool-call request.
    ToolCall,
    /// Tool result message.
    ToolResult,
    /// Existing bounded summary message.
    Summary,
}

/// Tool-call/result relationship metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiCompactionToolBinding {
    /// Tool call identity.
    pub call_id: String,
}

/// In-memory replay entry used by pure compaction preparation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiCompactionReplayEntry {
    /// Stable entry identity.
    pub id: String,
    /// Stable turn identity.
    pub turn_id: String,
    /// Parent entry identities for branch navigation.
    pub parent_ids: Vec<String>,
    /// Replay entry kind.
    pub kind: PiCompactionReplayEntryKind,
    /// Host-supplied context estimate units for this entry.
    pub estimate_units: u32,
    /// Tool binding for call/result entries.
    pub tool_binding: Option<PiCompactionToolBinding>,
    /// Safe content hash.
    pub content_hash: PiCompactionFingerprint,
}

/// Pure compaction preparation result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiCompactionPlan {
    /// Trigger state.
    pub trigger: PiCompactionTrigger,
    /// Index of first kept entry when summarization is required.
    pub cut_index: Option<usize>,
    /// Entry IDs summarized before the kept suffix.
    pub summarized_entry_ids: Vec<String>,
    /// Entry IDs kept verbatim.
    pub kept_entry_ids: Vec<String>,
    /// Entry IDs summarized from an early oversized turn prefix.
    pub split_turn_prefix_entry_ids: Vec<String>,
    /// Whether one turn was split by compaction.
    pub split_turn: bool,
    /// Diagnostics for invalid or non-required cases.
    pub diagnostics: Vec<PiCompactionDiagnostic>,
}

/// Branch summary mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PiBranchSummaryMode {
    /// Host did not request a branch handoff summary.
    NotRequested,
    /// Host requested a branch handoff summary.
    Requested,
}

/// Branch handoff summary request over already-loaded graph data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiBranchHandoffRequest {
    /// Old active leaf.
    pub old_leaf_id: String,
    /// Target leaf after navigation.
    pub target_leaf_id: String,
    /// Request mode.
    pub mode: PiBranchSummaryMode,
    /// Branch graph entries.
    pub entries: Vec<PiCompactionReplayEntry>,
}

/// Branch handoff plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiBranchHandoffPlan {
    /// Whether a summary should be requested from the host.
    pub summary_required: bool,
    /// Common ancestor entry identity.
    pub common_ancestor_id: Option<String>,
    /// Old branch span IDs to summarize.
    pub abandoned_entry_ids: Vec<String>,
    /// Target branch IDs preserved after navigation.
    pub target_entry_ids: Vec<String>,
    /// Diagnostics for invalid branch requests.
    pub diagnostics: Vec<PiCompactionDiagnostic>,
}

/// Status of an injected summarizer service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PiCompactionSummarizerState {
    /// Service is ready to receive a summarization request.
    Ready,
    /// Service is missing.
    Missing,
    /// Service denied the request.
    Denied,
    /// Service was cancelled.
    Cancelled,
    /// Service returned malformed output.
    MalformedOutput,
}

/// Pure summarizer request prepared for host dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiCompactionSummarizerRequest {
    /// Summary format selected by policy.
    pub summary_format: PiCompactionSummaryFormat,
    /// Summarizer selected by policy.
    pub summarizer_selection: PiCompactionSummarizerSelection,
    /// Entry IDs to summarize.
    pub summarized_entry_ids: Vec<String>,
    /// Kept suffix entry IDs.
    pub kept_entry_ids: Vec<String>,
}

/// Host-dispatch decision for required summarization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiCompactionDecision {
    /// Whether host dispatch may proceed.
    pub dispatch_summarizer: bool,
    /// Prepared request for host dispatch.
    pub request: Option<PiCompactionSummarizerRequest>,
    /// Diagnostics explaining failures.
    pub diagnostics: Vec<PiCompactionDiagnostic>,
}

/// Safe metadata entry produced after summarization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiCompactionSummaryEntry {
    /// Bounded summary text.
    pub summary_text: String,
    /// Number of source entries summarized.
    pub source_entry_count: u32,
    /// Number of source turns summarized.
    pub source_turn_count: u32,
    /// Redacted resource identifiers.
    pub redacted_resource_ids: Vec<String>,
    /// Cumulative read resource identifiers.
    pub cumulative_read_resource_ids: Vec<String>,
    /// Cumulative modified resource identifiers.
    pub cumulative_modified_resource_ids: Vec<String>,
    /// Source content hash.
    pub source_hash: PiCompactionFingerprint,
    /// Summary content hash.
    pub summary_hash: PiCompactionFingerprint,
    /// Whether the summary represents a split turn.
    pub split_turn: bool,
    /// Redaction behavior used for validation.
    pub redaction_behavior: PiCompactionRedactionBehavior,
}

/// Diagnostic class for Pi compaction contracts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PiCompactionDiagnosticKind {
    /// Required field is missing.
    MissingField,
    /// Numeric policy is invalid.
    InvalidBudget,
    /// Context estimate is missing.
    MissingEstimate,
    /// Hash is missing or not BLAKE3.
    InvalidHash,
    /// Cut point would orphan a tool result from its call.
    OrphanToolResult,
    /// Required summarizer service is unavailable or failed.
    SummarizerUnavailable,
    /// Summary text is empty or too long.
    InvalidSummaryText,
    /// Summary metadata contains unsafe raw material.
    UnsafeMetadata,
    /// Branch request is malformed.
    InvalidBranch,
}

/// Diagnostic emitted by compaction validation/preparation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiCompactionDiagnostic {
    /// Diagnostic class.
    pub kind: PiCompactionDiagnosticKind,
    /// Stable field label.
    pub field: String,
    /// Safe detail string.
    pub detail: String,
}

impl PiCompactionDiagnostic {
    /// Builds a compaction diagnostic.
    #[must_use]
    pub fn new(kind: PiCompactionDiagnosticKind, field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Compaction validation result.
pub type PiCompactionValidationResult = Result<(), Vec<PiCompactionDiagnostic>>;

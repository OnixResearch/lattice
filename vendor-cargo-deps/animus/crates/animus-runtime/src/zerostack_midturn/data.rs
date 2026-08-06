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
/// Fingerprint for mid-turn compaction content identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZerostackMidturnFingerprint {
    /// Hash algorithm name.
    pub algorithm: String,
    /// Lowercase hexadecimal digest value.
    pub value_hex: String,
}

impl ZerostackMidturnFingerprint {
    /// Builds a mid-turn fingerprint from precomputed parts.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value_hex: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value_hex: value_hex.into(),
        }
    }
}

/// Provider cached-token interpretation rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZerostackCachedTokenRule {
    /// Cached tokens are already included in input-token counts.
    IncludedInInputTokens,
    /// Cached tokens are reported separately and must be added.
    SeparateFromInputTokens,
    /// Route does not report cached-token fields.
    Absent,
    /// Route metadata is unknown or malformed.
    Unknown,
}

/// Provider route metadata used for prompt pressure calibration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZerostackRouteMetadata {
    /// Safe route label.
    pub route_label: String,
    /// Cached-token interpretation rule.
    pub cached_token_rule: ZerostackCachedTokenRule,
}

/// Provider usage metadata supplied by a host adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZerostackProviderUsage {
    /// Estimated replay token count from local context building.
    pub estimated_replay: u32,
    /// Provider-reported prompt/input token count.
    pub provider_input: Option<u32>,
    /// Provider-reported cached token count, if any.
    pub cached: Option<u32>,
}

/// Calibrated prompt pressure report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZerostackPromptPressure {
    /// Prompt pressure tokens after route-specific cache handling.
    pub prompt_pressure_tokens: u32,
    /// Estimated replay tokens kept separately for audit.
    pub estimated_replay_tokens: u32,
    /// Cache rule used in the calculation.
    pub cache_rule_used: ZerostackCachedTokenRule,
    /// Route label used in the calculation.
    pub route_label: String,
}

/// Required host service state for mid-turn compaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZerostackInjectedServiceState {
    /// Service is ready.
    Ready,
    /// Service is missing.
    Missing,
    /// Service denied the request.
    Denied,
    /// Service cancelled the request.
    Cancelled,
    /// Service returned malformed output.
    MalformedOutput,
}

/// Host services required by explicit stop/compact/resume planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZerostackMidturnServices {
    /// Ledger append service state.
    pub ledger: ZerostackInjectedServiceState,
    /// Summarizer service state.
    pub summarizer: ZerostackInjectedServiceState,
}

/// Mid-turn policy-selected decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZerostackMidturnDecision {
    /// Continue without a compaction plan.
    Continue,
    /// Continue with a warning.
    Warn,
    /// Fail closed without continuation.
    Fail,
    /// Stop, compact, append summary, and resume after host services succeed.
    StopCompactResume,
}

/// Mid-turn compaction policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZerostackMidturnPolicy {
    /// Whether mid-turn compaction is enabled.
    pub enabled: bool,
    /// Context window tokens.
    pub context_window_tokens: u32,
    /// Reserve tokens for the response.
    pub response_reserve_tokens: u32,
    /// Pressure threshold in basis points of usable context.
    pub threshold_basis_points: u32,
    /// Disabled-policy outcome.
    pub disabled_decision: ZerostackMidturnDecision,
}

/// Explicit mid-turn compaction plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZerostackMidturnPlan {
    /// Decision selected by policy and service availability.
    pub decision: ZerostackMidturnDecision,
    /// Whether runtime must stop the current turn before continuing.
    pub stop_before_continuing: bool,
    /// Whether a compaction summary is required.
    pub compact_before_resume: bool,
    /// Whether runtime may resume after host-owned ledger append.
    pub resume_after_append: bool,
    /// Prompt pressure report used by the plan.
    pub pressure: Option<ZerostackPromptPressure>,
    /// Diagnostics for fail-closed cases.
    pub diagnostics: Vec<ZerostackMidturnDiagnostic>,
}

/// Summary kind returned by an injected summarizer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZerostackMidturnSummaryKind {
    /// Mid-turn compaction summary.
    Midturn,
    /// Branch handoff summary.
    Branch,
    /// Unsupported summary kind fixture.
    Unsupported,
}

/// Summary output prepared for explicit ledger append.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZerostackMidturnSummary {
    /// Bounded summary text.
    pub text: String,
    /// Summary kind.
    pub kind: ZerostackMidturnSummaryKind,
    /// Safe metadata labels.
    pub metadata_labels: Vec<String>,
    /// Source content hash.
    pub source_hash: ZerostackMidturnFingerprint,
    /// Summary content hash.
    pub summary_hash: ZerostackMidturnFingerprint,
}

/// Tool-output artifact policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZerostackToolArtifactPolicy {
    /// Visible head character budget.
    pub visible_head: usize,
    /// Visible tail character budget.
    pub visible_tail: usize,
    /// Full output is artifactized when it exceeds this character limit.
    pub artifact_required_over: usize,
}

/// Bounded model-visible tool-output artifact descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZerostackToolArtifactDescriptor {
    /// Safe artifact label.
    pub artifact_label: String,
    /// Bounded visible head text.
    pub visible_head: String,
    /// Bounded visible tail text.
    pub visible_tail: String,
    /// Total output characters observed by the host.
    pub total_chars: usize,
    /// Whether output was truncated for model-visible context.
    pub truncated: bool,
    /// Whether full raw output is embedded in durable evidence.
    pub embeds_full_output: bool,
    /// BLAKE3 hash of the full output.
    pub artifact_hash: ZerostackMidturnFingerprint,
}

/// Zerostack mid-turn diagnostic class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZerostackMidturnDiagnosticKind {
    /// Required field is missing.
    MissingField,
    /// Provider usage metadata is malformed.
    MalformedProviderUsage,
    /// Policy budget or threshold is invalid.
    InvalidPolicy,
    /// Required ledger service is unavailable.
    LedgerUnavailable,
    /// Required summarizer service is unavailable.
    SummarizerUnavailable,
    /// Summary text, kind, metadata, or hash is invalid.
    InvalidSummary,
    /// Tool artifact policy is invalid.
    InvalidToolArtifactPolicy,
}

/// Typed mid-turn diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZerostackMidturnDiagnostic {
    /// Diagnostic class.
    pub kind: ZerostackMidturnDiagnosticKind,
    /// Stable field label.
    pub field: String,
    /// Safe detail text.
    pub detail: String,
}

impl ZerostackMidturnDiagnostic {
    /// Builds a mid-turn diagnostic.
    #[must_use]
    pub fn new(kind: ZerostackMidturnDiagnosticKind, field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Mid-turn validation result.
pub type ZerostackMidturnValidationResult = Result<(), Vec<ZerostackMidturnDiagnostic>>;

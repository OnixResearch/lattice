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
/// BLAKE3 fingerprint for dataset export rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetFingerprint {
    /// Hash algorithm.
    pub algorithm: String,
    /// Hex digest.
    pub value_hex: String,
}

impl DatasetFingerprint {
    /// Builds a fingerprint.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value_hex: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value_hex: value_hex.into(),
        }
    }
}

/// Dataset export target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatasetTarget {
    /// Supervised fine-tuning chat messages.
    ChatMessages,
    /// Preference row.
    Preference,
    /// Reinforcement-learning trajectory.
    RlTrajectory,
}

/// Replay role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatasetReplayRole {
    /// System text.
    System,
    /// User message.
    User,
    /// Assistant message.
    Assistant,
    /// Tool call marker.
    ToolCall,
    /// Tool result.
    ToolResult,
    /// Hidden reasoning.
    HiddenReasoning,
    /// Unsupported content.
    Unsupported,
}

/// Replay entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetReplayEntry {
    /// Entry ID.
    pub entry_id: String,
    /// Role.
    pub role: DatasetReplayRole,
    /// Content.
    pub content: String,
    /// Source span label.
    pub source_span: String,
}

/// Session source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetSessionSource {
    /// Session ID.
    pub session_id: String,
    /// Selected branch ID.
    pub branch_id: String,
    /// Target.
    pub target: DatasetTarget,
    /// Replay path.
    pub replay: Vec<DatasetReplayEntry>,
}

/// Metadata projection policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatasetMetadataProjection {
    /// Exclude metadata.
    Exclude,
    /// Include safe labels only.
    SafeLabels,
}

/// Export policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetExportPolicy {
    /// Include system text.
    pub system_text: DatasetMetadataProjection,
    /// Include tool calls.
    pub tool_calls: DatasetMetadataProjection,
    /// Include tool results.
    pub tool_results: DatasetMetadataProjection,
    /// Include hidden reasoning.
    pub hidden_reasoning: DatasetMetadataProjection,
    /// Include metadata.
    pub metadata: DatasetMetadataProjection,
    /// Minimum assistant characters.
    pub min_assistant_chars: usize,
}

/// Hugging Face chat message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetChatMessage {
    /// Chat role.
    pub role: String,
    /// Chat content.
    pub content: String,
}

/// Chat dataset row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetChatRow {
    /// Row ID.
    pub row_id: DatasetFingerprint,
    /// Ordered messages.
    pub messages: Vec<DatasetChatMessage>,
    /// Safe source spans.
    pub source_spans: Vec<String>,
}

/// Preference example input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetPreferenceExample {
    /// Prompt messages.
    pub prompt: Vec<DatasetChatMessage>,
    /// Chosen assistant response.
    pub chosen: String,
    /// Rejected assistant response.
    pub rejected: String,
    /// Outcome label.
    pub outcome_label: String,
    /// Safe metadata labels.
    pub metadata: Vec<String>,
}

/// Preference row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetPreferenceRow {
    /// Row ID.
    pub row_id: DatasetFingerprint,
    /// Prompt messages.
    pub prompt: Vec<DatasetChatMessage>,
    /// Chosen assistant response.
    pub chosen: String,
    /// Rejected assistant response.
    pub rejected: String,
    /// Safe metadata labels.
    pub metadata: Vec<String>,
}

/// RL step kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatasetRlStepKind {
    /// Prompt step.
    Prompt,
    /// Action step.
    Action,
    /// Observation step.
    Observation,
}

/// RL trajectory step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetRlStep {
    /// Step ID.
    pub step_id: String,
    /// Step kind.
    pub kind: DatasetRlStepKind,
    /// Content.
    pub content: String,
    /// Linked action or observation ID.
    pub link_id: Option<String>,
}

/// RL trajectory input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetRlTrajectory {
    /// Trajectory ID.
    pub trajectory_id: String,
    /// Ordered steps.
    pub steps: Vec<DatasetRlStep>,
    /// Reward label.
    pub reward_label: Option<String>,
    /// Terminal outcome label.
    pub terminal_outcome: Option<String>,
    /// Safe metadata labels.
    pub metadata: Vec<String>,
}

/// RL trajectory row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetRlTrajectoryRow {
    /// Row ID.
    pub row_id: DatasetFingerprint,
    /// Ordered steps.
    pub steps: Vec<DatasetRlStep>,
    /// Reward label.
    pub reward_label: String,
    /// Terminal outcome.
    pub terminal_outcome: String,
    /// Safe metadata labels.
    pub metadata: Vec<String>,
}

/// Skipped row reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetSkippedRow {
    /// Row label.
    pub row_label: String,
    /// Safe reason.
    pub reason: String,
}

/// Dataset inspection report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetInspectionReport {
    /// Exported row count.
    pub exported_rows: u32,
    /// Skipped rows.
    pub skipped_rows: Vec<DatasetSkippedRow>,
    /// Warning classes.
    pub warning_classes: Vec<String>,
    /// Policy decisions.
    pub policy_decisions: Vec<String>,
    /// Rows rejected for empty or too-short model-visible turns.
    pub quality_warning_count: usize,
    /// Rows rejected for tool, reasoning, metadata, or provenance leakage.
    pub leakage_warning_count: usize,
    /// Safe token or usage summary supplied by a host, when available.
    pub usage_summary: Option<String>,
}

/// Dataset diagnostic kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatasetDiagnosticKind {
    /// Invalid source.
    InvalidSource,
    /// Bad chat row.
    BadChatRow,
    /// Leakage risk.
    Leakage,
    /// Malformed preference row.
    MalformedPreference,
    /// Incomplete RL trajectory.
    IncompleteTrajectory,
}

/// Dataset diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetDiagnostic {
    /// Kind.
    pub kind: DatasetDiagnosticKind,
    /// Field.
    pub field: String,
    /// Detail.
    pub detail: String,
}

impl DatasetDiagnostic {
    /// Builds a diagnostic.
    #[must_use]
    pub fn new(kind: DatasetDiagnosticKind, field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Dataset export result.
pub type DatasetExportResult<T> = Result<T, Vec<DatasetDiagnostic>>;

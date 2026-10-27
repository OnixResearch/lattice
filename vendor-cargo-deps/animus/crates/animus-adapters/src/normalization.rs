//! Authority-free contracts for optional transcript normalization.

/// Reviewed package name.
pub const TRAJECTORY_PACKAGE: &str = "@letta-ai/trajectory";
/// Reviewed package version.
pub const TRAJECTORY_VERSION: &str = "0.2.0";
/// Reviewed source revision.
pub const TRAJECTORY_REVISION: &str = "a665556089d6a3c59f5806ee109b4d36f6c2ac83";
/// Reviewed trajectory-v1 schema identifier.
pub const TRAJECTORY_SCHEMA: &str = "https://letta.ai/schemas/trajectory/v1.json";
/// BLAKE3 identity of the reviewed schema bytes.
pub const TRAJECTORY_SCHEMA_BLAKE3: &str =
    "4193e80e761f1aaa9e59545f356de02d2aa33443b4430fae4d1d82e29bca1212";
/// BLAKE3 identity of the reviewed bundled helper bytes.
pub const TRAJECTORY_HELPER_BLAKE3: &str =
    "1ef86c7687ff8f7354cb08e515e391e66f4e894f892aa807597d4ce399f5de20";
/// Reviewed license identifier.
pub const TRAJECTORY_LICENSE: &str = "Apache-2.0";

/// Supported transcript source.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TrajectorySource {
    /// Claude Code JSONL.
    ClaudeCode,
    /// Codex JSONL.
    Codex,
    /// Hermes JSON.
    Hermes,
    /// Letta Code JSONL.
    LettaCode,
    /// `OpenClaw` JSONL.
    OpenClaw,
    /// `OpenHands` JSON.
    OpenHands,
    /// Pi JSONL.
    Pi,
}

impl TrajectorySource {
    /// Returns the reviewed upstream source label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::Hermes => "hermes",
            Self::LettaCode => "letta-code",
            Self::OpenClaw => "openclaw",
            Self::OpenHands => "openhands",
            Self::Pi => "pi",
        }
    }

    /// Parses one reviewed source label.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "claude-code" => Some(Self::ClaudeCode),
            "codex" => Some(Self::Codex),
            "hermes" => Some(Self::Hermes),
            "letta-code" => Some(Self::LettaCode),
            "openclaw" => Some(Self::OpenClaw),
            "openhands" => Some(Self::OpenHands),
            "pi" => Some(Self::Pi),
            _ => None,
        }
    }
}

/// Named finite bounds applied before effects and during result validation.
#[allow(clippy::struct_field_names)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrajectoryBounds {
    /// Maximum transcript input bytes.
    pub max_transcript_bytes: usize,
    /// Maximum helper output bytes.
    pub max_output_bytes: usize,
    /// Maximum normalized records.
    pub max_records: usize,
    /// Maximum bytes in one content field.
    pub max_content_bytes: usize,
    /// Maximum tool calls.
    pub max_tool_calls: usize,
    /// Maximum serialized tool argument bytes.
    pub max_tool_argument_bytes: usize,
    /// Maximum tool result bytes.
    pub max_tool_result_bytes: usize,
    /// Maximum diagnostics.
    pub max_diagnostics: usize,
}

/// Policy disposition for one reviewed diagnostic code.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticDisposition {
    /// Cleanup is admitted but remains explicit.
    AcceptedCleanup,
    /// The result is admitted as lossy.
    Lossy,
    /// The result is rejected.
    Rejected,
}

/// Reviewed diagnostic code.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TrajectoryDiagnosticCode {
    /// Invalid JSON line was dropped.
    InvalidJsonLine,
    /// Non-object JSON line was dropped.
    NonObjectJsonLine,
    /// Injected context was dropped.
    InjectedContextDropped,
    /// Noise record was dropped.
    NoiseRecordDropped,
    /// Sidechain record was dropped.
    SidechainRecordDropped,
    /// Tool call identity was synthesized.
    ToolCallIdSynthesized,
    /// Duplicate tool call identity was repaired.
    DuplicateToolCallId,
    /// Orphan tool result was observed.
    OrphanToolResult,
    /// Duplicate tool result was observed.
    DuplicateToolResult,
    /// Unknown tool name was observed.
    UnknownToolName,
    /// Tool arguments were reshaped.
    ToolArgumentsReshaped,
    /// Tool arguments were truncated.
    ToolArgumentsTruncated,
    /// Tool result was truncated.
    ToolResultTruncated,
    /// Timestamps were synthesized.
    TimestampsSynthesized,
    /// Timestamps were interpolated.
    TimestampsInterpolated,
}

impl TrajectoryDiagnosticCode {
    /// Returns the upstream diagnostic label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidJsonLine => "invalid_json_line",
            Self::NonObjectJsonLine => "non_object_json_line",
            Self::InjectedContextDropped => "injected_context_dropped",
            Self::NoiseRecordDropped => "noise_record_dropped",
            Self::SidechainRecordDropped => "sidechain_record_dropped",
            Self::ToolCallIdSynthesized => "tool_call_id_synthesized",
            Self::DuplicateToolCallId => "duplicate_tool_call_id",
            Self::OrphanToolResult => "orphan_tool_result",
            Self::DuplicateToolResult => "duplicate_tool_result",
            Self::UnknownToolName => "unknown_tool_name",
            Self::ToolArgumentsReshaped => "tool_arguments_reshaped",
            Self::ToolArgumentsTruncated => "tool_arguments_truncated",
            Self::ToolResultTruncated => "tool_result_truncated",
            Self::TimestampsSynthesized => "timestamps_synthesized",
            Self::TimestampsInterpolated => "timestamps_interpolated",
        }
    }

    /// Parses one reviewed diagnostic label.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "invalid_json_line" => Some(Self::InvalidJsonLine),
            "non_object_json_line" => Some(Self::NonObjectJsonLine),
            "injected_context_dropped" => Some(Self::InjectedContextDropped),
            "noise_record_dropped" => Some(Self::NoiseRecordDropped),
            "sidechain_record_dropped" => Some(Self::SidechainRecordDropped),
            "tool_call_id_synthesized" => Some(Self::ToolCallIdSynthesized),
            "duplicate_tool_call_id" => Some(Self::DuplicateToolCallId),
            "orphan_tool_result" => Some(Self::OrphanToolResult),
            "duplicate_tool_result" => Some(Self::DuplicateToolResult),
            "unknown_tool_name" => Some(Self::UnknownToolName),
            "tool_arguments_reshaped" => Some(Self::ToolArgumentsReshaped),
            "tool_arguments_truncated" => Some(Self::ToolArgumentsTruncated),
            "tool_result_truncated" => Some(Self::ToolResultTruncated),
            "timestamps_synthesized" => Some(Self::TimestampsSynthesized),
            "timestamps_interpolated" => Some(Self::TimestampsInterpolated),
            _ => None,
        }
    }
}

/// One diagnostic policy row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrajectoryDiagnosticPolicy {
    /// Diagnostic code.
    pub code: TrajectoryDiagnosticCode,
    /// Admission disposition.
    pub disposition: DiagnosticDisposition,
}

/// Content projection disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrajectoryContentDisposition {
    /// Include bounded content.
    Include,
    /// Omit content and record the omission class.
    Omit,
}

/// Revalidated transcript adapter policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrajectoryPolicy {
    /// Exact helper identity.
    pub helper_identity: alloc::string::String,
    /// Exact schema identity.
    pub schema_identity: alloc::string::String,
    /// Allowed sources.
    pub sources: alloc::vec::Vec<TrajectorySource>,
    /// Finite bounds.
    pub bounds: TrajectoryBounds,
    /// Diagnostic dispositions.
    pub diagnostics: alloc::vec::Vec<TrajectoryDiagnosticPolicy>,
    /// Reasoning projection posture.
    pub reasoning: TrajectoryContentDisposition,
    /// Tool-argument projection posture.
    pub tool_arguments: TrajectoryContentDisposition,
    /// Tool-result projection posture.
    pub tool_results: TrajectoryContentDisposition,
    /// Source path metadata posture.
    pub path_metadata: TrajectoryContentDisposition,
}

/// One transient normalization request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrajectoryNormalizeRequest {
    /// Transcript source.
    pub source: TrajectorySource,
    /// Already-loaded raw transcript, transient at the host boundary.
    pub transcript: alloc::string::String,
    /// Caller-supplied immutable source-generation identity.
    pub source_generation_identity: alloc::string::String,
    /// Revalidated policy identity.
    pub policy_identity: alloc::string::String,
    /// Expected helper identity.
    pub helper_identity: alloc::string::String,
}

/// One normalized tool call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrajectoryToolCall {
    /// Upstream call identity.
    pub id: alloc::string::String,
    /// Tool name.
    pub name: alloc::string::String,
    /// Serialized JSON object arguments.
    pub arguments: alloc::string::String,
}

/// One normalized trajectory-v1 record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrajectoryRecord {
    /// Leading source metadata.
    Meta {
        /// Source label.
        source: alloc::string::String,
        /// Optional working directory. Default policy omits it.
        cwd: Option<alloc::string::String>,
        /// Optional branch. Default policy omits it.
        git_branch: Option<alloc::string::String>,
        /// Optional model. Default policy omits it.
        model: Option<alloc::string::String>,
    },
    /// User content.
    User {
        /// Content.
        content: alloc::string::String,
        /// Descriptive timestamp.
        timestamp: alloc::string::String,
    },
    /// Hidden reasoning content.
    Reasoning {
        /// Content.
        content: alloc::string::String,
        /// Descriptive timestamp.
        timestamp: alloc::string::String,
    },
    /// Assistant content.
    Assistant {
        /// Content.
        content: alloc::string::String,
        /// Descriptive timestamp.
        timestamp: alloc::string::String,
    },
    /// Assistant tool-call batch.
    AssistantToolCalls {
        /// Calls.
        tool_calls: alloc::vec::Vec<TrajectoryToolCall>,
        /// Descriptive timestamp.
        timestamp: alloc::string::String,
    },
    /// Tool result.
    Tool {
        /// Correlated call identity.
        tool_call_id: alloc::string::String,
        /// Result content.
        content: alloc::string::String,
        /// Descriptive timestamp.
        timestamp: alloc::string::String,
    },
}

/// One bounded upstream diagnostic without prose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrajectoryDiagnostic {
    /// Reviewed code.
    pub code: TrajectoryDiagnosticCode,
    /// Optional bounded occurrence count.
    pub count: Option<u32>,
}

/// Loaded normalizer result passed to the pure validator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrajectoryNormalizeResult {
    /// Normalized records.
    pub records: alloc::vec::Vec<TrajectoryRecord>,
    /// Diagnostics without prose.
    pub diagnostics: alloc::vec::Vec<TrajectoryDiagnostic>,
}

/// Typed service failure that contains no transcript data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrajectoryServiceIssue {
    /// No service was injected.
    Unavailable,
    /// Helper identity does not match policy.
    CohortMismatch,
    /// Request exceeded an admitted bound.
    RequestBound,
    /// Host execution failed or was cancelled.
    Execution,
    /// Child protocol was malformed or exceeded a bound.
    Protocol,
    /// Requested API is outside transcript-string normalization.
    UnsupportedSurface,
}

/// Implementation-free transcript normalizer service.
pub trait TrajectoryNormalizerService {
    /// Normalizes one admitted transient request.
    ///
    /// # Errors
    ///
    /// Returns a bounded issue without raw transcript or helper output.
    fn normalize(
        &mut self,
        request: &TrajectoryNormalizeRequest,
    ) -> Result<TrajectoryNormalizeResult, TrajectoryServiceIssue>;
}

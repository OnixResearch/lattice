#![allow(clippy::struct_field_names)]

use alloc::{string::String, vec::Vec};

use crate::AtifObject;

/// Complete ATIF-v1.7 trajectory document.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct Trajectory {
    /// Exact schema version.
    pub schema_version: String,
    /// Run-scoped identity.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub session_id: Option<String>,
    /// Per-document identity.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub trajectory_id: Option<String>,
    /// Agent metadata.
    pub agent: Agent,
    /// One-based ordered steps.
    pub steps: Vec<Step>,
    /// Untrusted notes.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub notes: Option<String>,
    /// Aggregate metrics.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub final_metrics: Option<FinalMetrics>,
    /// Unresolved continuation descriptor.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub continued_trajectory_ref: Option<String>,
    /// Root extensions.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extra: Option<AtifObject>,
    /// Embedded subagent trajectories.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub subagent_trajectories: Option<Vec<Trajectory>>,
}

/// ATIF agent metadata.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct Agent {
    /// Agent name.
    pub name: String,
    /// Agent version.
    pub version: String,
    /// Optional default model label.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub model_name: Option<String>,
    /// Untrusted tool definition objects.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub tool_definitions: Option<Vec<AtifObject>>,
    /// Agent extensions.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extra: Option<AtifObject>,
}

/// ATIF step source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(rename_all = "lowercase"))]
pub enum StepSource {
    /// System event or prompt.
    System,
    /// User message.
    User,
    /// Agent output or deterministic dispatch.
    Agent,
}

/// Text or multimodal message.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(untagged))]
pub enum Message {
    /// Plain text.
    Text(String),
    /// Ordered content parts.
    Parts(Vec<ContentPart>),
}

/// Qualitative or numeric reasoning effort.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(untagged))]
pub enum ReasoningEffort {
    /// Qualitative label.
    Text(String),
    /// Numeric effort.
    Number(f64),
}

/// One ATIF step.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct Step {
    /// One-based ID.
    pub step_id: u32,
    /// Optional ISO 8601 timestamp.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub timestamp: Option<String>,
    /// Source role.
    pub source: StepSource,
    /// Per-step model label.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub model_name: Option<String>,
    /// Reasoning effort.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Required message.
    pub message: Message,
    /// Explicit reasoning body.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub reasoning_content: Option<String>,
    /// Tool calls.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Observation results.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub observation: Option<Observation>,
    /// Per-step metrics.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub metrics: Option<Metrics>,
    /// Copied-context marker.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub is_copied_context: Option<bool>,
    /// Number of LLM calls represented by this step.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub llm_call_count: Option<u32>,
    /// Step extensions.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extra: Option<AtifObject>,
}

/// Content part kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(rename_all = "lowercase"))]
pub enum ContentKind {
    /// Text content.
    Text,
    /// External image descriptor.
    Image,
}

/// One multimodal content part.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct ContentPart {
    /// Part kind.
    #[cfg_attr(feature = "json-codec", serde(rename = "type"))]
    pub kind: ContentKind,
    /// Text body for text parts.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub text: Option<String>,
    /// Image descriptor for image parts.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub source: Option<ImageSource>,
}

/// Admitted image media type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
pub enum ImageMediaType {
    /// JPEG image.
    #[cfg_attr(feature = "json-codec", serde(rename = "image/jpeg"))]
    Jpeg,
    /// PNG image.
    #[cfg_attr(feature = "json-codec", serde(rename = "image/png"))]
    Png,
    /// GIF image.
    #[cfg_attr(feature = "json-codec", serde(rename = "image/gif"))]
    Gif,
    /// WebP image.
    #[cfg_attr(feature = "json-codec", serde(rename = "image/webp"))]
    Webp,
}

/// Untrusted image reference descriptor.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct ImageSource {
    /// Declared media type.
    pub media_type: ImageMediaType,
    /// Unresolved path or URL descriptor.
    pub path: String,
}

/// One tool call.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct ToolCall {
    /// Correlation identity.
    pub tool_call_id: String,
    /// Function label.
    pub function_name: String,
    /// Untrusted argument object.
    pub arguments: AtifObject,
    /// Tool-call extensions.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extra: Option<AtifObject>,
}

/// Observation container.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct Observation {
    /// Ordered results.
    pub results: Vec<ObservationResult>,
}

/// One observation result.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct ObservationResult {
    /// Same-step tool-call identity.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub source_call_id: Option<String>,
    /// Text or multimodal result.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub content: Option<Message>,
    /// Subagent trajectory descriptors.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub subagent_trajectory_ref: Option<Vec<SubagentTrajectoryRef>>,
    /// Observation extensions.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extra: Option<AtifObject>,
}

/// Embedded or external subagent trajectory descriptor.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct SubagentTrajectoryRef {
    /// Embedded resolution key.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub trajectory_id: Option<String>,
    /// Informational run identity only.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub session_id: Option<String>,
    /// Unresolved external descriptor.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub trajectory_path: Option<String>,
    /// Reference extensions.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extra: Option<AtifObject>,
}

/// Per-step LLM metrics.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct Metrics {
    /// Total prompt tokens.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub prompt_tokens: Option<u64>,
    /// Completion tokens.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub completion_tokens: Option<u64>,
    /// Cached prompt-token subset.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cached_tokens: Option<u64>,
    /// Observed cost.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cost_usd: Option<f64>,
    /// Prompt token IDs.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub prompt_token_ids: Option<Vec<u64>>,
    /// Completion token IDs.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub completion_token_ids: Option<Vec<u64>>,
    /// Completion log probabilities.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub logprobs: Option<Vec<f64>>,
    /// Metric extensions.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extra: Option<AtifObject>,
}

/// Aggregate trajectory metrics.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct FinalMetrics {
    /// Total prompt tokens.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub total_prompt_tokens: Option<u64>,
    /// Total completion tokens.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub total_completion_tokens: Option<u64>,
    /// Total cached tokens.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub total_cached_tokens: Option<u64>,
    /// Total observed cost.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub total_cost_usd: Option<f64>,
    /// Declared total steps.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub total_steps: Option<u64>,
    /// Aggregate metric extensions.
    #[cfg_attr(
        feature = "json-codec",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extra: Option<AtifObject>,
}

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
    clippy::struct_excessive_bools,
    unbounded_collection_growth
)]
/// OpenAI-family API style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderApiStyle {
    /// `OpenAI` Responses API.
    Responses,
    /// `OpenAI` Chat Completions API.
    ChatCompletions,
    /// Ambiguous/unknown style.
    Unknown,
}

/// API style resolution policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderApiStylePolicy {
    /// Require explicit style.
    RequireExplicit,
    /// Default custom gateways to Chat Completions.
    CompatibleGatewayChatDefault,
    /// Default custom gateways to Responses.
    CompatibleGatewayResponsesDefault,
}

/// Shell-resolved secret source reference for a header value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSecretSourceReference {
    /// Redacted source label.
    pub label: String,
    /// Whether the shell found the source.
    pub available: bool,
}

/// Header value source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderHeaderValueSource {
    /// Literal safe value.
    Literal(String),
    /// Shell-resolved secret source label.
    SecretSource(ProviderSecretSourceReference),
}

/// Custom header descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderHeaderDescriptor {
    /// Header name.
    pub name: String,
    /// Value source.
    pub value_source: ProviderHeaderValueSource,
}

/// Header resolution report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderHeaderResolution {
    /// Header name.
    pub name: String,
    /// Redacted source label.
    pub redacted_source: String,
}

/// Extra body descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderExtraBodyDescriptor {
    /// Descriptor ID.
    pub descriptor_id: String,
    /// Provider scope.
    pub provider_id: String,
    /// Optional model scope.
    pub model_id: Option<String>,
    /// Field key.
    pub key: String,
    /// Redacted value class.
    pub value_class: String,
}

/// Normalized model class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderModelClass {
    /// Text agent model.
    TextAgent,
    /// Embedding model.
    Embedding,
    /// Image model.
    Image,
    /// Audio model.
    Audio,
    /// Moderation model.
    Moderation,
    /// Rerank model.
    Rerank,
    /// Other non-agent class.
    OtherNonAgent,
}

/// Normalized model metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderModelMetadata {
    /// Provider ID.
    pub provider_id: String,
    /// Model ID.
    pub model_id: String,
    /// Display name.
    pub display_name: String,
    /// Model class.
    pub model_class: ProviderModelClass,
    /// Context budget label.
    pub context_budget_label: String,
    /// Reasoning support.
    pub reasoning: bool,
}

/// Canonical content-part class supported by a model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderModelContentPartClass {
    /// Text content parts.
    Text,
    /// Descriptor-backed image content parts.
    Image,
}

/// Structured-output mode supported by a model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderStructuredOutputMode {
    /// JSON object mode.
    JsonObject,
    /// JSON schema mode.
    JsonSchema,
}

/// Streaming channel supported by a model adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderStreamChannel {
    /// Assistant-visible content frames.
    Content,
    /// Tool-call delta frames.
    ToolCall,
    /// Usage metadata frames.
    Usage,
}

/// Provider/model privacy posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderPrivacyPosture {
    /// Local-only model execution.
    LocalOnly,
    /// Private or enterprise gateway.
    PrivateGateway,
    /// Remote third-party provider.
    RemoteProvider,
}

/// Typed compatibility matrix for a selectable provider model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderModelCapabilityMatrix {
    /// Capability schema identity.
    pub schema_identity: String,
    /// Provider ID.
    pub provider_id: String,
    /// Model ID.
    pub model_id: String,
    /// Supported canonical message roles.
    pub roles: Vec<animus_adapters::CanonicalModelRole>,
    /// Supported canonical content-part classes.
    pub content_parts: Vec<ProviderModelContentPartClass>,
    /// Whether tool calls are supported.
    pub tool_calls: bool,
    /// Structured-output modes supported by this model.
    pub structured_outputs: Vec<ProviderStructuredOutputMode>,
    /// Stream channels supported by this model.
    pub stream_channels: Vec<ProviderStreamChannel>,
    /// Reasoning controls supported by this model.
    pub reasoning_controls: Vec<animus_adapters::CanonicalModelReasoningEffort>,
    /// Context budget in model-visible units.
    pub context_budget: u32,
    /// Output token budget.
    pub output_budget: u32,
    /// Privacy posture for compatibility decisions.
    pub privacy: ProviderPrivacyPosture,
}

/// Host policy for optional compatibility downgrades.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelCapabilityDowngradePolicy {
    /// Permit omitting optional image content.
    pub allow_image_omission: bool,
    /// Permit omitting optional structured-output intent.
    pub allow_structured_output_omission: bool,
    /// Permit omitting optional streaming.
    pub allow_streaming_omission: bool,
    /// Permit omitting optional reasoning controls.
    pub allow_reasoning_omission: bool,
}

/// Borrowed capability negotiation input.
#[derive(Clone, Copy)]
pub struct ModelCapabilityNegotiationInput<'a> {
    /// Canonical request requirements.
    pub request: &'a animus_adapters::CanonicalModelRequest,
    /// Selected model capability matrix.
    pub capabilities: &'a ProviderModelCapabilityMatrix,
    /// Host downgrade policy.
    pub downgrade_policy: ModelCapabilityDowngradePolicy,
    /// Required privacy posture.
    pub required_privacy: ProviderPrivacyPosture,
    /// Whether another provider fallback was requested.
    pub provider_fallback_requested: bool,
}

/// Capability negotiation decision status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelCapabilityDecisionStatus {
    /// All required features are admitted.
    Admitted,
    /// Required features are admitted and optional features were explicitly downgraded.
    Downgraded,
    /// Dispatch is denied before provider adapter execution.
    Denied,
}

/// Capability negotiation issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelCapabilityIssueKind {
    /// Capability metadata was missing, malformed, or stale.
    UnknownCapabilityMetadata,
    /// A required canonical role is unsupported.
    UnsupportedRole,
    /// A required content part is unsupported.
    UnsupportedContentPart,
    /// Tool calls are required but unsupported.
    UnsupportedToolCalls,
    /// Structured output is required but unsupported.
    UnsupportedStructuredOutput,
    /// Streaming is required but unsupported.
    UnsupportedStreaming,
    /// Reasoning controls are required but unsupported.
    UnsupportedReasoning,
    /// Context or output budget is exceeded.
    BudgetOverflow,
    /// Required privacy posture is not satisfied.
    UnsupportedPrivacy,
    /// Provider fallback was requested and denied.
    ProviderFallbackDenied,
}

/// Capability negotiation issue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelCapabilityIssue {
    /// Issue class.
    pub kind: ModelCapabilityIssueKind,
    /// Safe feature label.
    pub feature: String,
    /// Safe detail label.
    pub detail: String,
}

/// Optional feature downgrade evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelCapabilityDowngrade {
    /// Safe feature label.
    pub feature: String,
    /// Safe policy label.
    pub policy: String,
}

/// Capability negotiation decision with safe evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelCapabilityNegotiationDecision {
    /// Decision status.
    pub status: ModelCapabilityDecisionStatus,
    /// Provider ID.
    pub provider_id: String,
    /// Model ID.
    pub model_id: String,
    /// Safe admission/downgrade evidence labels.
    pub evidence: Vec<String>,
    /// Optional downgrade evidence.
    pub downgrades: Vec<ModelCapabilityDowngrade>,
    /// Denial issues.
    pub issues: Vec<ModelCapabilityIssue>,
}

/// Endpoint sensitivity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderEndpointSensitivity {
    /// Safe label can be shown.
    Public,
    /// Endpoint must be hashed/redacted.
    Sensitive,
}

/// Provider selection receipt input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSelectionReceiptInput {
    /// Provider ID.
    pub provider_id: String,
    /// Model metadata.
    pub model: ProviderModelMetadata,
    /// Endpoint label or URL supplied by the shell.
    pub endpoint: String,
    /// Endpoint sensitivity.
    pub sensitivity: ProviderEndpointSensitivity,
    /// API style.
    pub api_style: ProviderApiStyle,
    /// Extra body descriptor IDs.
    pub extra_body_descriptor_ids: Vec<String>,
    /// Diagnostics.
    pub diagnostics: Vec<ProviderDiagnostic>,
}

/// Provider selection receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSelectionReceipt {
    /// Provider ID.
    pub provider_id: String,
    /// Model ID.
    pub model_id: String,
    /// Endpoint identity or hash.
    pub endpoint_identity: String,
    /// API style.
    pub api_style: ProviderApiStyle,
    /// Capability class.
    pub capability_class: ProviderModelClass,
    /// Credential source class.
    pub credential_source_class: String,
    /// Extra body descriptor IDs.
    pub extra_body_descriptor_ids: Vec<String>,
    /// Diagnostics.
    pub diagnostics: Vec<ProviderDiagnostic>,
}

/// Provider diagnostic class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderDiagnosticKind {
    /// Ambiguous API style.
    AmbiguousApiStyle,
    /// Invalid header.
    InvalidHeader,
    /// Missing header secret.
    MissingHeaderSecret,
    /// Extra body out of scope.
    ExtraBodyOutOfScope,
    /// Unsupported model class.
    UnsupportedModelClass,
    /// Invalid capability metadata.
    InvalidCapabilityMetadata,
}

/// Provider diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderDiagnostic {
    /// Kind.
    pub kind: ProviderDiagnosticKind,
    /// Field.
    pub field: String,
    /// Detail.
    pub detail: String,
}

impl ProviderDiagnostic {
    /// Builds a diagnostic.
    #[must_use]
    pub fn new(kind: ProviderDiagnosticKind, field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Provider validation result.
pub type ProviderValidationResult<T> = Result<T, Vec<ProviderDiagnostic>>;

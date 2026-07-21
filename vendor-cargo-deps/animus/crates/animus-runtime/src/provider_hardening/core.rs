// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    clippy::too_many_arguments,
    unbounded_collection_growth
)]
use super::{
    BLAKE3_ALGORITHM, FIELD_API_STYLE, FIELD_EXTRA_BODY, FIELD_HEADER, FIELD_MODEL,
    HASH_PART_SEPARATOR, PROVIDER_HARDENING_HASH_DOMAIN,
    data::{
        ModelCapabilityDecisionStatus, ModelCapabilityDowngrade, ModelCapabilityDowngradePolicy,
        ModelCapabilityIssue, ModelCapabilityIssueKind, ModelCapabilityNegotiationDecision,
        ModelCapabilityNegotiationInput, ProviderApiStyle, ProviderApiStylePolicy,
        ProviderDiagnostic, ProviderDiagnosticKind, ProviderEndpointSensitivity,
        ProviderExtraBodyDescriptor, ProviderHeaderDescriptor, ProviderHeaderResolution,
        ProviderHeaderValueSource, ProviderModelCapabilityMatrix, ProviderModelClass,
        ProviderModelContentPartClass, ProviderModelMetadata, ProviderPrivacyPosture,
        ProviderSelectionReceipt, ProviderSelectionReceiptInput, ProviderStreamChannel,
        ProviderStructuredOutputMode, ProviderValidationResult,
    },
};

const HEADER_PREFIX: &str = "x-";
const REDACTED_ENV_PREFIX: &str = "env:";
const MODEL_CAPABILITY_MATRIX_SCHEMA_IDENTITY: &str = "animus.model-capability-matrix.v1";
const FEATURE_CAPABILITY_SCHEMA: &str = "capability_schema";
const FEATURE_PROVIDER_ID: &str = "provider_id";
const FEATURE_MODEL_ID: &str = "model_id";
const FEATURE_ROLES: &str = "roles";
const FEATURE_CONTENT_PARTS: &str = "content_parts";
const FEATURE_CONTEXT_BUDGET: &str = "context_budget";
const FEATURE_OUTPUT_BUDGET: &str = "output_budget";
const FEATURE_TOOL_CALLS: &str = "tool_calls";
const FEATURE_STRUCTURED_OUTPUT: &str = "structured_output";
const FEATURE_STREAMING: &str = "streaming";
const FEATURE_REASONING: &str = "reasoning";
const FEATURE_PRIVACY: &str = "privacy";
const FEATURE_PROVIDER_FALLBACK: &str = "provider_fallback";
const DOWNGRADE_POLICY_LABEL: &str = "host_optional_downgrade";
const DETAIL_MISSING_OR_STALE: &str = "missing_or_stale_capability_metadata";
const DETAIL_UNSUPPORTED: &str = "unsupported_required_feature";
const DETAIL_BUDGET_OVERFLOW: &str = "budget_overflow";
const DETAIL_PRIVACY: &str = "privacy_posture_not_satisfied";
const DETAIL_FALLBACK_DENIED: &str = "provider_fallback_denied";
const ONE_IMAGE_CONTEXT_UNIT: u32 = 1;

/// Validates a selected model capability matrix.
///
/// # Errors
///
/// Returns diagnostics for missing, stale, contradictory, or malformed metadata.
pub fn validate_model_capability_matrix(
    capabilities: &ProviderModelCapabilityMatrix,
) -> ProviderValidationResult<ProviderModelCapabilityMatrix> {
    let mut diagnostics = Vec::new();
    if capabilities.schema_identity != MODEL_CAPABILITY_MATRIX_SCHEMA_IDENTITY {
        diagnostics.push(ProviderDiagnostic::new(
            ProviderDiagnosticKind::InvalidCapabilityMetadata,
            FEATURE_CAPABILITY_SCHEMA,
            DETAIL_MISSING_OR_STALE,
        ));
    }
    push_if_blank(
        &mut diagnostics,
        FEATURE_PROVIDER_ID,
        &capabilities.provider_id,
    );
    push_if_blank(&mut diagnostics, FEATURE_MODEL_ID, &capabilities.model_id);
    push_if_empty(&mut diagnostics, FEATURE_ROLES, capabilities.roles.len());
    push_if_empty(
        &mut diagnostics,
        FEATURE_CONTENT_PARTS,
        capabilities.content_parts.len(),
    );
    push_if_zero(
        &mut diagnostics,
        FEATURE_CONTEXT_BUDGET,
        capabilities.context_budget,
    );
    push_if_zero(
        &mut diagnostics,
        FEATURE_OUTPUT_BUDGET,
        capabilities.output_budget,
    );
    if diagnostics.is_empty() {
        Ok(capabilities.clone())
    } else {
        Err(diagnostics)
    }
}

/// Negotiates canonical request requirements against a selected model capability matrix.
#[must_use]
pub fn negotiate_model_capability(
    input: ModelCapabilityNegotiationInput<'_>,
) -> ModelCapabilityNegotiationDecision {
    let mut evidence = vec![
        format!("schema:{}", input.capabilities.schema_identity),
        format!("provider:{}", input.capabilities.provider_id),
        format!("model:{}", input.capabilities.model_id),
    ];
    let mut downgrades = Vec::new();
    let mut issues = Vec::new();
    if input.provider_fallback_requested {
        issues.push(issue(
            ModelCapabilityIssueKind::ProviderFallbackDenied,
            FEATURE_PROVIDER_FALLBACK,
            DETAIL_FALLBACK_DENIED,
        ));
    }
    if let Err(diagnostics) = validate_model_capability_matrix(input.capabilities) {
        issues.extend(
            diagnostics
                .into_iter()
                .map(|diagnostic| ModelCapabilityIssue {
                    kind: ModelCapabilityIssueKind::UnknownCapabilityMetadata,
                    feature: diagnostic.field,
                    detail: diagnostic.detail,
                }),
        );
    }
    check_roles(input.request, input.capabilities, &mut issues);
    check_content_parts(
        input.request,
        input.capabilities,
        input.downgrade_policy,
        &mut downgrades,
        &mut issues,
    );
    check_tools(input.request, input.capabilities, &mut issues);
    check_structured_output(
        input.request,
        input.capabilities,
        input.downgrade_policy,
        &mut downgrades,
        &mut issues,
    );
    check_streaming(
        input.request,
        input.capabilities,
        input.downgrade_policy,
        &mut downgrades,
        &mut issues,
    );
    check_reasoning(
        input.request,
        input.capabilities,
        input.downgrade_policy,
        &mut downgrades,
        &mut issues,
    );
    check_budgets(
        input.request,
        input.capabilities,
        &mut evidence,
        &mut issues,
    );
    check_privacy(
        input.required_privacy,
        input.capabilities.privacy,
        &mut evidence,
        &mut issues,
    );
    let status = if issues.is_empty() {
        if downgrades.is_empty() {
            ModelCapabilityDecisionStatus::Admitted
        } else {
            ModelCapabilityDecisionStatus::Downgraded
        }
    } else {
        ModelCapabilityDecisionStatus::Denied
    };
    ModelCapabilityNegotiationDecision {
        status,
        provider_id: input.capabilities.provider_id.clone(),
        model_id: input.capabilities.model_id.clone(),
        evidence,
        downgrades,
        issues,
    }
}

/// Computes a provider-hardening BLAKE3 fingerprint.
#[must_use]
pub fn blake3_provider_hardening_fingerprint(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PROVIDER_HARDENING_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    format!("{BLAKE3_ALGORITHM}:{}", hasher.finalize().to_hex())
}

/// Resolves OpenAI-family API style before client construction.
///
/// # Errors
///
/// Returns an ambiguous-style diagnostic when policy requires explicit style and
/// no style is supplied.
pub fn resolve_provider_api_style(
    explicit: Option<ProviderApiStyle>,
    custom_gateway: bool,
    policy: ProviderApiStylePolicy,
) -> ProviderValidationResult<ProviderApiStyle> {
    if let Some(style) = explicit.filter(|style| *style != ProviderApiStyle::Unknown) {
        return Ok(style);
    }
    match (custom_gateway, policy) {
        (true, ProviderApiStylePolicy::CompatibleGatewayChatDefault) => {
            Ok(ProviderApiStyle::ChatCompletions)
        }
        (true, ProviderApiStylePolicy::CompatibleGatewayResponsesDefault) => {
            Ok(ProviderApiStyle::Responses)
        }
        _ => Err(vec![ProviderDiagnostic::new(
            ProviderDiagnosticKind::AmbiguousApiStyle,
            FIELD_API_STYLE,
            "provider API style is ambiguous before client construction",
        )]),
    }
}

/// Validates a custom header descriptor with redacted environment diagnostics.
///
/// # Errors
///
/// Returns diagnostics for invalid header names or unavailable environment-backed
/// sources without exposing resolved values.
pub fn validate_provider_header(
    header: &ProviderHeaderDescriptor,
) -> ProviderValidationResult<ProviderHeaderResolution> {
    if !valid_header_name(&header.name) {
        return Err(vec![ProviderDiagnostic::new(
            ProviderDiagnosticKind::InvalidHeader,
            FIELD_HEADER,
            "provider header name is invalid",
        )]);
    }
    match &header.value_source {
        ProviderHeaderValueSource::Literal(value) if value.trim().is_empty() => {
            Err(vec![ProviderDiagnostic::new(
                ProviderDiagnosticKind::InvalidHeader,
                FIELD_HEADER,
                "provider header literal is empty",
            )])
        }
        ProviderHeaderValueSource::Literal(_) => Ok(ProviderHeaderResolution {
            name: header.name.clone(),
            redacted_source: "literal:redacted".to_string(),
        }),
        ProviderHeaderValueSource::SecretSource(reference) => {
            if reference.available {
                Ok(ProviderHeaderResolution {
                    name: header.name.clone(),
                    redacted_source: format!("{REDACTED_ENV_PREFIX}{}", reference.label),
                })
            } else {
                Err(vec![ProviderDiagnostic::new(
                    ProviderDiagnosticKind::MissingHeaderSecret,
                    FIELD_HEADER,
                    format!(
                        "missing provider header source {REDACTED_ENV_PREFIX}{}",
                        reference.label
                    ),
                )])
            }
        }
    }
}

/// Resolves provider/model-scoped extra body descriptors.
///
/// # Errors
///
/// Returns an out-of-scope diagnostic when selected provider/model does not match.
pub fn resolve_extra_body_descriptor(
    descriptor: &ProviderExtraBodyDescriptor,
    selected_provider: &str,
    selected_model: &str,
) -> ProviderValidationResult<ProviderExtraBodyDescriptor> {
    let provider_matches = descriptor.provider_id == selected_provider;
    let model_matches = descriptor
        .model_id
        .as_ref()
        .is_none_or(|model| model == selected_model);
    if provider_matches && model_matches {
        Ok(descriptor.clone())
    } else {
        Err(vec![ProviderDiagnostic::new(
            ProviderDiagnosticKind::ExtraBodyOutOfScope,
            FIELD_EXTRA_BODY,
            "provider extra body descriptor is outside selected provider/model scope",
        )])
    }
}

/// Filters non-agent model classes unless host override is explicit.
///
/// # Errors
///
/// Returns unsupported-model-class diagnostics for embedding, image, audio,
/// moderation, rerank, or other non-agent classes.
pub fn filter_agent_model(
    model: &ProviderModelMetadata,
    allow_non_agent_override: bool,
) -> ProviderValidationResult<ProviderModelMetadata> {
    if model.model_class == ProviderModelClass::TextAgent || allow_non_agent_override {
        Ok(model.clone())
    } else {
        Err(vec![ProviderDiagnostic::new(
            ProviderDiagnosticKind::UnsupportedModelClass,
            FIELD_MODEL,
            "provider model class is not suitable for text agent turns",
        )])
    }
}

/// Builds a safe provider selection receipt.
#[must_use]
pub fn provider_selection_receipt(
    input: &ProviderSelectionReceiptInput,
) -> ProviderSelectionReceipt {
    let endpoint_identity = match input.sensitivity {
        ProviderEndpointSensitivity::Public => input.endpoint.clone(),
        ProviderEndpointSensitivity::Sensitive => {
            blake3_provider_hardening_fingerprint("endpoint", input.endpoint.as_bytes())
        }
    };
    ProviderSelectionReceipt {
        provider_id: input.provider_id.clone(),
        model_id: input.model.model_id.clone(),
        endpoint_identity,
        api_style: input.api_style,
        capability_class: input.model.model_class,
        credential_source_class: "redacted".to_string(),
        extra_body_descriptor_ids: input.extra_body_descriptor_ids.clone(),
        diagnostics: input.diagnostics.clone(),
    }
}

fn valid_header_name(name: &str) -> bool {
    !name.trim().is_empty()
        && name.starts_with(HEADER_PREFIX)
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn push_if_blank(diagnostics: &mut Vec<ProviderDiagnostic>, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(ProviderDiagnostic::new(
            ProviderDiagnosticKind::InvalidCapabilityMetadata,
            field,
            DETAIL_MISSING_OR_STALE,
        ));
    }
}

fn push_if_empty(diagnostics: &mut Vec<ProviderDiagnostic>, field: &str, count: usize) {
    if count == 0 {
        diagnostics.push(ProviderDiagnostic::new(
            ProviderDiagnosticKind::InvalidCapabilityMetadata,
            field,
            DETAIL_MISSING_OR_STALE,
        ));
    }
}

fn push_if_zero(diagnostics: &mut Vec<ProviderDiagnostic>, field: &str, value: u32) {
    if value == 0 {
        diagnostics.push(ProviderDiagnostic::new(
            ProviderDiagnosticKind::InvalidCapabilityMetadata,
            field,
            DETAIL_MISSING_OR_STALE,
        ));
    }
}

fn check_roles(
    request: &animus_adapters::CanonicalModelRequest,
    capabilities: &ProviderModelCapabilityMatrix,
    issues: &mut Vec<ModelCapabilityIssue>,
) {
    for message in request.messages() {
        if !capabilities.roles.contains(&message.role()) {
            issues.push(issue(
                ModelCapabilityIssueKind::UnsupportedRole,
                role_feature(message.role()),
                DETAIL_UNSUPPORTED,
            ));
        }
    }
}

fn check_content_parts(
    request: &animus_adapters::CanonicalModelRequest,
    capabilities: &ProviderModelCapabilityMatrix,
    policy: ModelCapabilityDowngradePolicy,
    downgrades: &mut Vec<ModelCapabilityDowngrade>,
    issues: &mut Vec<ModelCapabilityIssue>,
) {
    for message in request.messages() {
        for part in message.content() {
            let needed = if part.is_text() {
                ProviderModelContentPartClass::Text
            } else {
                ProviderModelContentPartClass::Image
            };
            if capabilities.content_parts.contains(&needed) {
                continue;
            }
            if needed == ProviderModelContentPartClass::Image && policy.allow_image_omission {
                downgrades.push(downgrade(FEATURE_CONTENT_PARTS));
            } else {
                issues.push(issue(
                    ModelCapabilityIssueKind::UnsupportedContentPart,
                    content_part_feature(needed),
                    DETAIL_UNSUPPORTED,
                ));
            }
        }
    }
}

fn check_tools(
    request: &animus_adapters::CanonicalModelRequest,
    capabilities: &ProviderModelCapabilityMatrix,
    issues: &mut Vec<ModelCapabilityIssue>,
) {
    if !request.tools().is_empty() && !capabilities.tool_calls {
        issues.push(issue(
            ModelCapabilityIssueKind::UnsupportedToolCalls,
            FEATURE_TOOL_CALLS,
            DETAIL_UNSUPPORTED,
        ));
    }
}

fn check_structured_output(
    request: &animus_adapters::CanonicalModelRequest,
    capabilities: &ProviderModelCapabilityMatrix,
    policy: ModelCapabilityDowngradePolicy,
    downgrades: &mut Vec<ModelCapabilityDowngrade>,
    issues: &mut Vec<ModelCapabilityIssue>,
) {
    let required = match request.response_format() {
        animus_adapters::CanonicalModelResponseFormatIntent::Text => return,
        animus_adapters::CanonicalModelResponseFormatIntent::JsonObject => {
            ProviderStructuredOutputMode::JsonObject
        }
        animus_adapters::CanonicalModelResponseFormatIntent::JsonSchema { .. } => {
            ProviderStructuredOutputMode::JsonSchema
        }
    };
    if capabilities.structured_outputs.contains(&required) {
        return;
    }
    if policy.allow_structured_output_omission {
        downgrades.push(downgrade(FEATURE_STRUCTURED_OUTPUT));
    } else {
        issues.push(issue(
            ModelCapabilityIssueKind::UnsupportedStructuredOutput,
            structured_feature(required),
            DETAIL_UNSUPPORTED,
        ));
    }
}

fn check_streaming(
    request: &animus_adapters::CanonicalModelRequest,
    capabilities: &ProviderModelCapabilityMatrix,
    policy: ModelCapabilityDowngradePolicy,
    downgrades: &mut Vec<ModelCapabilityDowngrade>,
    issues: &mut Vec<ModelCapabilityIssue>,
) {
    if !request.output().stream()
        || capabilities
            .stream_channels
            .contains(&ProviderStreamChannel::Content)
    {
        return;
    }
    if policy.allow_streaming_omission {
        downgrades.push(downgrade(FEATURE_STREAMING));
    } else {
        issues.push(issue(
            ModelCapabilityIssueKind::UnsupportedStreaming,
            FEATURE_STREAMING,
            DETAIL_UNSUPPORTED,
        ));
    }
}

fn check_reasoning(
    request: &animus_adapters::CanonicalModelRequest,
    capabilities: &ProviderModelCapabilityMatrix,
    policy: ModelCapabilityDowngradePolicy,
    downgrades: &mut Vec<ModelCapabilityDowngrade>,
    issues: &mut Vec<ModelCapabilityIssue>,
) {
    if !request.reasoning().enabled()
        || capabilities
            .reasoning_controls
            .contains(&request.reasoning().effort())
    {
        return;
    }
    if policy.allow_reasoning_omission {
        downgrades.push(downgrade(FEATURE_REASONING));
    } else {
        issues.push(issue(
            ModelCapabilityIssueKind::UnsupportedReasoning,
            FEATURE_REASONING,
            DETAIL_UNSUPPORTED,
        ));
    }
}

fn check_budgets(
    request: &animus_adapters::CanonicalModelRequest,
    capabilities: &ProviderModelCapabilityMatrix,
    evidence: &mut Vec<String>,
    issues: &mut Vec<ModelCapabilityIssue>,
) {
    let context_units = request_context_units(request);
    evidence.push(format!("context_units:{context_units}"));
    evidence.push(format!("output_budget:{}", capabilities.output_budget));
    if context_units > capabilities.context_budget {
        issues.push(issue(
            ModelCapabilityIssueKind::BudgetOverflow,
            FEATURE_CONTEXT_BUDGET,
            DETAIL_BUDGET_OVERFLOW,
        ));
    }
    if let Some(output_tokens) = request.output().max_output_tokens()
        && output_tokens > capabilities.output_budget
    {
        issues.push(issue(
            ModelCapabilityIssueKind::BudgetOverflow,
            FEATURE_OUTPUT_BUDGET,
            DETAIL_BUDGET_OVERFLOW,
        ));
    }
}

fn check_privacy(
    required: ProviderPrivacyPosture,
    actual: ProviderPrivacyPosture,
    evidence: &mut Vec<String>,
    issues: &mut Vec<ModelCapabilityIssue>,
) {
    evidence.push(format!("privacy:{actual:?}"));
    if privacy_satisfies(required, actual) {
        return;
    }
    issues.push(issue(
        ModelCapabilityIssueKind::UnsupportedPrivacy,
        FEATURE_PRIVACY,
        DETAIL_PRIVACY,
    ));
}

fn request_context_units(request: &animus_adapters::CanonicalModelRequest) -> u32 {
    let mut units = 0u32;
    for message in request.messages() {
        for part in message.content() {
            if let Some(text) = part.text() {
                units = units.saturating_add(usize_to_u32(text.chars().count()));
            } else {
                units = units.saturating_add(ONE_IMAGE_CONTEXT_UNIT);
            }
        }
    }
    units
}

#[allow(fallible_int_fallback, sentinel_fallback)]
fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

const fn privacy_satisfies(
    required: ProviderPrivacyPosture,
    actual: ProviderPrivacyPosture,
) -> bool {
    match required {
        ProviderPrivacyPosture::RemoteProvider => true,
        ProviderPrivacyPosture::PrivateGateway => matches!(
            actual,
            ProviderPrivacyPosture::LocalOnly | ProviderPrivacyPosture::PrivateGateway
        ),
        ProviderPrivacyPosture::LocalOnly => matches!(actual, ProviderPrivacyPosture::LocalOnly),
    }
}

fn role_feature(role: animus_adapters::CanonicalModelRole) -> String {
    format!("role:{}", role.as_str())
}

fn content_part_feature(class: ProviderModelContentPartClass) -> &'static str {
    match class {
        ProviderModelContentPartClass::Text => "content:text",
        ProviderModelContentPartClass::Image => "content:image",
    }
}

fn structured_feature(mode: ProviderStructuredOutputMode) -> &'static str {
    match mode {
        ProviderStructuredOutputMode::JsonObject => "structured:json_object",
        ProviderStructuredOutputMode::JsonSchema => "structured:json_schema",
    }
}

fn issue(
    kind: ModelCapabilityIssueKind,
    feature: impl Into<String>,
    detail: impl Into<String>,
) -> ModelCapabilityIssue {
    ModelCapabilityIssue {
        kind,
        feature: feature.into(),
        detail: detail.into(),
    }
}

fn downgrade(feature: &str) -> ModelCapabilityDowngrade {
    ModelCapabilityDowngrade {
        feature: feature.to_string(),
        policy: DOWNGRADE_POLICY_LABEL.to_string(),
    }
}

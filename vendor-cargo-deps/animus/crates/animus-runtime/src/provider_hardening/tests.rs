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

use super::*;

const ENDPOINT: &str = "https://gateway.internal/v1";
const MODEL_ID: &str = "agent-model";
const PROVIDER_ID: &str = "openai-compatible";
const CAPABILITY_SCHEMA: &str = "animus.model-capability-matrix.v1";
const USER_PROMPT: &str = "Plan safely.";
const SYSTEM_PROMPT: &str = "Stay bounded.";
const IMAGE_DESCRIPTOR_ID: &str = "image-one";
const IMAGE_MEDIA_TYPE: &str = "image/png";
const IMAGE_HASH: &str = "b3-image";
const TOOL_NAME: &str = "lookup";
const TOOL_DESCRIPTION: &str = "Bounded lookup.";
const TOOL_SCHEMA: &str = "schema.lookup";
const RESPONSE_SCHEMA: &str = "schema.answer";
const CONTEXT_BUDGET: u32 = 256;
const OUTPUT_BUDGET: u32 = 64;
const SMALL_CONTEXT_BUDGET: u32 = 1;
const SMALL_OUTPUT_BUDGET: u32 = 1;
const STREAM_CALL_ID: &str = "call-1";
const STREAM_TOOL_ID: &str = "tool-1";
const STREAM_PRIVATE_REASONING: &str = "private reasoning";
const EXPECTED_PARSER_INVENTORY_ENTRIES: usize = 1;
const EXPECTED_POSITIVE_STREAM_FRAMES: usize = 6;
const EXPECTED_POSITIVE_CONTENT_FRAMES: usize = 2;
const EXPECTED_TERMINAL_COUNT: usize = 1;
const TINY_FRAME_BUDGET: usize = 8;
const TINY_RECORD_BUDGET: usize = 16;
const TINY_AGGREGATE_BUDGET: usize = 32;
const ONE_RECORD_BUDGET: usize = 1;
const ONE_EVENT_BUDGET: usize = 1;
const ZERO_COMPLETION_BUDGET: usize = 0;
const INVALID_UTF8_CHUNK: &[u8] = &[u8::MAX];
const ORDERED_CONTENT_CHUNK: &[u8] = b"{\"event\":\"content\",\"call_id\":\"call-1\",\"order\":1,\"body\":\"a\"}\n";
const ORDERED_FINISH_CHUNK: &[u8] = b"{\"event\":\"finish\",\"call_id\":\"call-1\",\"order\":2}\n";

#[test]
fn api_style_resolution_handles_first_party_gateway_and_ambiguous_cases() {
    // r[agent_harness_runtime.zerostack_provider_hardening.api_style.gateway]
    assert_eq!(
        resolve_provider_api_style(Some(ProviderApiStyle::Responses), false, ProviderApiStylePolicy::RequireExplicit)
            .expect("explicit"),
        ProviderApiStyle::Responses
    );
    assert_eq!(
        resolve_provider_api_style(None, true, ProviderApiStylePolicy::CompatibleGatewayChatDefault)
            .expect("gateway default"),
        ProviderApiStyle::ChatCompletions
    );
    let diagnostics =
        resolve_provider_api_style(None, true, ProviderApiStylePolicy::RequireExplicit).expect_err("ambiguous");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ProviderDiagnosticKind::AmbiguousApiStyle })
    );
}

#[test]
fn header_resolution_redacts_missing_environment_sources() {
    // r[agent_harness_runtime.zerostack_provider_hardening.headers.missing]
    let header = ProviderHeaderDescriptor {
        name: "x-api-key".to_string(),
        value_source: ProviderHeaderValueSource::SecretSource(ProviderSecretSourceReference {
            label: "GATEWAY_KEY".to_string(),
            available: false,
        }),
    };
    let diagnostics = validate_provider_header(&header).expect_err("missing env");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ProviderDiagnosticKind::MissingHeaderSecret && !diagnostic.detail.contains("secret-value")
    }));

    let invalid = ProviderHeaderDescriptor {
        name: "Authorization".to_string(),
        value_source: ProviderHeaderValueSource::Literal("redacted".to_string()),
    };
    assert!(validate_provider_header(&invalid).is_err());
}

#[test]
fn extra_body_descriptor_cannot_cross_provider_or_model_scope() {
    // r[agent_harness_runtime.zerostack_provider_hardening.extra_body.scope]
    let descriptor = ProviderExtraBodyDescriptor {
        descriptor_id: "extra.reasoning".to_string(),
        provider_id: PROVIDER_ID.to_string(),
        model_id: Some(MODEL_ID.to_string()),
        key: "reasoning_effort".to_string(),
        value_class: "low".to_string(),
    };
    assert!(resolve_extra_body_descriptor(&descriptor, PROVIDER_ID, MODEL_ID).is_ok());
    assert!(resolve_extra_body_descriptor(&descriptor, "other", MODEL_ID).is_err());
    assert!(resolve_extra_body_descriptor(&descriptor, PROVIDER_ID, "other-model").is_err());
}

#[test]
fn model_filter_rejects_non_agent_model_classes() {
    // r[agent_harness_runtime.zerostack_provider_hardening.model_filter.reject]
    assert!(filter_agent_model(&model(ProviderModelClass::TextAgent), false).is_ok());
    for class in [
        ProviderModelClass::Embedding,
        ProviderModelClass::Image,
        ProviderModelClass::Audio,
        ProviderModelClass::Moderation,
        ProviderModelClass::Rerank,
        ProviderModelClass::OtherNonAgent,
    ] {
        let diagnostics = filter_agent_model(&model(class), false).expect_err("non agent");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.kind == ProviderDiagnosticKind::UnsupportedModelClass })
        );
    }
}

#[test]
fn capability_negotiation_admits_supported_requests_and_optional_downgrades() {
    // r[agent_harness_runtime.model_capability_matrix.negotiation.positive]
    let supported_request = structured_request(true, true, true);
    let supported = full_capabilities();
    let admitted = negotiate_model_capability(ModelCapabilityNegotiationInput {
        request: &supported_request,
        capabilities: &supported,
        downgrade_policy: deny_downgrades(),
        required_privacy: ProviderPrivacyPosture::PrivateGateway,
        provider_fallback_requested: false,
    });
    assert_eq!(admitted.status, ModelCapabilityDecisionStatus::Admitted);
    assert!(admitted.issues.is_empty());

    let optional_request = structured_request(false, true, true);
    let text_only = ProviderModelCapabilityMatrix {
        content_parts: vec![ProviderModelContentPartClass::Text],
        structured_outputs: Vec::new(),
        stream_channels: Vec::new(),
        reasoning_controls: Vec::new(),
        ..full_capabilities()
    };
    let downgraded = negotiate_model_capability(ModelCapabilityNegotiationInput {
        request: &optional_request,
        capabilities: &text_only,
        downgrade_policy: allow_optional_downgrades(),
        required_privacy: ProviderPrivacyPosture::RemoteProvider,
        provider_fallback_requested: false,
    });
    assert_eq!(downgraded.status, ModelCapabilityDecisionStatus::Downgraded);
    assert!(downgraded.issues.is_empty());
    assert!(
        downgraded
            .downgrades
            .iter()
            .any(|item| { item.feature == "content_parts" && item.policy == "host_optional_downgrade" })
    );
}

#[test]
fn capability_negotiation_denies_unsupported_required_features() {
    // r[agent_harness_runtime.model_capability_matrix.fail_closed]
    let request = structured_request(true, true, true);
    let role_limited = ProviderModelCapabilityMatrix {
        roles: vec![animus_adapters::CanonicalModelRole::User],
        ..full_capabilities()
    };
    assert_issue(&request, &role_limited, ModelCapabilityIssueKind::UnsupportedRole);

    let text_only = ProviderModelCapabilityMatrix {
        content_parts: vec![ProviderModelContentPartClass::Text],
        tool_calls: false,
        structured_outputs: Vec::new(),
        stream_channels: Vec::new(),
        reasoning_controls: Vec::new(),
        ..full_capabilities()
    };
    let denied = negotiate_model_capability(ModelCapabilityNegotiationInput {
        request: &request,
        capabilities: &text_only,
        downgrade_policy: deny_downgrades(),
        required_privacy: ProviderPrivacyPosture::RemoteProvider,
        provider_fallback_requested: false,
    });
    assert_eq!(denied.status, ModelCapabilityDecisionStatus::Denied);
    for kind in [
        ModelCapabilityIssueKind::UnsupportedContentPart,
        ModelCapabilityIssueKind::UnsupportedToolCalls,
        ModelCapabilityIssueKind::UnsupportedStructuredOutput,
        ModelCapabilityIssueKind::UnsupportedStreaming,
        ModelCapabilityIssueKind::UnsupportedReasoning,
    ] {
        assert!(denied.issues.iter().any(|issue| issue.kind == kind));
    }
}

#[test]
fn capability_negotiation_denies_bad_metadata_budgets_privacy_and_fallback() {
    // r[agent_harness_runtime.model_capability_matrix.fail_closed]
    let request = structured_request(false, false, false);
    let bad_metadata = ProviderModelCapabilityMatrix {
        schema_identity: "stale".to_string(),
        context_budget: SMALL_CONTEXT_BUDGET,
        output_budget: SMALL_OUTPUT_BUDGET,
        privacy: ProviderPrivacyPosture::RemoteProvider,
        ..full_capabilities()
    };
    let denied = negotiate_model_capability(ModelCapabilityNegotiationInput {
        request: &request,
        capabilities: &bad_metadata,
        downgrade_policy: deny_downgrades(),
        required_privacy: ProviderPrivacyPosture::LocalOnly,
        provider_fallback_requested: true,
    });
    assert_eq!(denied.status, ModelCapabilityDecisionStatus::Denied);
    for kind in [
        ModelCapabilityIssueKind::UnknownCapabilityMetadata,
        ModelCapabilityIssueKind::BudgetOverflow,
        ModelCapabilityIssueKind::UnsupportedPrivacy,
        ModelCapabilityIssueKind::ProviderFallbackDenied,
    ] {
        assert!(denied.issues.iter().any(|issue| issue.kind == kind));
    }
    assert!(validate_model_capability_matrix(&bad_metadata).is_err());
}

#[test]
fn receipts_hash_sensitive_endpoints_and_keep_reviewable_metadata() {
    // r[agent_harness_runtime.zerostack_provider_hardening.receipts.endpoint]
    let receipt = provider_selection_receipt(&ProviderSelectionReceiptInput {
        provider_id: PROVIDER_ID.to_string(),
        model: model(ProviderModelClass::TextAgent),
        endpoint: ENDPOINT.to_string(),
        sensitivity: ProviderEndpointSensitivity::Sensitive,
        api_style: ProviderApiStyle::Responses,
        extra_body_descriptor_ids: vec!["extra.reasoning".to_string()],
        diagnostics: Vec::new(),
    });
    assert!(receipt.endpoint_identity.starts_with(BLAKE3_ALGORITHM));
    assert_eq!(receipt.api_style, ProviderApiStyle::Responses);
    assert_eq!(receipt.capability_class, ProviderModelClass::TextAgent);
    assert_eq!(receipt.credential_source_class, "redacted");
}

#[test]
fn provider_stream_parser_inventory_names_entrypoint_and_raw_boundary_review() {
    // r[agent_harness_runtime.provider_stream_parser_rails.inventory]
    let inventory = provider_stream_parser_inventory();
    assert_eq!(inventory.len(), EXPECTED_PARSER_INVENTORY_ENTRIES);
    assert!(validate_provider_stream_parser_inventory(&inventory).is_ok());
    let entry = &inventory[0];
    assert_eq!(entry.entrypoint, "parse_provider_stream_chunks");
    assert!(entry.input_framing.contains("SSE"));
    assert!(entry.normalized_output.contains("ModelStreamFrame"));
    assert!(entry.budget_policy.contains("pre-buffer"));
    assert!(entry.receipt_projection.contains("ProviderStreamParserReceipt"));
    assert!(entry.raw_boundary_review.contains("budgeted"));

    let invalid = vec![ProviderStreamParserInventoryEntry {
        entrypoint: String::new(),
        input_framing: String::new(),
        normalized_output: String::new(),
        budget_policy: String::new(),
        receipt_projection: String::new(),
        raw_boundary_review: String::new(),
    }];
    assert!(validate_provider_stream_parser_inventory(&invalid).is_err());
}

#[test]
fn provider_stream_parser_accepts_sse_json_split_boundaries_and_safe_receipts() {
    // r[agent_harness_runtime.provider_stream_parser_rails.entrypoint_fixtures.split_boundary]
    // r[agent_harness_runtime.provider_stream_parser_rails.receipts.hidden_reasoning]
    let chunks: &[&[u8]] = &[
        b"data: {\"event\":\"content\",\"call_id\":\"call-1\",\"order\":1,\"body\":\"Hel\"}\n",
        b"data: {\"event\":\"content\",\"call_id\":\"call-1\",\"order\":2,\"body\":\"lo",
        b"\"}\n{\"event\":\"thought\",\"call_id\":\"call-1\",\"order\":3,\"body\":\"private reasoning\"}\n",
        b"{\"event\":\"tool\",\"call_id\":\"call-1\",\"order\":4,\"tool_call_id\":\"tool-1\",\"body\":\"arguments-hash\"}\n",
        b"{\"event\":\"usage\",\"call_id\":\"call-1\",\"order\":5,\"input_tokens\":7,\"output_tokens\":11}\n",
        b"{\"event\":\"finish\",\"call_id\":\"call-1\",\"order\":6}\n",
    ];
    let report = parse_provider_stream_chunks(provider_stream_input(chunks, ProviderStreamParserBudget::default()))
        .expect("provider stream should parse");

    assert_eq!(report.frames.len(), EXPECTED_POSITIVE_STREAM_FRAMES);
    assert_eq!(report.receipt.status, ProviderStreamParserStatus::Success);
    assert_eq!(report.receipt.record_count, EXPECTED_POSITIVE_STREAM_FRAMES);
    assert_eq!(report.receipt.event_count, EXPECTED_POSITIVE_STREAM_FRAMES);
    assert_eq!(report.receipt.completion_count, EXPECTED_TERMINAL_COUNT);
    assert!(report.receipt.stream_identity.starts_with(BLAKE3_ALGORITHM));
    assert!(report.receipt.redaction_markers.contains(&"thought:redacted".to_string()));
    assert!(report.receipt.redaction_markers.contains(&"tool_call:redacted".to_string()));
    assert!(!format!("{:?}", report.receipt).contains(STREAM_PRIVATE_REASONING));
    assert!(validate_provider_stream_parser_receipt(&report.receipt).is_ok());
    assert_eq!(report.frames[0].call_id().as_str(), STREAM_CALL_ID);
    assert_eq!(report.frames[0].body(), Some("Hel"));
    assert_eq!(report.frames[1].body(), Some("lo"));
    assert_eq!(report.frames[3].tool_call_id().expect("tool id").as_str(), STREAM_TOOL_ID);
    assert_eq!(
        report
            .frames
            .iter()
            .filter(|frame| frame.kind() == animus_adapters::ModelStreamFrameKind::ContentDelta)
            .count(),
        EXPECTED_POSITIVE_CONTENT_FRAMES
    );
}

#[test]
fn provider_stream_parser_rejects_malformed_order_and_terminal_cases() {
    // r[agent_harness_runtime.provider_stream_parser_rails.typed_failures.malformed_record]
    assert_parse_failure(
        &[INVALID_UTF8_CHUNK],
        ProviderStreamParserBudget::default(),
        ProviderStreamParseFailureKind::InvalidEncoding,
        None,
    );
    assert_parse_failure(
        &[b"data: {not-json}\n"],
        ProviderStreamParserBudget::default(),
        ProviderStreamParseFailureKind::MalformedRecord,
        None,
    );
    assert_parse_failure(
        &[b"{\"event\":\"audio\",\"call_id\":\"call-1\",\"order\":1}\n"],
        ProviderStreamParserBudget::default(),
        ProviderStreamParseFailureKind::UnsupportedEvent,
        None,
    );
    assert_parse_failure(
        &[b"{\"event\":\"terminal\",\"status\":\"mystery\",\"call_id\":\"call-1\",\"order\":1}\n"],
        ProviderStreamParserBudget::default(),
        ProviderStreamParseFailureKind::UnknownTerminalState,
        None,
    );
    assert_parse_failure(
        &[b"{\"event\":\"content\",\"call_id\":\"call-1\",\"order\":1,\"body\":\"partial\"}\n"],
        ProviderStreamParserBudget::default(),
        ProviderStreamParseFailureKind::MissingTerminal,
        None,
    );
    assert_parse_failure(
        &[
            b"{\"event\":\"content\",\"call_id\":\"call-1\",\"order\":2,\"body\":\"late\"}\n",
            b"{\"event\":\"content\",\"call_id\":\"call-1\",\"order\":1,\"body\":\"early\"}\n",
        ],
        ProviderStreamParserBudget::default(),
        ProviderStreamParseFailureKind::InvalidOrder,
        None,
    );
    assert_parse_failure(
        &[
            b"{\"event\":\"finish\",\"call_id\":\"call-1\",\"order\":1}\n",
            b"{\"event\":\"failure\",\"call_id\":\"call-1\",\"order\":2,\"reason\":\"provider\"}\n",
        ],
        ProviderStreamParserBudget::default(),
        ProviderStreamParseFailureKind::DuplicateTerminal,
        None,
    );
    assert_parse_failure(
        &[b"{\"event\":\"content\",\"call_id\":\"call-1\",\"order\":1,\"body\":\"unterminated\"}"],
        ProviderStreamParserBudget::default(),
        ProviderStreamParseFailureKind::MalformedRecord,
        None,
    );
}

#[test]
fn provider_stream_parser_enforces_pre_aggregation_budgets() {
    // r[agent_harness_runtime.provider_stream_parser_rails.budgets.oversized_frame]
    assert_parse_failure(
        &[b"{\"event\":\"finish\",\"call_id\":\"call-1\",\"order\":1}\n"],
        ProviderStreamParserBudget {
            frame_bytes: TINY_FRAME_BUDGET,
            ..ProviderStreamParserBudget::default()
        },
        ProviderStreamParseFailureKind::BudgetExceeded,
        Some(ProviderStreamBudgetKind::FrameBytes),
    );
    assert_parse_failure(
        &[b"{\"event\":\"finish\",\"call_id\":\"call-1\",\"order\":1}\n"],
        ProviderStreamParserBudget {
            record_bytes: TINY_RECORD_BUDGET,
            ..ProviderStreamParserBudget::default()
        },
        ProviderStreamParseFailureKind::BudgetExceeded,
        Some(ProviderStreamBudgetKind::RecordBytes),
    );
    assert_parse_failure(
        ordered_content_finish_chunks(),
        ProviderStreamParserBudget {
            aggregate_bytes: TINY_AGGREGATE_BUDGET,
            ..ProviderStreamParserBudget::default()
        },
        ProviderStreamParseFailureKind::BudgetExceeded,
        Some(ProviderStreamBudgetKind::AggregateBytes),
    );
    assert_parse_failure(
        ordered_content_finish_chunks(),
        ProviderStreamParserBudget {
            record_count: ONE_RECORD_BUDGET,
            ..ProviderStreamParserBudget::default()
        },
        ProviderStreamParseFailureKind::BudgetExceeded,
        Some(ProviderStreamBudgetKind::RecordCount),
    );
    assert_parse_failure(
        ordered_content_finish_chunks(),
        ProviderStreamParserBudget {
            event_count: ONE_EVENT_BUDGET,
            ..ProviderStreamParserBudget::default()
        },
        ProviderStreamParseFailureKind::BudgetExceeded,
        Some(ProviderStreamBudgetKind::EventCount),
    );
    assert_parse_failure(
        &[b"{\"event\":\"finish\",\"call_id\":\"call-1\",\"order\":1}\n"],
        ProviderStreamParserBudget {
            completion_count: ZERO_COMPLETION_BUDGET,
            ..ProviderStreamParserBudget::default()
        },
        ProviderStreamParseFailureKind::BudgetExceeded,
        Some(ProviderStreamBudgetKind::CompletionCount),
    );
}

#[test]
fn provider_stream_parser_receipt_validation_rejects_leak_markers() {
    // r[agent_harness_runtime.provider_stream_parser_rails.receipts.hidden_reasoning]
    // r[agent_harness_runtime.provider_stream_parser_rails.verification]
    let chunks: &[&[u8]] = &[b"{\"event\":\"finish\",\"call_id\":\"call-1\",\"order\":1}\n"];
    let report = parse_provider_stream_chunks(provider_stream_input(chunks, ProviderStreamParserBudget::default()))
        .expect("minimal finish should parse");
    let mut forged = report.receipt.clone();
    forged.redaction_markers.push("hidden_reasoning: private reasoning".to_string());

    let failure = validate_provider_stream_parser_receipt(&forged).expect_err("leak marker");
    assert_eq!(failure.kind, ProviderStreamParseFailureKind::ReceiptLeak);

    let mut forged_family = report.receipt.clone();
    forged_family.provider_family = "/home/provider/raw".to_string();
    let family_failure = validate_provider_stream_parser_receipt(&forged_family).expect_err("unsafe family");
    assert_eq!(family_failure.kind, ProviderStreamParseFailureKind::ReceiptLeak);
}

fn ordered_content_finish_chunks() -> &'static [&'static [u8]] {
    &[ORDERED_CONTENT_CHUNK, ORDERED_FINISH_CHUNK]
}

fn provider_stream_input<'a>(
    chunks: &'a [&'a [u8]],
    budget: ProviderStreamParserBudget,
) -> ProviderStreamParserInput<'a> {
    ProviderStreamParserInput {
        provider_family: PROVIDER_ID,
        chunks,
        budget,
    }
}

fn assert_parse_failure(
    chunks: &[&[u8]],
    budget: ProviderStreamParserBudget,
    expected: ProviderStreamParseFailureKind,
    expected_budget: Option<ProviderStreamBudgetKind>,
) {
    let report =
        parse_provider_stream_chunks(provider_stream_input(chunks, budget)).expect_err("parser should fail closed");
    assert_eq!(report.failure.kind, expected);
    assert_eq!(report.failure.budget_kind, expected_budget);
    assert_eq!(report.receipt.status, ProviderStreamParserStatus::ParserFailure);
    assert_eq!(report.receipt.failure_kind, Some(expected));
    assert_eq!(report.receipt.budget_kind, expected_budget);
    assert!(report.receipt.stream_identity.starts_with(BLAKE3_ALGORITHM));
    assert!(validate_provider_stream_parser_receipt(&report.receipt).is_ok());
}

fn model(model_class: ProviderModelClass) -> ProviderModelMetadata {
    ProviderModelMetadata {
        provider_id: PROVIDER_ID.to_string(),
        model_id: MODEL_ID.to_string(),
        display_name: "Agent Model".to_string(),
        model_class,
        context_budget_label: "large".to_string(),
        reasoning: true,
    }
}

fn structured_request(
    include_tools: bool,
    include_image: bool,
    include_controls: bool,
) -> animus_adapters::CanonicalModelRequest {
    let system_content = [animus_adapters::CanonicalMessageContentPartDraft::Text { text: SYSTEM_PROMPT }];
    let text_part = animus_adapters::CanonicalMessageContentPartDraft::Text { text: USER_PROMPT };
    let image_part = animus_adapters::CanonicalMessageContentPartDraft::ImageDescriptor {
        descriptor_id: IMAGE_DESCRIPTOR_ID,
        media_type: IMAGE_MEDIA_TYPE,
        content_hash: IMAGE_HASH,
    };
    let user_content_with_image = [text_part, image_part];
    let user_content_text_only = [text_part];
    let user_content = if include_image {
        user_content_with_image.as_slice()
    } else {
        user_content_text_only.as_slice()
    };
    let messages = [
        animus_adapters::CanonicalModelMessageDraft {
            role: animus_adapters::CanonicalModelRole::System.as_str(),
            content: &system_content,
            tool_call_id: None,
        },
        animus_adapters::CanonicalModelMessageDraft {
            role: animus_adapters::CanonicalModelRole::User.as_str(),
            content: user_content,
            tool_call_id: None,
        },
    ];
    let tools = [animus_adapters::CanonicalModelToolDeclarationDraft {
        name: TOOL_NAME,
        description: TOOL_DESCRIPTION,
        input_schema_label: TOOL_SCHEMA,
    }];
    let selected_tools = if include_tools { tools.as_slice() } else { &[] };
    animus_adapters::CanonicalModelRequest::from_draft(animus_adapters::CanonicalModelRequestDraft {
        contract_identity: animus_adapters::CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
        messages: &messages,
        tools: selected_tools,
        output: animus_adapters::CanonicalModelOutputControlsDraft {
            max_output_tokens: Some(OUTPUT_BUDGET),
            temperature_millis: None,
            stop_sequences: &[],
            stream: include_controls,
        },
        response_format: if include_controls {
            animus_adapters::CanonicalModelResponseFormatIntentDraft::JsonSchema {
                schema_label: RESPONSE_SCHEMA,
            }
        } else {
            animus_adapters::CanonicalModelResponseFormatIntentDraft::Text
        },
        reasoning: animus_adapters::CanonicalModelReasoningRequestDraft {
            enabled: include_controls,
            effort: if include_controls {
                animus_adapters::CanonicalModelReasoningEffort::Medium
            } else {
                animus_adapters::CanonicalModelReasoningEffort::None
            },
            summary_label: None,
        },
        provenance: &[],
    })
    .expect("canonical request fixture should validate")
}

fn full_capabilities() -> ProviderModelCapabilityMatrix {
    ProviderModelCapabilityMatrix {
        schema_identity: CAPABILITY_SCHEMA.to_string(),
        provider_id: PROVIDER_ID.to_string(),
        model_id: MODEL_ID.to_string(),
        roles: vec![
            animus_adapters::CanonicalModelRole::System,
            animus_adapters::CanonicalModelRole::User,
            animus_adapters::CanonicalModelRole::Assistant,
            animus_adapters::CanonicalModelRole::Tool,
        ],
        content_parts: vec![
            ProviderModelContentPartClass::Text,
            ProviderModelContentPartClass::Image,
        ],
        tool_calls: true,
        structured_outputs: vec![
            ProviderStructuredOutputMode::JsonObject,
            ProviderStructuredOutputMode::JsonSchema,
        ],
        stream_channels: vec![
            ProviderStreamChannel::Content,
            ProviderStreamChannel::ToolCall,
            ProviderStreamChannel::Usage,
        ],
        reasoning_controls: vec![
            animus_adapters::CanonicalModelReasoningEffort::Low,
            animus_adapters::CanonicalModelReasoningEffort::Medium,
            animus_adapters::CanonicalModelReasoningEffort::High,
        ],
        context_budget: CONTEXT_BUDGET,
        output_budget: OUTPUT_BUDGET,
        privacy: ProviderPrivacyPosture::PrivateGateway,
    }
}

const fn deny_downgrades() -> ModelCapabilityDowngradePolicy {
    ModelCapabilityDowngradePolicy {
        allow_image_omission: false,
        allow_structured_output_omission: false,
        allow_streaming_omission: false,
        allow_reasoning_omission: false,
    }
}

const fn allow_optional_downgrades() -> ModelCapabilityDowngradePolicy {
    ModelCapabilityDowngradePolicy {
        allow_image_omission: true,
        allow_structured_output_omission: true,
        allow_streaming_omission: true,
        allow_reasoning_omission: true,
    }
}

fn assert_issue(
    request: &animus_adapters::CanonicalModelRequest,
    capabilities: &ProviderModelCapabilityMatrix,
    expected: ModelCapabilityIssueKind,
) {
    let denied = negotiate_model_capability(ModelCapabilityNegotiationInput {
        request,
        capabilities,
        downgrade_policy: deny_downgrades(),
        required_privacy: ProviderPrivacyPosture::RemoteProvider,
        provider_fallback_requested: false,
    });
    assert_eq!(denied.status, ModelCapabilityDecisionStatus::Denied);
    assert!(denied.issues.iter().any(|issue| issue.kind == expected));
}

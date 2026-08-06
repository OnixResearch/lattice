//! Tests for provider projection conformance.
#![allow(unknown_lints, assertion_density, ambiguous_params, path_segment_repetition)]

use crate::model::*;

const DEVELOPER_TEXT: &str = "Preserve provider-neutral semantics.";
const USER_TEXT: &str = "Answer with safe structure.";
const IMAGE_DESCRIPTOR: &str = "image-one";
const IMAGE_MEDIA_TYPE: &str = "image/png";
const IMAGE_HASH: &str = "b3-image";
const TOOL_NAME: &str = "lookup";
const TOOL_DESCRIPTION: &str = "Bounded lookup.";
const TOOL_SCHEMA: &str = "schema.lookup";
const RESPONSE_SCHEMA: &str = "schema.answer";
const RESPONSE_TEXT: &str = "ok";
const PAYLOAD_LABEL: &str = "payload-hash";
const TOOL_CALL_ID: &str = "call-one";
const ARGUMENTS_LABEL: &str = "arguments-hash";
const REFUSAL_REASON: &str = "policy_refusal";
const SAFETY_CATEGORY: &str = "self_harm";
const MALFORMED_DIAGNOSTIC: &str = "malformed-json";
const CREDENTIAL_SENTINEL: &str = "credential:redacted-token";
const RAW_PAYLOAD: &str = "raw_provider_payload:{secret}";
const RAW_REQUEST_PAYLOAD_LABEL: &str = "provider_payload:request";
const HIDDEN_REASONING: &str = "hidden_reasoning: private";
const INPUT_TOKENS: u32 = 12;
const OUTPUT_TOKENS: u32 = 8;
const CACHED_INPUT_TOKENS: u32 = 4;
const EXPECTED_LITERAL_COUNT: usize = 11;
const STREAM_CALL_ID: &str = "call-1";
const STREAM_TOOL_ID: &str = "tool-1";
const INDEX_FEATURE: &str = "json_schema";
const INDEX_EVIDENCE: &str = "fixture-json-schema";
const INDEX_REASON: &str = "provider family has no image input";
const PRESERVED_FEATURE: ProviderProjectionFeatureSupport = ProviderProjectionFeatureSupport::Preserved;
const UNSUPPORTED_FEATURE: ProviderProjectionFeatureSupport = ProviderProjectionFeatureSupport::Unsupported;

#[test]
fn request_projection_matches_literal_fixtures_for_claimed_families() {
    // r[verify agent_harness_runtime.provider_projection_conformance.literal_fixtures]
    let request = canonical_projection_request();
    for family in [
        ProviderProjectionFamily::OpenAiResponses,
        ProviderProjectionFamily::OpenAiChat,
        ProviderProjectionFamily::AnthropicMessages,
        ProviderProjectionFamily::Mock,
    ] {
        let expected = expected_literals(family);
        let expected_refs: alloc::vec::Vec<_> = expected.iter().map(alloc::string::String::as_str).collect();
        let projection = assert_provider_request_fixture(&request, full_support(family), &expected_refs)
            .expect("literal provider projection should match reviewed fixture");
        assert_eq!(projection.family(), family);
        assert_eq!(projection.literal_parts().len(), EXPECTED_LITERAL_COUNT);
    }
}

#[test]
fn request_projection_fails_closed_for_lossy_provider_support() {
    // r[verify agent_harness_runtime.provider_projection_conformance.lossy_projection]
    let request = canonical_projection_request();
    for (support, feature) in [
        (
            ProviderProjectionSupport {
                developer_role: UNSUPPORTED_FEATURE,
                ..full_support(ProviderProjectionFamily::OpenAiResponses)
            },
            "role",
        ),
        (
            ProviderProjectionSupport {
                images: UNSUPPORTED_FEATURE,
                ..full_support(ProviderProjectionFamily::OpenAiResponses)
            },
            "content_part",
        ),
        (
            ProviderProjectionSupport {
                tools: UNSUPPORTED_FEATURE,
                ..full_support(ProviderProjectionFamily::OpenAiResponses)
            },
            "tool",
        ),
        (
            ProviderProjectionSupport {
                json_schema: UNSUPPORTED_FEATURE,
                ..full_support(ProviderProjectionFamily::OpenAiResponses)
            },
            "structured_output",
        ),
        (
            ProviderProjectionSupport {
                reasoning: UNSUPPORTED_FEATURE,
                ..full_support(ProviderProjectionFamily::OpenAiResponses)
            },
            "reasoning",
        ),
        (
            ProviderProjectionSupport {
                streaming: UNSUPPORTED_FEATURE,
                ..full_support(ProviderProjectionFamily::OpenAiResponses)
            },
            "streaming",
        ),
        (
            ProviderProjectionSupport {
                metadata: UNSUPPORTED_FEATURE,
                ..full_support(ProviderProjectionFamily::OpenAiResponses)
            },
            "metadata",
        ),
    ] {
        let error = project_provider_request_fixture(&request, support).expect_err("lossy support should fail closed");
        assert_eq!(error, ProviderProjectionError::LossyProjection { feature });
    }
}

#[test]
fn request_leaks_and_malformed_metadata_fail_before_provider_projection() {
    // r[verify agent_harness_runtime.provider_projection_conformance.lossy_projection]
    let raw_content = [CanonicalContentPartDraft::RawProviderPayload {
        label: RAW_REQUEST_PAYLOAD_LABEL,
    }];
    let raw_message = [CanonicalModelMessageDraft {
        role: "user",
        content: &raw_content,
        tool_call_id: None,
    }];
    let raw_payload_error = CanonicalModelRequest::from_draft(CanonicalModelRequestDraft {
        contract_identity: CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
        messages: &raw_message,
        tools: &[],
        output: compatibility_output(),
        response_format: CanonicalResponseFormatIntentDraft::Text,
        reasoning: compatibility_reasoning(),
        provenance: &[],
    })
    .expect_err("raw provider payload in request should fail before projection");
    assert!(matches!(raw_payload_error, CanonicalModelContractError::UnsafeText { .. }));

    let text_content = [CanonicalContentPartDraft::Text { text: USER_TEXT }];
    let text_message = [CanonicalModelMessageDraft {
        role: "user",
        content: &text_content,
        tool_call_id: None,
    }];
    let unsafe_metadata = [CanonicalSafeMetadataDraft {
        key: "trace_label",
        value: CREDENTIAL_SENTINEL,
    }];
    let metadata_error = CanonicalModelRequest::from_draft(CanonicalModelRequestDraft {
        contract_identity: CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
        messages: &text_message,
        tools: &[],
        output: compatibility_output(),
        response_format: CanonicalResponseFormatIntentDraft::Text,
        reasoning: compatibility_reasoning(),
        provenance: &unsafe_metadata,
    })
    .expect_err("unsafe metadata should fail before projection");
    assert!(matches!(metadata_error, CanonicalModelContractError::UnsafeText { .. }));
}

#[test]
fn response_normalization_preserves_supported_semantics() {
    // r[verify agent_harness_runtime.provider_projection_conformance.response_normalization]
    let text = normalize_provider_response_fixture(text_response_fixture(
        RESPONSE_TEXT,
        Some(CanonicalFinishReason::Stop.as_str()),
    ))
    .expect("text fixture should normalize");
    assert_eq!(text.items()[0].assistant_text(), Some(RESPONSE_TEXT));
    assert_eq!(text.usage().expect("usage should exist").input_tokens(), INPUT_TOKENS);

    let structured = normalize_provider_response_fixture(structured_response_fixture(
        PAYLOAD_LABEL,
        Some(CanonicalFinishReason::Stop.as_str()),
    ))
    .expect("structured fixture should normalize");
    assert!(matches!(structured.items()[0], CanonicalModelResponseItem::StructuredPayload { .. }));

    let tool = normalize_provider_response_fixture(tool_response_fixture(
        TOOL_CALL_ID,
        TOOL_NAME,
        ARGUMENTS_LABEL,
        Some(CanonicalFinishReason::ToolCall.as_str()),
    ))
    .expect("tool-call fixture should normalize");
    assert!(matches!(tool.items()[0], CanonicalModelResponseItem::ToolCall { .. }));

    let refusal = normalize_provider_response_fixture(refusal_response_fixture(
        REFUSAL_REASON,
        Some(CanonicalFinishReason::Refusal.as_str()),
    ))
    .expect("refusal fixture should normalize");
    assert!(matches!(refusal.items()[0], CanonicalModelResponseItem::Refusal { .. }));

    let safety = normalize_provider_response_fixture(ProviderResponseFixtureDraft {
        safety: Some(SAFETY_CATEGORY),
        ..base_response_fixture(Some(CanonicalFinishReason::Safety.as_str()))
    })
    .expect("safety fixture should normalize");
    assert!(matches!(safety.items()[0], CanonicalModelResponseItem::Safety { .. }));

    let malformed = normalize_provider_response_fixture(ProviderResponseFixtureDraft {
        malformed_output: Some(MALFORMED_DIAGNOSTIC),
        ..base_response_fixture(Some(CanonicalFinishReason::Malformed.as_str()))
    })
    .expect("malformed fixture should normalize");
    assert!(matches!(malformed.items()[0], CanonicalModelResponseItem::MalformedOutput { .. }));
}
#[path = "part2.rs"]
pub mod part2;
pub use part2::*;

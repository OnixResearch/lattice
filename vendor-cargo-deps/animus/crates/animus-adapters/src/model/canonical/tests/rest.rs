#![allow(unknown_lints)]
#![allow(non_trait_imports)]
use super::*;

#[test]
fn canonical_response_rejects_missing_finish_and_malformed_usage() {
    // r[verify agent_harness_runtime.model_compat.response_envelope.validation]
    let items = [CanonicalModelResponseItemDraft::AssistantText {
        text: RESPONSE_TEXT,
    }];
    let missing_finish = response_with_items(&items, None).expect_err("finish reason is required");
    assert_eq!(
        missing_finish,
        CanonicalModelContractError::MissingFinishReason {
            field: "model_finish_reason",
        }
    );

    let malformed_usage =
        CanonicalModelResponseEnvelope::from_draft(CanonicalModelResponseEnvelopeDraft {
            contract_identity: CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY,
            items: &items,
            usage: Some(CanonicalUsageDraft {
                input: INPUT_TOKENS,
                output: OUTPUT_TOKENS,
                cached_input: Some(TOO_MANY_CACHED_TOKENS),
            }),
            finish_reason: Some(CanonicalFinishReason::Stop.as_str()),
            diagnostics: &[],
            provenance: &[],
        })
        .expect_err("cached usage greater than input should fail");
    assert!(matches!(
        malformed_usage,
        CanonicalModelContractError::MalformedUsage { .. }
    ));
}

#[test]
fn canonical_response_rejects_impossible_combinations_and_leaks() {
    // r[verify agent_harness_runtime.model_compat.response_envelope.validation]
    let impossible = [
        CanonicalModelResponseItemDraft::AssistantText {
            text: RESPONSE_TEXT,
        },
        CanonicalModelResponseItemDraft::Safety {
            category: SAFETY_CATEGORY,
        },
    ];
    let error = response_with_items(&impossible, Some(CanonicalFinishReason::Safety.as_str()))
        .expect_err("success and safety should not combine");
    assert!(matches!(
        error,
        CanonicalModelContractError::ImpossibleResponseCombination { .. }
    ));

    let raw = [CanonicalModelResponseItemDraft::RawProviderPayload {
        label: RAW_PAYLOAD_LABEL,
    }];
    let error = response_with_items(&raw, Some(CanonicalFinishReason::Stop.as_str()))
        .expect_err("raw provider payload should fail");
    assert_eq!(
        error,
        CanonicalModelContractError::UnsafeText {
            field: "model_raw_provider_payload",
            reason: CanonicalUnsafeTextReason::RawProviderPayload,
        }
    );

    let hidden = [CanonicalModelResponseItemDraft::HiddenReasoning {
        label: HIDDEN_REASONING_LABEL,
    }];
    let error = response_with_items(&hidden, Some(CanonicalFinishReason::Stop.as_str()))
        .expect_err("hidden reasoning should fail");
    assert_eq!(
        error,
        CanonicalModelContractError::UnsafeText {
            field: "model_hidden_reasoning",
            reason: CanonicalUnsafeTextReason::HiddenReasoning,
        }
    );
}

#[test]
fn legacy_prompt_message_round_trips_through_canonical_defaults() {
    // r[verify agent_harness_runtime.model_compat.legacy_bridge.equivalent]
    let legacy_request = Request::new(USER_PROMPT).expect("legacy request should validate");
    let canonical_request = CanonicalModelRequest::from_legacy_request(&legacy_request)
        .expect("legacy request should project to canonical");
    let projected_request = canonical_request
        .to_legacy_request()
        .expect("canonical compatibility request should project back");
    assert_eq!(projected_request, legacy_request);

    let legacy_response = Response::new(RESPONSE_TEXT).expect("legacy response should validate");
    let canonical_response = CanonicalModelResponseEnvelope::from_legacy_response(&legacy_response)
        .expect("legacy response should project to canonical");
    let projected_response = canonical_response
        .to_legacy_response()
        .expect("canonical text response should project back");
    assert_eq!(projected_response, legacy_response);
}

#[test]
fn legacy_projection_fails_closed_for_lossy_canonical_responses() {
    // r[verify agent_harness_runtime.model_compat.legacy_bridge.equivalent]
    let structured = [CanonicalModelResponseItemDraft::StructuredPayload {
        schema_label: JSON_SCHEMA,
        payload_label: STRUCTURED_PAYLOAD,
    }];
    let canonical_response =
        response_with_items(&structured, Some(CanonicalFinishReason::Stop.as_str()))
            .expect("structured response should validate");
    let error = canonical_response
        .to_legacy_response()
        .expect_err("structured response should not collapse into legacy text");
    assert_eq!(
        error,
        CanonicalModelContractError::LossyLegacyProjection {
            reason: "canonical response carries non-text semantics",
        }
    );

    let text_content = [CanonicalContentPartDraft::Text { text: USER_PROMPT }];
    let message = [CanonicalModelMessageDraft {
        role: CanonicalModelRole::User.as_str(),
        content: &text_content,
        tool_call_id: None,
    }];
    let tools = [CanonicalToolDeclarationDraft {
        name: TOOL_NAME,
        description: TOOL_DESCRIPTION,
        input_schema_label: TOOL_SCHEMA,
    }];
    let canonical_request = CanonicalModelRequest::from_draft(CanonicalModelRequestDraft {
        contract_identity: CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
        messages: &message,
        tools: &tools,
        output: default_output(),
        response_format: CanonicalResponseFormatIntentDraft::Text,
        reasoning: default_reasoning(),
        provenance: &[],
    })
    .expect("tool-enabled request should validate");
    let error = canonical_request
        .to_legacy_request()
        .expect_err("tool-enabled request should not collapse into legacy prompt");
    assert_eq!(
        error,
        CanonicalModelContractError::LossyLegacyProjection {
            reason: "canonical request cannot collapse into one legacy prompt",
        }
    );
}

pub fn request_with_messages(
    messages: &[CanonicalModelMessageDraft<'_>],
) -> Result<CanonicalModelRequest, CanonicalModelContractError> {
    CanonicalModelRequest::from_draft(CanonicalModelRequestDraft {
        contract_identity: CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
        messages,
        tools: &[],
        output: default_output(),
        response_format: CanonicalResponseFormatIntentDraft::Text,
        reasoning: default_reasoning(),
        provenance: &[],
    })
}

pub fn response_with_items(
    items: &[CanonicalModelResponseItemDraft<'_>],
    finish_reason: Option<&str>,
) -> Result<CanonicalModelResponseEnvelope, CanonicalModelContractError> {
    CanonicalModelResponseEnvelope::from_draft(CanonicalModelResponseEnvelopeDraft {
        contract_identity: CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY,
        items,
        usage: Some(CanonicalUsageDraft {
            input: INPUT_TOKENS,
            output: OUTPUT_TOKENS,
            cached_input: Some(CACHED_TOKENS),
        }),
        finish_reason,
        diagnostics: &[],
        provenance: &[],
    })
}

pub const fn default_output<'a>() -> CanonicalOutputControlsDraft<'a> {
    CanonicalOutputControlsDraft {
        max_output_tokens: None,
        temperature_millis: None,
        stop_sequences: &[],
        stream: false,
    }
}

pub const fn default_reasoning<'a>() -> CanonicalReasoningRequestDraft<'a> {
    CanonicalReasoningRequestDraft {
        enabled: false,
        effort: CanonicalReasoningEffort::None,
        summary_label: None,
    }
}

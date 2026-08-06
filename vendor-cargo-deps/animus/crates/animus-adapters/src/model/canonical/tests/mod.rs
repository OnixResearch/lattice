//! Tests for canonical model message contracts.
#![allow(unknown_lints, function_length, numeric_units)]

use crate::model::*;

const USER_PROMPT: &str = "Summarize the safe plan.";
const SYSTEM_PROMPT: &str = "Stay inside adapter contracts.";
const TOOL_RESULT: &str = "tool result";
const TOOL_CALL_ID: &str = "call-lookup";
const TOOL_NAME: &str = "lookup";
const TOOL_DESCRIPTION: &str = "Returns bounded facts.";
const TOOL_SCHEMA: &str = "schema.lookup.v1";
const IMAGE_DESCRIPTOR: &str = "image-descriptor-1";
const IMAGE_MEDIA_TYPE: &str = "image/png";
const IMAGE_HASH: &str = "b3-safe-image-hash";
const JSON_SCHEMA: &str = "schema.answer.v1";
const METADATA_KEY: &str = "trace";
const METADATA_VALUE: &str = "safe-trace";
const STOP_SEQUENCE: &str = "<END>";
const RESPONSE_TEXT: &str = "done";
const STRUCTURED_PAYLOAD: &str = "payload-hash";
const REFUSAL_REASON: &str = "policy_refusal";
const SAFETY_CATEGORY: &str = "safety_boundary";
const MALFORMED_DIAGNOSTIC: &str = "missing_required_field";
const RAW_PAYLOAD_LABEL: &str = "raw_provider_payload:{secret}";
const HIDDEN_REASONING_LABEL: &str = "hidden_reasoning: private chain";
const UNSUPPORTED_ROLE: &str = "robot";
const UNSUPPORTED_CONTENT_KIND: &str = "audio-bytes";
const WRONG_CONTRACT_IDENTITY: &str = "animus.model-message-request.v0";
const OVERSIZED_METADATA_COUNT: usize = 33;
const INPUT_TOKENS: u32 = 10;
const OUTPUT_TOKENS: u32 = 6;
const CACHED_TOKENS: u32 = 4;
const TOO_MANY_CACHED_TOKENS: u32 = 11;
const TEMPERATURE_MILLIS: u16 = 4;

#[test]
fn canonical_request_accepts_structured_conversation() {
    // r[verify agent_harness_runtime.model_compat.message_request.validation]
    let system_content = [CanonicalContentPartDraft::Text { text: SYSTEM_PROMPT }];
    let user_content = [
        CanonicalContentPartDraft::Text { text: USER_PROMPT },
        CanonicalContentPartDraft::ImageDescriptor {
            descriptor_id: IMAGE_DESCRIPTOR,
            media_type: IMAGE_MEDIA_TYPE,
            content_hash: IMAGE_HASH,
        },
    ];
    let tool_content = [CanonicalContentPartDraft::Text { text: TOOL_RESULT }];
    let messages = [
        CanonicalModelMessageDraft {
            role: CanonicalModelRole::System.as_str(),
            content: &system_content,
            tool_call_id: None,
        },
        CanonicalModelMessageDraft {
            role: CanonicalModelRole::User.as_str(),
            content: &user_content,
            tool_call_id: None,
        },
        CanonicalModelMessageDraft {
            role: CanonicalModelRole::Tool.as_str(),
            content: &tool_content,
            tool_call_id: Some(TOOL_CALL_ID),
        },
    ];
    let tools = [CanonicalToolDeclarationDraft {
        name: TOOL_NAME,
        description: TOOL_DESCRIPTION,
        input_schema_label: TOOL_SCHEMA,
    }];
    let stop_sequences = [STOP_SEQUENCE];
    let provenance = [CanonicalSafeMetadataDraft {
        key: METADATA_KEY,
        value: METADATA_VALUE,
    }];

    let request = CanonicalModelRequest::from_draft(CanonicalModelRequestDraft {
        contract_identity: CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
        messages: &messages,
        tools: &tools,
        output: CanonicalOutputControlsDraft {
            max_output_tokens: Some(OUTPUT_TOKENS),
            temperature_millis: Some(TEMPERATURE_MILLIS),
            stop_sequences: &stop_sequences,
            stream: true,
        },
        response_format: CanonicalResponseFormatIntentDraft::JsonSchema {
            schema_label: JSON_SCHEMA,
        },
        reasoning: CanonicalReasoningRequestDraft {
            enabled: true,
            effort: CanonicalReasoningEffort::Medium,
            summary_label: Some("safe-summary"),
        },
        provenance: &provenance,
    })
    .expect("structured canonical request should validate");

    assert_eq!(request.messages().len(), messages.len());
    assert_eq!(request.tools()[0].name(), TOOL_NAME);
    assert!(request.output().stream());
    assert_eq!(request.reasoning().effort(), CanonicalReasoningEffort::Medium);
    assert_eq!(CanonicalModelRequest::contract_identity(), CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY);
}

#[test]
fn canonical_request_rejects_malformed_roles_and_empty_messages() {
    // r[verify agent_harness_runtime.model_compat.message_request.validation]
    let content = [CanonicalContentPartDraft::Text { text: USER_PROMPT }];
    let bad_role = [CanonicalModelMessageDraft {
        role: UNSUPPORTED_ROLE,
        content: &content,
        tool_call_id: None,
    }];
    let error = request_with_messages(&bad_role).expect_err("unsupported role should fail");
    assert!(matches!(error, CanonicalModelContractError::UnsupportedRole { .. }));

    let empty_messages: [CanonicalModelMessageDraft<'_>; 0] = [];
    let error = request_with_messages(&empty_messages).expect_err("empty messages should fail");
    assert_eq!(error, CanonicalModelContractError::EmptyCollection { field: "model_role" });
}

#[test]
fn canonical_request_rejects_unsupported_content_and_missing_required_fields() {
    // r[verify agent_harness_runtime.model_compat.message_request.validation]
    let unsupported_content = [CanonicalContentPartDraft::Unsupported {
        kind: UNSUPPORTED_CONTENT_KIND,
    }];
    let unsupported_message = [CanonicalModelMessageDraft {
        role: CanonicalModelRole::User.as_str(),
        content: &unsupported_content,
        tool_call_id: None,
    }];
    let error = request_with_messages(&unsupported_message).expect_err("unsupported part should fail");
    assert!(matches!(error, CanonicalModelContractError::UnsupportedContentPart { .. }));

    let text_content = [CanonicalContentPartDraft::Text { text: USER_PROMPT }];
    let message = [CanonicalModelMessageDraft {
        role: CanonicalModelRole::User.as_str(),
        content: &text_content,
        tool_call_id: None,
    }];
    let error = CanonicalModelRequest::from_draft(CanonicalModelRequestDraft {
        contract_identity: WRONG_CONTRACT_IDENTITY,
        messages: &message,
        tools: &[],
        output: default_output(),
        response_format: CanonicalResponseFormatIntentDraft::Text,
        reasoning: default_reasoning(),
        provenance: &[],
    })
    .expect_err("wrong contract identity should fail");
    assert!(matches!(error, CanonicalModelContractError::InvalidContractIdentity { .. }));
}

#[test]
fn canonical_request_rejects_oversized_metadata_and_leaks() {
    // r[verify agent_harness_runtime.model_compat.message_request.validation]
    let content = [CanonicalContentPartDraft::Text { text: USER_PROMPT }];
    let message = [CanonicalModelMessageDraft {
        role: CanonicalModelRole::User.as_str(),
        content: &content,
        tool_call_id: None,
    }];
    let oversized = alloc::vec![
        CanonicalSafeMetadataDraft {
            key: METADATA_KEY,
            value: METADATA_VALUE,
        };
        OVERSIZED_METADATA_COUNT
    ];
    let error = CanonicalModelRequest::from_draft(CanonicalModelRequestDraft {
        contract_identity: CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
        messages: &message,
        tools: &[],
        output: default_output(),
        response_format: CanonicalResponseFormatIntentDraft::Text,
        reasoning: default_reasoning(),
        provenance: &oversized,
    })
    .expect_err("oversized metadata should fail");
    assert!(matches!(error, CanonicalModelContractError::CollectionTooLarge { .. }));

    let leaky_content = [CanonicalContentPartDraft::RawProviderPayload {
        label: RAW_PAYLOAD_LABEL,
    }];
    let leaky_message = [CanonicalModelMessageDraft {
        role: CanonicalModelRole::User.as_str(),
        content: &leaky_content,
        tool_call_id: None,
    }];
    let error = request_with_messages(&leaky_message).expect_err("raw provider payload should fail");
    assert_eq!(error, CanonicalModelContractError::UnsafeText {
        field: "model_raw_provider_payload",
        reason: CanonicalUnsafeTextReason::RawProviderPayload,
    });
}

#[test]
fn canonical_response_accepts_text_structured_tool_and_refusal_cases() {
    // r[verify agent_harness_runtime.model_compat.response_envelope.validation]
    let text = [CanonicalModelResponseItemDraft::AssistantText { text: RESPONSE_TEXT }];
    let text_response =
        response_with_items(&text, Some(CanonicalFinishReason::Stop.as_str())).expect("text response should validate");
    assert_eq!(text_response.items()[0].assistant_text(), Some(RESPONSE_TEXT));
    assert_eq!(CanonicalModelResponseEnvelope::contract_identity(), CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY);

    let structured = [CanonicalModelResponseItemDraft::StructuredPayload {
        schema_label: JSON_SCHEMA,
        payload_label: STRUCTURED_PAYLOAD,
    }];
    response_with_items(&structured, Some(CanonicalFinishReason::Stop.as_str()))
        .expect("structured response should validate");

    let tool = [CanonicalModelResponseItemDraft::ToolCall {
        call_id: TOOL_CALL_ID,
        tool_name: TOOL_NAME,
        arguments_label: TOOL_SCHEMA,
    }];
    response_with_items(&tool, Some(CanonicalFinishReason::ToolCall.as_str()))
        .expect("tool-call response should validate");

    let refusal = [CanonicalModelResponseItemDraft::Refusal { reason: REFUSAL_REASON }];
    response_with_items(&refusal, Some(CanonicalFinishReason::Refusal.as_str()))
        .expect("refusal response should validate");

    let malformed = [CanonicalModelResponseItemDraft::MalformedOutput {
        diagnostic: MALFORMED_DIAGNOSTIC,
    }];
    response_with_items(&malformed, Some(CanonicalFinishReason::Malformed.as_str()))
        .expect("malformed-output diagnostic response should validate");
}
pub mod rest;
pub use rest::*;

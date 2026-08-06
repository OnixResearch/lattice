#![allow(unknown_lints)]
#![allow(non_trait_imports)]
use super::*;

pub fn canonical_projection_request() -> CanonicalModelRequest {
    let developer_content = [CanonicalContentPartDraft::Text { text: DEVELOPER_TEXT }];
    let user_content = [
        CanonicalContentPartDraft::Text { text: USER_TEXT },
        CanonicalContentPartDraft::ImageDescriptor {
            descriptor_id: IMAGE_DESCRIPTOR,
            media_type: IMAGE_MEDIA_TYPE,
            content_hash: IMAGE_HASH,
        },
    ];
    let messages = [
        CanonicalModelMessageDraft {
            role: "developer",
            content: &developer_content,
            tool_call_id: None,
        },
        CanonicalModelMessageDraft {
            role: "user",
            content: &user_content,
            tool_call_id: None,
        },
    ];
    let tools = [CanonicalToolDeclarationDraft {
        name: TOOL_NAME,
        description: TOOL_DESCRIPTION,
        input_schema_label: TOOL_SCHEMA,
    }];
    let provenance = [CanonicalSafeMetadataDraft {
        key: "trace_label",
        value: "projection-request",
    }];
    CanonicalModelRequest::from_draft(CanonicalModelRequestDraft {
        contract_identity: CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
        messages: &messages,
        tools: &tools,
        output: CanonicalOutputControlsDraft {
            max_output_tokens: None,
            temperature_millis: None,
            stop_sequences: &[],
            stream: true,
        },
        response_format: CanonicalResponseFormatIntentDraft::JsonSchema {
            schema_label: RESPONSE_SCHEMA,
        },
        reasoning: CanonicalReasoningRequestDraft {
            enabled: true,
            effort: CanonicalReasoningEffort::High,
            summary_label: Some("summary-label"),
        },
        provenance: &provenance,
    })
    .expect("canonical request fixture should validate")
}

pub fn compatibility_output() -> CanonicalOutputControlsDraft<'static> {
    CanonicalOutputControlsDraft {
        max_output_tokens: None,
        temperature_millis: None,
        stop_sequences: &[],
        stream: false,
    }
}

pub fn compatibility_reasoning() -> CanonicalReasoningRequestDraft<'static> {
    CanonicalReasoningRequestDraft {
        enabled: false,
        effort: CanonicalReasoningEffort::None,
        summary_label: None,
    }
}

pub fn full_support(family: ProviderProjectionFamily) -> ProviderProjectionSupport {
    ProviderProjectionSupport {
        family,
        developer_role: PRESERVED_FEATURE,
        images: PRESERVED_FEATURE,
        tools: PRESERVED_FEATURE,
        json_object: PRESERVED_FEATURE,
        json_schema: PRESERVED_FEATURE,
        reasoning: PRESERVED_FEATURE,
        streaming: PRESERVED_FEATURE,
        metadata: PRESERVED_FEATURE,
    }
}

pub fn expected_literals(family: ProviderProjectionFamily) -> [alloc::string::String; EXPECTED_LITERAL_COUNT] {
    [
        alloc::format!("family:{}", family.as_str()),
        alloc::string::String::from("role:developer"),
        alloc::format!("text:{DEVELOPER_TEXT}"),
        alloc::string::String::from("role:user"),
        alloc::format!("text:{USER_TEXT}"),
        alloc::string::String::from("image:descriptor"),
        alloc::format!("tool:{TOOL_NAME}"),
        alloc::format!("response:{RESPONSE_SCHEMA}"),
        alloc::string::String::from("reasoning:high"),
        alloc::string::String::from("stream:true"),
        alloc::string::String::from("metadata:trace_label=projection-request"),
    ]
}

pub fn base_response_fixture(finish_reason: Option<&str>) -> ProviderResponseFixtureDraft<'_> {
    ProviderResponseFixtureDraft {
        family: ProviderProjectionFamily::OpenAiResponses.as_str(),
        text: None,
        structured_payload: None,
        tool_call_id: None,
        tool_name: None,
        tool_arguments: None,
        refusal: None,
        safety: None,
        malformed_output: None,
        usage: Some(CanonicalUsageDraft {
            input: INPUT_TOKENS,
            output: OUTPUT_TOKENS,
            cached_input: Some(CACHED_INPUT_TOKENS),
        }),
        finish_reason,
        raw_provider_payload: None,
        hidden_reasoning: None,
    }
}

pub fn text_response_fixture(
    text: &'static str,
    finish_reason: Option<&'static str>,
) -> ProviderResponseFixtureDraft<'static> {
    ProviderResponseFixtureDraft {
        text: Some(text),
        ..base_response_fixture(finish_reason)
    }
}

pub fn structured_response_fixture(
    payload_label: &'static str,
    finish_reason: Option<&'static str>,
) -> ProviderResponseFixtureDraft<'static> {
    ProviderResponseFixtureDraft {
        structured_payload: Some(payload_label),
        ..base_response_fixture(finish_reason)
    }
}

pub fn tool_response_fixture(
    call_id: &'static str,
    tool_name: &'static str,
    arguments_label: &'static str,
    finish_reason: Option<&'static str>,
) -> ProviderResponseFixtureDraft<'static> {
    ProviderResponseFixtureDraft {
        tool_call_id: Some(call_id),
        tool_name: Some(tool_name),
        tool_arguments: Some(arguments_label),
        ..base_response_fixture(finish_reason)
    }
}

pub fn refusal_response_fixture(
    reason: &'static str,
    finish_reason: Option<&'static str>,
) -> ProviderResponseFixtureDraft<'static> {
    ProviderResponseFixtureDraft {
        refusal: Some(reason),
        ..base_response_fixture(finish_reason)
    }
}

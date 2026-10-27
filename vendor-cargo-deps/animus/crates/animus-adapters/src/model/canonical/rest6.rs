#![allow(unknown_lints, non_trait_imports, usize_in_public_api)]
use super::{
    CanonicalModelContractError, CanonicalModelResponseItem, CanonicalModelRole,
    CanonicalReasoningEffort, CanonicalSafeMetadata, CanonicalSafeMetadataDraft,
    CanonicalUnsafeTextReason, INVALID_MALFORMED_COMBINATION, INVALID_NON_TOOL_WITH_CALL,
    INVALID_REASONING_DISABLED_EFFORT, INVALID_REASONING_ENABLED_EFFORT,
    INVALID_REFUSAL_COMBINATION, INVALID_TEMPERATURE, INVALID_TOKEN_BUDGET,
    INVALID_TOOL_ROLE_WITHOUT_CALL, LEAK_MARKER_AUTHORIZATION_PREFIX, LEAK_MARKER_BEARER_PREFIX,
    LEAK_MARKER_CREDENTIAL_PREFIX, LEAK_MARKER_HIDDEN_REASONING,
    LEAK_MARKER_PROVIDER_PAYLOAD_PREFIX, LEAK_MARKER_RAW_PROVIDER_PAYLOAD,
    LEAK_MARKER_SECRET_PREFIX, LOSSY_LEGACY_REQUEST, LOSSY_LEGACY_RESPONSE,
    MAX_SAFE_METADATA_ITEMS, MAX_TEMPERATURE_MILLIS, MAX_TOKEN_BUDGET, METADATA_KEY_FIELD,
    METADATA_VALUE_FIELD, MIN_TOKEN_BUDGET, REASONING_SUMMARY_FIELD, RESPONSE_TEXT_FIELD,
    TOOL_CALL_ID_FIELD,
};

pub fn validate_metadata(
    drafts: &[CanonicalSafeMetadataDraft<'_>],
) -> Result<alloc::vec::Vec<CanonicalSafeMetadata>, CanonicalModelContractError> {
    validate_collection_len(METADATA_KEY_FIELD, drafts.len(), MAX_SAFE_METADATA_ITEMS)?;
    drafts
        .iter()
        .map(|draft| {
            Ok(CanonicalSafeMetadata {
                key: safe_text(METADATA_KEY_FIELD, draft.key)?,
                value: safe_text(METADATA_VALUE_FIELD, draft.value)?,
            })
        })
        .collect()
}

pub fn validate_tool_call_id(
    role: CanonicalModelRole,
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, CanonicalModelContractError> {
    match (role, value) {
        (CanonicalModelRole::Tool, Some(raw)) => Ok(Some(safe_text(TOOL_CALL_ID_FIELD, raw)?)),
        (CanonicalModelRole::Tool, None) => Err(CanonicalModelContractError::InvalidField {
            field: TOOL_CALL_ID_FIELD.as_str(),
            reason: INVALID_TOOL_ROLE_WITHOUT_CALL,
        }),
        (_, Some(_raw)) => Err(CanonicalModelContractError::InvalidField {
            field: TOOL_CALL_ID_FIELD.as_str(),
            reason: INVALID_NON_TOOL_WITH_CALL,
        }),
        (_, None) => Ok(None),
    }
}

pub fn validate_token_budget(value: Option<u32>) -> Result<(), CanonicalModelContractError> {
    if value.is_some_and(|tokens| !(MIN_TOKEN_BUDGET..=MAX_TOKEN_BUDGET).contains(&tokens)) {
        return Err(CanonicalModelContractError::InvalidField {
            field: RESPONSE_TEXT_FIELD.as_str(),
            reason: INVALID_TOKEN_BUDGET,
        });
    }
    Ok(())
}

pub fn validate_temperature(value: Option<u16>) -> Result<(), CanonicalModelContractError> {
    if value.is_some_and(|temperature| temperature > MAX_TEMPERATURE_MILLIS) {
        return Err(CanonicalModelContractError::InvalidField {
            field: RESPONSE_TEXT_FIELD.as_str(),
            reason: INVALID_TEMPERATURE,
        });
    }
    Ok(())
}

pub fn validate_reasoning_effort(
    enabled: bool,
    effort: CanonicalReasoningEffort,
) -> Result<(), CanonicalModelContractError> {
    match (enabled, effort) {
        (false, CanonicalReasoningEffort::None)
        | (
            true,
            CanonicalReasoningEffort::Low
            | CanonicalReasoningEffort::Medium
            | CanonicalReasoningEffort::High,
        ) => Ok(()),
        (false, _) => Err(CanonicalModelContractError::InvalidField {
            field: REASONING_SUMMARY_FIELD.as_str(),
            reason: INVALID_REASONING_DISABLED_EFFORT,
        }),
        (true, CanonicalReasoningEffort::None) => Err(CanonicalModelContractError::InvalidField {
            field: REASONING_SUMMARY_FIELD.as_str(),
            reason: INVALID_REASONING_ENABLED_EFFORT,
        }),
    }
}

pub fn validate_response_combination(
    items: &[CanonicalModelResponseItem],
) -> Result<(), CanonicalModelContractError> {
    let mut success_count = 0usize;
    let mut refusal_count = 0usize;
    let mut malformed_count = 0usize;
    for item in items {
        match item {
            CanonicalModelResponseItem::AssistantText { .. }
            | CanonicalModelResponseItem::StructuredPayload { .. }
            | CanonicalModelResponseItem::ToolCall { .. } => {
                success_count = success_count.saturating_add(1);
            }
            CanonicalModelResponseItem::Refusal { .. }
            | CanonicalModelResponseItem::Safety { .. } => {
                refusal_count = refusal_count.saturating_add(1);
            }
            CanonicalModelResponseItem::MalformedOutput { .. } => {
                malformed_count = malformed_count.saturating_add(1);
            }
        }
    }
    if refusal_count > 0 && success_count > 0 {
        return Err(CanonicalModelContractError::ImpossibleResponseCombination {
            reason: INVALID_REFUSAL_COMBINATION,
        });
    }
    if malformed_count > 0 && success_count > 0 {
        return Err(CanonicalModelContractError::ImpossibleResponseCombination {
            reason: INVALID_MALFORMED_COMBINATION,
        });
    }
    Ok(())
}

pub fn require_collection(
    field: crate::validation::FieldName,
    actual: usize,
    max_items: usize,
) -> Result<(), CanonicalModelContractError> {
    if actual == 0 {
        return Err(CanonicalModelContractError::EmptyCollection {
            field: field.as_str(),
        });
    }
    validate_collection_len(field, actual, max_items)
}

pub fn validate_collection_len(
    field: crate::validation::FieldName,
    actual: usize,
    max_items: usize,
) -> Result<(), CanonicalModelContractError> {
    if actual > max_items {
        return Err(CanonicalModelContractError::CollectionTooLarge {
            field: field.as_str(),
            max_items,
            actual_items: actual,
        });
    }
    Ok(())
}

fn safe_text(
    field: crate::validation::FieldName,
    raw: &str,
) -> Result<alloc::string::String, CanonicalModelContractError> {
    let normalized = crate::validation::normalized_adapter_field(field, raw)?;
    if contains_leak_marker(normalized.as_str(), LEAK_MARKER_RAW_PROVIDER_PAYLOAD)
        || contains_leak_marker(normalized.as_str(), LEAK_MARKER_PROVIDER_PAYLOAD_PREFIX)
    {
        return Err(CanonicalModelContractError::UnsafeText {
            field: field.as_str(),
            reason: CanonicalUnsafeTextReason::RawProviderPayload,
        });
    }
    if contains_leak_marker(normalized.as_str(), LEAK_MARKER_CREDENTIAL_PREFIX)
        || contains_leak_marker(normalized.as_str(), LEAK_MARKER_SECRET_PREFIX)
        || contains_leak_marker(normalized.as_str(), LEAK_MARKER_AUTHORIZATION_PREFIX)
        || contains_leak_marker(normalized.as_str(), LEAK_MARKER_BEARER_PREFIX)
    {
        return Err(CanonicalModelContractError::UnsafeText {
            field: field.as_str(),
            reason: CanonicalUnsafeTextReason::Credential,
        });
    }
    if contains_leak_marker(normalized.as_str(), LEAK_MARKER_HIDDEN_REASONING) {
        return Err(CanonicalModelContractError::UnsafeText {
            field: field.as_str(),
            reason: CanonicalUnsafeTextReason::HiddenReasoning,
        });
    }
    Ok(normalized)
}

fn contains_leak_marker(value: &str, marker: &str) -> bool {
    assert!(!marker.is_empty(), "leak marker must not be empty");
    value
        .as_bytes()
        .windows(marker.len())
        .any(|window| ascii_case_eq(window, marker.as_bytes()))
}

fn ascii_case_eq(left: &[u8], right: &[u8]) -> bool {
    assert!(left.len() == right.len(), "slices must be the same length");
    left.iter()
        .zip(right.iter())
        .all(|(left_byte, right_byte)| left_byte.eq_ignore_ascii_case(right_byte))
}

pub fn lossy_legacy_request() -> CanonicalModelContractError {
    CanonicalModelContractError::LossyLegacyProjection {
        reason: LOSSY_LEGACY_REQUEST,
    }
}

pub fn lossy_legacy_response() -> CanonicalModelContractError {
    CanonicalModelContractError::LossyLegacyProjection {
        reason: LOSSY_LEGACY_RESPONSE,
    }
}

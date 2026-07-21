#![allow(non_trait_imports)]

use super::{
    EVIDENCE_FIELD, FEATURE_CONTENT_PART, FEATURE_FIELD, FEATURE_METADATA, FEATURE_REASONING,
    FEATURE_ROLE, FEATURE_STREAMING, FEATURE_STRUCTURED, FEATURE_TOOL, LITERAL_FAMILY_PREFIX,
    LITERAL_IMAGE_PREFIX, LITERAL_KEY_VALUE_SEPARATOR, LITERAL_METADATA_PREFIX,
    LITERAL_REASONING_PREFIX, LITERAL_RESPONSE_PREFIX, LITERAL_ROLE_PREFIX, LITERAL_SEPARATOR,
    LITERAL_STREAM_PREFIX, LITERAL_TEXT_PREFIX, LITERAL_TOOL_PREFIX, ProviderProjectionError,
    ProviderProjectionFamily, ProviderProjectionIndexEntry, ProviderProjectionIndexEntryDraft,
    ProviderProjectionIndexStatus, ProviderProjectionSupport, ProviderRequestProjection,
    ProviderResponseFixtureDraft, REASON_FIELD,
};
use crate::model::{
    CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY, CanonicalModelRequest,
    CanonicalModelResponseEnvelope, CanonicalModelResponseEnvelopeDraft,
    CanonicalModelResponseItemDraft, CanonicalModelRole, CanonicalReasoningEffort,
    CanonicalResponseFormatIntent, CanonicalSafeMetadata,
};
/// # Errors
///
/// Returns [`ProviderProjectionError`] when the selected provider support would lose canonical semantics.
pub fn project_provider_request_fixture(
    request: &CanonicalModelRequest,
    support: ProviderProjectionSupport,
) -> Result<ProviderRequestProjection, ProviderProjectionError> {
    reject_unsupported_request_features(request, support)?;
    let mut literal_parts = alloc::vec![literal(LITERAL_FAMILY_PREFIX, support.family.as_str())];
    for message in request.messages() {
        literal_parts.push(literal(LITERAL_ROLE_PREFIX, message.role().as_str()));
        for part in message.content() {
            if let Some(text) = part.text() {
                literal_parts.push(literal(LITERAL_TEXT_PREFIX, text));
            } else {
                literal_parts.push(literal(LITERAL_IMAGE_PREFIX, "descriptor"));
            }
        }
    }
    for tool in request.tools() {
        literal_parts.push(literal(LITERAL_TOOL_PREFIX, tool.name()));
    }
    push_response_format(request.response_format(), &mut literal_parts);
    if request.reasoning().enabled() {
        literal_parts.push(literal(
            LITERAL_REASONING_PREFIX,
            reasoning_label(request.reasoning().effort()),
        ));
    }
    if request.output().stream() {
        literal_parts.push(literal(LITERAL_STREAM_PREFIX, "true"));
    }
    for metadata in request.provenance() {
        literal_parts.push(metadata_literal(metadata));
    }
    Ok(ProviderRequestProjection {
        family: support.family,
        literal_parts,
    })
}

/// Compares a provider request projection against reviewed literal fixture data.
///
/// # Errors
///
/// Returns [`ProviderProjectionError`] when projection is lossy or differs from expected literal data.
pub fn assert_provider_request_fixture(
    request: &CanonicalModelRequest,
    support: ProviderProjectionSupport,
    expected: &[&str],
) -> Result<ProviderRequestProjection, ProviderProjectionError> {
    let projection = project_provider_request_fixture(request, support)?;
    let expected: alloc::vec::Vec<alloc::string::String> = expected
        .iter()
        .map(|part| alloc::string::String::from(*part))
        .collect();
    if projection.literal_parts() == expected.as_slice() {
        return Ok(projection);
    }
    Err(ProviderProjectionError::LiteralMismatch {
        expected,
        actual: projection.literal_parts().to_vec(),
    })
}

/// Normalizes a provider response fixture into the canonical response envelope.
///
/// # Errors
///
/// Returns [`ProviderProjectionError`] when response semantics are malformed, leaky, or impossible.
pub fn normalize_provider_response_fixture(
    draft: ProviderResponseFixtureDraft<'_>,
) -> Result<CanonicalModelResponseEnvelope, ProviderProjectionError> {
    let _family = ProviderProjectionFamily::parse(draft.family)?;
    let mut items = alloc::vec::Vec::new();
    if let Some(label) = draft.raw_provider_payload {
        items.push(CanonicalModelResponseItemDraft::RawProviderPayload { label });
    }
    if let Some(label) = draft.hidden_reasoning {
        items.push(CanonicalModelResponseItemDraft::HiddenReasoning { label });
    }
    if let Some(text) = draft.text {
        items.push(CanonicalModelResponseItemDraft::AssistantText { text });
    }
    if let Some(payload_label) = draft.structured_payload {
        items.push(CanonicalModelResponseItemDraft::StructuredPayload {
            schema_label: "provider.schema",
            payload_label,
        });
    }
    if let (Some(call_id), Some(tool_name), Some(arguments_label)) =
        (draft.tool_call_id, draft.tool_name, draft.tool_arguments)
    {
        items.push(CanonicalModelResponseItemDraft::ToolCall {
            call_id,
            tool_name,
            arguments_label,
        });
    }
    if let Some(reason) = draft.refusal {
        items.push(CanonicalModelResponseItemDraft::Refusal { reason });
    }
    if let Some(category) = draft.safety {
        items.push(CanonicalModelResponseItemDraft::Safety { category });
    }
    if let Some(diagnostic) = draft.malformed_output {
        items.push(CanonicalModelResponseItemDraft::MalformedOutput { diagnostic });
    }
    CanonicalModelResponseEnvelope::from_draft(CanonicalModelResponseEnvelopeDraft {
        contract_identity: CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY,
        items: items.as_slice(),
        usage: draft.usage,
        finish_reason: draft.finish_reason,
        diagnostics: &[],
        provenance: &[],
    })
    .map_err(ProviderProjectionError::from)
}

/// Validates provider streaming fixture chunks through the production model-stream parser.
///
/// # Errors
///
/// Returns [`ProviderProjectionError`] when the stream fixture is malformed or frame kinds differ.
pub fn assert_provider_stream_fixture(
    chunks: &[&str],
    expected_kinds: &[crate::model::wire::parts::Kind],
) -> Result<alloc::vec::Vec<crate::model::wire::types::Frame>, ProviderProjectionError> {
    let frames = crate::model::wire::fixture::parse(chunks)?;
    let actual_kinds: alloc::vec::Vec<_> = frames
        .iter()
        .map(crate::model::wire::types::Frame::kind)
        .collect();
    if actual_kinds.as_slice() == expected_kinds {
        Ok(frames)
    } else {
        Err(ProviderProjectionError::LossyProjection {
            feature: FEATURE_STREAMING,
        })
    }
}

/// Validates a provider projection conformance index.
///
/// # Errors
///
/// Returns [`ProviderProjectionError`] when covered cases lack evidence or non-covered cases lack reasons.
pub fn validate_provider_projection_index(
    drafts: &[ProviderProjectionIndexEntryDraft<'_>],
) -> Result<alloc::vec::Vec<ProviderProjectionIndexEntry>, ProviderProjectionError> {
    drafts.iter().map(validate_index_entry).collect()
}

fn validate_index_entry(
    draft: &ProviderProjectionIndexEntryDraft<'_>,
) -> Result<ProviderProjectionIndexEntry, ProviderProjectionError> {
    let family = ProviderProjectionFamily::parse(draft.family)?;
    let feature = crate::validation::normalized_adapter_field(FEATURE_FIELD, draft.feature)?;
    let evidence = draft
        .evidence
        .map(|value| crate::validation::normalized_adapter_field(EVIDENCE_FIELD, value))
        .transpose()?;
    let reason = draft
        .reason
        .map(|value| crate::validation::normalized_adapter_field(REASON_FIELD, value))
        .transpose()?;
    match draft.status {
        ProviderProjectionIndexStatus::Covered if evidence.is_none() => {
            Err(ProviderProjectionError::MissingIndexField {
                field: EVIDENCE_FIELD.as_str(),
            })
        }
        ProviderProjectionIndexStatus::Unsupported
        | ProviderProjectionIndexStatus::NotApplicable
            if reason.is_none() =>
        {
            Err(ProviderProjectionError::MissingIndexField {
                field: REASON_FIELD.as_str(),
            })
        }
        _valid => Ok(ProviderProjectionIndexEntry {
            family,
            feature,
            status: draft.status,
            evidence,
            reason,
        }),
    }
}

fn reject_unsupported_request_features(
    request: &CanonicalModelRequest,
    support: ProviderProjectionSupport,
) -> Result<(), ProviderProjectionError> {
    for message in request.messages() {
        if message.role() == CanonicalModelRole::Developer && !support.developer_role.is_preserved()
        {
            return Err(lossy(FEATURE_ROLE));
        }
        for part in message.content() {
            if !part.is_text() && !support.images.is_preserved() {
                return Err(lossy(FEATURE_CONTENT_PART));
            }
        }
    }
    if !request.tools().is_empty() && !support.tools.is_preserved() {
        return Err(lossy(FEATURE_TOOL));
    }
    match request.response_format() {
        CanonicalResponseFormatIntent::Text => {}
        CanonicalResponseFormatIntent::JsonObject => {
            if !support.json_object.is_preserved() {
                return Err(lossy(FEATURE_STRUCTURED));
            }
        }
        CanonicalResponseFormatIntent::JsonSchema { .. } => {
            if !support.json_schema.is_preserved() {
                return Err(lossy(FEATURE_STRUCTURED));
            }
        }
    }
    if request.reasoning().enabled() && !support.reasoning.is_preserved() {
        return Err(lossy(FEATURE_REASONING));
    }
    if request.output().stream() && !support.streaming.is_preserved() {
        return Err(lossy(FEATURE_STREAMING));
    }
    if !request.provenance().is_empty() && !support.metadata.is_preserved() {
        return Err(lossy(FEATURE_METADATA));
    }
    Ok(())
}

fn push_response_format(
    format: &CanonicalResponseFormatIntent,
    literal_parts: &mut alloc::vec::Vec<alloc::string::String>,
) {
    match format {
        CanonicalResponseFormatIntent::Text => {
            literal_parts.push(literal(LITERAL_RESPONSE_PREFIX, "text"));
        }
        CanonicalResponseFormatIntent::JsonObject => {
            literal_parts.push(literal(LITERAL_RESPONSE_PREFIX, "json_object"));
        }
        CanonicalResponseFormatIntent::JsonSchema { schema_label } => {
            literal_parts.push(literal(LITERAL_RESPONSE_PREFIX, schema_label));
        }
    }
}

fn reasoning_label(effort: CanonicalReasoningEffort) -> &'static str {
    match effort {
        CanonicalReasoningEffort::None => "none",
        CanonicalReasoningEffort::Low => "low",
        CanonicalReasoningEffort::Medium => "medium",
        CanonicalReasoningEffort::High => "high",
    }
}

fn literal(prefix: &str, value: &str) -> alloc::string::String {
    let mut output = alloc::string::String::from(prefix);
    output.push_str(LITERAL_SEPARATOR);
    output.push_str(value);
    output
}

fn metadata_literal(metadata: &CanonicalSafeMetadata) -> alloc::string::String {
    let mut value = alloc::string::String::from(metadata.key());
    value.push_str(LITERAL_KEY_VALUE_SEPARATOR);
    value.push_str(metadata.value());
    literal(LITERAL_METADATA_PREFIX, value.as_str())
}

const fn lossy(feature: &'static str) -> ProviderProjectionError {
    ProviderProjectionError::LossyProjection { feature }
}

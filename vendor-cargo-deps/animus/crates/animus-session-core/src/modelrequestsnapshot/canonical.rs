use alloc::string::{String, ToString};
use alloc::vec::Vec;
use blake3::Hasher;

use super::data::{
    MAX_PRIVATE_PAYLOAD_BYTES, MODEL_REQUEST_BUDGET_IDENTITY_DOMAIN,
    MODEL_REQUEST_CANONICAL_IDENTITY_DOMAIN, MODEL_REQUEST_OUTPUT_IDENTITY_DOMAIN,
    MODEL_REQUEST_PRIVATE_PAYLOAD_DOMAIN, MODEL_REQUEST_ROUTE_IDENTITY_DOMAIN,
    MODEL_REQUEST_TERMINAL_PRIVATE_PAYLOAD_DOMAIN, MODEL_REQUEST_TOOL_SET_IDENTITY_DOMAIN,
    ModelRequestProviderTerminalObservation, ModelRequestProviderTerminalStatus,
    ModelRequestSnapshotDraft, ModelRequestSnapshotError, ModelRequestSnapshotIssue,
};

const FRAME_SEPARATOR: u8 = 0;
const OPTIONAL_ABSENT_TAG: u8 = 0;
const OPTIONAL_PRESENT_TAG: u8 = 1;
const CONTENT_TEXT_TAG: &str = "text";
const CONTENT_IMAGE_TAG: &str = "image_descriptor";
const RESPONSE_TEXT_TAG: &str = "text";
const RESPONSE_JSON_OBJECT_TAG: &str = "json_object";
const RESPONSE_JSON_SCHEMA_TAG: &str = "json_schema";
const REASONING_NONE_TAG: &str = "none";
const REASONING_LOW_TAG: &str = "low";
const REASONING_MEDIUM_TAG: &str = "medium";
const REASONING_HIGH_TAG: &str = "high";
const U64_BYTES: usize = core::mem::size_of::<u64>();
const U32_BYTES: usize = core::mem::size_of::<u32>();
const U16_BYTES: usize = core::mem::size_of::<u16>();

/// Computes the canonical BLAKE3 identity for an exact provider-neutral request.
///
/// # Errors
///
/// Returns a bound issue when the request cannot be framed within the private payload bound.
pub fn model_request_canonical_identity(
    request: &animus_adapters::CanonicalModelRequest,
) -> Result<String, ModelRequestSnapshotError> {
    let payload = canonical_request_payload(request)?;
    Ok(domain_hash(
        MODEL_REQUEST_CANONICAL_IDENTITY_DOMAIN,
        payload.as_slice(),
    ))
}

/// Computes the selected provider, transport, model, and dialect route identity.
#[must_use]
pub fn model_request_route_identity(
    provider_id: &str,
    provider_transport_id: &str,
    model_id: &str,
    model_dialect_id: &str,
) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, MODEL_REQUEST_ROUTE_IDENTITY_DOMAIN);
    update_str(&mut hasher, provider_id);
    update_str(&mut hasher, provider_transport_id);
    update_str(&mut hasher, model_id);
    update_str(&mut hasher, model_dialect_id);
    hasher.finalize().to_hex().to_string()
}

/// Computes the canonical ordered tool-schema-set identity.
#[must_use]
pub fn model_request_tool_schema_set_identity(
    request: &animus_adapters::CanonicalModelRequest,
) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, MODEL_REQUEST_TOOL_SET_IDENTITY_DOMAIN);
    update_usize(&mut hasher, request.tools.len());
    for tool in &request.tools {
        update_str(&mut hasher, tool.name());
        update_str(&mut hasher, tool.description());
        update_str(&mut hasher, tool.input_schema_label());
    }
    hasher.finalize().to_hex().to_string()
}

/// Computes the output, response-format, and reasoning-control identity.
#[must_use]
pub fn model_request_output_control_identity(
    request: &animus_adapters::CanonicalModelRequest,
) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, MODEL_REQUEST_OUTPUT_IDENTITY_DOMAIN);
    update_optional_u32(&mut hasher, request.output.max_output_tokens());
    update_optional_u16(&mut hasher, request.output.temperature_millis());
    update_usize(&mut hasher, request.output.stop_sequences().len());
    for stop in request.output.stop_sequences() {
        update_str(&mut hasher, stop);
    }
    update_bool(&mut hasher, request.output.stream());
    match &request.response_format {
        animus_adapters::CanonicalModelResponseFormatIntent::Text => {
            update_str(&mut hasher, RESPONSE_TEXT_TAG);
        }
        animus_adapters::CanonicalModelResponseFormatIntent::JsonObject => {
            update_str(&mut hasher, RESPONSE_JSON_OBJECT_TAG);
        }
        animus_adapters::CanonicalModelResponseFormatIntent::JsonSchema { schema_label } => {
            update_str(&mut hasher, RESPONSE_JSON_SCHEMA_TAG);
            update_str(&mut hasher, schema_label);
        }
    }
    update_bool(&mut hasher, request.reasoning.enabled());
    update_str(
        &mut hasher,
        reasoning_effort_label(request.reasoning.effort()),
    );
    update_optional_str(&mut hasher, request.reasoning.summary_label.as_deref());
    hasher.finalize().to_hex().to_string()
}

/// Computes the provider-neutral request budget identity.
#[must_use]
pub fn model_request_budget_identity(request: &animus_adapters::CanonicalModelRequest) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, MODEL_REQUEST_BUDGET_IDENTITY_DOMAIN);
    update_optional_u32(&mut hasher, request.output.max_output_tokens());
    update_optional_u16(&mut hasher, request.output.temperature_millis());
    update_usize(&mut hasher, request.output.stop_sequences().len());
    hasher.finalize().to_hex().to_string()
}

pub(super) fn private_snapshot_payload(
    draft: &ModelRequestSnapshotDraft<'_>,
    canonical_request_identity: &str,
) -> Result<Vec<u8>, ModelRequestSnapshotError> {
    let mut payload = Vec::new();
    push_str(&mut payload, MODEL_REQUEST_PRIVATE_PAYLOAD_DOMAIN)?;
    push_str(&mut payload, draft.schema)?;
    push_str(&mut payload, draft.session_id)?;
    push_str(&mut payload, draft.branch_id)?;
    push_str(&mut payload, draft.active_leaf_id)?;
    push_u64(&mut payload, draft.turn_generation)?;
    push_u64(&mut payload, draft.request_sequence)?;
    push_str(&mut payload, canonical_request_identity)?;
    push_str(&mut payload, draft.route.provider_id)?;
    push_str(&mut payload, draft.route.provider_transport_id)?;
    push_str(&mut payload, draft.route.model_id)?;
    push_str(&mut payload, draft.route.model_dialect_id)?;
    push_str(&mut payload, draft.route.route_identity)?;
    push_str(&mut payload, draft.model_invocation_identity)?;
    push_str(&mut payload, draft.request_context_plan_identity)?;
    push_usize(
        &mut payload,
        draft.selected_context_contribution_identities.len(),
    )?;
    for identity in draft.selected_context_contribution_identities {
        push_str(&mut payload, identity)?;
    }
    push_str(&mut payload, draft.tool_schema_set_identity)?;
    push_str(&mut payload, draft.output_control_identity)?;
    push_str(&mut payload, draft.budget_identity)?;
    push_str(&mut payload, draft.authority_seal_identity)?;
    push_str(&mut payload, draft.adapter_cohort_identity)?;
    let canonical = canonical_request_payload(draft.canonical_request)?;
    push_bytes(&mut payload, canonical.as_slice())?;
    if payload.len() > MAX_PRIVATE_PAYLOAD_BYTES {
        return Err(error(
            ModelRequestSnapshotIssue::BoundExceeded,
            "private_payload",
        ));
    }
    Ok(payload)
}

pub(super) fn canonical_request_payload(
    request: &animus_adapters::CanonicalModelRequest,
) -> Result<Vec<u8>, ModelRequestSnapshotError> {
    let mut payload = Vec::new();
    push_str(
        &mut payload,
        animus_adapters::CanonicalModelRequest::contract_identity(),
    )?;
    push_usize(&mut payload, request.messages.len())?;
    for message in &request.messages {
        push_str(&mut payload, message.role().as_str())?;
        push_usize(&mut payload, message.content().len())?;
        for part in message.content() {
            match part {
                animus_adapters::CanonicalMessageContentPart::Text { text } => {
                    push_str(&mut payload, CONTENT_TEXT_TAG)?;
                    push_str(&mut payload, text)?;
                }
                animus_adapters::CanonicalMessageContentPart::ImageDescriptor {
                    descriptor_id,
                    media_type,
                    content_hash,
                } => {
                    push_str(&mut payload, CONTENT_IMAGE_TAG)?;
                    push_str(&mut payload, descriptor_id)?;
                    push_str(&mut payload, media_type)?;
                    push_str(&mut payload, content_hash)?;
                }
            }
        }
        push_optional_str(&mut payload, message.tool_call_id())?;
    }
    push_usize(&mut payload, request.tools.len())?;
    for tool in &request.tools {
        push_str(&mut payload, tool.name())?;
        push_str(&mut payload, tool.description())?;
        push_str(&mut payload, tool.input_schema_label())?;
    }
    push_optional_u32(&mut payload, request.output.max_output_tokens())?;
    push_optional_u16(&mut payload, request.output.temperature_millis())?;
    push_usize(&mut payload, request.output.stop_sequences().len())?;
    for stop in request.output.stop_sequences() {
        push_str(&mut payload, stop)?;
    }
    push_bool(&mut payload, request.output.stream())?;
    match &request.response_format {
        animus_adapters::CanonicalModelResponseFormatIntent::Text => {
            push_str(&mut payload, RESPONSE_TEXT_TAG)?;
        }
        animus_adapters::CanonicalModelResponseFormatIntent::JsonObject => {
            push_str(&mut payload, RESPONSE_JSON_OBJECT_TAG)?;
        }
        animus_adapters::CanonicalModelResponseFormatIntent::JsonSchema { schema_label } => {
            push_str(&mut payload, RESPONSE_JSON_SCHEMA_TAG)?;
            push_str(&mut payload, schema_label)?;
        }
    }
    push_bool(&mut payload, request.reasoning.enabled())?;
    push_str(
        &mut payload,
        reasoning_effort_label(request.reasoning.effort()),
    )?;
    push_optional_str(&mut payload, request.reasoning.summary_label.as_deref())?;
    push_usize(&mut payload, request.provenance.len())?;
    for metadata in &request.provenance {
        push_str(&mut payload, metadata.key())?;
        push_str(&mut payload, metadata.value())?;
    }
    if payload.len() > MAX_PRIVATE_PAYLOAD_BYTES {
        return Err(error(
            ModelRequestSnapshotIssue::BoundExceeded,
            "canonical_request",
        ));
    }
    Ok(payload)
}

/// Encodes one validated metadata-only terminal observation for private durability.
///
/// # Errors
///
/// Returns a typed issue when the observation is stale or exceeds the private bound.
pub fn model_request_provider_terminal_private_payload(
    observation: &ModelRequestProviderTerminalObservation,
) -> Result<Vec<u8>, ModelRequestSnapshotError> {
    super::core::validate_model_request_provider_terminal_observation(observation)?;
    let mut payload = Vec::new();
    push_str(&mut payload, MODEL_REQUEST_TERMINAL_PRIVATE_PAYLOAD_DOMAIN)?;
    push_str(&mut payload, observation.snapshot_identity.as_str())?;
    push_str(&mut payload, observation.dispatch_plan_identity.as_str())?;
    push_str(&mut payload, observation.adapter_cohort_identity.as_str())?;
    push_u32(&mut payload, observation.attempt_number)?;
    push_str(&mut payload, observation.status.as_str())?;
    push_str(&mut payload, observation.observation_identity.as_str())?;
    Ok(payload)
}

/// Decodes and revalidates one private terminal provider observation payload.
///
/// # Errors
///
/// Returns a typed issue for malformed framing, an unsupported status, or identity drift.
pub fn decode_model_request_provider_terminal_private_payload(
    payload: &[u8],
) -> Result<ModelRequestProviderTerminalObservation, ModelRequestSnapshotError> {
    if payload.len() > MAX_PRIVATE_PAYLOAD_BYTES {
        return Err(error(
            ModelRequestSnapshotIssue::BoundExceeded,
            "provider_terminal",
        ));
    }
    let mut cursor = PayloadCursor::new(payload);
    cursor.expect_str(MODEL_REQUEST_TERMINAL_PRIVATE_PAYLOAD_DOMAIN)?;
    let snapshot_identity = cursor.read_string()?;
    let dispatch_plan_identity = cursor.read_string()?;
    let adapter_cohort_identity = cursor.read_string()?;
    let attempt_number = cursor.read_u32()?;
    let status = match cursor.read_string()?.as_str() {
        "succeeded" => ModelRequestProviderTerminalStatus::Succeeded,
        "failed" => ModelRequestProviderTerminalStatus::Failed,
        "cancelled" => ModelRequestProviderTerminalStatus::Cancelled,
        _unsupported => {
            return Err(error(
                ModelRequestSnapshotIssue::InvalidSchema,
                "provider_terminal",
            ));
        }
    };
    let observation = ModelRequestProviderTerminalObservation {
        snapshot_identity,
        dispatch_plan_identity,
        adapter_cohort_identity,
        attempt_number,
        status,
        observation_identity: cursor.read_string()?,
    };
    cursor.finish()?;
    super::core::validate_model_request_provider_terminal_observation(&observation)?;
    Ok(observation)
}

/// Decodes and revalidates one exact private provider-neutral snapshot payload.
///
/// # Errors
///
/// Returns a typed issue for malformed framing, unsupported tags, bound excess,
/// identity drift, or any request value that no longer passes canonical validation.
pub fn decode_model_request_snapshot_private_payload(
    payload: &[u8],
) -> Result<super::data::ModelRequestSnapshot, ModelRequestSnapshotError> {
    if payload.len() > MAX_PRIVATE_PAYLOAD_BYTES {
        return Err(error(
            ModelRequestSnapshotIssue::BoundExceeded,
            "private_payload",
        ));
    }
    let mut cursor = PayloadCursor::new(payload);
    cursor.expect_str(MODEL_REQUEST_PRIVATE_PAYLOAD_DOMAIN)?;
    let schema = cursor.read_string()?;
    let session_id = cursor.read_string()?;
    let branch_id = cursor.read_string()?;
    let active_leaf_id = cursor.read_string()?;
    let turn_generation = cursor.read_u64()?;
    let request_sequence = cursor.read_u64()?;
    let encoded_canonical_identity = cursor.read_string()?;
    let provider_id = cursor.read_string()?;
    let provider_transport_id = cursor.read_string()?;
    let model_id = cursor.read_string()?;
    let model_dialect_id = cursor.read_string()?;
    let route_identity = cursor.read_string()?;
    let model_invocation_identity = cursor.read_string()?;
    let request_context_plan_identity = cursor.read_string()?;
    let contribution_count = cursor.read_count()?;
    let mut contributions = Vec::with_capacity(contribution_count);
    for _index in 0..contribution_count {
        contributions.push(cursor.read_string()?);
    }
    let tool_schema_set_identity = cursor.read_string()?;
    let output_control_identity = cursor.read_string()?;
    let budget_identity = cursor.read_string()?;
    let authority_seal_identity = cursor.read_string()?;
    let adapter_cohort_identity = cursor.read_string()?;
    let canonical_payload = cursor.read_bytes()?;
    cursor.finish()?;

    let canonical_request = decode_canonical_request(canonical_payload)?;
    let contribution_refs = contributions.iter().map(String::as_str).collect::<Vec<_>>();
    let snapshot = super::core::validate_model_request_snapshot(&ModelRequestSnapshotDraft {
        schema: schema.as_str(),
        session_id: session_id.as_str(),
        branch_id: branch_id.as_str(),
        active_leaf_id: active_leaf_id.as_str(),
        turn_generation,
        request_sequence,
        canonical_request: &canonical_request,
        route: super::data::ModelRequestRouteDraft {
            provider_id: provider_id.as_str(),
            provider_transport_id: provider_transport_id.as_str(),
            model_id: model_id.as_str(),
            model_dialect_id: model_dialect_id.as_str(),
            route_identity: route_identity.as_str(),
        },
        model_invocation_identity: model_invocation_identity.as_str(),
        request_context_plan_identity: request_context_plan_identity.as_str(),
        selected_context_contribution_identities: contribution_refs.as_slice(),
        tool_schema_set_identity: tool_schema_set_identity.as_str(),
        output_control_identity: output_control_identity.as_str(),
        budget_identity: budget_identity.as_str(),
        authority_seal_identity: authority_seal_identity.as_str(),
        adapter_cohort_identity: adapter_cohort_identity.as_str(),
        unsupported: super::data::ModelRequestSnapshotUnsupportedDraft::default(),
    })?;
    if snapshot.canonical_request_identity() != encoded_canonical_identity
        || snapshot.private_payload() != payload
    {
        return Err(error(
            ModelRequestSnapshotIssue::IdentityMismatch,
            "private_payload",
        ));
    }
    Ok(snapshot)
}

#[derive(Debug)]
enum DecodedContentPart {
    Text(String),
    ImageDescriptor {
        descriptor_id: String,
        media_type: String,
        content_hash: String,
    },
}

#[derive(Debug)]
struct DecodedMessage {
    role: String,
    content: Vec<DecodedContentPart>,
    tool_call_id: Option<String>,
}

#[derive(Debug)]
struct DecodedTool {
    name: String,
    description: String,
    input_schema_label: String,
}

#[derive(Debug)]
enum DecodedResponseFormat {
    Text,
    JsonObject,
    JsonSchema(String),
}

#[derive(Debug)]
struct DecodedCanonicalRequest {
    messages: Vec<DecodedMessage>,
    tools: Vec<DecodedTool>,
    max_output_tokens: Option<u32>,
    temperature_millis: Option<u16>,
    stops: Vec<String>,
    stream: bool,
    response_format: DecodedResponseFormat,
    reasoning_enabled: bool,
    reasoning_effort: animus_adapters::CanonicalModelReasoningEffort,
    reasoning_summary: Option<String>,
    provenance: Vec<(String, String)>,
}

fn decode_canonical_request(
    payload: &[u8],
) -> Result<animus_adapters::CanonicalModelRequest, ModelRequestSnapshotError> {
    let mut cursor = PayloadCursor::new(payload);
    cursor.expect_str(animus_adapters::CanonicalModelRequest::contract_identity())?;
    let message_count = cursor.read_count()?;
    let mut messages = Vec::with_capacity(message_count);
    for _index in 0..message_count {
        let role = cursor.read_string()?;
        let content_count = cursor.read_count()?;
        let mut content = Vec::with_capacity(content_count);
        for _content_index in 0..content_count {
            let tag = cursor.read_string()?;
            match tag.as_str() {
                CONTENT_TEXT_TAG => content.push(DecodedContentPart::Text(cursor.read_string()?)),
                CONTENT_IMAGE_TAG => content.push(DecodedContentPart::ImageDescriptor {
                    descriptor_id: cursor.read_string()?,
                    media_type: cursor.read_string()?,
                    content_hash: cursor.read_string()?,
                }),
                _unsupported => {
                    return Err(error(
                        ModelRequestSnapshotIssue::InvalidSchema,
                        "canonical_content",
                    ));
                }
            }
        }
        messages.push(DecodedMessage {
            role,
            content,
            tool_call_id: cursor.read_optional_string()?,
        });
    }

    let tool_count = cursor.read_count()?;
    let mut tools = Vec::with_capacity(tool_count);
    for _index in 0..tool_count {
        tools.push(DecodedTool {
            name: cursor.read_string()?,
            description: cursor.read_string()?,
            input_schema_label: cursor.read_string()?,
        });
    }
    let max_output_tokens = cursor.read_optional_u32()?;
    let temperature_millis = cursor.read_optional_u16()?;
    let stop_count = cursor.read_count()?;
    let mut stops = Vec::with_capacity(stop_count);
    for _index in 0..stop_count {
        stops.push(cursor.read_string()?);
    }
    let stream = cursor.read_bool()?;
    let response_format = match cursor.read_string()?.as_str() {
        RESPONSE_TEXT_TAG => DecodedResponseFormat::Text,
        RESPONSE_JSON_OBJECT_TAG => DecodedResponseFormat::JsonObject,
        RESPONSE_JSON_SCHEMA_TAG => DecodedResponseFormat::JsonSchema(cursor.read_string()?),
        _unsupported => {
            return Err(error(
                ModelRequestSnapshotIssue::InvalidSchema,
                "response_format",
            ));
        }
    };
    let reasoning_enabled = cursor.read_bool()?;
    let reasoning_effort = match cursor.read_string()?.as_str() {
        REASONING_NONE_TAG => animus_adapters::CanonicalModelReasoningEffort::None,
        REASONING_LOW_TAG => animus_adapters::CanonicalModelReasoningEffort::Low,
        REASONING_MEDIUM_TAG => animus_adapters::CanonicalModelReasoningEffort::Medium,
        REASONING_HIGH_TAG => animus_adapters::CanonicalModelReasoningEffort::High,
        _unsupported => {
            return Err(error(
                ModelRequestSnapshotIssue::InvalidSchema,
                "reasoning_effort",
            ));
        }
    };
    let reasoning_summary = cursor.read_optional_string()?;
    let provenance_count = cursor.read_count()?;
    let mut provenance = Vec::with_capacity(provenance_count);
    for _index in 0..provenance_count {
        provenance.push((cursor.read_string()?, cursor.read_string()?));
    }
    cursor.finish()?;
    build_canonical_request(&DecodedCanonicalRequest {
        messages,
        tools,
        max_output_tokens,
        temperature_millis,
        stops,
        stream,
        response_format,
        reasoning_enabled,
        reasoning_effort,
        reasoning_summary,
        provenance,
    })
}

fn build_canonical_request(
    decoded: &DecodedCanonicalRequest,
) -> Result<animus_adapters::CanonicalModelRequest, ModelRequestSnapshotError> {
    let content_drafts = decoded
        .messages
        .iter()
        .map(|message| {
            message
                .content
                .iter()
                .map(|part| match part {
                    DecodedContentPart::Text(text) => {
                        animus_adapters::CanonicalMessageContentPartDraft::Text {
                            text: text.as_str(),
                        }
                    }
                    DecodedContentPart::ImageDescriptor {
                        descriptor_id,
                        media_type,
                        content_hash,
                    } => animus_adapters::CanonicalMessageContentPartDraft::ImageDescriptor {
                        descriptor_id: descriptor_id.as_str(),
                        media_type: media_type.as_str(),
                        content_hash: content_hash.as_str(),
                    },
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let message_drafts = decoded
        .messages
        .iter()
        .zip(content_drafts.iter())
        .map(
            |(message, content)| animus_adapters::CanonicalModelMessageDraft {
                role: message.role.as_str(),
                content: content.as_slice(),
                tool_call_id: message.tool_call_id.as_deref(),
            },
        )
        .collect::<Vec<_>>();
    let tool_drafts = decoded
        .tools
        .iter()
        .map(|tool| animus_adapters::CanonicalModelToolDeclarationDraft {
            name: tool.name.as_str(),
            description: tool.description.as_str(),
            input_schema_label: tool.input_schema_label.as_str(),
        })
        .collect::<Vec<_>>();
    let stop_refs = decoded.stops.iter().map(String::as_str).collect::<Vec<_>>();
    let response_format = match &decoded.response_format {
        DecodedResponseFormat::Text => {
            animus_adapters::CanonicalModelResponseFormatIntentDraft::Text
        }
        DecodedResponseFormat::JsonObject => {
            animus_adapters::CanonicalModelResponseFormatIntentDraft::JsonObject
        }
        DecodedResponseFormat::JsonSchema(schema_label) => {
            animus_adapters::CanonicalModelResponseFormatIntentDraft::JsonSchema {
                schema_label: schema_label.as_str(),
            }
        }
    };
    let provenance_drafts = decoded
        .provenance
        .iter()
        .map(
            |(key, value)| animus_adapters::CanonicalModelSafeMetadataDraft {
                key: key.as_str(),
                value: value.as_str(),
            },
        )
        .collect::<Vec<_>>();
    animus_adapters::CanonicalModelRequest::from_draft(
        animus_adapters::CanonicalModelRequestDraft {
            contract_identity: animus_adapters::CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
            messages: message_drafts.as_slice(),
            tools: tool_drafts.as_slice(),
            output: animus_adapters::CanonicalModelOutputControlsDraft {
                max_output_tokens: decoded.max_output_tokens,
                temperature_millis: decoded.temperature_millis,
                stop_sequences: stop_refs.as_slice(),
                stream: decoded.stream,
            },
            response_format,
            reasoning: animus_adapters::CanonicalModelReasoningRequestDraft {
                enabled: decoded.reasoning_enabled,
                effort: decoded.reasoning_effort,
                summary_label: decoded.reasoning_summary.as_deref(),
            },
            provenance: provenance_drafts.as_slice(),
        },
    )
    .map_err(|_error| {
        error(
            ModelRequestSnapshotIssue::InvalidSchema,
            "canonical_request",
        )
    })
}

struct PayloadCursor<'payload> {
    payload: &'payload [u8],
    offset: usize,
}

impl<'payload> PayloadCursor<'payload> {
    const fn new(payload: &'payload [u8]) -> Self {
        Self { payload, offset: 0 }
    }

    fn expect_str(&mut self, expected: &str) -> Result<(), ModelRequestSnapshotError> {
        if self.read_string()? == expected {
            Ok(())
        } else {
            Err(error(
                ModelRequestSnapshotIssue::InvalidSchema,
                "private_payload",
            ))
        }
    }

    fn read_string(&mut self) -> Result<String, ModelRequestSnapshotError> {
        let bytes = self.read_bytes()?;
        core::str::from_utf8(bytes)
            .map(String::from)
            .map_err(|_error| error(ModelRequestSnapshotIssue::InvalidSchema, "private_payload"))
    }

    fn read_bytes(&mut self) -> Result<&'payload [u8], ModelRequestSnapshotError> {
        let len = self.read_count_unbounded()?;
        self.take(len)
    }

    fn read_count(&mut self) -> Result<usize, ModelRequestSnapshotError> {
        let count = self.read_count_unbounded()?;
        if count > super::data::MAX_SNAPSHOT_IDENTITIES {
            return Err(error(
                ModelRequestSnapshotIssue::BoundExceeded,
                "private_payload",
            ));
        }
        Ok(count)
    }

    fn read_count_unbounded(&mut self) -> Result<usize, ModelRequestSnapshotError> {
        usize::try_from(self.read_u64()?)
            .map_err(|_error| error(ModelRequestSnapshotIssue::BoundExceeded, "private_payload"))
    }

    fn read_u64(&mut self) -> Result<u64, ModelRequestSnapshotError> {
        Ok(u64::from_le_bytes(self.read_array::<U64_BYTES>()?))
    }

    fn read_u32(&mut self) -> Result<u32, ModelRequestSnapshotError> {
        Ok(u32::from_le_bytes(self.read_array::<U32_BYTES>()?))
    }

    fn read_u16(&mut self) -> Result<u16, ModelRequestSnapshotError> {
        Ok(u16::from_le_bytes(self.read_array::<U16_BYTES>()?))
    }

    fn read_bool(&mut self) -> Result<bool, ModelRequestSnapshotError> {
        match self.take(1)? {
            [0] => Ok(false),
            [1] => Ok(true),
            _invalid => Err(error(
                ModelRequestSnapshotIssue::InvalidSchema,
                "private_payload",
            )),
        }
    }

    fn read_optional_string(&mut self) -> Result<Option<String>, ModelRequestSnapshotError> {
        if self.read_optional_tag()? {
            self.read_string().map(Some)
        } else {
            Ok(None)
        }
    }

    fn read_optional_u32(&mut self) -> Result<Option<u32>, ModelRequestSnapshotError> {
        if self.read_optional_tag()? {
            self.read_u32().map(Some)
        } else {
            Ok(None)
        }
    }

    fn read_optional_u16(&mut self) -> Result<Option<u16>, ModelRequestSnapshotError> {
        if self.read_optional_tag()? {
            self.read_u16().map(Some)
        } else {
            Ok(None)
        }
    }

    fn read_optional_tag(&mut self) -> Result<bool, ModelRequestSnapshotError> {
        match self.take(1)? {
            [OPTIONAL_ABSENT_TAG] => Ok(false),
            [OPTIONAL_PRESENT_TAG] => Ok(true),
            _invalid => Err(error(
                ModelRequestSnapshotIssue::InvalidSchema,
                "private_payload",
            )),
        }
    }

    fn read_array<const SIZE: usize>(&mut self) -> Result<[u8; SIZE], ModelRequestSnapshotError> {
        self.take(SIZE)?
            .try_into()
            .map_err(|_error| error(ModelRequestSnapshotIssue::InvalidSchema, "private_payload"))
    }

    fn take(&mut self, len: usize) -> Result<&'payload [u8], ModelRequestSnapshotError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| error(ModelRequestSnapshotIssue::BoundExceeded, "private_payload"))?;
        let bytes = self
            .payload
            .get(self.offset..end)
            .ok_or_else(|| error(ModelRequestSnapshotIssue::InvalidSchema, "private_payload"))?;
        self.offset = end;
        Ok(bytes)
    }

    fn finish(self) -> Result<(), ModelRequestSnapshotError> {
        if self.offset == self.payload.len() {
            Ok(())
        } else {
            Err(error(
                ModelRequestSnapshotIssue::InvalidSchema,
                "private_payload",
            ))
        }
    }
}

pub(super) fn domain_hash(domain: &str, payload: &[u8]) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, domain);
    hasher.update(payload);
    hasher.finalize().to_hex().to_string()
}

pub(super) fn update_str(hasher: &mut Hasher, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update(&[FRAME_SEPARATOR]);
}

pub(super) fn update_u64(hasher: &mut Hasher, value: u64) {
    hasher.update(&value.to_le_bytes());
    hasher.update(&[FRAME_SEPARATOR]);
}

pub(super) fn update_u32(hasher: &mut Hasher, value: u32) {
    hasher.update(&value.to_le_bytes());
    hasher.update(&[FRAME_SEPARATOR]);
}

fn update_usize(hasher: &mut Hasher, value: usize) {
    // Every supported Animus target has a usize width no greater than the
    // canonical u64 frame width.
    let canonical_value = value as u64;
    update_u64(hasher, canonical_value);
}

fn update_bool(hasher: &mut Hasher, value: bool) {
    hasher.update(&[u8::from(value)]);
    hasher.update(&[FRAME_SEPARATOR]);
}

fn update_optional_str(hasher: &mut Hasher, value: Option<&str>) {
    match value {
        Some(value) => {
            hasher.update(&[OPTIONAL_PRESENT_TAG]);
            update_str(hasher, value);
        }
        None => {
            hasher.update(&[OPTIONAL_ABSENT_TAG, FRAME_SEPARATOR]);
        }
    }
}

fn update_optional_u32(hasher: &mut Hasher, value: Option<u32>) {
    match value {
        Some(value) => {
            hasher.update(&[OPTIONAL_PRESENT_TAG]);
            update_u32(hasher, value);
        }
        None => {
            hasher.update(&[OPTIONAL_ABSENT_TAG, FRAME_SEPARATOR]);
        }
    }
}

fn update_optional_u16(hasher: &mut Hasher, value: Option<u16>) {
    match value {
        Some(value) => {
            hasher.update(&[OPTIONAL_PRESENT_TAG]);
            hasher.update(&value.to_le_bytes());
            hasher.update(&[FRAME_SEPARATOR]);
        }
        None => {
            hasher.update(&[OPTIONAL_ABSENT_TAG, FRAME_SEPARATOR]);
        }
    }
}

fn push_str(payload: &mut Vec<u8>, value: &str) -> Result<(), ModelRequestSnapshotError> {
    push_bytes(payload, value.as_bytes())
}

fn push_bytes(payload: &mut Vec<u8>, value: &[u8]) -> Result<(), ModelRequestSnapshotError> {
    push_usize(payload, value.len())?;
    extend_payload(payload, value)
}

fn push_usize(payload: &mut Vec<u8>, value: usize) -> Result<(), ModelRequestSnapshotError> {
    let value = u64::try_from(value)
        .map_err(|_error| error(ModelRequestSnapshotIssue::BoundExceeded, "private_payload"))?;
    push_u64(payload, value)
}

fn push_u64(payload: &mut Vec<u8>, value: u64) -> Result<(), ModelRequestSnapshotError> {
    extend_payload(payload, value.to_le_bytes().as_slice())
}

fn push_u32(payload: &mut Vec<u8>, value: u32) -> Result<(), ModelRequestSnapshotError> {
    extend_payload(payload, value.to_le_bytes().as_slice())
}

fn push_u16(payload: &mut Vec<u8>, value: u16) -> Result<(), ModelRequestSnapshotError> {
    extend_payload(payload, value.to_le_bytes().as_slice())
}

fn push_bool(payload: &mut Vec<u8>, value: bool) -> Result<(), ModelRequestSnapshotError> {
    extend_payload(payload, &[u8::from(value)])
}

fn push_optional_str(
    payload: &mut Vec<u8>,
    value: Option<&str>,
) -> Result<(), ModelRequestSnapshotError> {
    match value {
        Some(value) => {
            extend_payload(payload, &[OPTIONAL_PRESENT_TAG])?;
            push_str(payload, value)
        }
        None => extend_payload(payload, &[OPTIONAL_ABSENT_TAG]),
    }
}

fn push_optional_u32(
    payload: &mut Vec<u8>,
    value: Option<u32>,
) -> Result<(), ModelRequestSnapshotError> {
    match value {
        Some(value) => {
            extend_payload(payload, &[OPTIONAL_PRESENT_TAG])?;
            push_u32(payload, value)
        }
        None => extend_payload(payload, &[OPTIONAL_ABSENT_TAG]),
    }
}

fn push_optional_u16(
    payload: &mut Vec<u8>,
    value: Option<u16>,
) -> Result<(), ModelRequestSnapshotError> {
    match value {
        Some(value) => {
            extend_payload(payload, &[OPTIONAL_PRESENT_TAG])?;
            push_u16(payload, value)
        }
        None => extend_payload(payload, &[OPTIONAL_ABSENT_TAG]),
    }
}

fn extend_payload(payload: &mut Vec<u8>, value: &[u8]) -> Result<(), ModelRequestSnapshotError> {
    let next_len = payload
        .len()
        .checked_add(value.len())
        .ok_or_else(|| error(ModelRequestSnapshotIssue::BoundExceeded, "private_payload"))?;
    if next_len > MAX_PRIVATE_PAYLOAD_BYTES {
        return Err(error(
            ModelRequestSnapshotIssue::BoundExceeded,
            "private_payload",
        ));
    }
    payload.extend_from_slice(value);
    Ok(())
}

const fn reasoning_effort_label(
    effort: animus_adapters::CanonicalModelReasoningEffort,
) -> &'static str {
    match effort {
        animus_adapters::CanonicalModelReasoningEffort::None => REASONING_NONE_TAG,
        animus_adapters::CanonicalModelReasoningEffort::Low => REASONING_LOW_TAG,
        animus_adapters::CanonicalModelReasoningEffort::Medium => REASONING_MEDIUM_TAG,
        animus_adapters::CanonicalModelReasoningEffort::High => REASONING_HIGH_TAG,
    }
}

const fn error(issue: ModelRequestSnapshotIssue, field: &'static str) -> ModelRequestSnapshotError {
    ModelRequestSnapshotError::new(issue, field)
}

//! Canonical model message contracts.
#![allow(
    unknown_lints,
    path_segment_repetition,
    excessive_file_length,
    assertion_density,
    ambiguous_params,
    compound_condition,
    non_trait_imports
)]
#[cfg(test)]
mod tests;
const CONTRACT_IDENTITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_contract_identity");
const ROLE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("model_role");
const TOOL_CALL_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_tool_call_id");
const CONTENT_PART_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_content_part_kind");
const CONTENT_TEXT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_content_text");
const IMAGE_DESCRIPTOR_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_image_descriptor_id");
const IMAGE_MEDIA_TYPE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_image_media_type");
const IMAGE_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_image_content_hash");
const TOOL_NAME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_tool_name");
const TOOL_DESCRIPTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_tool_description");
const TOOL_SCHEMA_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_tool_schema_label");
const STOP_SEQUENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_stop_sequence");
const RESPONSE_SCHEMA_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_response_schema_label");
const REASONING_SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_reasoning_summary");
const METADATA_KEY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_metadata_key");
const METADATA_VALUE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_metadata_value");
const RESPONSE_TEXT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_response_text");
const STRUCTURED_PAYLOAD_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_structured_payload_label");
const TOOL_ARGUMENTS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_tool_arguments_label");
const REFUSAL_REASON_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_refusal_reason");
const SAFETY_CATEGORY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_safety_category");
const MALFORMED_DIAGNOSTIC_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_malformed_diagnostic");
const FINISH_REASON_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_finish_reason");
const RAW_PROVIDER_PAYLOAD_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_raw_provider_payload");
const HIDDEN_REASONING_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_hidden_reasoning");
const MAX_MODEL_MESSAGES: usize = 128;
const MAX_CONTENT_PARTS_PER_MESSAGE: usize = 64;
const MAX_MODEL_TOOLS: usize = 32;
const MAX_STOP_SEQUENCES: usize = 16;
const MAX_SAFE_METADATA_ITEMS: usize = 32;
const MAX_RESPONSE_ITEMS: usize = 64;
const MAX_RESPONSE_DIAGNOSTICS: usize = 16;
const MIN_TOKEN_BUDGET: u32 = 1;
const MAX_TOKEN_BUDGET: u32 = 1_000_000;
const MAX_TEMPERATURE_MILLIS: u16 = 2_000;
const ROLE_SYSTEM: &str = "system";
const ROLE_DEVELOPER: &str = "developer";
const ROLE_USER: &str = "user";
const ROLE_ASSISTANT: &str = "assistant";
const ROLE_TOOL: &str = "tool";
const FINISH_STOP: &str = "stop";
const FINISH_LENGTH: &str = "length";
const FINISH_TOOL_CALL: &str = "tool_call";
const FINISH_REFUSAL: &str = "refusal";
const FINISH_SAFETY: &str = "safety";
const FINISH_MALFORMED: &str = "malformed";
const LEAK_MARKER_RAW_PROVIDER_PAYLOAD: &str = "raw_provider_payload";
const LEAK_MARKER_PROVIDER_PAYLOAD_PREFIX: &str = "provider_payload:";
const LEAK_MARKER_CREDENTIAL_PREFIX: &str = "credential:";
const LEAK_MARKER_SECRET_PREFIX: &str = "secret:";
const LEAK_MARKER_AUTHORIZATION_PREFIX: &str = "authorization:";
const LEAK_MARKER_BEARER_PREFIX: &str = "bearer ";
const LEAK_MARKER_HIDDEN_REASONING: &str = "hidden_reasoning";
const INVALID_ZERO_USAGE: &str = "usage token counts must not both be zero";
const INVALID_CACHED_USAGE: &str = "cached input tokens must not exceed input tokens";
const INVALID_REASONING_DISABLED_EFFORT: &str = "disabled reasoning must use none effort";
const INVALID_REASONING_ENABLED_EFFORT: &str = "enabled reasoning must request non-none effort";
const INVALID_TOOL_ROLE_WITHOUT_CALL: &str = "tool role requires a tool-call id";
const INVALID_NON_TOOL_WITH_CALL: &str = "only tool role may carry a tool-call id";
const INVALID_TOKEN_BUDGET: &str = "token budget is outside accepted bounds";
const INVALID_TEMPERATURE: &str = "temperature millis is outside accepted bounds";
const INVALID_REFUSAL_COMBINATION: &str =
    "refusal or safety output cannot combine with success output";
const INVALID_MALFORMED_COMBINATION: &str = "malformed output cannot combine with success output";
const LOSSY_LEGACY_REQUEST: &str = "canonical request cannot collapse into one legacy prompt";
const LOSSY_LEGACY_RESPONSE: &str = "canonical response carries non-text semantics";
// r[impl agent_harness_runtime.model_compat.message_request]
/// Stable contract identity for canonical provider-neutral model request DTOs.
pub const CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY: &str = "animus.model-message-request.v1";
/// Stable contract identity for canonical provider-neutral model response envelopes.
pub const CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY: &str = "animus.model-response-envelope.v1";
/// Unsafe material class rejected from canonical model contracts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalUnsafeTextReason {
    /// Raw provider-native payload material was supplied at a provider-neutral boundary.
    RawProviderPayload,
    /// Credential or authorization material was supplied at a provider-neutral boundary.
    Credential,
    /// Hidden reasoning or thinking text would become assistant-visible.
    HiddenReasoning,
}
/// Validation error for canonical model request, response, and compatibility bridge contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalModelContractError {
    /// A shared adapter-field validator rejected a scalar field.
    AdapterContract {
        /// Underlying adapter contract error.
        source: crate::AdapterContractError,
    },
    /// The supplied stable contract identity was missing or not the expected value.
    InvalidContractIdentity {
        /// Expected stable contract identity.
        expected: &'static str,
        /// Supplied stable contract identity after validation.
        actual: alloc::string::String,
    },
    /// A required collection was empty.
    EmptyCollection {
        /// Collection field name.
        field: &'static str,
    },
    /// A bounded collection exceeded its maximum item count.
    CollectionTooLarge {
        /// Collection field name.
        field: &'static str,
        /// Maximum accepted item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A message role label is not part of the canonical role set.
    UnsupportedRole {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported role label.
        value: alloc::string::String,
    },
    /// A content part cannot be represented at the canonical boundary.
    UnsupportedContentPart {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported content-part kind.
        value: alloc::string::String,
    },
    /// A field contained unsafe provider, credential, or hidden-reasoning material.
    UnsafeText {
        /// Boundary field name.
        field: &'static str,
        /// Unsafe material class.
        reason: CanonicalUnsafeTextReason,
    },
    /// A role/correlation, output-control, or other scalar invariant failed.
    InvalidField {
        /// Boundary field name.
        field: &'static str,
        /// Human-readable invariant failure.
        reason: &'static str,
    },
    /// A response envelope omitted the required terminal finish reason.
    MissingFinishReason {
        /// Boundary field name.
        field: &'static str,
    },
    /// A response finish reason is unsupported.
    UnsupportedFinishReason {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported finish reason label.
        value: alloc::string::String,
    },
    /// Response usage metadata was malformed.
    MalformedUsage {
        /// Boundary field name.
        field: &'static str,
        /// Human-readable invariant failure.
        reason: &'static str,
    },
    /// Response items combined mutually exclusive semantic classes.
    ImpossibleResponseCombination {
        /// Human-readable invariant failure.
        reason: &'static str,
    },
    /// A compatibility projection would lose canonical semantics.
    LossyLegacyProjection {
        /// Human-readable projection failure.
        reason: &'static str,
    },
}
impl From<crate::AdapterContractError> for CanonicalModelContractError {
    fn from(source: crate::AdapterContractError) -> Self {
        Self::AdapterContract { source }
    }
}
/// Borrowed safe provenance metadata entry before canonical validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalSafeMetadataDraft<'a> {
    /// Safe metadata key.
    pub key: &'a str,
    /// Safe metadata value.
    pub value: &'a str,
}
/// Safe provenance metadata entry carried by canonical contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalSafeMetadata {
    key: alloc::string::String,
    value: alloc::string::String,
}

impl CanonicalSafeMetadata {
    /// Returns the metadata key.
    #[must_use]
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    /// Returns the metadata value.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

/// Canonical stable model message role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalModelRole {
    /// System-level instruction role.
    System,
    /// Developer-level instruction role.
    Developer,
    /// User-visible input role.
    User,
    /// Assistant output role.
    Assistant,
    /// Tool-result role correlated to a tool call.
    Tool,
}

impl CanonicalModelRole {
    /// Parses a stable role label.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when the label is empty, unsafe, or unsupported.
    pub fn parse(raw: &str) -> Result<Self, CanonicalModelContractError> {
        let normalized = text::safe_text(ROLE_FIELD, raw)?;
        match normalized.as_str() {
            ROLE_SYSTEM => Ok(Self::System),
            ROLE_DEVELOPER => Ok(Self::Developer),
            ROLE_USER => Ok(Self::User),
            ROLE_ASSISTANT => Ok(Self::Assistant),
            ROLE_TOOL => Ok(Self::Tool),
            _unsupported => Err(CanonicalModelContractError::UnsupportedRole {
                field: ROLE_FIELD.as_str(),
                value: normalized,
            }),
        }
    }

    /// Returns the stable role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => ROLE_SYSTEM,
            Self::Developer => ROLE_DEVELOPER,
            Self::User => ROLE_USER,
            Self::Assistant => ROLE_ASSISTANT,
            Self::Tool => ROLE_TOOL,
        }
    }
}
pub mod text;
pub(crate) use text::safe_text;
pub mod rest1;
pub mod rest2;
pub mod rest3;
pub mod rest4;
pub mod rest5;
pub mod rest6;
pub use rest1::*;
pub use rest2::*;
pub use rest3::*;
pub use rest4::*;
pub use rest5::*;
pub use rest6::*;

#![allow(non_trait_imports)]
use super::CanonicalModelContractError;
use super::CanonicalModelMessage;
use super::CanonicalModelMessageDraft;
use super::CanonicalOutputControlsDraft;
use super::CanonicalSafeMetadata;
use super::CanonicalSafeMetadataDraft;
use super::CanonicalToolDeclaration;
use super::CanonicalToolDeclarationDraft;
use super::MAX_STOP_SEQUENCES;
use super::REASONING_SUMMARY_FIELD;
use super::RESPONSE_SCHEMA_FIELD;
use super::STOP_SEQUENCE_FIELD;
use super::safe_text;
use super::validate_collection_len;
use super::validate_reasoning_effort;
use super::validate_temperature;
use super::validate_text_slice;
use super::validate_token_budget;

/// Validated canonical output controls.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalOutputControls {
    pub max_output_tokens: Option<u32>,
    pub temperature_millis: Option<u16>,
    pub stop_sequences: alloc::vec::Vec<alloc::string::String>,
    pub stream: bool,
}

impl CanonicalOutputControls {
    /// Validates borrowed output controls.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when token, sampling, or stop-sequence bounds fail.
    pub fn from_draft(draft: CanonicalOutputControlsDraft<'_>) -> Result<Self, CanonicalModelContractError> {
        validate_token_budget(draft.max_output_tokens)?;
        validate_temperature(draft.temperature_millis)?;
        validate_collection_len(STOP_SEQUENCE_FIELD, draft.stop_sequences.len(), MAX_STOP_SEQUENCES)?;
        Ok(Self {
            max_output_tokens: draft.max_output_tokens,
            temperature_millis: draft.temperature_millis,
            stop_sequences: validate_text_slice(STOP_SEQUENCE_FIELD, draft.stop_sequences)?,
            stream: draft.stream,
        })
    }

    /// Returns compatibility defaults for the legacy prompt/message path.
    #[must_use]
    pub fn compatibility_defaults() -> Self {
        Self {
            max_output_tokens: None,
            temperature_millis: None,
            stop_sequences: alloc::vec::Vec::new(),
            stream: false,
        }
    }

    /// Returns the optional output-token budget.
    #[must_use]
    pub const fn max_output_tokens(&self) -> Option<u32> {
        self.max_output_tokens
    }

    /// Returns the optional temperature in thousandths.
    #[must_use]
    pub const fn temperature_millis(&self) -> Option<u16> {
        self.temperature_millis
    }

    /// Returns stop sequences.
    #[must_use]
    pub fn stop_sequences(&self) -> &[alloc::string::String] {
        self.stop_sequences.as_slice()
    }

    /// Returns whether streaming is requested.
    #[must_use]
    pub const fn stream(&self) -> bool {
        self.stream
    }

    /// Returns true when controls match legacy compatibility defaults.
    #[must_use]
    pub fn is_legacy_compatible(&self) -> bool {
        self.max_output_tokens.is_none()
            && self.temperature_millis.is_none()
            && self.stop_sequences.is_empty()
            && !self.stream
    }
}

/// Borrowed response-format intent before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalResponseFormatIntentDraft<'a> {
    /// Plain assistant-visible text.
    Text,
    /// Provider-neutral JSON object intent.
    JsonObject,
    /// Provider-neutral JSON schema label or digest.
    JsonSchema {
        /// Safe schema label or digest.
        schema_label: &'a str,
    },
}

/// Validated response-format intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalResponseFormatIntent {
    /// Plain assistant-visible text.
    Text,
    /// Provider-neutral JSON object intent.
    JsonObject,
    /// Provider-neutral JSON schema label or digest.
    JsonSchema {
        /// Safe schema label or digest.
        schema_label: alloc::string::String,
    },
}

impl CanonicalResponseFormatIntent {
    /// Validates a borrowed response-format intent.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when a schema label is empty or unsafe.
    pub fn from_draft(draft: CanonicalResponseFormatIntentDraft<'_>) -> Result<Self, CanonicalModelContractError> {
        match draft {
            CanonicalResponseFormatIntentDraft::Text => Ok(Self::Text),
            CanonicalResponseFormatIntentDraft::JsonObject => Ok(Self::JsonObject),
            CanonicalResponseFormatIntentDraft::JsonSchema { schema_label } => Ok(Self::JsonSchema {
                schema_label: safe_text(RESPONSE_SCHEMA_FIELD, schema_label)?,
            }),
        }
    }
}

/// Canonical reasoning or thinking effort request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalReasoningEffort {
    /// No reasoning/thinking request.
    None,
    /// Low effort.
    Low,
    /// Medium effort.
    Medium,
    /// High effort.
    High,
}

/// Borrowed reasoning or thinking request before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalReasoningRequestDraft<'a> {
    /// Whether reasoning or thinking is requested.
    pub enabled: bool,
    /// Requested reasoning effort.
    pub effort: CanonicalReasoningEffort,
    /// Optional safe summary label; hidden reasoning text is rejected.
    pub summary_label: Option<&'a str>,
}

/// Validated reasoning or thinking request metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalReasoningRequest {
    pub enabled: bool,
    pub effort: CanonicalReasoningEffort,
    pub summary_label: Option<alloc::string::String>,
}

impl CanonicalReasoningRequest {
    /// Validates a borrowed reasoning request.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when enabled/effort invariants fail or summary
    /// metadata is unsafe.
    pub fn from_draft(draft: CanonicalReasoningRequestDraft<'_>) -> Result<Self, CanonicalModelContractError> {
        validate_reasoning_effort(draft.enabled, draft.effort)?;
        Ok(Self {
            enabled: draft.enabled,
            effort: draft.effort,
            summary_label: draft.summary_label.map(|raw| safe_text(REASONING_SUMMARY_FIELD, raw)).transpose()?,
        })
    }

    /// Returns compatibility defaults for the legacy prompt/message path.
    #[must_use]
    pub const fn compatibility_defaults() -> Self {
        Self {
            enabled: false,
            effort: CanonicalReasoningEffort::None,
            summary_label: None,
        }
    }

    /// Returns whether reasoning is requested.
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the requested reasoning effort.
    #[must_use]
    pub const fn effort(&self) -> CanonicalReasoningEffort {
        self.effort
    }
}

/// Borrowed canonical model request before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalModelRequestDraft<'a> {
    /// Stable contract identity.
    pub contract_identity: &'a str,
    /// Ordered messages.
    pub messages: &'a [CanonicalModelMessageDraft<'a>],
    /// Tool declarations available for this request.
    pub tools: &'a [CanonicalToolDeclarationDraft<'a>],
    /// Output controls.
    pub output: CanonicalOutputControlsDraft<'a>,
    /// Response-format intent.
    pub response_format: CanonicalResponseFormatIntentDraft<'a>,
    /// Reasoning or thinking request.
    pub reasoning: CanonicalReasoningRequestDraft<'a>,
    /// Safe provenance metadata.
    pub provenance: &'a [CanonicalSafeMetadataDraft<'a>],
}

/// Validated provider-neutral canonical model request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalModelRequest {
    pub messages: alloc::vec::Vec<CanonicalModelMessage>,
    pub tools: alloc::vec::Vec<CanonicalToolDeclaration>,
    pub output: CanonicalOutputControls,
    pub response_format: CanonicalResponseFormatIntent,
    pub reasoning: CanonicalReasoningRequest,
    pub provenance: alloc::vec::Vec<CanonicalSafeMetadata>,
}

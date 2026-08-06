//! Provider projection conformance validation and application logic.
#![allow(
    unknown_lints,
    path_segment_repetition,
    assertion_density,
    ambiguous_params,
    borrowed_argument_types
)]

pub mod rest;
pub use rest::*;

#[cfg(test)]
mod tests;

pub const FAMILY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("provider_projection_family");
pub const FEATURE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("provider_projection_feature");
pub const EVIDENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("provider_projection_evidence");
pub const REASON_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("provider_projection_reason");
pub const OPENAI_RESPONSES_LABEL: &str = "openai_responses";
pub const OPENAI_CHAT_LABEL: &str = "openai_chat";
pub const ANTHROPIC_MESSAGES_LABEL: &str = "anthropic_messages";
pub const MOCK_LABEL: &str = "mock";
pub const FEATURE_ROLE: &str = "role";
pub const FEATURE_CONTENT_PART: &str = "content_part";
pub const FEATURE_TOOL: &str = "tool";
pub const FEATURE_STRUCTURED: &str = "structured_output";
pub const FEATURE_REASONING: &str = "reasoning";
pub const FEATURE_STREAMING: &str = "streaming";
pub const FEATURE_METADATA: &str = "metadata";
pub const LITERAL_FAMILY_PREFIX: &str = "family";
pub const LITERAL_ROLE_PREFIX: &str = "role";
pub const LITERAL_TEXT_PREFIX: &str = "text";
pub const LITERAL_IMAGE_PREFIX: &str = "image";
pub const LITERAL_TOOL_PREFIX: &str = "tool";
pub const LITERAL_RESPONSE_PREFIX: &str = "response";
pub const LITERAL_REASONING_PREFIX: &str = "reasoning";
pub const LITERAL_STREAM_PREFIX: &str = "stream";
pub const LITERAL_METADATA_PREFIX: &str = "metadata";
pub const LITERAL_SEPARATOR: &str = ":";
pub const LITERAL_KEY_VALUE_SEPARATOR: &str = "=";

/// Provider family covered by projection conformance fixtures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderProjectionFamily {
    /// `OpenAI` Responses-style projection.
    OpenAiResponses,
    /// `OpenAI`-compatible Chat Completions or local `HTTP` projection.
    OpenAiChat,
    /// Anthropic Messages-style projection.
    AnthropicMessages,
    /// Deterministic mock/local fixture projection.
    Mock,
}

impl ProviderProjectionFamily {
    /// Parses a provider projection family label.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderProjectionError`] when the label is empty or unsupported.
    pub fn parse(raw: &str) -> Result<Self, ProviderProjectionError> {
        let normalized = crate::validation::normalized_adapter_field(FAMILY_FIELD, raw)?;
        match normalized.as_str() {
            OPENAI_RESPONSES_LABEL => Ok(Self::OpenAiResponses),
            OPENAI_CHAT_LABEL => Ok(Self::OpenAiChat),
            ANTHROPIC_MESSAGES_LABEL => Ok(Self::AnthropicMessages),
            MOCK_LABEL => Ok(Self::Mock),
            _unsupported => Err(ProviderProjectionError::UnsupportedFamily { value: normalized }),
        }
    }

    /// Returns the stable family label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenAiResponses => OPENAI_RESPONSES_LABEL,
            Self::OpenAiChat => OPENAI_CHAT_LABEL,
            Self::AnthropicMessages => ANTHROPIC_MESSAGES_LABEL,
            Self::Mock => MOCK_LABEL,
        }
    }
}

/// Feature support claim for provider projection fixtures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderProjectionFeatureSupport {
    /// The feature is preserved by the provider projection.
    Preserved,
    /// The feature is unsupported by the provider projection.
    Unsupported,
}

impl ProviderProjectionFeatureSupport {
    /// Returns true when the feature is preserved.
    #[must_use]
    pub const fn is_preserved(self) -> bool {
        matches!(self, Self::Preserved)
    }
}

/// Provider-family projection support claims for conformance fixtures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderProjectionSupport {
    /// Provider family.
    pub family: ProviderProjectionFamily,
    /// Developer message support.
    pub developer_role: ProviderProjectionFeatureSupport,
    /// Descriptor-backed image part support.
    pub images: ProviderProjectionFeatureSupport,
    /// Tool declaration support.
    pub tools: ProviderProjectionFeatureSupport,
    /// `JSON` object response-format support.
    pub json_object: ProviderProjectionFeatureSupport,
    /// `JSON` schema response-format support.
    pub json_schema: ProviderProjectionFeatureSupport,
    /// Reasoning control support.
    pub reasoning: ProviderProjectionFeatureSupport,
    /// Streaming support.
    pub streaming: ProviderProjectionFeatureSupport,
    /// Safe provenance metadata support.
    pub metadata: ProviderProjectionFeatureSupport,
}

/// Provider-native request projection fixture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRequestProjection {
    family: ProviderProjectionFamily,
    literal_parts: alloc::vec::Vec<alloc::string::String>,
}

impl ProviderRequestProjection {
    /// Returns the provider family.
    #[must_use]
    pub const fn family(&self) -> ProviderProjectionFamily {
        self.family
    }

    /// Returns reviewed literal projection parts.
    #[must_use]
    pub fn literal_parts(&self) -> &[alloc::string::String] {
        self.literal_parts.as_slice()
    }
}

/// Provider-native response fixture before canonical normalization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderResponseFixtureDraft<'a> {
    /// Provider family label.
    pub family: &'a str,
    /// Optional assistant text.
    pub text: Option<&'a str>,
    /// Optional structured payload label.
    pub structured_payload: Option<&'a str>,
    /// Optional tool call identifier.
    pub tool_call_id: Option<&'a str>,
    /// Optional tool name.
    pub tool_name: Option<&'a str>,
    /// Optional tool arguments label.
    pub tool_arguments: Option<&'a str>,
    /// Optional refusal reason.
    pub refusal: Option<&'a str>,
    /// Optional safety category.
    pub safety: Option<&'a str>,
    /// Optional malformed-output diagnostic label.
    pub malformed_output: Option<&'a str>,
    /// Optional usage metadata.
    pub usage: Option<super::CanonicalUsageDraft>,
    /// Optional finish reason label.
    pub finish_reason: Option<&'a str>,
    /// Optional raw provider payload leak sentinel.
    pub raw_provider_payload: Option<&'a str>,
    /// Optional hidden reasoning leak sentinel.
    pub hidden_reasoning: Option<&'a str>,
}

/// Conformance index case status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderProjectionIndexStatus {
    /// Covered by fixture evidence.
    Covered,
    /// Unsupported with an explicit reason.
    Unsupported,
    /// Not applicable with an explicit reason.
    NotApplicable,
}

/// Borrowed conformance index entry before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderProjectionIndexEntryDraft<'a> {
    /// Provider family label.
    pub family: &'a str,
    /// Feature/provider case label.
    pub feature: &'a str,
    /// Case status.
    pub status: ProviderProjectionIndexStatus,
    /// Fixture evidence label for covered cases.
    pub evidence: Option<&'a str>,
    /// Reason label for unsupported or not-applicable cases.
    pub reason: Option<&'a str>,
}

/// Validated conformance index entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderProjectionIndexEntry {
    family: ProviderProjectionFamily,
    feature: alloc::string::String,
    status: ProviderProjectionIndexStatus,
    evidence: Option<alloc::string::String>,
    reason: Option<alloc::string::String>,
}

impl ProviderProjectionIndexEntry {
    /// Returns the provider family.
    #[must_use]
    pub const fn family(&self) -> ProviderProjectionFamily {
        self.family
    }

    /// Returns the feature label.
    #[must_use]
    pub fn feature(&self) -> &str {
        self.feature.as_str()
    }

    /// Returns the case status.
    #[must_use]
    pub const fn status(&self) -> ProviderProjectionIndexStatus {
        self.status
    }
}

/// Provider projection conformance error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderProjectionError {
    /// Shared adapter scalar validation failed.
    AdapterContract {
        /// Underlying adapter error.
        source: crate::AdapterContractError,
    },
    /// Canonical model contract validation failed.
    CanonicalContract {
        /// Underlying canonical contract error.
        source: super::CanonicalModelContractError,
    },
    /// Model-stream fixture parsing or validation failed.
    StreamFixture {
        /// Underlying stream error.
        source: super::wire::failure::Error,
    },
    /// Provider family label is unsupported.
    UnsupportedFamily {
        /// Unsupported value.
        value: alloc::string::String,
    },
    /// Request projection would drop or misrepresent a canonical feature.
    LossyProjection {
        /// Safe feature label.
        feature: &'static str,
    },
    /// Literal fixture comparison failed.
    LiteralMismatch {
        /// Expected reviewed literal fixture.
        expected: alloc::vec::Vec<alloc::string::String>,
        /// Actual generated literal fixture.
        actual: alloc::vec::Vec<alloc::string::String>,
    },
    /// Conformance index entry is missing required evidence or reason.
    MissingIndexField {
        /// Missing field label.
        field: &'static str,
    },
}

impl From<crate::AdapterContractError> for ProviderProjectionError {
    fn from(source: crate::AdapterContractError) -> Self {
        Self::AdapterContract { source }
    }
}

impl From<super::CanonicalModelContractError> for ProviderProjectionError {
    fn from(source: super::CanonicalModelContractError) -> Self {
        Self::CanonicalContract { source }
    }
}

impl From<crate::model::wire::failure::Error> for ProviderProjectionError {
    fn from(source: crate::model::wire::failure::Error) -> Self {
        Self::StreamFixture { source }
    }
}

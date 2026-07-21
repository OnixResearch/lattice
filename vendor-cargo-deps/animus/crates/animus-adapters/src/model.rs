mod canonical;
mod conformance;
#[cfg(test)]
mod tests;
pub(crate) mod wire;

pub use canonical::{
    CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY, CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY,
    CanonicalContentPart, CanonicalContentPartDraft, CanonicalFinishReason,
    CanonicalModelContractError, CanonicalModelMessage, CanonicalModelMessageDraft,
    CanonicalModelRequest, CanonicalModelRequestDraft, CanonicalModelResponseEnvelope,
    CanonicalModelResponseEnvelopeDraft, CanonicalModelResponseItem,
    CanonicalModelResponseItemDraft, CanonicalModelRole, CanonicalOutputControls,
    CanonicalOutputControlsDraft, CanonicalReasoningEffort, CanonicalReasoningRequest,
    CanonicalReasoningRequestDraft, CanonicalResponseFormatIntent,
    CanonicalResponseFormatIntentDraft, CanonicalSafeMetadata, CanonicalSafeMetadataDraft,
    CanonicalToolDeclaration, CanonicalToolDeclarationDraft, CanonicalUnsafeTextReason,
    CanonicalUsage, CanonicalUsageDraft,
};
pub use conformance::{
    ProviderProjectionError, ProviderProjectionFamily, ProviderProjectionFeatureSupport,
    ProviderProjectionIndexEntry, ProviderProjectionIndexEntryDraft, ProviderProjectionIndexStatus,
    ProviderProjectionSupport, ProviderRequestProjection, ProviderResponseFixtureDraft,
    assert_provider_request_fixture, assert_provider_stream_fixture,
    normalize_provider_response_fixture, project_provider_request_fixture,
    validate_provider_projection_index,
};

const PROMPT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("prompt");
const MESSAGE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_message");

// r[impl agent_harness_runtime.boundary_versions.contract_identities]
/// Stable contract identity for public model request DTOs.
pub const MODEL_REQUEST_CONTRACT_IDENTITY: &str = "animus.model-request.v1";
/// Stable contract identity for public model response DTOs.
pub const MODEL_RESPONSE_CONTRACT_IDENTITY: &str = "animus.model-response.v1";

/// Request sent from the harness to a model component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    prompt: alloc::string::String,
}

impl Request {
    /// Builds a model request from validated prompt text.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the prompt is empty, too
    /// long, or contains a NUL character.
    pub fn new(prompt: &str) -> Result<Self, crate::AdapterContractError> {
        crate::validation::validate_required_adapter_field(PROMPT_FIELD, prompt)?;
        Ok(Self {
            prompt: alloc::string::String::from(prompt),
        })
    }

    /// Builds a model request from deterministic core state.
    #[must_use]
    pub fn from_state(state: &animus_core::AnimusState) -> Self {
        Self {
            prompt: state.summary(),
        }
    }

    /// Returns the prompt text.
    #[must_use]
    pub fn prompt(&self) -> &str {
        self.prompt.as_str()
    }

    /// Returns the stable model request contract identity.
    #[must_use]
    pub const fn contract_identity() -> &'static str {
        MODEL_REQUEST_CONTRACT_IDENTITY
    }
}

/// Response returned by a model component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    message: alloc::string::String,
}

impl Response {
    /// Builds a model response from validated message text.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the message is empty, too
    /// long, or contains a NUL character.
    pub fn new(message: &str) -> Result<Self, crate::AdapterContractError> {
        crate::validation::validate_required_adapter_field(MESSAGE_FIELD, message)?;
        Ok(Self {
            message: alloc::string::String::from(message),
        })
    }

    /// Returns the model message text.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    /// Returns the stable model response contract identity.
    #[must_use]
    pub const fn contract_identity() -> &'static str {
        MODEL_RESPONSE_CONTRACT_IDENTITY
    }
}

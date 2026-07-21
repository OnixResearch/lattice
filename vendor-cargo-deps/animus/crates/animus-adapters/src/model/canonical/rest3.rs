#![allow(non_trait_imports)]
use super::{
    CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY, CanonicalContentPartDraft,
    CanonicalModelContractError, CanonicalModelMessage, CanonicalModelMessageDraft,
    CanonicalModelRequest, CanonicalModelRequestDraft, CanonicalModelRole, CanonicalOutputControls,
    CanonicalOutputControlsDraft, CanonicalReasoningEffort, CanonicalReasoningRequest,
    CanonicalReasoningRequestDraft, CanonicalResponseFormatIntent,
    CanonicalResponseFormatIntentDraft, CanonicalSafeMetadata, CanonicalToolDeclaration,
    INVALID_CACHED_USAGE, INVALID_ZERO_USAGE, MAX_MODEL_MESSAGES, MAX_MODEL_TOOLS,
    RESPONSE_TEXT_FIELD, ROLE_FIELD, ROLE_USER, TOOL_NAME_FIELD, lossy_legacy_request,
    require_collection, validate_collection_len, validate_contract_identity, validate_messages,
    validate_metadata, validate_tools,
};

impl CanonicalModelRequest {
    /// Validates a borrowed canonical request draft.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when any request field is malformed, unsafe, or unsupported.
    pub fn from_draft(
        draft: CanonicalModelRequestDraft<'_>,
    ) -> Result<Self, CanonicalModelContractError> {
        validate_contract_identity(
            CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
            draft.contract_identity,
        )?;
        require_collection(ROLE_FIELD, draft.messages.len(), MAX_MODEL_MESSAGES)?;
        validate_collection_len(TOOL_NAME_FIELD, draft.tools.len(), MAX_MODEL_TOOLS)?;
        Ok(Self {
            messages: validate_messages(draft.messages)?,
            tools: validate_tools(draft.tools)?,
            output: CanonicalOutputControls::from_draft(draft.output)?,
            response_format: CanonicalResponseFormatIntent::from_draft(draft.response_format)?,
            reasoning: CanonicalReasoningRequest::from_draft(draft.reasoning)?,
            provenance: validate_metadata(draft.provenance)?,
        })
    }

    /// Builds a canonical compatibility request from the legacy prompt DTO.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when the legacy prompt somehow fails canonical validation.
    pub fn from_legacy_request(
        request: &crate::model::Request,
    ) -> Result<Self, CanonicalModelContractError> {
        let content = [CanonicalContentPartDraft::Text {
            text: request.prompt(),
        }];
        let messages = [CanonicalModelMessageDraft {
            role: ROLE_USER,
            content: &content,
            tool_call_id: None,
        }];
        Self::from_draft(CanonicalModelRequestDraft {
            contract_identity: CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
            messages: &messages,
            tools: &[],
            output: CanonicalOutputControlsDraft {
                max_output_tokens: None,
                temperature_millis: None,
                stop_sequences: &[],
                stream: false,
            },
            response_format: CanonicalResponseFormatIntentDraft::Text,
            reasoning: CanonicalReasoningRequestDraft {
                enabled: false,
                effort: CanonicalReasoningEffort::None,
                summary_label: None,
            },
            provenance: &[],
        })
    }

    /// Projects a canonical compatibility request back to the legacy prompt DTO.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when projection would lose structured, tool, stream, or reasoning semantics.
    pub fn to_legacy_request(&self) -> Result<crate::model::Request, CanonicalModelContractError> {
        if !self.tools.is_empty()
            || !self.output.is_legacy_compatible()
            || self.response_format != CanonicalResponseFormatIntent::Text
            || self.reasoning.enabled()
            || !self.provenance.is_empty()
        {
            return Err(lossy_legacy_request());
        }
        let [message] = self.messages.as_slice() else {
            return Err(lossy_legacy_request());
        };
        if message.role() != CanonicalModelRole::User || message.tool_call_id().is_some() {
            return Err(lossy_legacy_request());
        }
        let [part] = message.content() else {
            return Err(lossy_legacy_request());
        };
        let Some(text) = part.text() else {
            return Err(lossy_legacy_request());
        };
        Ok(crate::model::Request::new(text)?)
    }

    /// Returns validated messages.
    #[must_use]
    pub fn messages(&self) -> &[CanonicalModelMessage] {
        self.messages.as_slice()
    }

    /// Returns validated tool declarations.
    #[must_use]
    pub fn tools(&self) -> &[CanonicalToolDeclaration] {
        self.tools.as_slice()
    }

    /// Returns output controls.
    #[must_use]
    pub const fn output(&self) -> &CanonicalOutputControls {
        &self.output
    }

    /// Returns response-format intent.
    #[must_use]
    pub const fn response_format(&self) -> &CanonicalResponseFormatIntent {
        &self.response_format
    }

    /// Returns reasoning request metadata.
    #[must_use]
    pub const fn reasoning(&self) -> &CanonicalReasoningRequest {
        &self.reasoning
    }

    /// Returns safe provenance metadata.
    #[must_use]
    pub fn provenance(&self) -> &[CanonicalSafeMetadata] {
        self.provenance.as_slice()
    }

    /// Returns the stable canonical request contract identity.
    #[must_use]
    pub const fn contract_identity() -> &'static str {
        CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY
    }
}

/// Borrowed usage metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalUsageDraft {
    /// Provider-reported input token count.
    pub input: u32,
    /// Provider-reported output token count.
    pub output: u32,
    /// Optional cached input token count.
    pub cached_input: Option<u32>,
}

/// Validated usage metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalUsage {
    input: u32,
    output: u32,
    cached_input: Option<u32>,
}

impl CanonicalUsage {
    /// Validates usage metadata.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when usage counts are malformed.
    pub fn from_draft(draft: CanonicalUsageDraft) -> Result<Self, CanonicalModelContractError> {
        if draft.input == 0 && draft.output == 0 {
            return Err(CanonicalModelContractError::MalformedUsage {
                field: RESPONSE_TEXT_FIELD.as_str(),
                reason: INVALID_ZERO_USAGE,
            });
        }
        if let Some(cached) = draft.cached_input
            && cached > draft.input
        {
            return Err(CanonicalModelContractError::MalformedUsage {
                field: RESPONSE_TEXT_FIELD.as_str(),
                reason: INVALID_CACHED_USAGE,
            });
        }
        Ok(Self {
            input: draft.input,
            output: draft.output,
            cached_input: draft.cached_input,
        })
    }

    /// Returns input-token count.
    #[must_use]
    pub const fn input_tokens(&self) -> u32 {
        self.input
    }

    /// Returns output-token count.
    #[must_use]
    pub const fn output_tokens(&self) -> u32 {
        self.output
    }

    /// Returns cached input-token count when provided.
    #[must_use]
    pub const fn cached_input_tokens(&self) -> Option<u32> {
        self.cached_input
    }
}

#![allow(non_trait_imports)]
use super::{
    CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY, CONTRACT_IDENTITY_FIELD, CanonicalContentPart,
    CanonicalContentPartDraft, CanonicalFinishReason, CanonicalModelContractError,
    CanonicalModelMessage, CanonicalModelMessageDraft, CanonicalModelResponseItem,
    CanonicalModelResponseItemDraft, CanonicalSafeMetadata, CanonicalSafeMetadataDraft,
    CanonicalToolDeclaration, CanonicalToolDeclarationDraft, CanonicalUsage, CanonicalUsageDraft,
    FINISH_REASON_FIELD, FINISH_STOP, MALFORMED_DIAGNOSTIC_FIELD, MAX_RESPONSE_DIAGNOSTICS,
    MAX_RESPONSE_ITEMS, RESPONSE_TEXT_FIELD, lossy_legacy_response, require_collection, safe_text,
    validate_collection_len, validate_metadata, validate_response_combination,
};

/// Borrowed canonical response envelope before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalModelResponseEnvelopeDraft<'a> {
    /// Stable contract identity.
    pub contract_identity: &'a str,
    /// Ordered response items.
    pub items: &'a [CanonicalModelResponseItemDraft<'a>],
    /// Optional usage metadata.
    pub usage: Option<CanonicalUsageDraft>,
    /// Required finish reason label.
    pub finish_reason: Option<&'a str>,
    /// Safe malformed-output or normalization diagnostics.
    pub diagnostics: &'a [&'a str],
    /// Safe provenance metadata.
    pub provenance: &'a [CanonicalSafeMetadataDraft<'a>],
}

/// Validated canonical model response envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalModelResponseEnvelope {
    items: alloc::vec::Vec<CanonicalModelResponseItem>,
    usage: Option<CanonicalUsage>,
    finish_reason: CanonicalFinishReason,
    diagnostics: alloc::vec::Vec<alloc::string::String>,
    provenance: alloc::vec::Vec<CanonicalSafeMetadata>,
}

impl CanonicalModelResponseEnvelope {
    /// Validates a borrowed response-envelope draft.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when any response field is malformed, unsafe, or semantically impossible.
    pub fn from_draft(
        draft: CanonicalModelResponseEnvelopeDraft<'_>,
    ) -> Result<Self, CanonicalModelContractError> {
        validate_contract_identity(
            CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY,
            draft.contract_identity,
        )?;
        require_collection(RESPONSE_TEXT_FIELD, draft.items.len(), MAX_RESPONSE_ITEMS)?;
        validate_collection_len(
            MALFORMED_DIAGNOSTIC_FIELD,
            draft.diagnostics.len(),
            MAX_RESPONSE_DIAGNOSTICS,
        )?;
        let Some(raw_finish_reason) = draft.finish_reason else {
            return Err(CanonicalModelContractError::MissingFinishReason {
                field: FINISH_REASON_FIELD.as_str(),
            });
        };
        let items = validate_response_items(draft.items)?;
        validate_response_combination(items.as_slice())?;
        Ok(Self {
            items,
            usage: draft.usage.map(CanonicalUsage::from_draft).transpose()?,
            finish_reason: CanonicalFinishReason::parse(raw_finish_reason)?,
            diagnostics: validate_text_slice(MALFORMED_DIAGNOSTIC_FIELD, draft.diagnostics)?,
            provenance: validate_metadata(draft.provenance)?,
        })
    }

    /// Builds a canonical compatibility response from the legacy message DTO.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when the legacy message somehow fails canonical validation.
    pub fn from_legacy_response(
        response: &crate::model::Response,
    ) -> Result<Self, CanonicalModelContractError> {
        let items = [CanonicalModelResponseItemDraft::AssistantText {
            text: response.message(),
        }];
        Self::from_draft(CanonicalModelResponseEnvelopeDraft {
            contract_identity: CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY,
            items: &items,
            usage: None,
            finish_reason: Some(FINISH_STOP),
            diagnostics: &[],
            provenance: &[],
        })
    }

    /// Projects a canonical compatibility response back to the legacy message DTO.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when projection would lose structured, tool, refusal, safety, or malformed semantics.
    pub fn to_legacy_response(
        &self,
    ) -> Result<crate::model::Response, CanonicalModelContractError> {
        if !matches!(
            self.finish_reason,
            CanonicalFinishReason::Stop | CanonicalFinishReason::Length
        ) || !self.diagnostics.is_empty()
            || self.usage.is_some()
            || !self.provenance.is_empty()
        {
            return Err(lossy_legacy_response());
        }
        let [item] = self.items.as_slice() else {
            return Err(lossy_legacy_response());
        };
        let Some(text) = item.assistant_text() else {
            return Err(lossy_legacy_response());
        };
        Ok(crate::model::Response::new(text)?)
    }

    /// Returns response items.
    #[must_use]
    pub fn items(&self) -> &[CanonicalModelResponseItem] {
        self.items.as_slice()
    }

    /// Returns optional usage metadata.
    #[must_use]
    pub const fn usage(&self) -> Option<CanonicalUsage> {
        self.usage
    }

    /// Returns the terminal finish reason.
    #[must_use]
    pub const fn finish_reason(&self) -> CanonicalFinishReason {
        self.finish_reason
    }

    /// Returns safe diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[alloc::string::String] {
        self.diagnostics.as_slice()
    }

    /// Returns safe provenance metadata.
    #[must_use]
    pub fn provenance(&self) -> &[CanonicalSafeMetadata] {
        self.provenance.as_slice()
    }

    /// Returns the stable canonical response contract identity.
    #[must_use]
    pub const fn contract_identity() -> &'static str {
        CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY
    }
}

pub fn validate_contract_identity(
    expected: &'static str,
    raw: &str,
) -> Result<(), CanonicalModelContractError> {
    let actual = safe_text(CONTRACT_IDENTITY_FIELD, raw)?;
    if actual == expected {
        return Ok(());
    }
    Err(CanonicalModelContractError::InvalidContractIdentity { expected, actual })
}

pub fn validate_messages(
    drafts: &[CanonicalModelMessageDraft<'_>],
) -> Result<alloc::vec::Vec<CanonicalModelMessage>, CanonicalModelContractError> {
    drafts
        .iter()
        .copied()
        .map(CanonicalModelMessage::from_draft)
        .collect()
}

pub fn validate_content_parts(
    drafts: &[CanonicalContentPartDraft<'_>],
) -> Result<alloc::vec::Vec<CanonicalContentPart>, CanonicalModelContractError> {
    drafts
        .iter()
        .copied()
        .map(CanonicalContentPart::from_draft)
        .collect()
}

pub fn validate_tools(
    drafts: &[CanonicalToolDeclarationDraft<'_>],
) -> Result<alloc::vec::Vec<CanonicalToolDeclaration>, CanonicalModelContractError> {
    drafts
        .iter()
        .copied()
        .map(CanonicalToolDeclaration::from_draft)
        .collect()
}

fn validate_response_items(
    drafts: &[CanonicalModelResponseItemDraft<'_>],
) -> Result<alloc::vec::Vec<CanonicalModelResponseItem>, CanonicalModelContractError> {
    drafts
        .iter()
        .copied()
        .map(CanonicalModelResponseItem::from_draft)
        .collect()
}

pub fn validate_text_slice(
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, CanonicalModelContractError> {
    values.iter().map(|value| safe_text(field, value)).collect()
}

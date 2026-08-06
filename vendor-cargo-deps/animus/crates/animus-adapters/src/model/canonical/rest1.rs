#![allow(non_trait_imports)]
use super::CONTENT_PART_KIND_FIELD;
use super::CONTENT_TEXT_FIELD;
use super::CanonicalModelContractError;
use super::CanonicalModelRole;
use super::CanonicalUnsafeTextReason;
use super::IMAGE_DESCRIPTOR_FIELD;
use super::IMAGE_HASH_FIELD;
use super::IMAGE_MEDIA_TYPE_FIELD;
use super::MAX_CONTENT_PARTS_PER_MESSAGE;
use super::RAW_PROVIDER_PAYLOAD_FIELD;
use super::TOOL_DESCRIPTION_FIELD;
use super::TOOL_NAME_FIELD;
use super::TOOL_SCHEMA_FIELD;
use super::require_collection;
use super::safe_text;
use super::validate_content_parts;
use super::validate_tool_call_id;

/// Borrowed canonical content part before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalContentPartDraft<'a> {
    /// Assistant-visible or model-visible text content.
    Text {
        /// Text body.
        text: &'a str,
    },
    /// Descriptor-backed image content without raw bytes or URLs.
    ImageDescriptor {
        /// Safe descriptor identifier.
        descriptor_id: &'a str,
        /// Safe media type label.
        media_type: &'a str,
        /// Safe content hash.
        content_hash: &'a str,
    },
    /// Unsupported content part used to prove fail-closed behavior.
    Unsupported {
        /// Unsupported content-part kind label.
        kind: &'a str,
    },
    /// Raw provider-native payload used to prove leak rejection.
    RawProviderPayload {
        /// Unsafe payload label.
        label: &'a str,
    },
}

/// Validated canonical provider-neutral content part.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalContentPart {
    /// Text content part.
    Text {
        /// Validated text body.
        text: alloc::string::String,
    },
    /// Descriptor-backed image content part.
    ImageDescriptor {
        /// Safe descriptor identifier.
        descriptor_id: alloc::string::String,
        /// Safe media type label.
        media_type: alloc::string::String,
        /// Safe content hash.
        content_hash: alloc::string::String,
    },
}

impl CanonicalContentPart {
    /// Validates a borrowed content-part draft.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when content is empty, unsupported, or unsafe.
    pub fn from_draft(draft: CanonicalContentPartDraft<'_>) -> Result<Self, CanonicalModelContractError> {
        match draft {
            CanonicalContentPartDraft::Text { text } => Ok(Self::Text {
                text: safe_text(CONTENT_TEXT_FIELD, text)?,
            }),
            CanonicalContentPartDraft::ImageDescriptor {
                descriptor_id,
                media_type,
                content_hash,
            } => Ok(Self::ImageDescriptor {
                descriptor_id: safe_text(IMAGE_DESCRIPTOR_FIELD, descriptor_id)?,
                media_type: safe_text(IMAGE_MEDIA_TYPE_FIELD, media_type)?,
                content_hash: safe_text(IMAGE_HASH_FIELD, content_hash)?,
            }),
            CanonicalContentPartDraft::Unsupported { kind } => {
                let value = safe_text(CONTENT_PART_KIND_FIELD, kind)?;
                Err(CanonicalModelContractError::UnsupportedContentPart {
                    field: CONTENT_PART_KIND_FIELD.as_str(),
                    value,
                })
            }
            CanonicalContentPartDraft::RawProviderPayload { label } => {
                let _label = crate::validation::normalized_adapter_field(RAW_PROVIDER_PAYLOAD_FIELD, label)?;
                Err(CanonicalModelContractError::UnsafeText {
                    field: RAW_PROVIDER_PAYLOAD_FIELD.as_str(),
                    reason: CanonicalUnsafeTextReason::RawProviderPayload,
                })
            }
        }
    }

    /// Returns true when the part is text.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    /// Returns the text body when this is a text part.
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text.as_str()),
            Self::ImageDescriptor { .. } => None,
        }
    }
}

/// Borrowed canonical message before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalModelMessageDraft<'a> {
    /// Stable role label.
    pub role: &'a str,
    /// Ordered content parts.
    pub content: &'a [CanonicalContentPartDraft<'a>],
    /// Tool-call identifier required only for tool-result messages.
    pub tool_call_id: Option<&'a str>,
}

/// Validated canonical message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalModelMessage {
    role: CanonicalModelRole,
    content: alloc::vec::Vec<CanonicalContentPart>,
    tool_call_id: Option<alloc::string::String>,
}

impl CanonicalModelMessage {
    /// Validates a borrowed message draft.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when the role, content, or tool correlation is
    /// invalid.
    pub fn from_draft(draft: CanonicalModelMessageDraft<'_>) -> Result<Self, CanonicalModelContractError> {
        let role = CanonicalModelRole::parse(draft.role)?;
        require_collection(CONTENT_PART_KIND_FIELD, draft.content.len(), MAX_CONTENT_PARTS_PER_MESSAGE)?;
        let content = validate_content_parts(draft.content)?;
        let tool_call_id = validate_tool_call_id(role, draft.tool_call_id)?;
        Ok(Self {
            role,
            content,
            tool_call_id,
        })
    }

    /// Returns the stable role.
    #[must_use]
    pub const fn role(&self) -> CanonicalModelRole {
        self.role
    }

    /// Returns validated content parts.
    #[must_use]
    pub fn content(&self) -> &[CanonicalContentPart] {
        self.content.as_slice()
    }

    /// Returns the optional tool-call correlation identifier.
    #[must_use]
    pub fn tool_call_id(&self) -> Option<&str> {
        self.tool_call_id.as_deref()
    }
}

/// Borrowed tool declaration before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalToolDeclarationDraft<'a> {
    /// Safe tool name.
    pub name: &'a str,
    /// Safe human-readable description.
    pub description: &'a str,
    /// Safe input schema label or digest.
    pub input_schema_label: &'a str,
}

/// Validated canonical tool declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalToolDeclaration {
    name: alloc::string::String,
    description: alloc::string::String,
    input_schema_label: alloc::string::String,
}

impl CanonicalToolDeclaration {
    /// Validates a borrowed tool declaration.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when a tool field is empty or unsafe.
    pub fn from_draft(draft: CanonicalToolDeclarationDraft<'_>) -> Result<Self, CanonicalModelContractError> {
        Ok(Self {
            name: safe_text(TOOL_NAME_FIELD, draft.name)?,
            description: safe_text(TOOL_DESCRIPTION_FIELD, draft.description)?,
            input_schema_label: safe_text(TOOL_SCHEMA_FIELD, draft.input_schema_label)?,
        })
    }

    /// Returns the tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the safe description.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the input schema label.
    #[must_use]
    pub fn input_schema_label(&self) -> &str {
        self.input_schema_label.as_str()
    }
}

/// Borrowed output controls before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalOutputControlsDraft<'a> {
    /// Optional maximum output token budget.
    pub max_output_tokens: Option<u32>,
    /// Optional deterministic temperature encoded as thousandths.
    pub temperature_millis: Option<u16>,
    /// Stop sequences requested by the host.
    pub stop_sequences: &'a [&'a str],
    /// Whether the caller requests streaming frames.
    pub stream: bool,
}

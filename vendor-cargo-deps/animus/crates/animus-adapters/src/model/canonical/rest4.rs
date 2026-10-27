#![allow(non_trait_imports)]
use super::{
    CanonicalModelContractError, CanonicalUnsafeTextReason, FINISH_LENGTH, FINISH_MALFORMED,
    FINISH_REASON_FIELD, FINISH_REFUSAL, FINISH_SAFETY, FINISH_STOP, FINISH_TOOL_CALL,
    HIDDEN_REASONING_FIELD, MALFORMED_DIAGNOSTIC_FIELD, RAW_PROVIDER_PAYLOAD_FIELD,
    REFUSAL_REASON_FIELD, RESPONSE_SCHEMA_FIELD, RESPONSE_TEXT_FIELD, SAFETY_CATEGORY_FIELD,
    STRUCTURED_PAYLOAD_FIELD, TOOL_ARGUMENTS_FIELD, TOOL_CALL_ID_FIELD, TOOL_NAME_FIELD, safe_text,
};

/// Canonical terminal finish reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalFinishReason {
    /// Provider reached normal stop.
    Stop,
    /// Provider stopped because output length was exhausted.
    Length,
    /// Provider requested one or more tool calls.
    ToolCall,
    /// Provider refused the request.
    Refusal,
    /// Provider returned a safety block.
    Safety,
    /// Provider output was malformed and normalized as a diagnostic.
    Malformed,
}

impl CanonicalFinishReason {
    /// Parses a stable finish-reason label.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when the label is empty, unsafe, or unsupported.
    pub fn parse(raw: &str) -> Result<Self, CanonicalModelContractError> {
        let normalized = safe_text(FINISH_REASON_FIELD, raw)?;
        match normalized.as_str() {
            FINISH_STOP => Ok(Self::Stop),
            FINISH_LENGTH => Ok(Self::Length),
            FINISH_TOOL_CALL => Ok(Self::ToolCall),
            FINISH_REFUSAL => Ok(Self::Refusal),
            FINISH_SAFETY => Ok(Self::Safety),
            FINISH_MALFORMED => Ok(Self::Malformed),
            _unsupported => Err(CanonicalModelContractError::UnsupportedFinishReason {
                field: FINISH_REASON_FIELD.as_str(),
                value: normalized,
            }),
        }
    }

    /// Returns the stable finish-reason label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stop => FINISH_STOP,
            Self::Length => FINISH_LENGTH,
            Self::ToolCall => FINISH_TOOL_CALL,
            Self::Refusal => FINISH_REFUSAL,
            Self::Safety => FINISH_SAFETY,
            Self::Malformed => FINISH_MALFORMED,
        }
    }
}

/// Borrowed response item before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalModelResponseItemDraft<'a> {
    /// Assistant-visible text.
    AssistantText {
        /// Text body.
        text: &'a str,
    },
    /// Structured output represented by safe labels rather than raw provider payloads.
    StructuredPayload {
        /// Safe schema label.
        schema_label: &'a str,
        /// Safe payload label or digest.
        payload_label: &'a str,
    },
    /// Tool-call request.
    ToolCall {
        /// Safe call identifier.
        call_id: &'a str,
        /// Safe tool name.
        tool_name: &'a str,
        /// Safe arguments label or digest.
        arguments_label: &'a str,
    },
    /// Refusal block.
    Refusal {
        /// Safe refusal reason.
        reason: &'a str,
    },
    /// Safety block.
    Safety {
        /// Safe safety category.
        category: &'a str,
    },
    /// Malformed-output diagnostic.
    MalformedOutput {
        /// Safe diagnostic label.
        diagnostic: &'a str,
    },
    /// Raw provider-native payload used to prove leak rejection.
    RawProviderPayload {
        /// Unsafe payload label.
        label: &'a str,
    },
    /// Hidden reasoning text used to prove leak rejection.
    HiddenReasoning {
        /// Unsafe hidden reasoning label.
        label: &'a str,
    },
}

/// Validated canonical response item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalModelResponseItem {
    /// Assistant-visible text.
    AssistantText {
        /// Validated text body.
        text: alloc::string::String,
    },
    /// Structured output represented by safe labels rather than raw provider payloads.
    StructuredPayload {
        /// Safe schema label.
        schema_label: alloc::string::String,
        /// Safe payload label or digest.
        payload_label: alloc::string::String,
    },
    /// Tool-call request.
    ToolCall {
        /// Safe call identifier.
        call_id: alloc::string::String,
        /// Safe tool name.
        tool_name: alloc::string::String,
        /// Safe arguments label or digest.
        arguments_label: alloc::string::String,
    },
    /// Refusal block.
    Refusal {
        /// Safe refusal reason.
        reason: alloc::string::String,
    },
    /// Safety block.
    Safety {
        /// Safe safety category.
        category: alloc::string::String,
    },
    /// Malformed-output diagnostic.
    MalformedOutput {
        /// Safe diagnostic label.
        diagnostic: alloc::string::String,
    },
}

impl CanonicalModelResponseItem {
    /// Validates a borrowed response item.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModelContractError`] when the item is empty, unsafe, or leaky.
    pub fn from_draft(
        draft: CanonicalModelResponseItemDraft<'_>,
    ) -> Result<Self, CanonicalModelContractError> {
        match draft {
            CanonicalModelResponseItemDraft::AssistantText { text } => Ok(Self::AssistantText {
                text: safe_text(RESPONSE_TEXT_FIELD, text)?,
            }),
            CanonicalModelResponseItemDraft::StructuredPayload {
                schema_label,
                payload_label,
            } => Ok(Self::StructuredPayload {
                schema_label: safe_text(RESPONSE_SCHEMA_FIELD, schema_label)?,
                payload_label: safe_text(STRUCTURED_PAYLOAD_FIELD, payload_label)?,
            }),
            CanonicalModelResponseItemDraft::ToolCall {
                call_id,
                tool_name,
                arguments_label,
            } => Ok(Self::ToolCall {
                call_id: safe_text(TOOL_CALL_ID_FIELD, call_id)?,
                tool_name: safe_text(TOOL_NAME_FIELD, tool_name)?,
                arguments_label: safe_text(TOOL_ARGUMENTS_FIELD, arguments_label)?,
            }),
            CanonicalModelResponseItemDraft::Refusal { reason } => Ok(Self::Refusal {
                reason: safe_text(REFUSAL_REASON_FIELD, reason)?,
            }),
            CanonicalModelResponseItemDraft::Safety { category } => Ok(Self::Safety {
                category: safe_text(SAFETY_CATEGORY_FIELD, category)?,
            }),
            CanonicalModelResponseItemDraft::MalformedOutput { diagnostic } => {
                Ok(Self::MalformedOutput {
                    diagnostic: safe_text(MALFORMED_DIAGNOSTIC_FIELD, diagnostic)?,
                })
            }
            CanonicalModelResponseItemDraft::RawProviderPayload { label } => {
                let _label =
                    crate::validation::normalized_adapter_field(RAW_PROVIDER_PAYLOAD_FIELD, label)?;
                Err(CanonicalModelContractError::UnsafeText {
                    field: RAW_PROVIDER_PAYLOAD_FIELD.as_str(),
                    reason: CanonicalUnsafeTextReason::RawProviderPayload,
                })
            }
            CanonicalModelResponseItemDraft::HiddenReasoning { label } => {
                let _label =
                    crate::validation::normalized_adapter_field(HIDDEN_REASONING_FIELD, label)?;
                Err(CanonicalModelContractError::UnsafeText {
                    field: HIDDEN_REASONING_FIELD.as_str(),
                    reason: CanonicalUnsafeTextReason::HiddenReasoning,
                })
            }
        }
    }

    /// Returns true when the item is assistant text.
    #[must_use]
    pub const fn is_assistant_text(&self) -> bool {
        matches!(self, Self::AssistantText { .. })
    }

    /// Returns the assistant text when present.
    #[must_use]
    pub fn assistant_text(&self) -> Option<&str> {
        match self {
            Self::AssistantText { text } => Some(text.as_str()),
            Self::StructuredPayload { .. }
            | Self::ToolCall { .. }
            | Self::Refusal { .. }
            | Self::Safety { .. }
            | Self::MalformedOutput { .. } => None,
        }
    }
}

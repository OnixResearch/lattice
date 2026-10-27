const NUL_CHARACTER: char = '\0';

/// Smallest accepted non-empty adapter field length after trimming.
pub const MIN_ADAPTER_FIELD_CHARS: usize = 1;

/// Largest accepted adapter field length.
pub const MAX_ADAPTER_FIELD_CHARS: usize = 8_192;

const _: () = assert!(MAX_ADAPTER_FIELD_CHARS >= MIN_ADAPTER_FIELD_CHARS);

/// A validated adapter field name constant used for error diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldName {
    value: &'static str,
}

impl FieldName {
    /// Creates a new field name constant.
    #[must_use]
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }

    /// Returns the static field name string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.value
    }
}

/// Contract errors raised before data crosses an adapter boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterContractError {
    /// A required field contained no user-visible characters.
    EmptyField {
        /// Boundary field name.
        field: &'static str,
    },
    /// A field exceeded [`MAX_ADAPTER_FIELD_CHARS`].
    FieldTooLong {
        /// Boundary field name.
        field: &'static str,
        /// Maximum accepted character count.
        max_chars: usize,
        /// Actual character count.
        actual_chars: usize,
    },
    /// A field contained a NUL character.
    NulCharacter {
        /// Boundary field name.
        field: &'static str,
        /// Byte index where the NUL character starts.
        byte_index: usize,
    },
    /// A semantic agent status state was not supported.
    UnsupportedStatusState {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported value after normalization.
        value: alloc::string::String,
    },
    /// A status-observation authority kind was not supported.
    UnsupportedStatusObservationKind {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported value after normalization.
        value: alloc::string::String,
    },
    /// A session-reference kind was not supported.
    UnsupportedSessionReferenceKind {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported value after normalization.
        value: alloc::string::String,
    },
    /// A safe evidence hash failed deterministic validation.
    InvalidEvidenceHash {
        /// Boundary field name.
        field: &'static str,
        /// Human-readable validation reason.
        reason: &'static str,
    },
    /// A workspace field value was not supported.
    UnsupportedWorkspaceValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported value after normalization.
        value: alloc::string::String,
    },
    /// A dataspace field value was not supported.
    UnsupportedDataspaceValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported value after normalization.
        value: alloc::string::String,
    },
    /// A model-router field value was not supported.
    UnsupportedRouterValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported value after normalization.
        value: alloc::string::String,
    },
    /// A required collection contained no values.
    EmptyCollection {
        /// Boundary collection field name.
        field: &'static str,
    },
}

impl core::fmt::Display for AdapterContractError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyField { field } | Self::EmptyCollection { field } => {
                write!(formatter, "{field} must not be empty")
            }
            Self::FieldTooLong {
                field,
                max_chars,
                actual_chars,
            } => write!(
                formatter,
                "{field} must be at most {max_chars} characters; got {actual_chars}"
            ),
            Self::NulCharacter { field, byte_index } => write!(
                formatter,
                "{field} contains a NUL character at byte index {byte_index}"
            ),
            Self::UnsupportedStatusState { field, value } => {
                write!(
                    formatter,
                    "{field} contains unsupported status state {value}"
                )
            }
            Self::UnsupportedStatusObservationKind { field, value } => write!(
                formatter,
                "{field} contains unsupported status observation kind {value}"
            ),
            Self::UnsupportedSessionReferenceKind { field, value } => write!(
                formatter,
                "{field} contains unsupported session reference kind {value}"
            ),
            Self::InvalidEvidenceHash { field, reason } => {
                write!(formatter, "{field} is invalid: {reason}")
            }
            Self::UnsupportedWorkspaceValue { field, value } => {
                write!(
                    formatter,
                    "{field} contains unsupported workspace value {value}"
                )
            }
            Self::UnsupportedDataspaceValue { field, value } => {
                write!(
                    formatter,
                    "{field} contains unsupported dataspace value {value}"
                )
            }
            Self::UnsupportedRouterValue { field, value } => {
                write!(
                    formatter,
                    "{field} contains unsupported model-router value {value}"
                )
            }
        }
    }
}

pub(crate) fn validate_required_adapter_field(
    field_name: FieldName,
    field_text: &str,
) -> Result<(), AdapterContractError> {
    assert!(
        !field_name.as_str().is_empty(),
        "field name must not be empty"
    );
    let raw_chars = field_text.chars().count();
    let visible_chars = field_text.trim().chars().count();
    assert!(
        visible_chars <= raw_chars,
        "trimmed field character count must fit in raw character count"
    );

    if visible_chars < MIN_ADAPTER_FIELD_CHARS {
        return Err(AdapterContractError::EmptyField {
            field: field_name.as_str(),
        });
    }

    validate_optional_adapter_field(field_name, field_text)
}

/// Normalizes an adapter field value by trimming whitespace and rejecting
/// empty, too-long, or NUL-containing inputs.
///
/// # Errors
///
/// Returns [`AdapterContractError`] when the field is empty, exceeds
/// [`MAX_ADAPTER_FIELD_CHARS`], or contains a NUL character.
///
/// # Panics
///
/// Panics only if required-field validation succeeds while the normalized field
/// still violates the adapter length invariants.
pub fn normalized_adapter_field(
    field_name: FieldName,
    field_text: &str,
) -> Result<alloc::string::String, AdapterContractError> {
    validate_required_adapter_field(field_name, field_text)?;
    let normalized = field_text.trim();
    let normalized_chars = normalized.chars().count();
    assert!(
        normalized_chars >= MIN_ADAPTER_FIELD_CHARS,
        "normalized required field must not be empty"
    );
    assert!(
        normalized_chars <= MAX_ADAPTER_FIELD_CHARS,
        "normalized required field must fit adapter limit"
    );
    Ok(alloc::string::String::from(normalized))
}

pub(crate) fn validate_optional_adapter_field(
    field_name: FieldName,
    field_text: &str,
) -> Result<(), AdapterContractError> {
    assert!(
        !field_name.as_str().is_empty(),
        "field name must not be empty"
    );
    let byte_len = field_text.len();
    let actual_chars = field_text.chars().count();
    assert!(
        actual_chars <= byte_len,
        "field character count must fit in byte length"
    );

    if actual_chars > MAX_ADAPTER_FIELD_CHARS {
        return Err(AdapterContractError::FieldTooLong {
            field: field_name.as_str(),
            max_chars: MAX_ADAPTER_FIELD_CHARS,
            actual_chars,
        });
    }

    if let Some((byte_index, _character)) = field_text
        .char_indices()
        .find(|(_byte_index, character)| *character == NUL_CHARACTER)
    {
        return Err(AdapterContractError::NulCharacter {
            field: field_name.as_str(),
            byte_index,
        });
    }

    Ok(())
}

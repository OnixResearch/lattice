const NUL_CHARACTER: char = '\0';

/// Smallest accepted non-empty adapter field length after trimming.
pub(crate) const MIN_ADAPTER_FIELD_CHARS: usize = 1;

/// Largest accepted adapter field length.
pub(crate) const MAX_ADAPTER_FIELD_CHARS: usize = 8_192;

const _: () = assert!(MAX_ADAPTER_FIELD_CHARS >= MIN_ADAPTER_FIELD_CHARS);

/// A validated adapter field name constant used for error diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FieldName {
    value: &'static str,
}

impl FieldName {
    #[must_use]
    pub(crate) const fn new(value: &'static str) -> Self {
        Self { value }
    }

    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        self.value
    }
}

/// Contract errors raised before data crosses a router boundary.
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
    /// A safe evidence hash failed deterministic validation.
    InvalidEvidenceHash {
        /// Boundary field name.
        field: &'static str,
        /// Human-readable validation reason.
        reason: &'static str,
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
            } => write!(formatter, "{field} must be at most {max_chars} characters; got {actual_chars}"),
            Self::NulCharacter { field, byte_index } => {
                write!(formatter, "{field} contains a NUL character at byte index {byte_index}")
            }
            Self::UnsupportedRouterValue { field, value } => {
                write!(formatter, "{field} contains unsupported model-router value {value}")
            }
            Self::InvalidEvidenceHash { field, reason } => {
                write!(formatter, "{field} is invalid: {reason}")
            }
        }
    }
}

/// Normalizes an adapter field value by trimming whitespace and rejecting
/// empty, too-long, or NUL-containing inputs.
pub(crate) fn normalized_adapter_field(
    field_name: FieldName,
    field_text: &str,
) -> Result<alloc::string::String, AdapterContractError> {
    assert!(
        field_name.as_str().chars().count() >= MIN_ADAPTER_FIELD_CHARS,
        "diagnostic field name must be non-empty"
    );
    assert!(!field_name.as_str().contains(NUL_CHARACTER), "diagnostic field name must not contain NUL");

    let raw_chars = field_text.chars().count();
    let trimmed_text = field_text.trim();
    let visible_chars = trimmed_text.chars().count();
    assert!(visible_chars <= raw_chars, "trimmed field character count must fit in raw character count");

    if visible_chars < MIN_ADAPTER_FIELD_CHARS {
        return Err(AdapterContractError::EmptyField {
            field: field_name.as_str(),
        });
    }

    if raw_chars > MAX_ADAPTER_FIELD_CHARS {
        return Err(AdapterContractError::FieldTooLong {
            field: field_name.as_str(),
            max_chars: MAX_ADAPTER_FIELD_CHARS,
            actual_chars: raw_chars,
        });
    }

    if let Some((byte_index, _character)) =
        field_text.char_indices().find(|(_byte_index, character)| *character == NUL_CHARACTER)
    {
        return Err(AdapterContractError::NulCharacter {
            field: field_name.as_str(),
            byte_index,
        });
    }

    Ok(alloc::string::String::from(trimmed_text))
}

/// Safe BLAKE3 hash used by router receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SafeHash {
    value: alloc::string::String,
}

impl SafeHash {
    /// Character length of a lowercase BLAKE3 receipt hash in hex form.
    pub(crate) const HEX_CHARS: usize = 64;

    /// Builds a validated receipt hash.
    pub(crate) fn new(value: &str) -> Result<Self, AdapterContractError> {
        let field = FieldName::new("router_effect_receipt_hash");
        validate_hash(field, value)?;
        Ok(Self {
            value: alloc::string::String::from(value),
        })
    }

    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

const _: () =
    assert!(SafeHash::HEX_CHARS > MIN_ADAPTER_FIELD_CHARS, "BLAKE3 hash length must exceed a single character");

fn validate_hash(field: FieldName, value: &str) -> Result<(), AdapterContractError> {
    let field_name = field.as_str();
    assert!(
        field_name.chars().count() >= MIN_ADAPTER_FIELD_CHARS,
        "hash diagnostic field name must be non-empty"
    );
    assert!(field_name.starts_with("router_"), "hash diagnostic field name must stay router-scoped");
    assert!(field_name.ends_with("_hash"), "hash diagnostic field name must identify hash material");

    if value.is_empty() {
        return Err(AdapterContractError::EmptyField { field: field_name });
    }
    if value.len() != SafeHash::HEX_CHARS {
        return Err(AdapterContractError::InvalidEvidenceHash {
            field: field_name,
            reason: "effect hash must use the configured BLAKE3 hex length",
        });
    }
    if !value.chars().all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase()) {
        return Err(AdapterContractError::InvalidEvidenceHash {
            field: field_name,
            reason: "effect hash must contain lowercase hex only",
        });
    }
    Ok(())
}

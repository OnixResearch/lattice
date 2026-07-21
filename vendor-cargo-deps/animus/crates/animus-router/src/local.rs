//! Router-local adapter field validation.
//!
//! These types mirror a small subset of `animus-adapters::validation` and
//! `animus-adapters::effect` so that `animus-router` can be a standalone crate
//! with no circular dependency on `animus-adapters`. When `animus-adapters`
//! re-exports the router types, only `animus-adapters` depends on
//! `animus-router` — never the reverse.

use alloc::string::String;

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
    /// Creates a new field name constant.
    #[must_use]
    pub(crate) const fn new(value: &'static str) -> Self {
        Self { value }
    }

    /// Returns the static field name string.
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        self.value
    }
}

/// Contract errors raised before data crosses a router boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AdapterContractError {
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
        value: String,
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
            } => write!(
                formatter,
                "{field} must be at most {max_chars} characters; got {actual_chars}"
            ),
            Self::NulCharacter { field, byte_index } => write!(
                formatter,
                "{field} contains a NUL character at byte index {byte_index}"
            ),
            Self::UnsupportedRouterValue { field, value } => write!(
                formatter,
                "{field} contains unsupported model-router value {value}"
            ),
            Self::InvalidEvidenceHash { field, reason } => {
                write!(formatter, "{field} is invalid: {reason}")
            }
        }
    }
}

/// Normalizes an adapter field value by trimming whitespace and rejecting
/// empty, too-long, or NUL-containing inputs.
///
/// # Errors
///
/// Returns [`AdapterContractError`] when the field is empty, exceeds
/// [`MAX_ADAPTER_FIELD_CHARS`], or contains a NUL character.
pub(crate) fn normalized_adapter_field(
    field_name: FieldName,
    field_text: &str,
) -> Result<String, AdapterContractError> {
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

    if raw_chars > MAX_ADAPTER_FIELD_CHARS {
        return Err(AdapterContractError::FieldTooLong {
            field: field_name.as_str(),
            max_chars: MAX_ADAPTER_FIELD_CHARS,
            actual_chars: raw_chars,
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

    Ok(String::from(field_text.trim()))
}

/// Safe BLAKE3 hash used by router receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SafeHash {
    value: String,
}

impl SafeHash {
    /// Character length of a lowercase BLAKE3 receipt hash in hex form.
    pub(crate) const HEX_CHARS: usize = 64;

    /// Builds a validated receipt hash.
    ///
    /// # Errors
    ///
    /// Returns [`AdapterContractError`] when the hash is empty, has the wrong
    /// length, contains uppercase hex, or contains non-hex characters.
    pub(crate) fn new(value: &str) -> Result<Self, AdapterContractError> {
        let field = FieldName::new("router_effect_receipt_hash");
        validate_hash(field, value)?;
        Ok(Self {
            value: String::from(value),
        })
    }

    /// Builds a safe hash directly from raw bytes with BLAKE3.
    #[must_use]
    pub(crate) fn from_blake3_bytes(bytes: &[u8]) -> Self {
        Self {
            value: alloc::format!("{}", blake3::hash(bytes).to_hex()),
        }
    }

    /// Returns the validated hash text.
    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

fn validate_hash(
    field: FieldName,
    value: &str,
) -> Result<(), AdapterContractError> {
    if value.is_empty() {
        return Err(AdapterContractError::EmptyField {
            field: field.as_str(),
        });
    }
    if value.len() != SafeHash::HEX_CHARS {
        return Err(AdapterContractError::InvalidEvidenceHash {
            field: field.as_str(),
            reason: "effect hash must use the configured BLAKE3 hex length",
        });
    }
    if !value.chars().all(|c| c.is_ascii_hexdigit() && c.is_ascii_lowercase()) {
        return Err(AdapterContractError::InvalidEvidenceHash {
            field: field.as_str(),
            reason: "effect hash must contain lowercase hex only",
        });
    }
    Ok(())
}
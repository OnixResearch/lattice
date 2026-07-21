type Text = super::Text;

/// Prompt resource validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A required descriptor field was absent.
    MissingField {
        /// Missing field name.
        field: &'static str,
    },
    /// A shared adapter field contract failed.
    Contract(crate::AdapterContractError),
    /// A name-like field used an unsupported character or separator shape.
    MalformedName {
        /// Field containing the malformed name.
        field: &'static str,
        /// Malformed normalized value.
        value: Text,
    },
    /// A description exceeded the safe summary bound.
    DescriptionTooLong {
        /// Maximum accepted description characters.
        max_chars: usize,
        /// Actual description characters.
        actual_chars: usize,
    },
    /// Provenance looked like an unsafe host path rather than a safe label.
    MalformedProvenance {
        /// Malformed normalized provenance value.
        value: Text,
    },
    /// Content identity failed lowercase BLAKE3 validation.
    InvalidContentIdentity {
        /// Field containing the unsafe identity.
        field: &'static str,
        /// Human-readable validation reason.
        reason: &'static str,
    },
    /// A descriptor label used an unsupported value.
    UnsupportedValue {
        /// Field containing the unsupported value.
        field: &'static str,
        /// Unsupported normalized value.
        value: Text,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingField { field } => write!(formatter, "{field} is required"),
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::MalformedName { field, value } => {
                write!(formatter, "{field} has malformed resource name {value}")
            }
            Self::DescriptionTooLong {
                max_chars,
                actual_chars,
            } => write!(
                formatter,
                "resource description supports at most {max_chars} characters; got {actual_chars}"
            ),
            Self::MalformedProvenance { value } => {
                write!(
                    formatter,
                    "resource provenance must be a safe label: {value}"
                )
            }
            Self::InvalidContentIdentity { field, reason } => {
                write!(formatter, "{field} is invalid: {reason}")
            }
            Self::UnsupportedValue { field, value } => {
                write!(formatter, "{field} has unsupported resource value {value}")
            }
        }
    }
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

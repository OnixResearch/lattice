/// Permission policy validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A required field was absent.
    MissingField {
        /// Missing field name.
        field: &'static str,
    },
    /// Adapter-boundary validation failed.
    Contract(crate::AdapterContractError),
    /// Unsupported descriptor label.
    UnsupportedValue {
        /// Field containing the unsupported value.
        field: &'static str,
        /// Unsupported normalized value.
        value: super::Text,
    },
    /// Policy descriptor was not trusted by host policy.
    UntrustedDescriptor {
        /// Untrusted descriptor scope.
        trust_scope: super::TrustScope,
    },
    /// Precedence was outside supported bounds.
    PrecedenceOutOfRange {
        /// Minimum supported precedence.
        min: u16,
        /// Maximum supported precedence.
        max: u16,
        /// Actual precedence.
        actual: u16,
    },
    /// Pattern syntax was malformed.
    InvalidPattern {
        /// Field containing the malformed pattern.
        field: &'static str,
    },
    /// Redaction collection exceeded receipt bounds.
    TooManyRedactions {
        /// Maximum supported redaction markers.
        max_items: usize,
        /// Actual marker count.
        actual_items: usize,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingField { field } => write!(formatter, "{field} is required"),
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::UnsupportedValue { field, value } => {
                write!(formatter, "{field} has unsupported permission value {value}")
            }
            Self::UntrustedDescriptor { trust_scope } => {
                write!(formatter, "{} permission descriptor is untrusted", trust_scope.as_str())
            }
            Self::PrecedenceOutOfRange { min, max, actual } => {
                write!(formatter, "permission precedence must be between {min} and {max}; got {actual}")
            }
            Self::InvalidPattern { field } => {
                write!(formatter, "{field} has malformed permission pattern syntax")
            }
            Self::TooManyRedactions {
                max_items,
                actual_items,
            } => write!(formatter, "permission receipts support at most {max_items} redactions; got {actual_items}"),
        }
    }
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

/// Exec-policy validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A shared adapter-boundary contract check failed.
    Contract(crate::AdapterContractError),
    /// A required field was absent.
    MissingField {
        /// Boundary field name.
        field: &'static str,
    },
    /// A decision label was not supported.
    UnsupportedDecision {
        /// Unsupported decision label after normalization.
        value: super::Text,
    },
    /// A collection exceeded the exec-policy bound.
    TooManyItems {
        /// Boundary collection field name.
        field: &'static str,
        /// Maximum accepted item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A positive rule example failed to match its rule prefix.
    PositiveExampleMismatch {
        /// Validated rule identity.
        rule_id: super::Text,
        /// BLAKE3 command fingerprint for the failing example.
        command_fingerprint: super::Text,
    },
    /// A negative rule example unexpectedly matched its rule prefix.
    NegativeExampleMatched {
        /// Validated rule identity.
        rule_id: super::Text,
        /// BLAKE3 command fingerprint for the failing example.
        command_fingerprint: super::Text,
    },
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

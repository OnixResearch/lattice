/// Retrieval-budget validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A required field is missing.
    MissingField {
        /// Field name.
        field: &'static str,
    },
    /// A nested adapter field failed validation.
    InvalidField {
        /// Adapter validation error.
        error: crate::AdapterContractError,
    },
    /// A condition list is empty.
    EmptyConditionList {
        /// Field name.
        field: &'static str,
    },
    /// A condition list exceeds the safe entry limit.
    ConditionLimit {
        /// Field name.
        field: &'static str,
        /// Maximum accepted entries.
        max: usize,
        /// Actual entries.
        actual: usize,
    },
    /// Optional max retrievals was zero.
    InvalidLimit,
    /// Host policy requires a budget for this capability.
    MissingBudget {
        /// Retrieval capability missing a budget.
        capability: super::contract::Capability,
    },
    /// Missing-evidence names are required for the current status.
    MissingEvidenceNames,
    /// Raw fetched content was embedded in receipt metadata.
    RawBodyEmbedded,
    /// Safe reference contained a host path or traversal marker.
    UnsafeReference {
        /// Unsafe reference value.
        reference: alloc::string::String,
    },
    /// Optional content hash was not lowercase BLAKE3 hex.
    InvalidHash,
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::InvalidField { error }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingField { field } => write!(formatter, "{field} is required"),
            Self::InvalidField { error } => write!(formatter, "invalid retrieval field: {error}"),
            Self::EmptyConditionList { field } => write!(formatter, "{field} must not be empty"),
            Self::ConditionLimit { field, max, actual } => write!(
                formatter,
                "{field} accepts at most {max} entries; got {actual}"
            ),
            Self::InvalidLimit => write!(formatter, "max retrievals must be non-zero"),
            Self::MissingBudget { capability } => write!(
                formatter,
                "retrieval budget is required for {}",
                capability.as_str()
            ),
            Self::MissingEvidenceNames => write!(formatter, "missing evidence names are required"),
            Self::RawBodyEmbedded => {
                write!(formatter, "raw retrieved body is not receipt metadata")
            }
            Self::UnsafeReference { reference } => {
                write!(formatter, "retrieval reference is unsafe: {reference}")
            }
            Self::InvalidHash => write!(formatter, "content hash must be lowercase BLAKE3 hex"),
        }
    }
}

/// Agent-definition validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A required manifest or receipt field was absent.
    MissingField {
        /// Missing field name.
        field: &'static str,
    },
    /// A field failed adapter-boundary validation.
    Contract(crate::AdapterContractError),
    /// A manifest label used an unsupported value.
    UnsupportedValue {
        /// Field containing the unsupported value.
        field: &'static str,
        /// Unsupported normalized value.
        value: super::Text,
    },
    /// A collection exceeded the bounded item limit.
    TooManyItems {
        /// Field containing too many items.
        field: &'static str,
        /// Maximum supported items.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// Prompt fragments exceeded the bounded prompt budget.
    PromptTooLong {
        /// Maximum prompt-fragment characters.
        max_chars: usize,
        /// Actual prompt-fragment characters.
        actual_chars: usize,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingField { field } => write!(formatter, "{field} is required"),
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::UnsupportedValue { field, value } => {
                write!(formatter, "{field} has unsupported agent value {value}")
            }
            Self::TooManyItems {
                field,
                max_items,
                actual_items,
            } => write!(formatter, "{field} supports at most {max_items} items; got {actual_items}"),
            Self::PromptTooLong {
                max_chars,
                actual_chars,
            } => write!(formatter, "agent prompt fragments support at most {max_chars} characters; got {actual_chars}"),
        }
    }
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

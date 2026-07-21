/// Error returned by Steel loop fabric operations.
#[derive(Debug)]
pub enum Failure {
    /// Adapter DTO validation failed after a Steel decision was decoded.
    AdapterContract {
        /// Adapter contract validation error.
        source: animus_adapters::AdapterContractError,
    },
    /// Required loop input was empty.
    EmptyInput {
        /// Input field name.
        field: &'static str,
    },
    /// Steel code failed while loading the loop program.
    Load {
        /// Steel interpreter error.
        source: steel::SteelErr,
    },
    /// Steel code failed while evaluating one turn.
    Evaluate {
        /// Steel interpreter error.
        source: steel::SteelErr,
    },
    /// Steel code returned no decision value.
    MissingDecision,
    /// Steel code returned a value that is not a valid loop decision.
    InvalidDecision {
        /// Human-readable validation reason.
        reason: String,
    },
}

impl From<animus_adapters::AdapterContractError> for Failure {
    fn from(source: animus_adapters::AdapterContractError) -> Self {
        Self::AdapterContract { source }
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AdapterContract { source } => {
                write!(formatter, "adapter contract failed: {source}")
            }
            Self::EmptyInput { field } => write!(formatter, "loop input {field} must not be empty"),
            Self::Load { source } => {
                write!(formatter, "steel loop program failed to load: {source}")
            }
            Self::Evaluate { source } => {
                write!(formatter, "steel loop turn failed to evaluate: {source}")
            }
            Self::MissingDecision => formatter.write_str("steel loop returned no decision"),
            Self::InvalidDecision { reason } => {
                write!(formatter, "invalid steel loop decision: {reason}")
            }
        }
    }
}

impl std::error::Error for Failure {}

const FIELD: crate::validation::FieldName = crate::validation::FieldName::new("dataspace_assertion_status");

const ACTIVE: &str = "active";
const COMPLETED: &str = "completed";
const CANCELLED: &str = "cancelled";
const EXPIRED: &str = "expired";
const FAILED: &str = "failed";
const BLOCKED: &str = "blocked";

/// Safe lifecycle status class for a dataspace assertion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    /// Assertion is active.
    Active,
    /// Assertion reached a completed state.
    Completed,
    /// Assertion was cancelled.
    Cancelled,
    /// Assertion expired or was marked stale.
    Expired,
    /// Assertion reached a failed state.
    Failed,
    /// Assertion is blocked by policy, dependencies, or host state.
    Blocked,
}

impl Value {
    /// Parses a stable assertion-status label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(FIELD, value, Self::from_normalized)
    }

    fn from_normalized(normalized: &str) -> Option<Self> {
        match normalized {
            ACTIVE => Some(Self::Active),
            COMPLETED => Some(Self::Completed),
            CANCELLED => Some(Self::Cancelled),
            EXPIRED => Some(Self::Expired),
            FAILED => Some(Self::Failed),
            BLOCKED => Some(Self::Blocked),
            _ => None,
        }
    }
}

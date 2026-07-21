#![allow(unknown_lints, path_segment_repetition)]

const DEADLINE_FIELD: crate::FieldName = crate::FieldName::new("router_timeout_class");
const FALLBACK_FIELD: crate::FieldName = crate::FieldName::new("router_fallback_policy");

/// Timeout/deadline class for bounded host execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeadlineClass {
    /// Short deadline bucket.
    Short,
    /// Standard deadline bucket.
    Standard,
    /// Extended deadline bucket.
    Extended,
}

impl DeadlineClass {
    /// Parses a deadline class.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        crate::parse_value(DEADLINE_FIELD, value, |normalized| match normalized {
            crate::policy::DEADLINE_SHORT => Some(Self::Short),
            crate::policy::DEADLINE_STANDARD => Some(Self::Standard),
            crate::policy::DEADLINE_EXTENDED => Some(Self::Extended),
            _ => None,
        })
    }

    /// Returns stable deadline label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Short => crate::policy::DEADLINE_SHORT,
            Self::Standard => crate::policy::DEADLINE_STANDARD,
            Self::Extended => crate::policy::DEADLINE_EXTENDED,
        }
    }
}

/// Fallback policy for collaborative route failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackPolicy {
    /// Deny fallback.
    Deny,
    /// Return best valid evidence when policy permits.
    BestValidEvidence,
    /// Fall back to a safe single model.
    SafeSingle,
}

impl FallbackPolicy {
    /// Parses fallback policy.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        crate::parse_value(FALLBACK_FIELD, value, |normalized| match normalized {
            crate::policy::FALLBACK_DENY => Some(Self::Deny),
            crate::policy::FALLBACK_BEST_VALID_EVIDENCE => Some(Self::BestValidEvidence),
            crate::policy::FALLBACK_SAFE_SINGLE => Some(Self::SafeSingle),
            _ => None,
        })
    }

    /// Returns stable fallback label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deny => crate::policy::FALLBACK_DENY,
            Self::BestValidEvidence => crate::policy::FALLBACK_BEST_VALID_EVIDENCE,
            Self::SafeSingle => crate::policy::FALLBACK_SAFE_SINGLE,
        }
    }
}

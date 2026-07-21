const DEADLINE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_timeout_class");
const FALLBACK_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_fallback_policy");
const ZERO_COUNT: u32 = 0;

const DEADLINE_SHORT: &str = "short";
const DEADLINE_STANDARD: &str = "standard";
const DEADLINE_EXTENDED: &str = "extended";
const FALLBACK_DENY: &str = "deny";
const FALLBACK_BEST_VALID_EVIDENCE: &str = "best_valid_evidence";
const FALLBACK_SAFE_SINGLE: &str = "safe_single";

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
        super::parse_value(DEADLINE_FIELD, value, |normalized| match normalized {
            DEADLINE_SHORT => Some(Self::Short),
            DEADLINE_STANDARD => Some(Self::Standard),
            DEADLINE_EXTENDED => Some(Self::Extended),
            _ => None,
        })
    }

    /// Returns stable deadline label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Short => DEADLINE_SHORT,
            Self::Standard => DEADLINE_STANDARD,
            Self::Extended => DEADLINE_EXTENDED,
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
        super::parse_value(FALLBACK_FIELD, value, |normalized| match normalized {
            FALLBACK_DENY => Some(Self::Deny),
            FALLBACK_BEST_VALID_EVIDENCE => Some(Self::BestValidEvidence),
            FALLBACK_SAFE_SINGLE => Some(Self::SafeSingle),
            _ => None,
        })
    }

    /// Returns stable fallback label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deny => FALLBACK_DENY,
            Self::BestValidEvidence => FALLBACK_BEST_VALID_EVIDENCE,
            Self::SafeSingle => FALLBACK_SAFE_SINGLE,
        }
    }
}

/// Hard execution bounds validated before host execution begins.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bounds {
    max_candidates: u32,
    max_parallelism: u32,
    min_success_quorum: u32,
    deadline: DeadlineClass,
    max_synthesis_attempts: u32,
    max_repair_attempts: u32,
    fallback_policy: FallbackPolicy,
}

impl Bounds {
    /// Builds validated execution bounds.
    ///
    /// # Errors
    ///
    /// Returns [`super::ValidationError`] for unbounded fan-out, impossible quorum,
    /// missing deadline, unlimited repair, or disallowed fallback shapes.
    pub fn new(draft: BoundsDraft<'_>) -> Result<Self, super::ValidationError> {
        if draft.max_candidates == ZERO_COUNT || draft.max_parallelism == ZERO_COUNT {
            return Err(super::ValidationError::UnboundedFanOut);
        }
        if draft.min_success_quorum == ZERO_COUNT {
            return Err(super::ValidationError::ZeroQuorum);
        }
        if draft.min_success_quorum > draft.max_candidates {
            return Err(super::ValidationError::ImpossibleQuorum);
        }
        let deadline =
            DeadlineClass::parse(draft.timeout_class).map_err(super::ValidationError::Contract)?;
        let max_repair_attempts = draft
            .max_repair_attempts
            .ok_or(super::ValidationError::UnlimitedRepair)?;

        Ok(Self {
            max_candidates: draft.max_candidates,
            max_parallelism: draft.max_parallelism,
            min_success_quorum: draft.min_success_quorum,
            deadline,
            max_synthesis_attempts: draft.max_synthesis_attempts,
            max_repair_attempts,
            fallback_policy: FallbackPolicy::parse(draft.fallback_policy)
                .map_err(super::ValidationError::Contract)?,
        })
    }

    /// Returns max candidates.
    #[must_use]
    pub const fn max_candidates(&self) -> u32 {
        self.max_candidates
    }

    /// Returns max parallelism.
    #[must_use]
    pub const fn max_parallelism(&self) -> u32 {
        self.max_parallelism
    }

    /// Returns min-success quorum.
    #[must_use]
    pub const fn min_success_quorum(&self) -> u32 {
        self.min_success_quorum
    }

    /// Returns deadline class.
    #[must_use]
    pub const fn timeout_class(&self) -> DeadlineClass {
        self.deadline
    }

    /// Returns max synthesis attempts.
    #[must_use]
    pub const fn max_synthesis_attempts(&self) -> u32 {
        self.max_synthesis_attempts
    }

    /// Returns max repair attempts.
    #[must_use]
    pub const fn max_repair_attempts(&self) -> u32 {
        self.max_repair_attempts
    }

    /// Returns fallback policy.
    #[must_use]
    pub const fn fallback_policy(&self) -> FallbackPolicy {
        self.fallback_policy
    }
}

/// Borrowed fields used to validate execution bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundsDraft<'a> {
    /// Hard candidate count cap.
    pub max_candidates: u32,
    /// Hard parallelism cap.
    pub max_parallelism: u32,
    /// Minimum successful candidates required.
    pub min_success_quorum: u32,
    /// Timeout class label.
    pub timeout_class: &'a str,
    /// Hard synthesis attempt cap.
    pub max_synthesis_attempts: u32,
    /// Bounded repair attempt cap; `None` is rejected as unlimited repair.
    pub max_repair_attempts: Option<u32>,
    /// Fallback policy label.
    pub fallback_policy: &'a str,
}

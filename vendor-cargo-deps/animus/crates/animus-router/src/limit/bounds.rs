#![allow(unknown_lints, path_segment_repetition)]

use super::DeadlineClass;
use super::FallbackPolicy;
use super::ZERO_COUNT;

/// Hard execution bounds validated before host execution begins.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bounds {
    pub(super) max_candidates: u32,
    pub(super) max_parallelism: u32,
    pub(super) min_success_quorum: u32,
    pub(super) deadline: DeadlineClass,
    pub(super) max_synthesis_attempts: u32,
    pub(super) max_repair_attempts: u32,
    pub(super) fallback_policy: FallbackPolicy,
}

impl Bounds {
    /// Builds validated execution bounds.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ValidationError`] for unbounded fan-out, impossible quorum,
    /// missing deadline, unlimited repair, or disallowed fallback shapes.
    pub fn new(draft: BoundsDraft<'_>) -> Result<Self, crate::ValidationError> {
        if draft.max_candidates == ZERO_COUNT || draft.max_parallelism == ZERO_COUNT {
            return Err(crate::ValidationError::UnboundedFanOut);
        }
        if draft.min_success_quorum == ZERO_COUNT {
            return Err(crate::ValidationError::ZeroQuorum);
        }
        if draft.min_success_quorum > draft.max_candidates {
            return Err(crate::ValidationError::ImpossibleQuorum);
        }
        let deadline = DeadlineClass::parse(draft.timeout_class).map_err(crate::ValidationError::Contract)?;
        let max_repair_attempts = draft.max_repair_attempts.ok_or(crate::ValidationError::UnlimitedRepair)?;

        Ok(Self {
            max_candidates: draft.max_candidates,
            max_parallelism: draft.max_parallelism,
            min_success_quorum: draft.min_success_quorum,
            deadline,
            max_synthesis_attempts: draft.max_synthesis_attempts,
            max_repair_attempts,
            fallback_policy: FallbackPolicy::parse(draft.fallback_policy).map_err(crate::ValidationError::Contract)?,
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

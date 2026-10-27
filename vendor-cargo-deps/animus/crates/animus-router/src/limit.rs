#![allow(non_trait_imports, missing_docs)]

mod bounds;
mod budget;
mod class;

pub(super) const ZERO_COUNT: u32 = 0;

pub use bounds::{Bounds, BoundsDraft};
pub use budget::BudgetLedger;
pub use class::{DeadlineClass, FallbackPolicy};

#[cfg(test)]
pub(crate) fn test_bounds(fixture: crate::test_support::BoundsFixture) -> Bounds {
    Bounds {
        max_candidates: fixture.max_candidates,
        max_parallelism: fixture.max_parallelism,
        min_success_quorum: fixture.min_success_quorum,
        deadline: fixture.timeout_class,
        max_synthesis_attempts: fixture.max_synthesis_attempts,
        max_repair_attempts: fixture.max_repair_attempts,
        fallback_policy: fixture.fallback_policy,
    }
}

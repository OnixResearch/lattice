//! Pure identity and retention decisions for bounded tool-result reuse.
//!
//! Callers supply every request field, bound, generation, and retention fact.
//! This module reads no clock, process state, memory pressure, or scheduler state.

mod decision;
mod identity;

pub use decision::{
    DecisionError, Entry, InsertDisposition, InsertPlan, Policy, PromotionFacts, ReleasePlan,
    TrimPlan, decide_promotion, plan_insert, plan_trim, select_oldest,
};
pub use identity::{DIGEST_BYTES, Digest, Projection, project_key};

#[cfg(test)]
mod tests;

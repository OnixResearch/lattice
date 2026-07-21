//! Router-receipt evidence assembly, validation, and leak screening.
//!
//! Keep this in sync with `src/evidence.rs` so builds that resolve the
//! directory-style module entry point still expose the full evidence surface.

mod label;
mod plan;
mod record;
mod screen;

pub use label::{DurationBucket, QuorumClass, Redaction};
pub use record::{Receipt, ReceiptDraft, ReceiptLeak, validate_receipt_text};
pub use screen::{
    ExecutionPlanError, FallbackChain, FallbackStep, RoleStep, RouterExecutionPlan,
    materialize_execution_plan, materialize_execution_plan_with_rotation,
};

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct FallbackStepFixture<'a> {
    pub(crate) role: crate::Role,
    pub(crate) model_label: &'a str,
    pub(crate) backend_label: &'a str,
    pub(crate) is_local: bool,
}

#[cfg(test)]
pub(crate) fn test_fallback_step(fixture: FallbackStepFixture<'_>) -> FallbackStep {
    FallbackStep {
        role: fixture.role,
        model_label: alloc::string::String::from(fixture.model_label),
        backend: alloc::string::String::from(fixture.backend_label),
        local: fixture.is_local,
    }
}

#[cfg(test)]
pub(crate) fn test_fallback_chain(step_list: alloc::vec::Vec<FallbackStep>) -> FallbackChain {
    FallbackChain { steps: step_list }
}

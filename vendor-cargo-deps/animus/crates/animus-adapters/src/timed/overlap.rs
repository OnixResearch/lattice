use super::ScheduleConcurrencyPolicy;

/// Concurrency decision for overlapping runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduledConcurrencyDecision {
    /// Start immediately.
    Start,
    /// Serialize behind active run.
    Serialize,
    /// Start in isolated workspace.
    Isolate,
    /// Deny due to overlap.
    Deny,
}

/// Concurrency evaluation input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduledConcurrencyInput {
    /// Concurrency policy.
    pub policy: ScheduleConcurrencyPolicy,
    /// Whether another run is active.
    pub active_run: bool,
    /// Whether active run may write.
    pub active_run_writes: bool,
    /// Whether new run may write.
    pub new_run_writes: bool,
}

/// Evaluates overlapping scheduled-run concurrency.
#[must_use]
pub const fn evaluate_scheduled_concurrency(input: ScheduledConcurrencyInput) -> ScheduledConcurrencyDecision {
    if !input.active_run || !input.active_run_writes || !input.new_run_writes {
        return ScheduledConcurrencyDecision::Start;
    }
    match input.policy {
        ScheduleConcurrencyPolicy::Serialize => ScheduledConcurrencyDecision::Serialize,
        ScheduleConcurrencyPolicy::Isolate => ScheduledConcurrencyDecision::Isolate,
        ScheduleConcurrencyPolicy::Deny => ScheduledConcurrencyDecision::Deny,
    }
}

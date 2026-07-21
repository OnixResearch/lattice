/// Policy-selected outcome for repeated identical actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepeatedActionPolicy {
    /// Return coaching status without dispatching.
    Coach,
    /// Require a prompt before dispatching.
    Ask,
    /// Deny before dispatching.
    Deny,
    /// Ignore the repeated-action signal.
    Ignore,
}

impl RepeatedActionPolicy {
    /// Returns stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Coach => COACH_REPEAT_LABEL,
            Self::Ask => ASK_REPEAT_LABEL,
            Self::Deny => DENY_REPEAT_LABEL,
            Self::Ignore => IGNORE_REPEAT_LABEL,
        }
    }
}

/// Borrowed repeated-action input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepeatedActionInput<'a> {
    /// Guarded action class.
    pub action: super::super::GuardedAction,
    /// Safe resource identity hash.
    pub resource_hash: &'a crate::SafeHash,
    /// Number of prior identical observations.
    pub prior_observations: u32,
    /// Threshold at which the repeated-action policy applies.
    pub threshold: u32,
    /// Whether this action class is destructive under active policy.
    pub destructive: bool,
    /// Selected repeated-action policy.
    pub policy: RepeatedActionPolicy,
}

/// Repeated-action detection status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepeatedActionStatus {
    /// Threshold has not been reached.
    NotRepeated,
    /// Repeated action should produce coaching, not dispatch.
    Coach,
    /// Repeated action requires a prompt before dispatch.
    Ask,
    /// Repeated action is denied before dispatch.
    Denied,
    /// Repeated action signal was ignored by policy.
    Ignored,
}

/// Repeated-action detection output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepeatedActionResolution {
    status: RepeatedActionStatus,
    adapter_dispatched: bool,
}

impl RepeatedActionResolution {
    /// Returns repeated-action status.
    #[must_use]
    pub const fn status(self) -> RepeatedActionStatus {
        self.status
    }

    /// Returns whether an adapter dispatch has occurred inside detection.
    #[must_use]
    pub const fn adapter_dispatched(self) -> bool {
        self.adapter_dispatched
    }
}

/// Detects repeated identical action identities without dispatching adapters.
#[must_use]
// r[impl agent_harness_runtime.zerostack_permission_kernel.repeated_action]
// r[impl agent_harness_runtime.zerostack_permission_kernel.repeated_action.deny]
pub fn detect_repeated_action(input: RepeatedActionInput<'_>) -> RepeatedActionResolution {
    assert!(
        input.threshold >= MIN_REPEAT_THRESHOLD,
        "repeated-action threshold must be positive"
    );
    assert!(
        !input.resource_hash.as_str().is_empty(),
        "resource hash must be present"
    );
    if input
        .prior_observations
        .saturating_add(MIN_REPEAT_THRESHOLD)
        < input.threshold
    {
        return repeated_resolution(RepeatedActionStatus::NotRepeated);
    }
    match input.policy {
        RepeatedActionPolicy::Coach => repeated_resolution(RepeatedActionStatus::Coach),
        RepeatedActionPolicy::Ask => repeated_resolution(RepeatedActionStatus::Ask),
        RepeatedActionPolicy::Deny => {
            if input.destructive {
                repeated_resolution(RepeatedActionStatus::Denied)
            } else {
                repeated_resolution(RepeatedActionStatus::Coach)
            }
        }
        RepeatedActionPolicy::Ignore => repeated_resolution(RepeatedActionStatus::Ignored),
    }
}

fn repeated_resolution(status: RepeatedActionStatus) -> RepeatedActionResolution {
    RepeatedActionResolution {
        status,
        adapter_dispatched: false,
    }
}

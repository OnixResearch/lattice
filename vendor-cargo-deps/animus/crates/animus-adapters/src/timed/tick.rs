use super::{MIN_INTERVAL_TICKS, ScheduleCadence, ScheduleError, ScheduledWorkflow};

/// Explicit next-run planning input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NextRunInput<'a> {
    /// Validated schedule.
    pub schedule: &'a ScheduledWorkflow,
    /// Explicit current logical tick.
    pub current_tick: u64,
    /// Explicit recurrence interval in ticks.
    pub interval_ticks: u64,
}

/// Deterministic next-run plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NextRunPlan {
    next_tick: u64,
}

impl NextRunPlan {
    /// Returns the planned next logical tick.
    #[must_use]
    pub const fn next_tick(self) -> u64 {
        self.next_tick
    }
}

/// Plans the next run over explicit logical clock inputs only.
///
/// # Errors
///
/// Returns [`ScheduleError`] when interval inputs are invalid.
pub fn plan_next_run(input: NextRunInput<'_>) -> Result<NextRunPlan, ScheduleError> {
    if input.interval_ticks < MIN_INTERVAL_TICKS {
        return Err(ScheduleError::InvalidPlanningInput);
    }
    let next_tick = match input.schedule.cadence() {
        ScheduleCadence::RunNow => input.current_tick,
        ScheduleCadence::Hourly | ScheduleCadence::Daily => input
            .current_tick
            .checked_add(input.interval_ticks)
            .ok_or(ScheduleError::InvalidPlanningInput)?,
    };
    Ok(NextRunPlan { next_tick })
}

/// Lifecycle phase for one supervised cycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CyclePhase {
    /// No calls prepared.
    New,
    /// Exact call plans prepared.
    Prepared,
    /// Guarded parallel work completed and awaits commit.
    ReadyToCommit,
    /// Human input is required; the cycle is not terminal.
    Parked,
    /// Every commit completed.
    Finished,
    /// At least one guarded run failed.
    Failed,
}

/// Bounded supervised cycle state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cycle {
    identity: alloc::string::String,
    max_calls: u32,
    call_ids: alloc::vec::Vec<alloc::string::String>,
    phase: CyclePhase,
}

impl Cycle {
    /// Builds a new empty supervised cycle.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error::InvalidSupervision`] for blank identity or zero bound.
    pub fn new(cycle_id: &str, max_calls: u32) -> Result<Self, super::Error> {
        if cycle_id.trim().is_empty() || max_calls == 0 {
            return Err(super::Error::InvalidSupervision);
        }
        Ok(Self {
            identity: alloc::string::String::from(cycle_id),
            max_calls,
            call_ids: alloc::vec::Vec::new(),
            phase: CyclePhase::New,
        })
    }

    /// Returns the cycle identity.
    #[must_use]
    pub fn cycle_id(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the current phase.
    #[must_use]
    pub const fn phase(&self) -> CyclePhase {
        self.phase
    }

    /// Returns exact prepared call identities.
    #[must_use]
    pub fn call_ids(&self) -> &[alloc::string::String] {
        &self.call_ids
    }
}

/// Guarded run result for one prepared call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunStatus {
    /// Guarded work produced a commit candidate.
    Completed,
    /// Guarded work needs a human decision.
    NeedsHuman,
    /// Guarded work failed without commit.
    Failed,
}

/// Supplied guarded-run observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunObservation {
    /// Exact prepared call identity.
    pub call_id: alloc::string::String,
    /// Guarded run status.
    pub status: RunStatus,
    /// Optional bounded telemetry.
    pub telemetry: Option<super::ResourceTelemetry>,
}

/// Commit-boundary observation supplied by the shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuardObservation {
    /// Whether the synchronization guard is released.
    pub guard_released: bool,
    /// Whether any user side effect ran while guarded.
    pub side_effects_inside_guard: bool,
    /// Number of exact calls committed after guarded work.
    pub committed_call_count: u32,
}

/// Prepares exact calls for a new cycle without running effects.
///
/// # Errors
///
/// Returns [`super::Error::InvalidSupervision`] for invalid phase, bounds, or identities.
pub fn prepare_cycle(cycle: &Cycle, call_ids: &[&str]) -> Result<Cycle, super::Error> {
    let call_count = u32::try_from(call_ids.len()).map_err(|_| super::Error::InvalidSupervision)?;
    if cycle.phase != CyclePhase::New || call_count == 0 || call_count > cycle.max_calls {
        return Err(super::Error::InvalidSupervision);
    }
    let mut prepared = alloc::vec::Vec::with_capacity(call_ids.len());
    for call_id in call_ids {
        if call_id.trim().is_empty() || prepared.iter().any(|prior| prior == call_id) {
            return Err(super::Error::InvalidSupervision);
        }
        prepared.push(alloc::string::String::from(*call_id));
    }
    Ok(Cycle {
        identity: cycle.identity.clone(),
        max_calls: cycle.max_calls,
        call_ids: prepared,
        phase: CyclePhase::Prepared,
    })
}

/// Observes a real parallel batch after every worker joined.
///
/// # Errors
///
/// Returns [`super::Error::InvalidSupervision`] unless each prepared call has
/// one exact observation and the shell confirms parallel execution.
pub fn observe_batch(
    cycle: &Cycle,
    observations: &[RunObservation],
    parallel_execution: bool,
) -> Result<Cycle, super::Error> {
    validate_observations(cycle, observations, parallel_execution)?;
    debug_assert_eq!(observations.len(), cycle.call_ids.len());
    debug_assert!(parallel_execution);
    let phase = if observations
        .iter()
        .any(|observation| observation.status == RunStatus::Failed)
    {
        CyclePhase::Failed
    } else if observations
        .iter()
        .any(|observation| observation.status == RunStatus::NeedsHuman)
    {
        CyclePhase::Parked
    } else {
        CyclePhase::ReadyToCommit
    };
    Ok(Cycle {
        phase,
        ..cycle.clone()
    })
}

/// Parks a prepared cycle without presenting terminal completion.
///
/// # Errors
///
/// Returns [`super::Error::InvalidSupervision`] from any non-prepared phase.
pub fn park_cycle(cycle: &Cycle) -> Result<Cycle, super::Error> {
    if cycle.phase != CyclePhase::Prepared {
        return Err(super::Error::InvalidSupervision);
    }
    Ok(Cycle {
        phase: CyclePhase::Parked,
        ..cycle.clone()
    })
}

/// Commits a ready cycle only after guard release and exact call commits.
///
/// # Errors
///
/// Returns [`super::Error::InvalidSupervision`] for early, guarded, partial,
/// or duplicate commit observations.
pub fn commit_cycle(cycle: &Cycle, observation: GuardObservation) -> Result<Cycle, super::Error> {
    let call_count =
        u32::try_from(cycle.call_ids.len()).map_err(|_| super::Error::InvalidSupervision)?;
    if cycle.phase != CyclePhase::ReadyToCommit || !observation.guard_released {
        return Err(super::Error::InvalidSupervision);
    }
    if observation.side_effects_inside_guard || observation.committed_call_count != call_count {
        return Err(super::Error::InvalidSupervision);
    }
    Ok(Cycle {
        phase: CyclePhase::Finished,
        ..cycle.clone()
    })
}

fn validate_observations(
    cycle: &Cycle,
    observations: &[RunObservation],
    parallel_execution: bool,
) -> Result<(), super::Error> {
    if cycle.phase != CyclePhase::Prepared {
        return Err(super::Error::InvalidSupervision);
    }
    if !parallel_execution {
        return Err(super::Error::InvalidSupervision);
    }
    if observations.len() != cycle.call_ids.len() {
        return Err(super::Error::InvalidSupervision);
    }
    for call_id in &cycle.call_ids {
        let matches = observations
            .iter()
            .filter(|observation| observation.call_id == *call_id)
            .count();
        if matches != 1 {
            return Err(super::Error::InvalidSupervision);
        }
    }
    debug_assert_eq!(cycle.phase, CyclePhase::Prepared);
    debug_assert!(parallel_execution);
    Ok(())
}

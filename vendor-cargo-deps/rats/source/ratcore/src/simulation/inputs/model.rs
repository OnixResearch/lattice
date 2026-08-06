/// Explicit deterministic effect result supplied by a shell or scripted adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SimulationEffectResult {
    /// Effect completed successfully.
    Completed,
    /// Effect failed with a redacted semantic error class.
    Failed,
    /// Effect was cancelled.
    Cancelled,
    /// Effect was replaced by a newer request.
    Replaced,
    /// Effect exceeded its logical deadline.
    TimedOut,
    /// Effect source closed.
    SourceClosed,
    /// External VM-controlled behavior produced a typed result.
    ExternallyControlledFailure,
}

impl SimulationEffectResult {
    /// Returns a stable semantic outcome label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Replaced => "replaced",
            Self::TimedOut => "timed-out",
            Self::SourceClosed => "source-closed",
            Self::ExternallyControlledFailure => "externally-controlled-failure",
        }
    }
}

/// Forbidden ambient source observed while deterministic evidence is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ForbiddenSimulationSource {
    /// Host wall or monotonic clock read.
    HostClock,
    /// Host entropy or random generator.
    HostRandom,
    /// Direct filesystem, process, network, browser, or similar I/O.
    DirectExternalIo,
}

/// Replayable input that is allowed to influence a deterministic scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulationControlInput {
    /// Explicit logical timer delivery.
    LogicalTick {
        /// Registered scheduler source.
        source_id: String,
        /// Monotonic logical tick.
        tick: u64,
    },
    /// Explicit finite choice selection.
    FiniteChoice {
        /// Registered choice point.
        choice_point: SimulationChoicePointId,
        /// Non-zero exclusive upper bound.
        bound: u32,
        /// Selected value in `0..bound`.
        selected: u32,
    },
    /// Explicit arbitration among ready semantic event sources.
    EventSelection {
        /// Strictly ordered ready source identities.
        ready_sources: Vec<String>,
        /// Selected source, which must be ready.
        selected_source: String,
    },
    /// Typed effect result returned through the application message boundary.
    EffectResult {
        /// Stable request identity.
        request_id: String,
        /// Registered effect class.
        effect_class: SimulationEffectClassId,
        /// Typed result.
        result: SimulationEffectResult,
    },
    /// Explicit evidence that an ambient source was attempted.
    ForbiddenSource(ForbiddenSimulationSource),
}

/// Stable deterministic-controller failure code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationControlDiagnosticCode {
    /// Script exceeds its fixed reviewed bound.
    ScriptTooLarge,
    /// Logical tick regressed.
    RegressingLogicalTick,
    /// Scheduler source is empty or unregistered.
    UnknownSchedulerSource,
    /// Choice point is empty or unregistered.
    UnknownChoicePoint,
    /// Choice has an empty set.
    EmptyChoiceSet,
    /// Selected choice is outside the declared bound.
    ChoiceOutOfRange,
    /// Ready-source list is empty, unordered, duplicate, or oversized.
    InvalidReadySources,
    /// Selected scheduler source was not ready.
    SelectedSourceNotReady,
    /// Effect class is unregistered.
    UnknownEffectClass,
    /// Request identity is malformed.
    InvalidRequestIdentity,
    /// More than one terminal result was supplied for a request.
    DuplicateEffectResult,
    /// Host clock cannot supply deterministic logical time.
    HostClockForbidden,
    /// Host entropy cannot supply deterministic finite choices.
    HostRandomForbidden,
    /// Direct external I/O cannot execute inside the headless controller.
    DirectExternalIoForbidden,
    /// Scripted provider has no remaining input.
    MissingScriptedInput,
    /// Scripted provider received an unexpected input class.
    UnexpectedScriptedInput,
}

/// Stable deterministic-controller diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationControlDiagnostic {
    /// Machine-comparable issue code.
    pub code: SimulationControlDiagnosticCode,
    /// Safe semantic subject class.
    pub subject: &'static str,
}

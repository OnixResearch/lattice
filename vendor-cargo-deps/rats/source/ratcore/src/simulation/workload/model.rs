/// Event-order selection for the bounded headless pilot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessPilotEventOrder {
    /// Deliver logical time before the finite choice.
    TickThenChoice,
    /// Deliver the finite choice before logical time.
    ChoiceThenTick,
}

impl HeadlessPilotEventOrder {
    const fn selected_source(self) -> &'static str {
        match self {
            Self::TickThenChoice => "source.tick",
            Self::ChoiceThenTick => "source.choice",
        }
    }
}

/// Explicit controls for one bounded headless pilot run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeadlessPilotControls {
    /// Explicit logical tick.
    pub logical_tick: u64,
    /// Selected pane class in the closed range `0..2`.
    pub selected_choice: u32,
    /// Explicit semantic source ordering.
    pub event_order: HeadlessPilotEventOrder,
    /// Typed effect result.
    pub effect_result: SimulationEffectResult,
}

impl Default for HeadlessPilotControls {
    fn default() -> Self {
        Self {
            logical_tick: 10,
            selected_choice: 1,
            event_order: HeadlessPilotEventOrder::TickThenChoice,
            effect_result: SimulationEffectResult::Completed,
        }
    }
}

/// Accepted state of the terminal/browser-free pilot.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HeadlessPilotState {
    /// Last accepted logical tick.
    pub logical_tick: u64,
    /// Selected finite pane class.
    pub selected_choice: u32,
    /// Last accepted semantic viewport width in cells.
    pub viewport_columns: u32,
    /// Last accepted semantic viewport height in cells.
    pub viewport_rows: u32,
    /// Effect lifecycle result, when returned.
    pub effect_result: Option<SimulationEffectResult>,
    /// Number of deterministic application diagnostics.
    pub diagnostic_count: u32,
    /// Whether shutdown was accepted.
    pub shutdown: bool,
}

/// Typed pilot application message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessPilotMessage {
    /// Explicit logical timer delivery.
    LogicalTick(u64),
    /// Explicit finite choice selection.
    SelectPane(u32),
    /// Normalized semantic viewport dimensions.
    Resize {
        /// Viewport width in cells.
        columns: u32,
        /// Viewport height in cells.
        rows: u32,
    },
    /// Request one declarative persistence effect.
    RequestPersistence,
    /// Return one typed effect result.
    EffectResult(SimulationEffectResult),
    /// Request orderly shutdown.
    Shutdown,
}

impl HeadlessPilotMessage {
    const fn as_str(self) -> &'static str {
        match self {
            Self::LogicalTick(_) => "logical-tick",
            Self::SelectPane(_) => "select-pane",
            Self::Resize { .. } => "resize",
            Self::RequestPersistence => "request-persistence",
            Self::EffectResult(_) => "effect-result",
            Self::Shutdown => "shutdown",
        }
    }
}

/// Declarative effect emitted by the pure pilot update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessPilotEffectRequest {
    /// Stable request identity.
    pub request_id: String,
    /// Stable operation class.
    pub effect_class: SimulationEffectClassId,
}

/// Pilot diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessPilotDiagnosticCode {
    /// Choice was outside the pilot's closed finite set.
    ChoiceOutOfRange,
    /// Logical time regressed.
    LogicalTickRegressed,
    /// Viewport dimensions were empty.
    EmptyViewport,
    /// Persistence failed or was otherwise not completed.
    PersistenceNotCompleted,
    /// Duplicate effect result arrived after terminal completion.
    DuplicateEffectResult,
}

impl HeadlessPilotDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ChoiceOutOfRange => "choice-out-of-range",
            Self::LogicalTickRegressed => "logical-tick-regressed",
            Self::EmptyViewport => "empty-viewport",
            Self::PersistenceNotCompleted => "persistence-not-completed",
            Self::DuplicateEffectResult => "duplicate-effect-result",
        }
    }
}

/// Logical invalidation produced by the pure pilot update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessPilotInvalidation {
    /// No redraw or timer work.
    None,
    /// Accepted state requires immediate semantic projection refresh.
    Immediate,
    /// Logical timer remains controlled by the shell.
    ContinueLogicalTimer,
}

impl HeadlessPilotInvalidation {
    const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Immediate => "immediate",
            Self::ContinueLogicalTimer => "continue-logical-timer",
        }
    }
}

/// Pure update outcome class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessPilotTransitionKind {
    /// Authoritative state changed.
    Accepted,
    /// Message was rejected and state was preserved except diagnostic accounting.
    Rejected,
    /// Message was a valid no-op.
    Ignored,
}

impl HeadlessPilotTransitionKind {
    const fn probe_outcome(self) -> SimulationProbeOutcome {
        match self {
            Self::Accepted => SimulationProbeOutcome::Accepted,
            Self::Rejected => SimulationProbeOutcome::Rejected,
            Self::Ignored => SimulationProbeOutcome::Ignored,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Ignored => "ignored",
        }
    }
}

/// Complete pure update output for one pilot message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessPilotTransition {
    /// Outcome class.
    pub kind: HeadlessPilotTransitionKind,
    /// Declarative effects.
    pub effects: Vec<HeadlessPilotEffectRequest>,
    /// Deterministic diagnostics.
    pub diagnostics: Vec<HeadlessPilotDiagnosticCode>,
    /// Logical invalidation.
    pub invalidation: HeadlessPilotInvalidation,
}

/// Applies one typed pilot message without I/O, clocks, randomness, or observers.
#[must_use]
pub fn update_headless_pilot(
    state: &mut HeadlessPilotState,
    message: HeadlessPilotMessage,
) -> HeadlessPilotTransition {
    match message {
        HeadlessPilotMessage::LogicalTick(tick) => match tick.cmp(&state.logical_tick) {
            std::cmp::Ordering::Less => {
                rejected_transition(state, HeadlessPilotDiagnosticCode::LogicalTickRegressed)
            }
            std::cmp::Ordering::Equal => ignored_transition(),
            std::cmp::Ordering::Greater => {
                state.logical_tick = tick;
                accepted_transition(HeadlessPilotInvalidation::ContinueLogicalTimer)
            }
        },
        HeadlessPilotMessage::SelectPane(selected) => {
            if selected >= 2 {
                rejected_transition(state, HeadlessPilotDiagnosticCode::ChoiceOutOfRange)
            } else if selected == state.selected_choice {
                ignored_transition()
            } else {
                state.selected_choice = selected;
                accepted_transition(HeadlessPilotInvalidation::Immediate)
            }
        }
        HeadlessPilotMessage::Resize { columns, rows } => {
            if columns == 0 || rows == 0 {
                rejected_transition(state, HeadlessPilotDiagnosticCode::EmptyViewport)
            } else if columns == state.viewport_columns && rows == state.viewport_rows {
                ignored_transition()
            } else {
                state.viewport_columns = columns;
                state.viewport_rows = rows;
                accepted_transition(HeadlessPilotInvalidation::Immediate)
            }
        }
        HeadlessPilotMessage::RequestPersistence => HeadlessPilotTransition {
            kind: HeadlessPilotTransitionKind::Accepted,
            effects: vec![HeadlessPilotEffectRequest {
                request_id: PILOT_REQUEST.to_string(),
                effect_class: SimulationEffectClassId::new(PILOT_EFFECT_CLASS),
            }],
            diagnostics: Vec::new(),
            invalidation: HeadlessPilotInvalidation::None,
        },
        HeadlessPilotMessage::EffectResult(result) => {
            if state.effect_result.is_some() {
                rejected_transition(state, HeadlessPilotDiagnosticCode::DuplicateEffectResult)
            } else {
                state.effect_result = Some(result);
                if result == SimulationEffectResult::Completed {
                    accepted_transition(HeadlessPilotInvalidation::Immediate)
                } else {
                    state.diagnostic_count = state.diagnostic_count.saturating_add(1);
                    HeadlessPilotTransition {
                        kind: HeadlessPilotTransitionKind::Accepted,
                        effects: Vec::new(),
                        diagnostics: vec![HeadlessPilotDiagnosticCode::PersistenceNotCompleted],
                        invalidation: HeadlessPilotInvalidation::Immediate,
                    }
                }
            }
        }
        HeadlessPilotMessage::Shutdown => {
            if state.shutdown {
                ignored_transition()
            } else {
                state.shutdown = true;
                accepted_transition(HeadlessPilotInvalidation::Immediate)
            }
        }
    }
}

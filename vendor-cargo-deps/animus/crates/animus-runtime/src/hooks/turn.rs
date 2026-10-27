#![allow(
    ambiguous_params,
    non_trait_imports,
    path_segment_repetition,
    underscore_in_module_filename,
    usize_in_public_api
)]

use super::LifecycleHookEventKind;

const INITIAL_TURN_INDEX: usize = 0;

/// Pure turn-lifecycle FSM state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum State {
    /// No startup lifecycle boundary has been emitted.
    #[default]
    New,
    /// Startup was emitted and the first turn may be entered.
    Started,
    /// A pre-turn boundary has been emitted for the indexed turn.
    BeforeTurn {
        /// Zero-based deterministic turn index.
        turn_index: usize,
    },
    /// A post-turn boundary has been emitted for the indexed turn.
    AfterTurn {
        /// Zero-based deterministic turn index.
        turn_index: usize,
    },
    /// The lifecycle is complete.
    Completed,
}

/// Turn result class recorded at the post-turn boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The turn completed after a model adapter response.
    Model,
    /// The turn completed after a tool adapter response.
    Tool,
    /// The turn completed with a final response.
    Respond,
    /// The turn completed with a stop decision.
    Stop,
    /// The run completed by exhausting the configured turn budget.
    MaxTurnsExceeded,
    /// The turn failed closed.
    Failed,
    /// The turn was cancelled.
    Cancelled,
}

/// Event advanced through the pure turn-lifecycle FSM.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    /// Runtime startup boundary. Emits `session_start`.
    Startup,
    /// Boundary before loop-fabric evaluation for one turn.
    PreTurn {
        /// Zero-based deterministic turn index.
        turn_index: usize,
    },
    /// Boundary after a turn result has been recorded.
    PostTurn {
        /// Zero-based deterministic turn index.
        turn_index: usize,
        /// Safe turn outcome class.
        outcome: Outcome,
    },
    /// Mark the lifecycle complete after all required turn boundaries.
    Complete,
}

/// Lifecycle hook effect emitted by the pure FSM.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Effect {
    event_kind: LifecycleHookEventKind,
    turn_index: Option<usize>,
}

impl Effect {
    const fn new(event_kind: LifecycleHookEventKind, turn_index: Option<usize>) -> Self {
        Self {
            event_kind,
            turn_index,
        }
    }

    /// Returns the lifecycle hook event kind to route through host-owned adapters.
    #[must_use]
    pub const fn event_kind(self) -> LifecycleHookEventKind {
        self.event_kind
    }

    /// Returns the turn index for turn-scoped effects.
    #[must_use]
    pub const fn turn_index(self) -> Option<usize> {
        self.turn_index
    }
}

/// Pure transition result for a turn-lifecycle event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Transition {
    state: State,
    effect: Option<Effect>,
}

impl Transition {
    const fn new(state: State, effect: Option<Effect>) -> Self {
        Self { state, effect }
    }

    /// Returns the next pure FSM state.
    #[must_use]
    pub const fn state(self) -> State {
        self.state
    }

    /// Returns the lifecycle hook effect emitted by this transition, if any.
    #[must_use]
    pub const fn effect(self) -> Option<Effect> {
        self.effect
    }
}

/// Typed turn-lifecycle transition failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// Event is not legal from the current state.
    InvalidTransition,
    /// Event turn index did not match the expected deterministic turn index.
    TurnIndexMismatch,
    /// The next deterministic turn index would overflow.
    TurnIndexOverflow,
}

/// Error returned by the pure turn-lifecycle FSM.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    state: State,
    event: Event,
}

impl Error {
    const fn new(kind: ErrorKind, state: State, event: Event) -> Self {
        Self { kind, state, event }
    }

    /// Returns the transition failure class.
    #[must_use]
    pub const fn kind(self) -> ErrorKind {
        self.kind
    }

    /// Returns the state that rejected the event.
    #[must_use]
    pub const fn state(self) -> State {
        self.state
    }

    /// Returns the event that was rejected.
    #[must_use]
    pub const fn event(self) -> Event {
        self.event
    }
}

/// Advances the pure turn-lifecycle FSM without executing adapters or hooks.
///
/// # Errors
///
/// Returns [`Error`] when the event is out of order, the turn index
/// is stale or skipped, or the next deterministic turn index would overflow.
pub fn advance(state: State, event: Event) -> Result<Transition, Error> {
    // r[impl agent_harness_runtime.turn_lifecycle_hooks.fsm]
    match (state, event) {
        (State::New, Event::Startup) => Ok(Transition::new(
            State::Started,
            Some(Effect::new(LifecycleHookEventKind::SessionStart, None)),
        )),
        (State::Started, Event::PreTurn { turn_index }) => begin_initial(state, event, turn_index),
        (
            State::AfterTurn {
                turn_index: previous,
            },
            Event::PreTurn { turn_index },
        ) => begin_next(state, event, previous, turn_index),
        (
            State::BeforeTurn {
                turn_index: expected,
            },
            Event::PostTurn {
                turn_index,
                outcome: _,
            },
        ) => finish_current(state, event, expected, turn_index),
        (State::Started | State::AfterTurn { .. }, Event::Complete) => {
            Ok(Transition::new(State::Completed, None))
        }
        _ => Err(Error::new(ErrorKind::InvalidTransition, state, event)),
    }
}

fn begin_initial(state: State, event: Event, turn_index: usize) -> Result<Transition, Error> {
    if turn_index != INITIAL_TURN_INDEX {
        return Err(Error::new(ErrorKind::TurnIndexMismatch, state, event));
    }
    Ok(begin_transition(turn_index))
}

fn begin_next(
    state: State,
    event: Event,
    previous_turn_index: usize,
    turn_index: usize,
) -> Result<Transition, Error> {
    let Some(expected_turn_index) = previous_turn_index.checked_add(1) else {
        return Err(Error::new(ErrorKind::TurnIndexOverflow, state, event));
    };
    if turn_index != expected_turn_index {
        return Err(Error::new(ErrorKind::TurnIndexMismatch, state, event));
    }
    Ok(begin_transition(turn_index))
}

fn begin_transition(turn_index: usize) -> Transition {
    Transition::new(
        State::BeforeTurn { turn_index },
        Some(Effect::new(
            LifecycleHookEventKind::PreTurn,
            Some(turn_index),
        )),
    )
}

fn finish_current(
    state: State,
    event: Event,
    expected_turn_index: usize,
    turn_index: usize,
) -> Result<Transition, Error> {
    if turn_index != expected_turn_index {
        return Err(Error::new(ErrorKind::TurnIndexMismatch, state, event));
    }
    Ok(Transition::new(
        State::AfterTurn { turn_index },
        Some(Effect::new(
            LifecycleHookEventKind::PostTurn,
            Some(turn_index),
        )),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_TURN_INDEX: usize = 0;
    const SECOND_TURN_INDEX: usize = 1;

    fn must_advance(state: State, event: Event) -> Transition {
        advance(state, event).expect("turn lifecycle transition should be valid")
    }

    #[test]
    fn emits_hooks_in_order() {
        // r[verify agent_harness_runtime.turn_lifecycle_hooks.fsm.positive]
        let startup = must_advance(State::New, Event::Startup);
        let pre_turn = must_advance(
            startup.state(),
            Event::PreTurn {
                turn_index: FIRST_TURN_INDEX,
            },
        );
        let post_turn = must_advance(
            pre_turn.state(),
            Event::PostTurn {
                turn_index: FIRST_TURN_INDEX,
                outcome: Outcome::Model,
            },
        );
        let complete = must_advance(post_turn.state(), Event::Complete);

        assert_eq!(
            startup.effect().map(Effect::event_kind),
            Some(LifecycleHookEventKind::SessionStart)
        );
        assert_eq!(
            pre_turn.effect().map(Effect::event_kind),
            Some(LifecycleHookEventKind::PreTurn)
        );
        assert_eq!(
            pre_turn.effect().and_then(Effect::turn_index),
            Some(FIRST_TURN_INDEX)
        );
        assert_eq!(
            post_turn.effect().map(Effect::event_kind),
            Some(LifecycleHookEventKind::PostTurn)
        );
        assert_eq!(
            post_turn.effect().and_then(Effect::turn_index),
            Some(FIRST_TURN_INDEX)
        );
        assert_eq!(complete.state(), State::Completed);
        assert_eq!(complete.effect(), None);
    }

    #[test]
    fn requires_post_before_next_pre() {
        // r[verify agent_harness_runtime.turn_lifecycle_hooks.fsm.negative]
        let startup = must_advance(State::New, Event::Startup);
        let pre_turn = must_advance(
            startup.state(),
            Event::PreTurn {
                turn_index: FIRST_TURN_INDEX,
            },
        );
        let error = advance(
            pre_turn.state(),
            Event::PreTurn {
                turn_index: SECOND_TURN_INDEX,
            },
        )
        .expect_err("duplicate pre-turn without post-turn should fail");

        assert_eq!(error.kind(), ErrorKind::InvalidTransition);
        assert_eq!(error.state(), pre_turn.state());
    }

    #[test]
    fn rejects_duplicate_startup_and_early_post() {
        // r[verify agent_harness_runtime.turn_lifecycle_hooks.fsm.negative]
        let startup = must_advance(State::New, Event::Startup);
        let duplicate_startup =
            advance(startup.state(), Event::Startup).expect_err("duplicate startup should fail");
        let post_without_pre = advance(
            startup.state(),
            Event::PostTurn {
                turn_index: FIRST_TURN_INDEX,
                outcome: Outcome::Tool,
            },
        )
        .expect_err("post-turn before pre-turn should fail");

        assert_eq!(duplicate_startup.kind(), ErrorKind::InvalidTransition);
        assert_eq!(post_without_pre.kind(), ErrorKind::InvalidTransition);
    }

    #[test]
    fn rejects_skipped_or_stale_indices() {
        // r[verify agent_harness_runtime.turn_lifecycle_hooks.fsm.negative]
        let startup = must_advance(State::New, Event::Startup);
        let skipped_first = advance(
            startup.state(),
            Event::PreTurn {
                turn_index: SECOND_TURN_INDEX,
            },
        )
        .expect_err("first pre-turn must use the initial index");
        let first_pre = must_advance(
            startup.state(),
            Event::PreTurn {
                turn_index: FIRST_TURN_INDEX,
            },
        );
        let mismatched_post = advance(
            first_pre.state(),
            Event::PostTurn {
                turn_index: SECOND_TURN_INDEX,
                outcome: Outcome::Respond,
            },
        )
        .expect_err("post-turn index must match the pre-turn index");

        assert_eq!(skipped_first.kind(), ErrorKind::TurnIndexMismatch);
        assert_eq!(mismatched_post.kind(), ErrorKind::TurnIndexMismatch);
    }
}

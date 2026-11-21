//! Lifecycle, cancellation and single-shot liveness derived from one owned
//! attempt's retained state and the shell's observation time only.

type Cancellation = super::retained::Cancellation;
type Lifecycle = super::retained::Lifecycle;
type Liveness = super::retained::Liveness;
type LivenessState = super::retained::LivenessState;
type State<'a> = super::retained::AttemptState<'a>;
type StateDefect = super::retained::StateDefect;
type UnknownCause = super::retained::UnknownCause;

pub(super) fn lifecycle(state: &State<'_>) -> Result<Lifecycle, StateDefect> {
    let lifecycle = if state.has_revision_block {
        match (state.has_intent, state.has_refusal, state.terminal_disposition) {
            (true, false, None) => Lifecycle::RevisionBlockedBeforeDispatch,
            (false, _, _) => return Err(StateDefect::RefusalWithoutIntent),
            (true, _, _) => return Err(StateDefect::RefusalWithTerminal),
        }
    } else {
        match (state.has_intent, state.has_refusal, state.terminal_disposition) {
            (false, false, None) => Lifecycle::AdmittedNotDispatched,
            (false, true, None) => return Err(StateDefect::RefusalWithoutIntent),
            (false, _, Some(_)) => return Err(StateDefect::TerminalWithoutIntent),
            (true, true, Some(_)) => return Err(StateDefect::RefusalWithTerminal),
            (true, true, None) => Lifecycle::BindingRejectedBeforeDispatch,
            (true, false, None) => Lifecycle::DispatchRecordedOutcomeUnknown,
            (true, false, Some(disposition)) => Lifecycle::TerminalObserved {
                disposition: disposition.to_owned(),
            },
        }
    };
    // Only a retained intent can have dispatched, been refused or blocked, or
    // finished; a terminal state is reported exactly when one was retained.
    debug_assert!(state.has_intent || lifecycle == Lifecycle::AdmittedNotDispatched);
    debug_assert_eq!(matches!(lifecycle, Lifecycle::TerminalObserved { .. }), state.terminal_disposition.is_some());
    Ok(lifecycle)
}

pub(super) fn cancellation(state: &State<'_>) -> Cancellation {
    if state.terminal_disposition == Some("cancelled") {
        Cancellation::ObservedTerminated
    } else if state.has_cancel_request {
        Cancellation::RequestedUnobserved
    } else {
        Cancellation::NotRequested
    }
}

/// Whether the observation is past the request's admitted window.
pub(super) fn is_past_window(state: &State<'_>, observed_at_epoch_ms: i64) -> bool {
    u64::try_from(observed_at_epoch_ms).is_ok_and(|observed| observed > state.admitted_until_epoch_ms)
}

/// Single-shot liveness: only an observed terminal result is `exited`, and a
/// dispatched attempt without one is `unknown`, never idle or live.
pub(super) fn liveness(state: &State<'_>, lifecycle: &Lifecycle, observed_at_epoch_ms: i64) -> Liveness {
    let (liveness_state, unknown_cause) = match lifecycle {
        Lifecycle::AdmittedNotDispatched
        | Lifecycle::BindingRejectedBeforeDispatch
        | Lifecycle::RevisionBlockedBeforeDispatch => (LivenessState::NotStarted, None),
        Lifecycle::TerminalObserved { .. } => (LivenessState::Exited, None),
        Lifecycle::DispatchRecordedOutcomeUnknown if is_past_window(state, observed_at_epoch_ms) => {
            (LivenessState::Unknown, Some(UnknownCause::Stale))
        }
        Lifecycle::DispatchRecordedOutcomeUnknown => (LivenessState::Unknown, Some(UnknownCause::Unobserved)),
    };
    let retained_generation = state
        .artifacts
        .iter()
        .map(|_| 1_u64)
        .chain(state.terminal_disposition.map(|_| 1_u64))
        .fold(0_u64, u64::saturating_add);
    debug_assert!(retained_generation >= u64::from(state.terminal_disposition.is_some()));
    debug_assert_eq!(unknown_cause.is_some(), liveness_state == LivenessState::Unknown);
    Liveness {
        runtime: super::retained::RuntimeClass::SingleShot,
        state: liveness_state,
        unknown_cause,
        retained_generation,
        observed_at_epoch_ms,
    }
}

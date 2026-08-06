use super::super::catalog::ActiveBindingLayers;
use super::super::catalog::BindingCatalogSnapshot;
use super::super::resolution::BindingResolution;
use super::super::resolution::BindingResolutionOutcome;
use super::KeySequenceReducerOutcome;
use super::KeySequenceState;
use super::KeySequenceTransition;
use super::PendingSequence;

pub(super) fn transition_from_resolution(
    resolution: BindingResolution,
    catalog: &BindingCatalogSnapshot,
    active: &ActiveBindingLayers,
) -> KeySequenceTransition {
    let (state, outcome) = match &resolution.outcome {
        BindingResolutionOutcome::Resolved(binding) => {
            (KeySequenceState::default(), KeySequenceReducerOutcome::Resolved(binding.clone()))
        }
        BindingResolutionOutcome::Pending(continuations) => (
            KeySequenceState {
                pending: Some(PendingSequence {
                    sequence: resolution.sequence.clone(),
                    catalog_generation: catalog.generation,
                    context_generation: active.context_generation,
                }),
            },
            KeySequenceReducerOutcome::Pending(continuations.clone()),
        ),
        BindingResolutionOutcome::Unbound => (KeySequenceState::default(), KeySequenceReducerOutcome::Unbound),
        BindingResolutionOutcome::Blocked(reason) => {
            (KeySequenceState::default(), KeySequenceReducerOutcome::Blocked(reason.clone()))
        }
        BindingResolutionOutcome::Ambiguous(ids) => {
            (KeySequenceState::default(), KeySequenceReducerOutcome::Ambiguous(ids.clone()))
        }
        BindingResolutionOutcome::Stale(reason) => {
            (KeySequenceState::default(), KeySequenceReducerOutcome::Stale(reason.clone()))
        }
    };
    transition(state, outcome, Some(resolution))
}

pub(super) fn terminal_input(
    state: &KeySequenceState,
    observed: (u64, u64),
    catalog: &BindingCatalogSnapshot,
    active: &ActiveBindingLayers,
    outcome: KeySequenceReducerOutcome,
) -> KeySequenceTransition {
    if !current_generations(observed, catalog, active) || pending_is_stale(state, catalog, active) {
        stale_transition("terminal sequence input observes stale generation data")
    } else {
        transition(KeySequenceState::default(), outcome, None)
    }
}

pub(super) fn replace_snapshots(
    state: &KeySequenceState,
    previous: (u64, u64),
    catalog: &BindingCatalogSnapshot,
    active: &ActiveBindingLayers,
) -> KeySequenceTransition {
    let pending_matches_previous = state
        .pending
        .as_ref()
        .is_some_and(|pending| pending.catalog_generation == previous.0 && pending.context_generation == previous.1);
    let snapshots_changed = previous.0 != catalog.generation || previous.1 != active.context_generation;
    if state.pending.is_some() && pending_matches_previous && snapshots_changed {
        stale_transition("accepted snapshot replacement cancelled the pending prefix")
    } else if pending_is_stale(state, catalog, active) {
        stale_transition("pending prefix does not match accepted snapshot generations")
    } else {
        transition(state.clone(), KeySequenceReducerOutcome::Reset, None)
    }
}

pub(super) fn unsupported_input(
    state: &KeySequenceState,
    key: &str,
    observed: (u64, u64),
    catalog: &BindingCatalogSnapshot,
    active: &ActiveBindingLayers,
) -> KeySequenceTransition {
    if !current_generations(observed, catalog, active) || pending_is_stale(state, catalog, active) {
        return stale_transition("unsupported input observes stale generation data");
    }
    transition(
        state.clone(),
        KeySequenceReducerOutcome::Unsupported(format!("renderer cannot normalize key `{key}`")),
        None,
    )
}

pub(super) fn pending_is_stale(
    state: &KeySequenceState,
    catalog: &BindingCatalogSnapshot,
    active: &ActiveBindingLayers,
) -> bool {
    state.pending.as_ref().is_some_and(|pending| {
        pending.catalog_generation != catalog.generation || pending.context_generation != active.context_generation
    })
}

pub(super) fn current_generations(
    observed: (u64, u64),
    catalog: &BindingCatalogSnapshot,
    active: &ActiveBindingLayers,
) -> bool {
    observed.0 == catalog.generation && observed.1 == active.context_generation
}

pub(super) fn stale_transition(message: &str) -> KeySequenceTransition {
    transition(KeySequenceState::default(), KeySequenceReducerOutcome::Stale(message.to_string()), None)
}

pub(super) fn transition(
    state: KeySequenceState,
    outcome: KeySequenceReducerOutcome,
    resolution: Option<BindingResolution>,
) -> KeySequenceTransition {
    KeySequenceTransition {
        state,
        outcome,
        resolution,
    }
}

pub(super) fn format_sequence_diagnostics(diagnostics: &[super::super::KeySequenceDiagnostic]) -> String {
    diagnostics.iter().map(|diagnostic| diagnostic.message.clone()).collect::<Vec<_>>().join("; ")
}

use super::ActiveBindingLayers;
use super::BindingCatalogSnapshot;
use super::KeySequenceInput;
use super::KeySequenceState;
use super::KeySequenceTransition;
use super::ResolvedBindingDispatch;

/// One deterministic layered key-sequence replay step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySequenceReplayStep {
    /// Explicit production reducer input.
    pub input: KeySequenceInput,
    /// Renderer provenance used only after a binding resolves.
    pub renderer: super::super::ActionRendererAdapter,
}

impl KeySequenceReplayStep {
    /// Creates a replay step.
    #[must_use]
    pub const fn new(input: KeySequenceInput, renderer: super::super::ActionRendererAdapter) -> Self {
        Self { input, renderer }
    }
}

/// One production reducer and optional contextual-routing receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySequenceReplayReceipt {
    /// Stable replay sequence number.
    pub sequence: u32,
    /// Production pending-sequence transition.
    pub transition: KeySequenceTransition,
    /// Subsequent production contextual dispatch for resolved input.
    pub dispatch: Option<ResolvedBindingDispatch>,
}

/// Deterministic layered key-sequence replay artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySequenceReplay {
    /// Catalog generation used by replay.
    pub catalog_generation: u64,
    /// Context generation used by replay.
    pub context_generation: u64,
    /// Ordered receipts.
    pub receipts: Vec<KeySequenceReplayReceipt>,
    /// Final pure pending state.
    pub final_state: KeySequenceState,
}

/// Replays logical inputs through the production reducer and contextual router.
#[must_use]
pub fn replay_key_sequences(
    catalog: &BindingCatalogSnapshot,
    registry: &super::super::ActionRegistrySnapshot,
    active: &ActiveBindingLayers,
    routing: &super::super::ActionRoutingSnapshot,
    steps: &[KeySequenceReplayStep],
) -> KeySequenceReplay {
    let mut state = KeySequenceState::default();
    let mut receipts = Vec::with_capacity(steps.len());
    for (index, step) in steps.iter().enumerate() {
        let transition = super::reduce_key_sequence(&state, &step.input, catalog, registry, active);
        let dispatch = match &transition.outcome {
            super::KeySequenceReducerOutcome::Resolved(binding) => {
                Some(super::dispatch_resolved_binding(routing, catalog, active, binding, step.renderer))
            }
            _ => None,
        };
        state = transition.state.clone();
        receipts.push(KeySequenceReplayReceipt {
            sequence: public_sequence(index),
            transition,
            dispatch,
        });
    }
    KeySequenceReplay {
        catalog_generation: catalog.generation(),
        context_generation: active.context_generation(),
        receipts,
        final_state: state,
    }
}

fn public_sequence(index: usize) -> u32 {
    u32::try_from(index).map_or(u32::MAX, |value| value)
}

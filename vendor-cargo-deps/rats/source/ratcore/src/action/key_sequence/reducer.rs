use super::ActionBindingId;
use super::KeySequence;
use super::LogicalKeystroke;
use super::catalog::ActiveBindingLayers;
use super::catalog::BindingCatalogSnapshot;
use super::resolution::BindingResolution;
use super::resolution::PendingContinuation;
use super::resolution::ResolvedActionBinding;

/// Pure pending sequence state bound to accepted catalog/context generations.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeySequenceState {
    pending: Option<PendingSequence>,
}

impl KeySequenceState {
    /// Returns the pending sequence, when one exists.
    #[must_use]
    pub fn pending_sequence(&self) -> Option<&KeySequence> {
        self.pending.as_ref().map(|pending| &pending.sequence)
    }

    /// Returns the pending catalog generation.
    #[must_use]
    pub fn catalog_generation(&self) -> Option<u64> {
        self.pending.as_ref().map(|pending| pending.catalog_generation)
    }

    /// Returns the pending context generation.
    #[must_use]
    pub fn context_generation(&self) -> Option<u64> {
        self.pending.as_ref().map(|pending| pending.context_generation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingSequence {
    sequence: KeySequence,
    catalog_generation: u64,
    context_generation: u64,
}

/// Explicit reducer input; clocks and physical events remain shell-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySequenceInput {
    /// Submit one normalized logical stroke.
    Stroke {
        /// Logical stroke.
        stroke: LogicalKeystroke,
        /// Catalog generation observed by the input surface.
        observed_catalog_generation: u64,
        /// Context generation observed by the input surface.
        observed_context_generation: u64,
    },
    /// Cancel a pending sequence.
    Cancel {
        /// Catalog generation observed by the cancellation source.
        observed_catalog_generation: u64,
        /// Context generation observed by the cancellation source.
        observed_context_generation: u64,
    },
    /// Expire a pending sequence after a shell-owned timer fires.
    Expire {
        /// Catalog generation attached to the scheduled expiry.
        observed_catalog_generation: u64,
        /// Context generation attached to the scheduled expiry.
        observed_context_generation: u64,
    },
    /// Reset state unconditionally.
    Reset,
    /// Notify the reducer that accepted snapshots were replaced.
    AcceptedSnapshotReplacement {
        /// Catalog generation against which pending state began.
        previous_catalog_generation: u64,
        /// Context generation against which pending state began.
        previous_context_generation: u64,
    },
    /// Report a physical key that the renderer could not normalize.
    UnsupportedKey {
        /// Renderer-local diagnostic label.
        key: String,
        /// Catalog generation observed by the renderer.
        observed_catalog_generation: u64,
        /// Context generation observed by the renderer.
        observed_context_generation: u64,
    },
}

/// Observable result of one sequence transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySequenceReducerOutcome {
    /// A complete binding resolved to an action identity.
    Resolved(ResolvedActionBinding),
    /// A valid prefix remains pending.
    Pending(Vec<PendingContinuation>),
    /// Pending input was cancelled explicitly.
    Cancelled,
    /// Pending input expired explicitly.
    Expired,
    /// Input has no active binding.
    Unbound,
    /// Generation data was stale and pending state was cleared.
    Stale(String),
    /// Modal, enablement, or sequence policy blocked input.
    Blocked(String),
    /// Equal effective precedence was ambiguous.
    Ambiguous(Vec<ActionBindingId>),
    /// Renderer could not normalize the physical key; prior state is preserved.
    Unsupported(String),
    /// State was reset without resolving input.
    Reset,
}

/// Accepted reducer state plus outcome and production resolver evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySequenceTransition {
    /// State after the transition.
    pub state: KeySequenceState,
    /// Observable outcome.
    pub outcome: KeySequenceReducerOutcome,
    /// Production resolver receipt for stroke inputs.
    pub resolution: Option<BindingResolution>,
}

/// Reduces one explicit input against accepted catalog and active-layer snapshots.
#[must_use]
pub fn reduce_key_sequence(
    state: &KeySequenceState,
    input: &KeySequenceInput,
    catalog: &BindingCatalogSnapshot,
    registry: &super::super::ActionRegistrySnapshot,
    active: &ActiveBindingLayers,
) -> KeySequenceTransition {
    let context = ReducerContext {
        catalog,
        registry,
        active,
    };
    match input {
        KeySequenceInput::Stroke {
            stroke,
            observed_catalog_generation,
            observed_context_generation,
        } => reduce_stroke(state, stroke, (*observed_catalog_generation, *observed_context_generation), &context),
        KeySequenceInput::Cancel {
            observed_catalog_generation,
            observed_context_generation,
        } => transitions::terminal_input(
            state,
            (*observed_catalog_generation, *observed_context_generation),
            catalog,
            active,
            KeySequenceReducerOutcome::Cancelled,
        ),
        KeySequenceInput::Expire {
            observed_catalog_generation,
            observed_context_generation,
        } => transitions::terminal_input(
            state,
            (*observed_catalog_generation, *observed_context_generation),
            catalog,
            active,
            KeySequenceReducerOutcome::Expired,
        ),
        KeySequenceInput::Reset => {
            transitions::transition(KeySequenceState::default(), KeySequenceReducerOutcome::Reset, None)
        }
        KeySequenceInput::AcceptedSnapshotReplacement {
            previous_catalog_generation,
            previous_context_generation,
        } => transitions::replace_snapshots(
            state,
            (*previous_catalog_generation, *previous_context_generation),
            catalog,
            active,
        ),
        KeySequenceInput::UnsupportedKey {
            key,
            observed_catalog_generation,
            observed_context_generation,
        } => transitions::unsupported_input(
            state,
            key,
            (*observed_catalog_generation, *observed_context_generation),
            catalog,
            active,
        ),
    }
}

struct ReducerContext<'a> {
    catalog: &'a BindingCatalogSnapshot,
    registry: &'a super::super::ActionRegistrySnapshot,
    active: &'a ActiveBindingLayers,
}

fn reduce_stroke(
    state: &KeySequenceState,
    stroke: &LogicalKeystroke,
    observed: (u64, u64),
    context: &ReducerContext<'_>,
) -> KeySequenceTransition {
    if !transitions::current_generations(observed, context.catalog, context.active) {
        return transitions::stale_transition("stroke observes stale catalog or context generation");
    }
    if transitions::pending_is_stale(state, context.catalog, context.active) {
        return transitions::stale_transition("pending prefix belongs to a replaced catalog or context");
    }
    let sequence = match next_sequence(state, stroke, context.catalog) {
        Ok(sequence) => sequence,
        Err(message) => {
            return transitions::transition(
                KeySequenceState::default(),
                KeySequenceReducerOutcome::Blocked(message),
                None,
            );
        }
    };
    let resolution =
        super::resolution::resolve_key_sequence(context.catalog, context.registry, context.active, &sequence);
    transitions::transition_from_resolution(resolution, context.catalog, context.active)
}

fn next_sequence(
    state: &KeySequenceState,
    stroke: &LogicalKeystroke,
    catalog: &BindingCatalogSnapshot,
) -> Result<KeySequence, String> {
    if let Some(pending) = &state.pending {
        return pending
            .sequence
            .pushed(stroke.clone(), &catalog.limits)
            .map_err(|diagnostics| transitions::format_sequence_diagnostics(&diagnostics));
    }
    KeySequence::try_new(vec![stroke.clone()], &catalog.limits)
        .map_err(|diagnostics| transitions::format_sequence_diagnostics(&diagnostics))
}

#[path = "reducer/transitions.rs"]
mod transitions;

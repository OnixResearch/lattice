use super::ActionBindingId;
use super::BindingLayerId;
use super::KeySequence;
use super::LogicalKeystroke;
use super::catalog::ActionBindingSource;
use super::catalog::ActiveBindingLayers;
use super::catalog::BindingCatalogSnapshot;

/// Resolver decision for one catalog binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingDecision {
    /// Binding resolved the complete sequence.
    Winner,
    /// Binding contributes a continuation to the accepted pending prefix.
    Pending,
    /// A higher-precedence binding or prefix controls the input.
    Shadowed,
    /// Binding is disabled.
    Disabled,
    /// Binding's layer is not active.
    InactiveLayer,
    /// Bound action is disabled in the action registry.
    ActionDisabled,
    /// An active modal layer blocked fallback.
    ModalBlocked,
    /// Equal effective precedence could not be resolved uniquely.
    Ambiguous,
}

/// Inspectable resolver consideration for one relevant binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingConsideration {
    /// Binding identity.
    pub binding_id: ActionBindingId,
    /// Bound action identity.
    pub action_id: super::super::ActionId,
    /// Owning layer.
    pub layer_id: BindingLayerId,
    /// Effective layer precedence.
    pub precedence: i16,
    /// Resolver decision.
    pub decision: BindingDecision,
}

/// One immediate continuation for an accepted pending prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingContinuation {
    /// Next logical stroke.
    pub stroke: LogicalKeystroke,
    /// Bindings reachable through this stroke.
    pub binding_ids: Vec<ActionBindingId>,
    /// Actions reachable through this stroke.
    pub action_ids: Vec<super::super::ActionId>,
}

/// Accepted binding result before contextual route selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedActionBinding {
    /// Complete accepted sequence.
    pub sequence: KeySequence,
    /// Winning binding identity.
    pub binding_id: ActionBindingId,
    /// Resolved action identity.
    pub action_id: super::super::ActionId,
    /// Winning layer identity.
    pub layer_id: BindingLayerId,
    /// Binding source/default/override provenance.
    pub source: ActionBindingSource,
    /// Catalog generation used for resolution.
    pub catalog_generation: u64,
    /// Context generation used for layer activation.
    pub context_generation: u64,
}

/// Full deterministic binding-resolution provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingResolutionProvenance {
    /// Active layers in effective order.
    pub active_layers: Vec<BindingLayerId>,
    /// Relevant binding decisions in stable resolution order.
    pub considerations: Vec<BindingConsideration>,
}

/// Pure sequence-resolution outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingResolutionOutcome {
    /// One complete binding resolved.
    Resolved(ResolvedActionBinding),
    /// Sequence is an accepted prefix awaiting another stroke.
    Pending(Vec<PendingContinuation>),
    /// No active binding accepts this sequence.
    Unbound,
    /// A modal layer, disabled binding/action, or policy blocked resolution.
    Blocked(String),
    /// Equal effective precedence could not be resolved uniquely.
    Ambiguous(Vec<ActionBindingId>),
    /// Observed catalog/context data is stale.
    Stale(String),
}

impl BindingResolutionOutcome {
    /// Returns the resolved binding when accepted.
    #[must_use]
    pub const fn resolved(&self) -> Option<&ResolvedActionBinding> {
        match self {
            Self::Resolved(binding) => Some(binding),
            _ => None,
        }
    }

    /// Returns pending continuations when the sequence is a prefix.
    #[must_use]
    pub fn continuations(&self) -> &[PendingContinuation] {
        match self {
            Self::Pending(continuations) => continuations,
            _ => &[],
        }
    }
}

/// Complete resolver receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingResolution {
    /// Sequence that was evaluated.
    pub sequence: KeySequence,
    /// Accepted outcome.
    pub outcome: BindingResolutionOutcome,
    /// Deterministic layer and binding evidence.
    pub provenance: BindingResolutionProvenance,
}

/// Resolves one complete or partial sequence through accepted active layers.
#[must_use]
pub fn resolve_key_sequence(
    catalog: &BindingCatalogSnapshot,
    registry: &super::super::ActionRegistrySnapshot,
    active: &ActiveBindingLayers,
    sequence: &KeySequence,
) -> BindingResolution {
    engine::resolve(catalog, registry, active, sequence)
}

#[path = "resolution/engine.rs"]
mod engine;

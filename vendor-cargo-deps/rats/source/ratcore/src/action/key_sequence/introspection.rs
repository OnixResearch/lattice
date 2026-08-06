use std::collections::BTreeMap;

use super::ActionBindingId;
use super::BindingLayerId;
use super::KeySequence;
use super::RendererKeyCapabilities;
use super::catalog::ActionBinding;
use super::catalog::ActionBindingSource;
use super::catalog::ActiveBindingLayers;
use super::catalog::BindingCatalogSnapshot;
use super::resolution::BindingDecision;
use super::resolution::BindingResolutionOutcome;

/// Renderer support projection for one accepted binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererBindingSupport {
    /// Whether every stroke can be represented by the renderer.
    pub supported: bool,
    /// Explicit unsupported reasons.
    pub reasons: Vec<String>,
}

/// Introspection state for one binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingInspectionState {
    /// Binding wins for its complete sequence.
    Winner,
    /// Binding contributes to a pending prefix.
    Pending,
    /// A higher layer shadows this binding.
    Shadowed,
    /// Binding is disabled.
    Disabled,
    /// Binding layer is not active.
    InactiveLayer,
    /// Bound action is disabled.
    ActionDisabled,
    /// Modal or other binding policy blocks this binding.
    Blocked,
    /// Resolution is ambiguous.
    Ambiguous,
    /// Binding/catalog generation is stale.
    Stale,
}

/// One inspectable binding with source, layer, and renderer support evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingInspection {
    /// Binding identity.
    pub binding_id: ActionBindingId,
    /// Bound action identity.
    pub action_id: super::super::ActionId,
    /// Complete logical sequence.
    pub sequence: KeySequence,
    /// Neutral fallback label.
    pub sequence_label: String,
    /// Owning layer.
    pub layer_id: BindingLayerId,
    /// Source/default/override provenance.
    pub source: ActionBindingSource,
    /// Current resolver state.
    pub state: BindingInspectionState,
    /// Human-readable state reason when non-winning.
    pub reason: Option<String>,
    /// Renderer capability projection.
    pub renderer_support: RendererBindingSupport,
}

/// Describe-sequence projection built from the production resolver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescribeSequenceProjection {
    /// Production resolution receipt.
    pub resolution: super::BindingResolution,
    /// Relevant winning, pending, shadowed, disabled, and inactive bindings.
    pub bindings: Vec<BindingInspection>,
}

/// Describes one sequence using the same catalog and resolver as dispatch.
#[must_use]
pub fn describe_key_sequence(
    catalog: &BindingCatalogSnapshot,
    registry: &super::super::ActionRegistrySnapshot,
    active: &ActiveBindingLayers,
    sequence: &KeySequence,
    capabilities: &RendererKeyCapabilities,
) -> DescribeSequenceProjection {
    let resolution = super::resolve_key_sequence(catalog, registry, active, sequence);
    let decisions = resolution
        .provenance
        .considerations
        .iter()
        .map(|item| (item.binding_id.clone(), item.decision))
        .collect::<BTreeMap<_, _>>();
    let mut bindings = catalog
        .bindings
        .iter()
        .filter(|binding| related(sequence, &binding.sequence))
        .map(|binding| {
            let decision = decisions.get(&binding.id).copied().unwrap_or(BindingDecision::InactiveLayer);
            inspection_from_decision(binding, decision, capabilities)
        })
        .collect::<Vec<_>>();
    bindings.sort_by(|left, right| left.binding_id.cmp(&right.binding_id));
    DescribeSequenceProjection { resolution, bindings }
}

/// Projects every current accepted binding for one action.
#[must_use]
pub fn where_is_action(
    catalog: &BindingCatalogSnapshot,
    registry: &super::super::ActionRegistrySnapshot,
    active: &ActiveBindingLayers,
    action_id: &super::super::ActionId,
    capabilities: &RendererKeyCapabilities,
) -> Vec<BindingInspection> {
    let mut inspections = catalog
        .bindings
        .iter()
        .filter(|binding| &binding.action_id == action_id)
        .map(|binding| {
            let resolution = super::resolve_key_sequence(catalog, registry, active, &binding.sequence);
            let (state, reason) = state_for_binding(binding, &resolution.outcome);
            inspection(binding, state, reason, capabilities)
        })
        .collect::<Vec<_>>();
    inspections.sort_by(|left, right| {
        left.sequence
            .cmp(&right.sequence)
            .then_with(|| left.layer_id.cmp(&right.layer_id))
            .then_with(|| left.binding_id.cmp(&right.binding_id))
    });
    inspections
}

fn state_for_binding(
    binding: &ActionBinding,
    outcome: &BindingResolutionOutcome,
) -> (BindingInspectionState, Option<String>) {
    if !binding.enabled {
        return (BindingInspectionState::Disabled, binding.disabled_reason.clone());
    }
    match outcome {
        BindingResolutionOutcome::Resolved(winner) if winner.binding_id == binding.id => {
            (BindingInspectionState::Winner, None)
        }
        BindingResolutionOutcome::Resolved(winner) => {
            (BindingInspectionState::Shadowed, Some(format!("shadowed by binding `{}`", winner.binding_id)))
        }
        BindingResolutionOutcome::Pending(_) => (
            BindingInspectionState::Shadowed,
            Some("higher-precedence prefix controls this sequence".to_string()),
        ),
        BindingResolutionOutcome::Unbound => {
            (BindingInspectionState::InactiveLayer, Some("binding layer is not active".to_string()))
        }
        BindingResolutionOutcome::Blocked(reason) => (BindingInspectionState::Blocked, Some(reason.clone())),
        BindingResolutionOutcome::Ambiguous(ids) => {
            (BindingInspectionState::Ambiguous, Some(format!("ambiguous bindings: {}", join_binding_ids(ids))))
        }
        BindingResolutionOutcome::Stale(reason) => (BindingInspectionState::Stale, Some(reason.clone())),
    }
}

fn inspection_from_decision(
    binding: &ActionBinding,
    decision: BindingDecision,
    capabilities: &RendererKeyCapabilities,
) -> BindingInspection {
    let (state, reason) = match decision {
        BindingDecision::Winner => (BindingInspectionState::Winner, None),
        BindingDecision::Pending => (BindingInspectionState::Pending, None),
        BindingDecision::Shadowed => (
            BindingInspectionState::Shadowed,
            Some("higher-precedence binding controls the sequence".to_string()),
        ),
        BindingDecision::Disabled => (BindingInspectionState::Disabled, binding.disabled_reason.clone()),
        BindingDecision::InactiveLayer => {
            (BindingInspectionState::InactiveLayer, Some("binding layer is not active".to_string()))
        }
        BindingDecision::ActionDisabled => {
            (BindingInspectionState::ActionDisabled, Some("bound action is disabled".to_string()))
        }
        BindingDecision::ModalBlocked => {
            (BindingInspectionState::Blocked, Some("active modal layer blocks this binding".to_string()))
        }
        BindingDecision::Ambiguous => {
            (BindingInspectionState::Ambiguous, Some("equal effective precedence is ambiguous".to_string()))
        }
    };
    inspection(binding, state, reason, capabilities)
}

fn inspection(
    binding: &ActionBinding,
    state: BindingInspectionState,
    reason: Option<String>,
    capabilities: &RendererKeyCapabilities,
) -> BindingInspection {
    BindingInspection {
        binding_id: binding.id.clone(),
        action_id: binding.action_id.clone(),
        sequence: binding.sequence.clone(),
        sequence_label: binding.sequence.canonical_label(),
        layer_id: binding.layer_id.clone(),
        source: binding.source.clone(),
        state,
        reason,
        renderer_support: renderer_support(&binding.sequence, capabilities),
    }
}

fn renderer_support(sequence: &KeySequence, capabilities: &RendererKeyCapabilities) -> RendererBindingSupport {
    let mut reasons = Vec::new();
    for stroke in sequence.strokes() {
        if !capabilities.supports_key(stroke.key()) {
            reasons.push(format!(
                "renderer {:?} cannot represent key `{}`",
                capabilities.renderer,
                stroke.canonical_label()
            ));
        }
        for modifier in stroke.modifiers() {
            if !capabilities.supports_modifier(modifier) {
                reasons.push(format!("renderer {:?} cannot represent modifier `{modifier:?}`", capabilities.renderer));
            }
        }
    }
    reasons.sort();
    reasons.dedup();
    RendererBindingSupport {
        supported: reasons.is_empty(),
        reasons,
    }
}

fn related(input: &KeySequence, binding: &KeySequence) -> bool {
    input == binding || binding.starts_with(input) || input.starts_with(binding)
}

fn join_binding_ids(ids: &[ActionBindingId]) -> String {
    ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
}

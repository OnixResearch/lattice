use std::collections::BTreeSet;

use super::super::ActionBindingId;
use super::super::KeySequence;
use super::super::catalog::ActionBinding;
use super::super::catalog::ActiveBindingLayers;
use super::super::catalog::BindingCatalogSnapshot;
use super::super::catalog::BindingLayer;
use super::super::catalog::BindingLayerKind;
use super::BindingConsideration;
use super::BindingDecision;
use super::BindingResolution;
use super::BindingResolutionOutcome;
use super::BindingResolutionProvenance;
use super::ResolvedActionBinding;

pub(super) fn resolve(
    catalog: &BindingCatalogSnapshot,
    registry: &super::super::super::ActionRegistrySnapshot,
    active: &ActiveBindingLayers,
    sequence: &KeySequence,
) -> BindingResolution {
    let view = ResolutionView::new(catalog, active, sequence);
    let selection = select_level(&view, registry);
    let outcome = selection.outcome(catalog, active, sequence);
    let provenance = view.provenance(&selection);
    BindingResolution {
        sequence: sequence.clone(),
        outcome,
        provenance,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchKind {
    Exact,
    Prefix,
}

#[derive(Debug, Clone, Copy)]
struct Match<'a> {
    binding: &'a ActionBinding,
    layer: &'a BindingLayer,
    kind: MatchKind,
}

struct ResolutionView<'a> {
    matches: Vec<Match<'a>>,
    active_layers: Vec<&'a BindingLayer>,
}

impl<'a> ResolutionView<'a> {
    fn new(catalog: &'a BindingCatalogSnapshot, active: &ActiveBindingLayers, sequence: &KeySequence) -> Self {
        let active_ids = active.layers.iter().map(|item| &item.layer_id).collect::<BTreeSet<_>>();
        let active_layers = active.layers.iter().filter_map(|item| catalog.layer(&item.layer_id)).collect::<Vec<_>>();
        let matches = catalog
            .bindings
            .iter()
            .filter_map(|binding| {
                let layer = catalog.layer(&binding.layer_id)?;
                let kind = helpers::relation(sequence, &binding.sequence)?;
                Some(Match { binding, layer, kind })
            })
            .collect::<Vec<_>>();
        debug_assert!(active_layers.iter().all(|layer| active_ids.contains(&layer.id)));
        Self { matches, active_layers }
    }

    fn active_match(&self, candidate: &Match<'_>) -> bool {
        self.active_layers.iter().any(|layer| layer.id == candidate.layer.id)
    }

    fn provenance(&self, selection: &Selection<'_>) -> BindingResolutionProvenance {
        let mut considerations = self
            .matches
            .iter()
            .map(|candidate| BindingConsideration {
                binding_id: candidate.binding.id.clone(),
                action_id: candidate.binding.action_id.clone(),
                layer_id: candidate.layer.id.clone(),
                precedence: candidate.layer.precedence,
                decision: self.decision(candidate, selection),
            })
            .collect::<Vec<_>>();
        considerations.sort_by(|left, right| {
            right
                .precedence
                .cmp(&left.precedence)
                .then_with(|| left.layer_id.cmp(&right.layer_id))
                .then_with(|| left.binding_id.cmp(&right.binding_id))
        });
        BindingResolutionProvenance {
            active_layers: self.active_layers.iter().map(|layer| layer.id.clone()).collect(),
            considerations,
        }
    }

    fn decision(&self, candidate: &Match<'_>, selection: &Selection<'_>) -> BindingDecision {
        if !self.active_match(candidate) {
            return BindingDecision::InactiveLayer;
        }
        if !candidate.binding.enabled {
            return BindingDecision::Disabled;
        }
        if selection.modal_blocked {
            return BindingDecision::ModalBlocked;
        }
        if selection.ambiguous_ids.contains(&candidate.binding.id) {
            return BindingDecision::Ambiguous;
        }
        if selection.winner_id.as_ref() == Some(&candidate.binding.id) {
            return selection.winner_decision;
        }
        if selection.precedence.is_some_and(|value| candidate.layer.precedence < value) {
            return BindingDecision::Shadowed;
        }
        BindingDecision::Shadowed
    }
}

struct Selection<'a> {
    winner: Option<Match<'a>>,
    winner_id: Option<ActionBindingId>,
    winner_decision: BindingDecision,
    pending: Vec<Match<'a>>,
    precedence: Option<i16>,
    ambiguous_ids: Vec<ActionBindingId>,
    modal_blocked: bool,
    blocked_reason: Option<String>,
}

impl Selection<'_> {
    fn outcome(
        &self,
        catalog: &BindingCatalogSnapshot,
        active: &ActiveBindingLayers,
        sequence: &KeySequence,
    ) -> BindingResolutionOutcome {
        if let Some(reason) = &self.blocked_reason {
            return BindingResolutionOutcome::Blocked(reason.clone());
        }
        if !self.ambiguous_ids.is_empty() {
            return BindingResolutionOutcome::Ambiguous(self.ambiguous_ids.clone());
        }
        if let Some(winner) = self.winner {
            return BindingResolutionOutcome::Resolved(ResolvedActionBinding {
                sequence: sequence.clone(),
                binding_id: winner.binding.id.clone(),
                action_id: winner.binding.action_id.clone(),
                layer_id: winner.layer.id.clone(),
                source: winner.binding.source.clone(),
                catalog_generation: catalog.generation,
                context_generation: active.context_generation,
            });
        }
        if !self.pending.is_empty() {
            return BindingResolutionOutcome::Pending(helpers::project_continuations(sequence, &self.pending));
        }
        BindingResolutionOutcome::Unbound
    }
}

fn select_level<'a>(
    view: &'a ResolutionView<'a>,
    registry: &super::super::super::ActionRegistrySnapshot,
) -> Selection<'a> {
    let mut precedences = view.active_layers.iter().map(|layer| layer.precedence).collect::<Vec<_>>();
    precedences.sort_by(|left, right| right.cmp(left));
    precedences.dedup();
    for precedence in precedences {
        let eligible = view
            .matches
            .iter()
            .copied()
            .filter(|candidate| view.active_match(candidate))
            .filter(|candidate| candidate.binding.enabled)
            .filter(|candidate| candidate.layer.precedence == precedence)
            .collect::<Vec<_>>();
        if !eligible.is_empty() {
            return select_matches(&eligible, precedence, registry);
        }
        let modal = view
            .active_layers
            .iter()
            .find(|layer| layer.precedence == precedence && layer.kind == BindingLayerKind::Modal);
        if let Some(layer) = modal {
            return helpers::blocked_selection(
                precedence,
                format!("active modal layer `{}` blocks lower binding layers", layer.id),
            );
        }
    }
    helpers::empty_selection()
}

fn select_matches<'a>(
    matches: &[Match<'a>],
    precedence: i16,
    registry: &super::super::super::ActionRegistrySnapshot,
) -> Selection<'a> {
    let exact = matches.iter().copied().filter(|candidate| candidate.kind == MatchKind::Exact).collect::<Vec<_>>();
    let pending = matches.iter().copied().filter(|candidate| candidate.kind == MatchKind::Prefix).collect::<Vec<_>>();
    if exact.len() > 1 || (!exact.is_empty() && !pending.is_empty()) {
        return helpers::ambiguous_selection(matches, precedence);
    }
    if let Some(winner) = exact.first().copied() {
        let Some(action) = registry.action(&winner.binding.action_id) else {
            return helpers::blocked_selection(precedence, "bound action is absent".to_string());
        };
        if !registry.is_active(action) {
            let reason = registry.inactive_reason(action).unwrap_or_else(|| "bound action is disabled".to_string());
            return helpers::blocked_selection(precedence, reason);
        }
        return Selection {
            winner: Some(winner),
            winner_id: Some(winner.binding.id.clone()),
            winner_decision: BindingDecision::Winner,
            pending: Vec::new(),
            precedence: Some(precedence),
            ambiguous_ids: Vec::new(),
            modal_blocked: false,
            blocked_reason: None,
        };
    }
    Selection {
        winner: None,
        winner_id: None,
        winner_decision: BindingDecision::Pending,
        pending,
        precedence: Some(precedence),
        ambiguous_ids: Vec::new(),
        modal_blocked: false,
        blocked_reason: None,
    }
}

#[path = "engine/helpers.rs"]
mod helpers;

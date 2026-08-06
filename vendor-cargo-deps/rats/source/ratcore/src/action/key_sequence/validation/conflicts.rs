use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::super::LogicalKeystroke;
use super::super::catalog::ActionBinding;
use super::super::catalog::BindingCatalogCandidate;
use super::super::catalog::BindingCatalogDiagnostic;
use super::super::catalog::BindingCatalogDiagnosticCode;

pub(super) fn conflict_diagnostics(candidate: &BindingCatalogCandidate) -> Vec<BindingCatalogDiagnostic> {
    let precedence = candidate.layers.iter().map(|layer| (&layer.id, layer.precedence)).collect::<BTreeMap<_, _>>();
    let active = candidate
        .bindings
        .iter()
        .filter(|binding| binding.enabled)
        .filter_map(|binding| precedence.get(&binding.layer_id).map(|value| (binding, *value)))
        .collect::<Vec<_>>();
    let mut diagnostics = Vec::new();
    for left_index in 0..active.len() {
        let left = active[left_index];
        for right in &active[left_index.saturating_add(1)..] {
            if left.1 != right.1 {
                continue;
            }
            if left.0.sequence == right.0.sequence {
                diagnostics.push(conflict_diagnostic(
                    BindingCatalogDiagnosticCode::DuplicateSequence,
                    left.0,
                    right.0,
                    "equal-precedence bindings duplicate one complete sequence",
                ));
            } else if left.0.sequence.is_prefix_of(&right.0.sequence) || right.0.sequence.is_prefix_of(&left.0.sequence)
            {
                diagnostics.push(conflict_diagnostic(
                    BindingCatalogDiagnosticCode::PrefixConflict,
                    left.0,
                    right.0,
                    "equal-precedence exact and prefix bindings conflict",
                ));
            }
        }
    }
    diagnostics
}

fn conflict_diagnostic(
    code: BindingCatalogDiagnosticCode,
    left: &ActionBinding,
    right: &ActionBinding,
    message: &str,
) -> BindingCatalogDiagnostic {
    BindingCatalogDiagnostic::new(code, "binding.sequence", message)
        .with_binding(left.id.clone())
        .with_binding(right.id.clone())
        .with_layer(left.layer_id.clone())
        .with_layer(right.layer_id.clone())
}

pub(super) fn fanout_diagnostics(candidate: &BindingCatalogCandidate) -> Vec<BindingCatalogDiagnostic> {
    let precedence = candidate.layers.iter().map(|layer| (&layer.id, layer.precedence)).collect::<BTreeMap<_, _>>();
    let mut fanout: BTreeMap<(i16, Vec<LogicalKeystroke>), BTreeSet<LogicalKeystroke>> = BTreeMap::new();
    for binding in candidate.bindings.iter().filter(|binding| binding.enabled) {
        let Some(layer_precedence) = precedence.get(&binding.layer_id) else {
            continue;
        };
        for next_index in 0..binding.sequence.strokes().len() {
            let prefix = binding.sequence.strokes()[..next_index].to_vec();
            let next = binding.sequence.strokes()[next_index].clone();
            fanout.entry((*layer_precedence, prefix)).or_default().insert(next);
        }
    }
    fanout
        .into_iter()
        .filter(|(_, continuations)| continuations.len() > usize::from(candidate.limits.continuations))
        .map(|((layer_precedence, prefix), continuations)| {
            BindingCatalogDiagnostic::new(
                BindingCatalogDiagnosticCode::ContinuationFanoutExceeded,
                "binding.sequence",
                format!(
                    "precedence {layer_precedence} prefix length {} has {} continuations; limit is {}",
                    prefix.len(),
                    continuations.len(),
                    candidate.limits.continuations
                ),
            )
        })
        .collect()
}

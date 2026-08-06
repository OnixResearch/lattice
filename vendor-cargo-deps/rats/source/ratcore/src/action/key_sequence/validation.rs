use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::catalog::ActionBinding;
use super::catalog::ActionBindingSourceKind;
use super::catalog::BindingCatalogBuild;
use super::catalog::BindingCatalogCandidate;
use super::catalog::BindingCatalogDiagnostic;
use super::catalog::BindingCatalogDiagnosticCode;
use super::catalog::BindingCatalogSnapshot;
use super::catalog::BindingLayer;

pub(super) fn accept(
    mut candidate: BindingCatalogCandidate,
    registry: &super::super::ActionRegistrySnapshot,
    previous: Option<&BindingCatalogSnapshot>,
) -> BindingCatalogBuild {
    let mut diagnostics = validate_candidate(&candidate, registry, previous);
    sort_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        candidate.layers.sort_by(|left, right| left.id.cmp(&right.id));
        candidate.bindings.sort_by(|left, right| left.id.cmp(&right.id));
        BindingCatalogBuild {
            snapshot: Some(BindingCatalogSnapshot {
                generation: candidate.generation,
                layers: candidate.layers,
                bindings: candidate.bindings,
                reserved: candidate.reserved,
                limits: candidate.limits,
            }),
            diagnostics,
            candidate_accepted: true,
        }
    } else {
        BindingCatalogBuild {
            snapshot: previous.cloned(),
            diagnostics,
            candidate_accepted: false,
        }
    }
}

fn validate_candidate(
    candidate: &BindingCatalogCandidate,
    registry: &super::super::ActionRegistrySnapshot,
    previous: Option<&BindingCatalogSnapshot>,
) -> Vec<BindingCatalogDiagnostic> {
    let mut diagnostics = generation_diagnostics(candidate, previous);
    diagnostics.extend(limit_diagnostics(candidate));
    diagnostics.extend(layer_diagnostics(&candidate.layers));
    diagnostics.extend(binding_shape_diagnostics(candidate, registry));
    diagnostics.extend(override_diagnostics(&candidate.bindings));
    diagnostics.extend(conflict_diagnostics(candidate));
    diagnostics.extend(fanout_diagnostics(candidate));
    diagnostics
}

fn generation_diagnostics(
    candidate: &BindingCatalogCandidate,
    previous: Option<&BindingCatalogSnapshot>,
) -> Vec<BindingCatalogDiagnostic> {
    let monotonic = previous.is_none_or(|snapshot| candidate.generation > snapshot.generation);
    if candidate.generation > 0 && monotonic {
        Vec::new()
    } else {
        vec![BindingCatalogDiagnostic::new(
            BindingCatalogDiagnosticCode::NonMonotonicGeneration,
            "catalog.generation",
            "catalog generation must be non-zero and newer than the previous snapshot",
        )]
    }
}

fn limit_diagnostics(candidate: &BindingCatalogCandidate) -> Vec<BindingCatalogDiagnostic> {
    let mut diagnostics = Vec::new();
    if !candidate.limits.valid() {
        diagnostics.push(BindingCatalogDiagnostic::new(
            BindingCatalogDiagnosticCode::InvalidLimits,
            "catalog.limits",
            "catalog limits must be non-zero and within hard safety maxima",
        ));
    }
    if count_exceeds_u32(candidate.bindings.len(), candidate.limits.bindings) {
        diagnostics.push(BindingCatalogDiagnostic::new(
            BindingCatalogDiagnosticCode::LimitExceeded,
            "catalog.bindings",
            format!("binding count {} exceeds limit {}", candidate.bindings.len(), candidate.limits.bindings),
        ));
    }
    if candidate.layers.len() > usize::from(candidate.limits.layers) {
        diagnostics.push(BindingCatalogDiagnostic::new(
            BindingCatalogDiagnosticCode::LimitExceeded,
            "catalog.layers",
            format!("layer count {} exceeds limit {}", candidate.layers.len(), candidate.limits.layers),
        ));
    }
    diagnostics
}

fn layer_diagnostics(layers: &[BindingLayer]) -> Vec<BindingCatalogDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = BTreeSet::new();
    for layer in layers {
        if malformed_identity(layer.id.as_str()) {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::MalformedIdentity,
                    "layer.id",
                    "layer identity must be non-empty normalized text",
                )
                .with_layer(layer.id.clone()),
            );
        }
        if layer.generation == 0 {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::NonMonotonicGeneration,
                    "layer.generation",
                    "layer generation must be non-zero",
                )
                .with_layer(layer.id.clone()),
            );
        }
        if !seen.insert(layer.id.clone()) {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::DuplicateLayerId,
                    "layer.id",
                    "layer identity is duplicated",
                )
                .with_layer(layer.id.clone()),
            );
        }
    }
    diagnostics
}

fn binding_shape_diagnostics(
    candidate: &BindingCatalogCandidate,
    registry: &super::super::ActionRegistrySnapshot,
) -> Vec<BindingCatalogDiagnostic> {
    let layers = candidate.layers.iter().map(|layer| &layer.id).collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for binding in &candidate.bindings {
        diagnostics.extend(binding_identity_diagnostics(binding));
        if !seen.insert(binding.id.clone()) {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::DuplicateBindingId,
                    "binding.id",
                    "binding identity is duplicated",
                )
                .with_binding(binding.id.clone()),
            );
        }
        if registry.action(&binding.action_id).is_none() {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::UnknownAction,
                    "binding.action_id",
                    "binding references an action absent from the accepted registry",
                )
                .with_binding(binding.id.clone())
                .with_action(binding.action_id.clone()),
            );
        }
        if !layers.contains(&binding.layer_id) {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::UnknownLayer,
                    "binding.layer_id",
                    "binding references an undeclared layer",
                )
                .with_binding(binding.id.clone())
                .with_layer(binding.layer_id.clone()),
            );
        }
        if candidate.reserved.iter().any(|reserved| reserved.sequence == binding.sequence) {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::ReservedSequence,
                    "binding.sequence",
                    "binding uses a policy-reserved sequence",
                )
                .with_binding(binding.id.clone())
                .with_action(binding.action_id.clone()),
            );
        }
    }
    diagnostics
}

fn binding_identity_diagnostics(binding: &ActionBinding) -> Vec<BindingCatalogDiagnostic> {
    let mut diagnostics = Vec::new();
    let fields = [
        ("binding.id", binding.id.as_str()),
        ("binding.action_id", binding.action_id.as_str()),
        ("binding.layer_id", binding.layer_id.as_str()),
        ("binding.source.name", binding.source.name.as_str()),
    ];
    for (field, value) in fields {
        if malformed_identity(value) {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::MalformedIdentity,
                    field,
                    "binding identity/source must be non-empty normalized text",
                )
                .with_binding(binding.id.clone())
                .with_action(binding.action_id.clone())
                .with_layer(binding.layer_id.clone()),
            );
        }
    }
    diagnostics
}

fn override_diagnostics(bindings: &[ActionBinding]) -> Vec<BindingCatalogDiagnostic> {
    let by_id = bindings.iter().map(|binding| (&binding.id, binding)).collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    for binding in bindings {
        let Some(replaced_id) = binding.source.replaces.as_ref() else {
            continue;
        };
        let valid_kind = binding.source.kind == ActionBindingSourceKind::UserOverride;
        let replaced = by_id.get(replaced_id).copied();
        let valid_target = replaced.is_some_and(|item| item.action_id == binding.action_id);
        let distinct = replaced_id != &binding.id;
        if valid_kind && valid_target && distinct {
            continue;
        }
        let mut diagnostic = BindingCatalogDiagnostic::new(
            BindingCatalogDiagnosticCode::MalformedOverride,
            "binding.source.replaces",
            "replacement overrides must target a distinct accepted binding for the same action",
        )
        .with_binding(binding.id.clone())
        .with_binding(replaced_id.clone())
        .with_action(binding.action_id.clone());
        diagnostic.layer_ids.push(binding.layer_id.clone());
        diagnostics.push(diagnostic);
    }
    diagnostics
}

fn malformed_identity(value: &str) -> bool {
    value.is_empty() || value.trim() != value
}

fn count_exceeds_u32(count: usize, limit: u32) -> bool {
    match u32::try_from(count) {
        Ok(value) => value > limit,
        Err(_) => true,
    }
}

pub(super) fn sort_diagnostics(diagnostics: &mut Vec<BindingCatalogDiagnostic>) {
    diagnostics.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.binding_ids.cmp(&right.binding_ids))
            .then_with(|| left.layer_ids.cmp(&right.layer_ids))
            .then_with(|| left.action_id.cmp(&right.action_id))
            .then_with(|| left.field.cmp(&right.field))
            .then_with(|| left.message.cmp(&right.message))
    });
    diagnostics.dedup();
}

#[path = "validation/active.rs"]
mod active;
pub(super) use active::validate_active_layers;

#[path = "validation/conflicts.rs"]
mod conflicts;
use conflicts::conflict_diagnostics;
use conflicts::fanout_diagnostics;

use std::collections::BTreeSet;

use super::super::catalog::ActiveBindingLayer;
use super::super::catalog::ActiveBindingLayers;
use super::super::catalog::BindingCatalogDiagnostic;
use super::super::catalog::BindingCatalogDiagnosticCode;
use super::super::catalog::BindingCatalogSnapshot;

pub(in crate::nexus_core::binding_core) fn validate_active_layers(
    snapshot: &BindingCatalogSnapshot,
    context_generation: u64,
    mut active: Vec<ActiveBindingLayer>,
) -> Result<ActiveBindingLayers, Vec<BindingCatalogDiagnostic>> {
    let mut diagnostics = Vec::new();
    if context_generation == 0 {
        diagnostics.push(BindingCatalogDiagnostic::new(
            BindingCatalogDiagnosticCode::StaleContextGeneration,
            "active.context_generation",
            "active context generation must be non-zero",
        ));
    }
    if active.len() > usize::from(snapshot.limits.active_layers) {
        diagnostics.push(BindingCatalogDiagnostic::new(
            BindingCatalogDiagnosticCode::LimitExceeded,
            "active.layers",
            format!("active layer count {} exceeds limit {}", active.len(), snapshot.limits.active_layers),
        ));
    }
    diagnostics.extend(active_layer_diagnostics(snapshot, &active));
    super::sort_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        active.sort_by(|left, right| active_layer_order(snapshot, left, right));
        Ok(ActiveBindingLayers {
            context_generation,
            layers: active,
        })
    } else {
        Err(diagnostics)
    }
}

fn active_layer_diagnostics(
    snapshot: &BindingCatalogSnapshot,
    active: &[ActiveBindingLayer],
) -> Vec<BindingCatalogDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = BTreeSet::new();
    for item in active {
        if !seen.insert(item.layer_id.clone()) {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::DuplicateActiveLayer,
                    "active.layers",
                    "active layer stack repeats a layer",
                )
                .with_layer(item.layer_id.clone()),
            );
        }
        let Some(layer) = snapshot.layer(&item.layer_id) else {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::UnknownLayer,
                    "active.layers",
                    "active layer stack references an unknown layer",
                )
                .with_layer(item.layer_id.clone()),
            );
            continue;
        };
        if item.observed_generation != layer.generation {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::StaleLayerGeneration,
                    "active.observed_generation",
                    "active layer observes a stale lifecycle generation",
                )
                .with_layer(item.layer_id.clone()),
            );
        }
        if !layer.enabled {
            diagnostics.push(
                BindingCatalogDiagnostic::new(
                    BindingCatalogDiagnosticCode::DisabledLayer,
                    "active.layers",
                    layer.disabled_reason.clone().unwrap_or_else(|| "active layer is disabled".to_string()),
                )
                .with_layer(item.layer_id.clone()),
            );
        }
    }
    diagnostics
}

fn active_layer_order(
    snapshot: &BindingCatalogSnapshot,
    left: &ActiveBindingLayer,
    right: &ActiveBindingLayer,
) -> std::cmp::Ordering {
    let left_precedence = snapshot.layer(&left.layer_id).map_or(i16::MIN, |layer| layer.precedence);
    let right_precedence = snapshot.layer(&right.layer_id).map_or(i16::MIN, |layer| layer.precedence);
    right_precedence.cmp(&left_precedence).then_with(|| left.layer_id.cmp(&right.layer_id))
}

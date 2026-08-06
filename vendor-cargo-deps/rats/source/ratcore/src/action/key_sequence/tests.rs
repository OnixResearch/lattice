use super::ActionBinding;
use super::ActionBindingSource;
use super::ActionBindingSourceKind;
use super::ActiveBindingLayer;
use super::ActiveBindingLayers;
use super::BindingCatalogCandidate;
use super::BindingCatalogSnapshot;
use super::BindingLayer;
use super::BindingLayerKind;
use super::KeyModifier;
use super::KeySequence;
use super::LogicalKey;
use super::LogicalKeystroke;
use crate::nexus_core::ActionDescriptor;
use crate::nexus_core::ActionRegistrySnapshot;
use crate::nexus_core::ActionSource;
use crate::nexus_core::BindingCatalogDiagnostic;

pub(super) const ACTION_OPEN: &str = "file.open";
pub(super) const ACTION_SAVE: &str = "file.save";
pub(super) const ACTION_QUIT: &str = "app.quit";
pub(super) const LAYER_APP: &str = "layer.application";
pub(super) const LAYER_FOCUS: &str = "layer.focus";
pub(super) const LAYER_MODAL: &str = "layer.modal";
pub(super) const LAYER_TRANSIENT: &str = "layer.transient";

pub(super) fn registry() -> ActionRegistrySnapshot {
    crate::nexus_core::build_registry(vec![
        ActionDescriptor::new(ACTION_OPEN, "Open", ActionSource::new("test")),
        ActionDescriptor::new(ACTION_SAVE, "Save", ActionSource::new("test")),
        ActionDescriptor::new(ACTION_QUIT, "Quit", ActionSource::new("test")),
    ])
    .snapshot
}

pub(super) fn stroke(key: char, modifiers: Vec<KeyModifier>) -> Result<LogicalKeystroke, String> {
    LogicalKeystroke::try_new(LogicalKey::Character(key), modifiers)
        .map_err(|issues| format!("stroke diagnostics: {issues:?}"))
}

pub(super) fn sequence(strokes: Vec<LogicalKeystroke>) -> Result<KeySequence, String> {
    KeySequence::try_new(strokes, &super::BindingCatalogLimits::default())
        .map_err(|issues| format!("sequence diagnostics: {issues:?}"))
}

pub(super) fn binding(id: &str, action: &str, sequence: KeySequence, layer: &str) -> ActionBinding {
    ActionBinding::new(
        id,
        action,
        sequence,
        layer,
        ActionBindingSource::new("test", ActionBindingSourceKind::Application),
    )
}

pub(super) fn layers(generation: u64) -> Vec<BindingLayer> {
    vec![
        BindingLayer::new(LAYER_APP, 10, BindingLayerKind::Application, generation),
        BindingLayer::new(LAYER_FOCUS, 20, BindingLayerKind::FocusedComponent, generation),
        BindingLayer::new(LAYER_TRANSIENT, 30, BindingLayerKind::Transient, generation),
        BindingLayer::new(LAYER_MODAL, 40, BindingLayerKind::Modal, generation),
    ]
}

pub(super) fn accepted_catalog(
    generation: u64,
    bindings: Vec<ActionBinding>,
) -> Result<BindingCatalogSnapshot, String> {
    let build = super::accept_binding_catalog(
        BindingCatalogCandidate::new(generation, layers(generation), bindings),
        &registry(),
        None,
    );
    accepted_snapshot(build.snapshot, &build.diagnostics)
}

pub(super) fn active(
    catalog: &BindingCatalogSnapshot,
    context_generation: u64,
    ids: &[&str],
) -> Result<ActiveBindingLayers, String> {
    let layers = ids.iter().map(|id| ActiveBindingLayer::new(*id, catalog.generation())).collect();
    catalog
        .activate(context_generation, layers)
        .map_err(|issues| format!("active layer diagnostics: {issues:?}"))
}

fn accepted_snapshot(
    snapshot: Option<BindingCatalogSnapshot>,
    diagnostics: &[BindingCatalogDiagnostic],
) -> Result<BindingCatalogSnapshot, String> {
    snapshot.ok_or_else(|| format!("catalog rejected: {diagnostics:?}"))
}

#[path = "tests/catalog.rs"]
mod catalog;
#[path = "tests/editing.rs"]
mod editing;
#[path = "tests/logical.rs"]
mod logical;
#[path = "tests/reducer.rs"]
mod reducer;
#[path = "tests/replay.rs"]
mod replay;
#[path = "tests/resolution.rs"]
mod resolution;

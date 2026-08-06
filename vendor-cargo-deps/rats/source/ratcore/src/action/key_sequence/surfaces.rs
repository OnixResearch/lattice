use super::ActiveBindingLayers;
use super::BindingCatalogSnapshot;
use super::BindingInspection;
use super::RendererKeyCapabilities;

/// Multiple-binding projection for one action on command/help surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionBindingsProjection {
    /// Stable action identity.
    pub action_id: super::super::ActionId,
    /// Action display label.
    pub label: String,
    /// Action category.
    pub category: String,
    /// Every accepted sequence in deterministic neutral notation.
    pub sequence_labels: Vec<String>,
    /// Full winner/shadow/source/capability evidence.
    pub bindings: Vec<BindingInspection>,
    /// Whether action metadata is active.
    pub enabled: bool,
    /// Registry inactive reason.
    pub disabled_reason: Option<String>,
}

/// Shared multiple-binding views for existing command surface adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingCommandViews {
    /// Keymap-like action rows.
    pub keymap: Vec<ActionBindingsProjection>,
    /// Leader/which-key-like rows.
    pub leader: Vec<ActionBindingsProjection>,
    /// Menu-like rows.
    pub menu: Vec<ActionBindingsProjection>,
    /// Command-palette-like rows.
    pub palette: Vec<ActionBindingsProjection>,
    /// Shortcut-help rows.
    pub shortcut_help: Vec<ActionBindingsProjection>,
}

/// Projects multiple sequences per action from the accepted catalog.
///
/// Renderer shells may format labels differently, but they must retain the same
/// sequence and binding identities represented in each row.
#[must_use]
pub fn project_binding_command_views(
    catalog: &BindingCatalogSnapshot,
    registry: &super::super::ActionRegistrySnapshot,
    active: &ActiveBindingLayers,
    capabilities: &RendererKeyCapabilities,
) -> BindingCommandViews {
    let rows = registry
        .actions()
        .iter()
        .map(|action| action_row(catalog, registry, active, capabilities, action))
        .collect::<Vec<_>>();
    BindingCommandViews {
        keymap: active_rows(&rows),
        leader: bound_rows(&rows),
        menu: rows.clone(),
        palette: rows.clone(),
        shortcut_help: bound_rows(&rows),
    }
}

fn action_row(
    catalog: &BindingCatalogSnapshot,
    registry: &super::super::ActionRegistrySnapshot,
    active: &ActiveBindingLayers,
    capabilities: &RendererKeyCapabilities,
    action: &super::super::ActionDescriptor,
) -> ActionBindingsProjection {
    let bindings = super::where_is_action(catalog, registry, active, &action.id, capabilities);
    let sequence_labels = bindings.iter().map(|binding| binding.sequence_label.clone()).collect();
    ActionBindingsProjection {
        action_id: action.id.clone(),
        label: action.label.clone(),
        category: action.category.clone(),
        sequence_labels,
        bindings,
        enabled: registry.is_active(action),
        disabled_reason: registry.inactive_reason(action),
    }
}

fn active_rows(rows: &[ActionBindingsProjection]) -> Vec<ActionBindingsProjection> {
    rows.iter().filter(|row| row.enabled).filter(|row| !row.bindings.is_empty()).cloned().collect()
}

fn bound_rows(rows: &[ActionBindingsProjection]) -> Vec<ActionBindingsProjection> {
    rows.iter().filter(|row| !row.bindings.is_empty()).cloned().collect()
}

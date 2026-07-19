//! Workspace shell action and view projections.

const ACTION_CATEGORY_SPACE: &str = "workspace";
const ACTION_SOURCE_SHELL: &str = "workspace-shell";

/// Renderable diagnostic view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticView {
    /// Stable diagnostic code label.
    pub code: String,
    /// Affected subject.
    pub subject: String,
    /// Human-readable message.
    pub message: String,
}

/// Renderable pane view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneView {
    /// Pane live ID used for typed intents.
    pub live_id: super::types::LiveId,
    /// User-facing title.
    pub title: String,
    /// Status label.
    pub status: String,
    /// Whether this pane owns focus.
    pub focused: bool,
    /// Logical content summary.
    pub content_summary: String,
    /// Pane-local visible diagnostics.
    pub diagnostics: Vec<DiagnosticView>,
}

/// Renderable tab view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabView {
    /// Tab live ID used for typed intents.
    pub live_id: super::types::LiveId,
    /// User-facing title.
    pub title: String,
    /// Whether this tab is active.
    pub active: bool,
    /// Aggregate status rollup.
    pub status_rollup: super::types::StatusRollup,
    /// Pane rows.
    pub panes: Vec<PaneView>,
}

/// Renderable workspace view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceView {
    /// Workspace live ID used for typed intents.
    pub live_id: super::types::LiveId,
    /// User-facing title.
    pub title: String,
    /// Whether this workspace owns active focus.
    pub active: bool,
    /// Aggregate status rollup.
    pub status_rollup: super::types::StatusRollup,
    /// Tab rows.
    pub tabs: Vec<TabView>,
}

/// Meteorite-facing workspace shell view model with renderable data only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewModel {
    /// Workspace rows.
    pub workspaces: Vec<SpaceView>,
    /// Workspace-specific action projections.
    pub actions: Vec<super::commands::ActionProjection>,
    /// Model-level visible diagnostics.
    pub diagnostics: Vec<DiagnosticView>,
    /// Overall workspace status.
    pub status_rollup: super::types::StatusRollup,
}

/// Projects workspace action descriptors through the shared registry.
#[must_use]
pub fn project_actions(
    actions: Vec<super::commands::ActionDescriptor>,
) -> super::commands::ActionSurfaces {
    debug_assert!(actions.iter().all(|action| !action.id.is_empty()));
    debug_assert!(actions.iter().all(|action| !action.label.is_empty()));
    let descriptors: Vec<_> = actions.iter().map(descriptor_from_action).collect();
    let registry = crate::action_registry::build_registry(descriptors.clone());
    let snapshot = registry.snapshot;
    let workspace = actions.iter().map(action_projection).collect();
    super::commands::ActionSurfaces {
        descriptors,
        workspace,
        keymap: snapshot.keymap_bindings(),
        menu: snapshot.menu_items(),
        palette: snapshot.command_palette_items(),
        help: shortcut_help(&snapshot),
    }
}

/// Builds a renderer-neutral view model.
#[must_use]
pub fn build_view_model(
    model: &super::state::Model,
    actions: &super::commands::ActionSurfaces,
) -> ViewModel {
    debug_assert!(model.workspaces.len() <= model.workspaces.capacity());
    debug_assert!(actions.workspace.len() <= actions.workspace.capacity());
    let workspaces = model
        .workspaces
        .iter()
        .map(|space| space_view(space, model))
        .collect();
    let diagnostics = model.diagnostics.iter().map(diagnostic_view).collect();
    let status_rollup = super::types::rollup(
        model
            .workspaces
            .iter()
            .flat_map(|space| space.tabs.iter())
            .flat_map(|tab| tab.panes.iter().map(|pane| pane.status)),
    );
    ViewModel {
        workspaces,
        actions: actions.workspace.clone(),
        diagnostics,
        status_rollup,
    }
}

fn descriptor_from_action(
    action: &super::commands::ActionDescriptor,
) -> crate::action_registry::ActionDescriptor {
    debug_assert!(!action.id.is_empty());
    debug_assert!(!action.label.is_empty());
    let mut descriptor = crate::action_registry::ActionDescriptor::new(
        action.id.clone(),
        action.label.clone(),
        crate::action_registry::ActionSource::new(ACTION_SOURCE_SHELL),
    )
    .with_category(ACTION_CATEGORY_SPACE);
    if let Some(description) = &action.description {
        descriptor = descriptor.with_description(description.clone());
    }
    if let Some(shortcut) = &action.shortcut {
        descriptor = descriptor.with_shortcut(shortcut.clone());
    }
    if !action.enabled {
        descriptor =
            descriptor.with_enablement(crate::action_registry::ActionEnablement::Disabled {
                reason: action
                    .disabled_reason
                    .clone()
                    .unwrap_or_else(|| String::from("disabled")),
            });
    }
    descriptor
}

fn action_projection(
    action: &super::commands::ActionDescriptor,
) -> super::commands::ActionProjection {
    debug_assert!(!action.id.is_empty());
    debug_assert!(!action.label.is_empty());
    let is_enabled = action.enabled;
    let diagnostic = disabled_diagnostic(action, is_enabled);
    let shortcut_label = action
        .shortcut
        .as_ref()
        .map(crate::action_registry::ActionShortcut::display);
    super::commands::ActionProjection {
        action_id: action.id.clone(),
        label: action.label.clone(),
        category: ACTION_CATEGORY_SPACE.to_string(),
        shortcut_label,
        enabled: is_enabled,
        disabled_reason: action.disabled_reason.clone(),
        focus_policy: action.focus_policy,
        diagnostic,
    }
}

fn disabled_diagnostic(
    action: &super::commands::ActionDescriptor,
    is_enabled: bool,
) -> Option<super::types::Diagnostic> {
    if is_enabled {
        return None;
    }
    Some(super::types::Diagnostic::new(
        super::types::DiagnosticCode::DisabledAction,
        action.id.clone(),
        action
            .disabled_reason
            .clone()
            .unwrap_or_else(|| String::from("disabled")),
    ))
}

fn shortcut_help(
    snapshot: &crate::action_registry::ActionRegistrySnapshot,
) -> std::collections::BTreeMap<String, String> {
    snapshot
        .shortcut_help_sections()
        .into_iter()
        .flat_map(|section| section.entries.into_iter())
        .map(|entry| (entry.action_id, entry.shortcut_label))
        .collect()
}

fn space_view(space: &super::state::Space, model: &super::state::Model) -> SpaceView {
    let is_active = model
        .focus
        .as_ref()
        .map(|focus| focus.workspace == space.live_id)
        .unwrap_or(false);
    let tabs = space
        .tabs
        .iter()
        .map(|tab| tab_view(tab, space, model))
        .collect();
    SpaceView {
        live_id: space.live_id.clone(),
        title: space.title.clone(),
        active: is_active,
        status_rollup: super::types::space_rollup(space),
        tabs,
    }
}

fn tab_view(
    tab: &super::state::Tab,
    space: &super::state::Space,
    model: &super::state::Model,
) -> TabView {
    let is_active = space.active_tab.as_ref() == Some(&tab.live_id);
    let panes = tab
        .panes
        .iter()
        .map(|pane| pane_view(pane, tab, space, model))
        .collect();
    TabView {
        live_id: tab.live_id.clone(),
        title: tab.title.clone(),
        active: is_active,
        status_rollup: super::types::tab_rollup(tab),
        panes,
    }
}

fn pane_view(
    pane: &super::state::Pane,
    tab: &super::state::Tab,
    space: &super::state::Space,
    model: &super::state::Model,
) -> PaneView {
    let is_focused = model
        .focus
        .as_ref()
        .map(|focus| {
            focus.workspace == space.live_id
                && focus.tab == tab.live_id
                && focus.pane == pane.live_id
        })
        .unwrap_or(false);
    PaneView {
        live_id: pane.live_id.clone(),
        title: pane.title.clone(),
        status: pane.status.label().to_string(),
        focused: is_focused,
        content_summary: pane.content_summary.clone(),
        diagnostics: pane.diagnostics.iter().map(diagnostic_view).collect(),
    }
}

fn diagnostic_view(diagnostic: &super::types::Diagnostic) -> DiagnosticView {
    DiagnosticView {
        code: format!("{:?}", diagnostic.code),
        subject: diagnostic.subject.clone(),
        message: diagnostic.message.clone(),
    }
}

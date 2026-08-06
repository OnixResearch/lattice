//! Workspace shell renderer-neutral model types.

/// Shell focus target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Focus {
    /// Focused workspace live ID.
    pub workspace: super::types::WorkspaceId,
    /// Focused tab live ID.
    pub tab: super::types::TabId,
    /// Focused pane live ID.
    pub pane: super::types::PaneId,
}

impl Focus {
    /// Creates a focus target.
    #[must_use]
    pub fn new(
        workspace: impl Into<super::types::WorkspaceId>,
        tab: impl Into<super::types::TabId>,
        pane: impl Into<super::types::PaneId>,
    ) -> Self {
        Self {
            workspace: workspace.into(),
            tab: tab.into(),
            pane: pane.into(),
        }
    }
}

/// Workspace pane state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
    /// Compact session-visible pane ID.
    pub live_id: super::types::PaneId,
    /// Durable persisted pane ID.
    pub persisted_id: super::types::PersistedPaneId,
    /// User-facing pane title.
    pub title: String,
    /// Current portable task status.
    pub status: super::types::TaskStatus,
    /// Logical content summary used by replay and status projections.
    pub content_summary: String,
    /// Pane-local deterministic diagnostics.
    pub diagnostics: Vec<super::types::Diagnostic>,
}

impl Pane {
    /// Creates a pane with empty content and no diagnostics.
    #[must_use]
    pub fn new(
        live_id: impl Into<super::types::PaneId>,
        persisted_id: impl Into<super::types::PersistedPaneId>,
        title: impl Into<String>,
        status: super::types::TaskStatus,
    ) -> Self {
        Self {
            live_id: live_id.into(),
            persisted_id: persisted_id.into(),
            title: title.into(),
            status,
            content_summary: String::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Sets a deterministic content summary.
    #[must_use]
    pub fn with_content_summary(mut self, content_summary: impl Into<String>) -> Self {
        self.content_summary = content_summary.into();
        self
    }

    /// Adds a visible pane diagnostic.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: super::types::Diagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }
}

/// Workspace tab state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    /// Compact session-visible tab ID.
    pub live_id: super::types::TabId,
    /// Durable persisted tab ID.
    pub persisted_id: super::types::PersistedTabId,
    /// User-facing tab title.
    pub title: String,
    /// Panes inside the tab.
    pub panes: Vec<Pane>,
    /// Active pane live ID when a pane is selected.
    pub active_pane: Option<super::types::PaneId>,
}

impl Tab {
    /// Creates a tab and selects its first pane when present.
    #[must_use]
    pub fn new(
        live_id: impl Into<super::types::TabId>,
        persisted_id: impl Into<super::types::PersistedTabId>,
        title: impl Into<String>,
        panes: Vec<Pane>,
    ) -> Self {
        let active_pane = panes.first().map(|pane| pane.live_id.clone());
        Self {
            live_id: live_id.into(),
            persisted_id: persisted_id.into(),
            title: title.into(),
            panes,
            active_pane,
        }
    }
}

/// Workspace state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Space {
    /// Compact session-visible workspace ID.
    pub live_id: super::types::WorkspaceId,
    /// Durable persisted workspace ID.
    pub persisted_id: super::types::PersistedWorkspaceId,
    /// User-facing workspace title.
    pub title: String,
    /// Tabs inside the workspace.
    pub tabs: Vec<Tab>,
    /// Active tab live ID when a tab is selected.
    pub active_tab: Option<super::types::TabId>,
}

impl Space {
    /// Creates a workspace and selects its first tab when present.
    #[must_use]
    pub fn new(
        live_id: impl Into<super::types::WorkspaceId>,
        persisted_id: impl Into<super::types::PersistedWorkspaceId>,
        title: impl Into<String>,
        tabs: Vec<Tab>,
    ) -> Self {
        let active_tab = tabs.first().map(|tab| tab.live_id.clone());
        Self {
            live_id: live_id.into(),
            persisted_id: persisted_id.into(),
            title: title.into(),
            tabs,
            active_tab,
        }
    }
}

/// Workspace shell state accepted by pure reducers and renderer projections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    /// Workspaces in deterministic display order.
    pub workspaces: Vec<Space>,
    /// Focused pane target when a focusable pane exists.
    pub focus: Option<Focus>,
    /// Retired live IDs that must be rejected as stale.
    pub retired_live_ids: std::collections::BTreeSet<super::types::RetiredLiveId>,
    /// Model-level deterministic diagnostics.
    pub diagnostics: Vec<super::types::Diagnostic>,
}

impl Model {
    /// Creates a shell model and chooses the first focusable pane.
    #[must_use]
    pub fn new(workspaces: Vec<Space>) -> Self {
        let focus = first_focus(&workspaces);
        Self {
            workspaces,
            focus,
            retired_live_ids: std::collections::BTreeSet::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Overrides the focused pane target.
    #[must_use]
    pub fn with_focus(mut self, focus: Focus) -> Self {
        self.focus = Some(focus);
        self
    }

    /// Records a retired live ID for deterministic stale-ID rejection.
    #[must_use]
    pub fn with_retired_live_id(mut self, live_id: impl Into<super::types::RetiredLiveId>) -> Self {
        self.retired_live_ids.insert(live_id.into());
        self
    }

    /// Adds a model-level visible diagnostic.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: super::types::Diagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }
}

pub(crate) fn first_focus(workspaces: &[Space]) -> Option<Focus> {
    workspaces.iter().find_map(|workspace| {
        workspace.tabs.iter().find_map(|tab| {
            tab.panes
                .first()
                .map(|pane| Focus::new(workspace.live_id.clone(), tab.live_id.clone(), pane.live_id.clone()))
        })
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TabPosition {
    pub workspace: usize,
    pub tab: usize,
}

pub(crate) fn repair_focus_after_close(model: &mut Model, position: TabPosition, closed_pane: &super::types::PaneId) {
    if model.focus.as_ref().map(|focus| &focus.pane) != Some(closed_pane) {
        return;
    }
    let workspace_id = model.workspaces[position.workspace].live_id.clone();
    let tab = &mut model.workspaces[position.workspace].tabs[position.tab];
    let tab_id = tab.live_id.clone();
    let next_active_pane = tab.panes.first().map(|pane| pane.live_id.clone());
    tab.active_pane.clone_from(&next_active_pane);
    model.focus = next_active_pane.map(|pane| Focus::new(workspace_id, tab_id, pane));
}

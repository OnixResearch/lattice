//! Pure workspace shell model, reducer, action projection, and view models.
//!
//! The shell owns renderer-neutral state only. Applications own process handles,
//! terminals, browser APIs, sockets, filesystems, and live status adapters.

#[path = "parts/commands.rs"]
mod commands;
#[path = "parts/reducer.rs"]
mod reducer;
#[path = "parts/state.rs"]
mod state;
#[path = "parts/support.rs"]
mod support;
#[path = "parts/types.rs"]
mod types;
#[path = "parts/validation.rs"]
mod validation;
#[path = "parts/views.rs"]
mod views;

/// Workspace action metadata before registry projection.
pub type WorkspaceShellActionDescriptor = commands::ActionDescriptor;
/// Workspace action projection with focus-policy metadata.
pub type WorkspaceShellActionProjection = commands::ActionProjection;
/// Workspace command surfaces projected through the shared action registry.
pub type WorkspaceShellActionSurfaces = commands::ActionSurfaces;
/// Pure workspace shell command.
pub type WorkspaceShellCommand = commands::Command;
/// Result of reducing one workspace command.
pub type WorkspaceShellCommandOutput = commands::CommandOutput;
/// Deterministic workspace shell diagnostic.
pub type WorkspaceShellDiagnostic = types::Diagnostic;
/// Stable workspace shell diagnostic code.
pub type WorkspaceShellDiagnosticCode = types::DiagnosticCode;
/// Renderable diagnostic view.
pub type WorkspaceShellDiagnosticView = views::DiagnosticView;
/// Declarative effect request returned by the pure reducer.
pub type WorkspaceShellEffectRequest = commands::EffectRequest;
/// Shell focus target.
pub type WorkspaceShellFocus = state::Focus;
/// Typed UI intent emitted by workspace shell components.
pub type WorkspaceShellIntent = commands::Intent;
/// Workspace shell state accepted by reducers and projections.
pub type WorkspaceShellModel = state::Model;
/// Workspace pane state.
pub type WorkspaceShellPane = state::Pane;
/// Renderable pane view.
pub type WorkspaceShellPaneView = views::PaneView;
/// Workspace tab state.
pub type WorkspaceShellTab = state::Tab;
/// Renderable tab view.
pub type WorkspaceShellTabView = views::TabView;
/// Meteorite-facing workspace shell view model.
pub type WorkspaceShellViewModel = views::ViewModel;
/// Workspace state.
pub type WorkspaceShellWorkspace = state::Space;
/// Renderable workspace view.
pub type WorkspaceShellWorkspaceView = views::SpaceView;
/// Focus behavior for commands that create panes or tabs.
pub type WorkspaceFocusPolicy = commands::FocusPolicy;
/// Compact session-visible workspace, tab, or pane identifier.
pub type WorkspaceLiveId = types::LiveId;
/// Durable persisted workspace, tab, pane, or layout identifier.
pub type WorkspacePersistedId = types::PersistedId;
/// Split axis requested by a workspace shell command.
pub type WorkspaceSplitAxis = commands::SplitAxis;
/// Aggregated task status counts and overall state.
pub type WorkspaceStatusRollup = types::StatusRollup;
/// Portable task status vocabulary for workspace panes.
pub type WorkspaceTaskStatus = types::TaskStatus;

/// Projects workspace action descriptors through the shared action registry.
#[must_use]
pub fn project_workspace_shell_actions(
    actions: Vec<WorkspaceShellActionDescriptor>,
) -> WorkspaceShellActionSurfaces {
    views::project_actions(actions)
}

/// Builds a renderer-neutral workspace shell view model.
#[must_use]
pub fn build_workspace_shell_view_model(
    model: &WorkspaceShellModel,
    actions: &WorkspaceShellActionSurfaces,
) -> WorkspaceShellViewModel {
    views::build_view_model(model, actions)
}

/// Reduces a command against a workspace shell model.
#[must_use]
pub fn reduce_workspace_shell_command(
    model: &WorkspaceShellModel,
    command: &WorkspaceShellCommand,
) -> WorkspaceShellCommandOutput {
    reducer::reduce_command(model, command)
}

/// Rolls pane statuses into stable counts and an overall status.
#[must_use]
pub fn rollup_workspace_statuses(
    statuses: impl IntoIterator<Item = WorkspaceTaskStatus>,
) -> WorkspaceStatusRollup {
    types::rollup(statuses)
}

/// Rolls a tab's panes into stable status counts.
#[must_use]
pub fn tab_status_rollup(tab: &WorkspaceShellTab) -> WorkspaceStatusRollup {
    types::tab_rollup(tab)
}

/// Rolls a workspace's panes into stable status counts.
#[must_use]
pub fn workspace_status_rollup(workspace: &WorkspaceShellWorkspace) -> WorkspaceStatusRollup {
    types::space_rollup(workspace)
}

/// Validates live IDs, durable IDs, and focus targets.
#[must_use]
pub fn validate_workspace_shell(model: &WorkspaceShellModel) -> Vec<WorkspaceShellDiagnostic> {
    validation::validate_model(model)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

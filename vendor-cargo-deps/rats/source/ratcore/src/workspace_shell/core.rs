//! Pure workspace shell model, reducer, action projection, and view models.
//!
//! The shell owns renderer-neutral state only. Applications own process handles,
//! terminals, browser APIs, sockets, filesystems, and live status adapters.

#[path = "parts/commands.rs"]
mod commands;
#[path = "parts/reducer.rs"]
mod reducer;
#[path = "parts/routing/mod.rs"]
mod routing;
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
/// Representative contextual action used by the workspace pilot.
pub type WorkspaceContextualAction = routing::ContextualAction;
/// Focused editor/tree kind used by contextual routing.
pub type WorkspaceContextualContentKind = routing::ContextualContentKind;
/// Active modal state used by contextual routing.
pub type WorkspaceContextualModalState = routing::ContextualModalState;
/// Closed availability state for a workspace contextual capability.
pub type WorkspaceContextualAvailability = routing::ContextualAvailability;
/// Application-owned inputs for one contextual routing generation.
pub type WorkspaceContextualRoutingOptions = routing::Options;
/// Atomic contextual routing build plus shared command surfaces.
pub type WorkspaceContextualActionPilot = routing::ContextualActionPilot;
/// Typed contextual owner delivered after pure resolution.
pub type WorkspaceContextualActionTarget = routing::ContextualActionTarget;
/// Typed runtime message emitted after accepted contextual routing.
pub type WorkspaceContextualActionMessage = routing::ContextualActionMessage;
/// Generation-bound workspace contextual dispatch result.
pub type WorkspaceContextualActionDispatch = routing::ContextualActionDispatch;
/// Layered key binding plus workspace contextual dispatch result.
pub type WorkspaceContextualKeyDispatch = routing::ContextualKeyDispatch;
/// Inputs for dispatching a resolved key binding into a workspace message.
pub type WorkspaceContextualKeyDispatchRequest<'a> = routing::ContextualKeyDispatchRequest<'a>;
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
/// Compact session-visible workspace identifier.
pub type WorkspaceId = types::WorkspaceId;
/// Durable persisted workspace identifier.
pub type PersistedWorkspaceId = types::PersistedWorkspaceId;
/// Compact session-visible tab identifier.
pub type WorkspaceTabId = types::TabId;
/// Durable persisted tab identifier.
pub type PersistedWorkspaceTabId = types::PersistedTabId;
/// Compact session-visible pane identifier.
pub type WorkspacePaneId = types::PaneId;
/// Durable persisted pane identifier.
pub type PersistedWorkspacePaneId = types::PersistedPaneId;
/// Typed identity retained after an entity is removed.
pub type RetiredWorkspaceLiveId = types::RetiredLiveId;
/// Compatibility name for the session-visible workspace identity.
pub type WorkspaceLiveId = WorkspaceId;
/// Compatibility name for the durable workspace identity.
pub type WorkspacePersistedId = PersistedWorkspaceId;
/// Split axis requested by a workspace shell command.
pub type WorkspaceSplitAxis = commands::SplitAxis;
/// Aggregated task status counts and overall state.
pub type WorkspaceStatusRollup = types::StatusRollup;
/// Portable task status vocabulary for workspace panes.
pub type WorkspaceTaskStatus = types::TaskStatus;

/// Projects workspace action descriptors through the shared action registry.
#[must_use]
pub fn project_workspace_shell_actions(actions: Vec<WorkspaceShellActionDescriptor>) -> WorkspaceShellActionSurfaces {
    views::project_actions(actions)
}

/// Builds shared contextual routing and command surfaces for the workspace pilot.
#[must_use]
pub fn build_workspace_contextual_action_pilot(
    model: &WorkspaceShellModel,
    options: &WorkspaceContextualRoutingOptions,
    previous: Option<&crate::action_registry::ActionRoutingSnapshot>,
) -> WorkspaceContextualActionPilot {
    routing::build_contextual_action_pilot(model, options, previous)
}

/// Converts an accepted contextual route into a typed workspace runtime message.
#[must_use]
pub fn dispatch_workspace_contextual_action(
    model: &WorkspaceShellModel,
    snapshot: &crate::action_registry::ActionRoutingSnapshot,
    invocation: &crate::action_registry::ActionInvocation,
) -> WorkspaceContextualActionDispatch {
    routing::dispatch_contextual_action(model, snapshot, invocation)
}

/// Routes a resolved key binding into a typed workspace runtime message.
#[must_use]
pub fn dispatch_workspace_contextual_key_sequence(
    model: &WorkspaceShellModel,
    request: &WorkspaceContextualKeyDispatchRequest<'_>,
) -> WorkspaceContextualKeyDispatch {
    routing::dispatch_contextual_key_sequence(model, request)
}

/// Replays workspace activations through production routing and message adaptation.
#[must_use]
pub fn replay_workspace_contextual_actions(
    model: &WorkspaceShellModel,
    snapshot: &crate::action_registry::ActionRoutingSnapshot,
    steps: &[crate::action_registry::ActionRouteReplayStep],
) -> Vec<WorkspaceContextualActionDispatch> {
    routing::replay_contextual_actions(model, snapshot, steps)
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
pub fn rollup_workspace_statuses(statuses: impl IntoIterator<Item = WorkspaceTaskStatus>) -> WorkspaceStatusRollup {
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

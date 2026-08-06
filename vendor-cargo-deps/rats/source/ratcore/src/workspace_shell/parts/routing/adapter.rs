/// Generation-bound invocation result before application execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextualActionDispatch {
    /// Production contextual invocation receipt.
    pub receipt: crate::action_registry::ActionInvocationReceipt,
    /// Typed runtime message, absent when resolution rejected the request.
    pub message: Option<super::ContextualActionMessage>,
}

/// Layered key binding plus contextual route and typed runtime adaptation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextualKeyDispatch {
    /// Shared binding-to-contextual-route receipt.
    pub binding: crate::action_registry::ResolvedBindingDispatch,
    /// Typed runtime message, absent when binding/routing rejected dispatch.
    pub message: Option<super::ContextualActionMessage>,
}

/// Builds the workspace contextual routing pilot from accepted shell state.
#[must_use]
pub fn build_contextual_action_pilot(
    model: &super::super::state::Model,
    options: &super::Options,
    previous: Option<&crate::action_registry::ActionRoutingSnapshot>,
) -> super::ContextualActionPilot {
    let registry = super::catalog::action_registry();
    let candidate = super::catalog::candidate(model, options);
    let routing =
        crate::action_registry::accept_contextual_action_snapshot(candidate, &registry.snapshot, None, previous);
    let surfaces = routing.snapshot.as_ref().map(crate::action_registry::project_contextual_action_surfaces);
    super::ContextualActionPilot { routing, surfaces }
}

/// Normalizes one accepted route into a typed workspace runtime message.
#[must_use]
pub fn dispatch_contextual_action(
    model: &super::super::state::Model,
    snapshot: &crate::action_registry::ActionRoutingSnapshot,
    invocation: &crate::action_registry::ActionInvocation,
) -> ContextualActionDispatch {
    let receipt = crate::action_registry::invoke_contextual_action(snapshot, invocation);
    let message = receipt.probe.outcome.route().and_then(|route| message_from_route(model, route));
    ContextualActionDispatch { receipt, message }
}

/// Inputs required to route one accepted key binding into a workspace message.
pub struct ContextualKeyDispatchRequest<'a> {
    /// Current contextual routing snapshot.
    pub routing: &'a crate::action_registry::ActionRoutingSnapshot,
    /// Current accepted binding catalog.
    pub catalog: &'a crate::action_registry::BindingCatalogSnapshot,
    /// Current accepted active layers.
    pub active: &'a crate::action_registry::ActiveBindingLayers,
    /// Previously resolved binding provenance.
    pub binding: &'a crate::action_registry::ResolvedActionBinding,
    /// Renderer adapter provenance.
    pub renderer: crate::action_registry::ActionRendererAdapter,
}

/// Dispatches a resolved key binding through contextual routing and typed messages.
#[must_use]
pub fn dispatch_contextual_key_sequence(
    model: &super::super::state::Model,
    request: &ContextualKeyDispatchRequest<'_>,
) -> ContextualKeyDispatch {
    let dispatch = crate::action_registry::dispatch_resolved_binding(
        request.routing,
        request.catalog,
        request.active,
        request.binding,
        request.renderer,
    );
    let message = dispatch
        .invocation
        .as_ref()
        .and_then(|receipt| receipt.probe.outcome.route())
        .and_then(|route| message_from_route(model, route));
    ContextualKeyDispatch {
        binding: dispatch,
        message,
    }
}

/// Replays normalized actions through production routing and message adaptation.
#[must_use]
pub fn replay_contextual_actions(
    model: &super::super::state::Model,
    snapshot: &crate::action_registry::ActionRoutingSnapshot,
    steps: &[crate::action_registry::ActionRouteReplayStep],
) -> Vec<ContextualActionDispatch> {
    steps
        .iter()
        .map(|step| {
            let mut invocation = crate::action_registry::ActionInvocation::new(
                step.action_id.clone(),
                step.observed_generation,
                step.surface,
                step.renderer,
            );
            invocation.active_chain.clone_from(&step.active_chain);
            dispatch_contextual_action(model, snapshot, &invocation)
        })
        .collect()
}

pub(super) fn focused_entities(
    model: &super::super::state::Model,
) -> Option<(&super::super::state::Space, &super::super::state::Tab, &super::super::state::Pane)> {
    let focus = model.focus.as_ref()?;
    let workspace = model.workspaces.iter().find(|workspace| workspace.live_id == focus.workspace)?;
    let tab = workspace.tabs.iter().find(|tab| tab.live_id == focus.tab)?;
    let pane = tab.panes.iter().find(|pane| pane.live_id == focus.pane)?;
    Some((workspace, tab, pane))
}

fn message_from_route(
    model: &super::super::state::Model,
    route: &crate::action_registry::ActionRoute,
) -> Option<super::ContextualActionMessage> {
    debug_assert!(route.routing_generation > 0);
    debug_assert!(!route.action_id.as_str().is_empty());
    let (workspace, tab, pane) = focused_entities(model)?;
    let target = target_from_route(workspace, pane, route);
    let action_id = route.action_id.as_str();
    match action_id {
        super::ACTION_CLOSE_PANE => Some(super::ContextualActionMessage::ClosePane {
            workspace: workspace.live_id.clone(),
            tab: tab.live_id.clone(),
            pane: pane.live_id.clone(),
            target,
        }),
        super::ACTION_FIND => Some(super::ContextualActionMessage::Find { target }),
        super::ACTION_SELECT_ALL => Some(super::ContextualActionMessage::SelectAll { target }),
        super::ACTION_REFRESH => Some(super::ContextualActionMessage::Refresh { target }),
        super::ACTION_OPEN_PALETTE => Some(super::ContextualActionMessage::OpenCommandPalette { target }),
        super::ACTION_DISMISS_MODAL => Some(super::ContextualActionMessage::DismissModal { target }),
        _ => None,
    }
}

fn target_from_route(
    workspace: &super::super::state::Space,
    pane: &super::super::state::Pane,
    route: &crate::action_registry::ActionRoute,
) -> super::ContextualActionTarget {
    if route.owner_context == super::CONTEXT_APPLICATION {
        return super::ContextualActionTarget::Application;
    }
    if route.owner_context == super::catalog::ids::workspace_context_id(&workspace.live_id) {
        return super::ContextualActionTarget::Workspace(workspace.live_id.clone());
    }
    if route.owner_context == super::catalog::ids::pane_context_id(&pane.live_id) {
        return super::ContextualActionTarget::Pane(pane.live_id.clone());
    }
    if route.owner_context.as_str().starts_with("workspace-context.modal.") {
        return super::ContextualActionTarget::Modal(route.owner_context.clone());
    }
    super::ContextualActionTarget::Content(pane.live_id.clone())
}

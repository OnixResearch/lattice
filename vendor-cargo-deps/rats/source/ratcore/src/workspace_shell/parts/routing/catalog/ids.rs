pub(in crate::terminalkit) fn workspace_context_id(
    id: &super::super::super::types::WorkspaceId,
) -> crate::action_registry::ActionContextId {
    crate::action_registry::ActionContextId::new(format!("workspace-context.workspace.{id}"))
}

pub(in crate::terminalkit) fn pane_context_id(
    id: &super::super::super::types::PaneId,
) -> crate::action_registry::ActionContextId {
    crate::action_registry::ActionContextId::new(format!("workspace-context.pane.{id}"))
}

pub(in crate::terminalkit) fn content_context_id(
    id: &super::super::super::types::PaneId,
    kind: super::super::ContextualContentKind,
) -> crate::action_registry::ActionContextId {
    crate::action_registry::ActionContextId::new(format!("workspace-context.{}.{id}", kind.fact_value()))
}

pub(in crate::terminalkit) fn modal_context_id(id: &str) -> crate::action_registry::ActionContextId {
    crate::action_registry::ActionContextId::new(format!("workspace-context.modal.{id}"))
}

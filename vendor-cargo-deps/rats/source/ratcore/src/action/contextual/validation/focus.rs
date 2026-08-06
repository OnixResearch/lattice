pub(super) fn validate_links(
    contexts: &std::collections::BTreeMap<super::super::ActionContextId, super::super::model::ActionContext>,
    active_chain: &[super::super::ActionContextId],
    focus_graph: Option<&crate::focus::FocusGraph>,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    debug_assert!(diagnostics.items.capacity() >= super::MAX_ROUTING_DIAGNOSTICS);
    let has_links = contexts.values().any(|context| context.focus_scope.is_some() || context.focus_node.is_some());
    let Some(graph) = focus_graph else {
        if has_links {
            diagnostics.push(super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::StaleFocusLink,
                "context.focus",
                "focus-linked contexts require an accepted focus graph",
            ));
        }
        return;
    };
    if graph.validate().is_err() {
        diagnostics.push(super::super::model::RoutingDiagnostic::new(
            super::super::model::RoutingDiagnosticCode::StaleFocusLink,
            "context.focus",
            "associated focus graph is invalid",
        ));
        return;
    }
    for context in contexts.values() {
        validate_context_link(context, graph, diagnostics);
    }
    if let Some(focused) = active_chain.first().and_then(|id| contexts.get(id))
        && let Some(node_id) = &focused.focus_node
        && graph.active_id() != Some(node_id)
    {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::StaleFocusLink,
                "context.focus_node",
                "focused action context does not match the active focus node",
            )
            .with_context(focused.id.clone()),
        );
    }
    if let Some(modal) = active_chain.iter().filter_map(|id| contexts.get(id)).find(|context| context.modal.is_some())
        && let Some(scope_id) = &modal.focus_scope
        && graph.modal_scope() != Some(scope_id)
    {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::StaleFocusLink,
                "context.focus_scope",
                "active modal context does not match the modal focus scope",
            )
            .with_context(modal.id.clone()),
        );
    }
}

pub(super) fn validate_context_link(
    context: &super::super::model::ActionContext,
    graph: &crate::focus::FocusGraph,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    debug_assert!(diagnostics.items.capacity() >= super::MAX_ROUTING_DIAGNOSTICS);
    if let Some(scope_id) = &context.focus_scope
        && graph.scope(scope_id).is_none()
    {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::StaleFocusLink,
                "context.focus_scope",
                "context references an unknown focus scope",
            )
            .with_context(context.id.clone()),
        );
    }
    if let Some(node_id) = &context.focus_node {
        let Some(node) = graph.node(node_id) else {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::StaleFocusLink,
                    "context.focus_node",
                    "context references an unknown focus node",
                )
                .with_context(context.id.clone()),
            );
            return;
        };
        if context.focus_scope.as_ref().is_some_and(|scope_id| scope_id != &node.scope) {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::StaleFocusLink,
                    "context.focus_node",
                    "context focus node belongs to a different focus scope",
                )
                .with_context(context.id.clone()),
            );
        }
    }
}

pub(super) fn validate_hierarchy(
    contexts: &std::collections::BTreeMap<super::super::ActionContextId, super::super::model::ActionContext>,
    candidate: &super::super::model::ActionRoutingCandidate,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    debug_assert!(diagnostics.items.capacity() >= super::MAX_ROUTING_DIAGNOSTICS);
    let roots = contexts.values().filter(|context| context.parent.is_none()).collect::<Vec<_>>();
    if roots.len() != 1 || roots[0].kind != super::super::model::ActionContextKind::Application {
        diagnostics.push(super::super::model::RoutingDiagnostic::new(
            super::super::model::RoutingDiagnosticCode::InvalidRoot,
            "context.parent",
            "context graph requires exactly one application root",
        ));
    }
    for context in contexts.values() {
        if context.generation > candidate.generation {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::StaleContextGeneration,
                    "context.generation",
                    "context generation cannot exceed routing generation",
                )
                .with_context(context.id.clone()),
            );
        }
        if let Some(parent) = &context.parent
            && !contexts.contains_key(parent)
        {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::MissingParent,
                    "context.parent",
                    format!("context references missing parent `{parent}`"),
                )
                .with_context(context.id.clone())
                .with_context(parent.clone()),
            );
        }
        detect_cycle(context, contexts, diagnostics);
    }
    validate_sibling_order(contexts, diagnostics);
}

pub(super) fn detect_cycle(
    start: &super::super::model::ActionContext,
    contexts: &std::collections::BTreeMap<super::super::ActionContextId, super::super::model::ActionContext>,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    debug_assert!(diagnostics.items.capacity() >= super::MAX_ROUTING_DIAGNOSTICS);
    let mut seen = std::collections::BTreeSet::new();
    let mut current = Some(start.id.clone());
    while let Some(context_id) = current {
        if !seen.insert(context_id.clone()) {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::ContextCycle,
                    "context.parent",
                    "context graph contains a parent cycle",
                )
                .with_context(context_id),
            );
            return;
        }
        current = contexts.get(&context_id).and_then(|context| context.parent.clone());
    }
}

pub(super) fn validate_sibling_order(
    contexts: &std::collections::BTreeMap<super::super::ActionContextId, super::super::model::ActionContext>,
    diagnostics: &mut super::DiagnosticBag,
) {
    let mut order = std::collections::BTreeMap::new();
    for context in contexts.values() {
        let key = (context.parent.clone(), context.order);
        if let Some(first) = order.insert(key, context.id.clone()) {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::MalformedIdentity,
                    "context.order",
                    "sibling context order must be unique",
                )
                .with_context(first)
                .with_context(context.id.clone()),
            );
        }
    }
}

pub(super) fn derive_active_chain(
    contexts: &std::collections::BTreeMap<super::super::ActionContextId, super::super::model::ActionContext>,
    focused_context: &super::super::ActionContextId,
) -> Vec<super::super::ActionContextId> {
    let mut chain = Vec::with_capacity(contexts.len());
    let mut current = Some(focused_context.clone());
    while let Some(context_id) = current {
        if chain.contains(&context_id) || chain.len() >= contexts.len() {
            break;
        }
        chain.push(context_id.clone());
        current = contexts.get(&context_id).and_then(|context| context.parent.clone());
    }
    chain
}

pub(super) fn validate_active_chain(
    contexts: &std::collections::BTreeMap<super::super::ActionContextId, super::super::model::ActionContext>,
    active_chain: &[super::super::ActionContextId],
    candidate: &super::super::model::ActionRoutingCandidate,
    diagnostics: &mut super::DiagnosticBag,
) {
    debug_assert!(diagnostics.items.len() <= super::MAX_ROUTING_DIAGNOSTICS);
    debug_assert!(diagnostics.items.capacity() >= super::MAX_ROUTING_DIAGNOSTICS);
    if super::malformed_id(candidate.focused_context.as_str())
        || !contexts.contains_key(&candidate.focused_context)
        || active_chain.is_empty()
    {
        diagnostics.push(
            super::super::model::RoutingDiagnostic::new(
                super::super::model::RoutingDiagnosticCode::InvalidActiveChain,
                "routing.focused_context",
                "focused context is unknown or malformed",
            )
            .with_context(candidate.focused_context.clone()),
        );
        return;
    }
    for context_id in active_chain {
        let Some(context) = contexts.get(context_id) else {
            continue;
        };
        if !context.active || !context.visible {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::InvalidActiveChain,
                    "routing.active_chain",
                    "active chain contains an inactive or hidden context",
                )
                .with_context(context_id.clone()),
            );
        }
    }
    let active_modals =
        contexts.values().filter(|context| context.active && context.modal.is_some()).collect::<Vec<_>>();
    for modal in active_modals {
        if !active_chain.contains(&modal.id) {
            diagnostics.push(
                super::super::model::RoutingDiagnostic::new(
                    super::super::model::RoutingDiagnosticCode::ContradictoryModalRoot,
                    "routing.active_modal",
                    "active modal context is outside the focused chain",
                )
                .with_context(modal.id.clone()),
            );
        }
    }
}

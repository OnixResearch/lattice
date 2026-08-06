#[path = "catalog/ids.rs"]
pub(in crate::terminalkit) mod ids;

pub(super) fn action_registry() -> crate::action_registry::ActionRegistryBuild {
    crate::action_registry::build_registry(vec![
        crate::action_registry::ActionDescriptor::new(
            super::ACTION_CLOSE_PANE,
            "Close Pane",
            crate::action_registry::ActionSource::new(super::SOURCE_WORKSPACE_CONTEXT),
        )
        .with_category("workspace")
        .with_shortcut(crate::action_registry::ActionShortcut::global("w", vec!["Ctrl".to_string()])),
        crate::action_registry::ActionDescriptor::new(
            super::ACTION_FIND,
            "Find",
            crate::action_registry::ActionSource::new(super::SOURCE_WORKSPACE_CONTEXT),
        )
        .with_category("edit")
        .with_shortcut(crate::action_registry::ActionShortcut::global("f", vec!["Ctrl".to_string()])),
        crate::action_registry::ActionDescriptor::new(
            super::ACTION_SELECT_ALL,
            "Select All",
            crate::action_registry::ActionSource::new(super::SOURCE_WORKSPACE_CONTEXT),
        )
        .with_category("edit")
        .with_shortcut(crate::action_registry::ActionShortcut::global("a", vec!["Ctrl".to_string()])),
        crate::action_registry::ActionDescriptor::new(
            super::ACTION_REFRESH,
            "Refresh",
            crate::action_registry::ActionSource::new(super::SOURCE_WORKSPACE_CONTEXT),
        )
        .with_category("workspace")
        .with_shortcut(crate::action_registry::ActionShortcut::global("r", vec!["Ctrl".to_string()])),
        crate::action_registry::ActionDescriptor::new(
            super::ACTION_OPEN_PALETTE,
            "Open Command Palette",
            crate::action_registry::ActionSource::new(super::SOURCE_WORKSPACE_CONTEXT),
        )
        .with_category("application")
        .with_shortcut(crate::action_registry::ActionShortcut::global("p", vec![
            "Ctrl".to_string(),
            "Shift".to_string(),
        ])),
        crate::action_registry::ActionDescriptor::new(
            super::ACTION_DISMISS_MODAL,
            "Dismiss Modal",
            crate::action_registry::ActionSource::new(super::SOURCE_WORKSPACE_CONTEXT),
        )
        .with_category("application")
        .with_shortcut(crate::action_registry::ActionShortcut::global("Escape", Vec::new())),
    ])
}

pub(super) fn candidate(
    model: &super::super::state::Model,
    options: &super::Options,
) -> crate::action_registry::ActionRoutingCandidate {
    debug_assert!(!super::CONTEXT_APPLICATION.is_empty());
    debug_assert!(!super::SOURCE_WORKSPACE_CONTEXT.is_empty());
    let generation = options.generation;
    let Some((workspace, _tab, pane)) = super::adapter::focused_entities(model) else {
        return crate::action_registry::ActionRoutingCandidate::new(
            generation,
            "workspace-context.invalid-focus",
            vec![crate::action_registry::ActionContext::root(
                super::CONTEXT_APPLICATION,
                crate::action_registry::ActionContextKind::Application,
                generation,
            )],
            Vec::new(),
        );
    };
    let workspace_context = ids::workspace_context_id(&workspace.live_id);
    let pane_context = ids::pane_context_id(&pane.live_id);
    let content_context = ids::content_context_id(&pane.live_id, options.content_kind);
    let mut contexts = vec![
        crate::action_registry::ActionContext::root(
            super::CONTEXT_APPLICATION,
            crate::action_registry::ActionContextKind::Application,
            generation,
        ),
        crate::action_registry::ActionContext::child(
            workspace_context.clone(),
            super::CONTEXT_APPLICATION,
            crate::action_registry::ActionContextKind::Workspace,
            generation,
        ),
        crate::action_registry::ActionContext::child(
            pane_context.clone(),
            workspace_context.clone(),
            crate::action_registry::ActionContextKind::Pane,
            generation,
        ),
        content_context_record(&content_context, &pane_context, options),
    ];
    let mut registrations = registrations(generation, &workspace_context, &pane_context, &content_context, options);
    let focused_context = if let Some(records) = modal_records(content_context.clone(), options) {
        contexts.push(records.context);
        registrations.push(records.registration);
        records.focused_context
    } else {
        content_context
    };
    crate::action_registry::ActionRoutingCandidate::new(generation, focused_context, contexts, registrations)
        .with_fact_definitions(vec![
            crate::action_registry::ActionContextFactDefinition::new(
                super::FACT_CONTENT_KIND,
                crate::action_registry::ActionContextFactValueKind::Text,
            ),
            crate::action_registry::ActionContextFactDefinition::new(
                super::FACT_SELECTION_AVAILABLE,
                crate::action_registry::ActionContextFactValueKind::Boolean,
            ),
        ])
}

struct ModalRecords {
    context: crate::action_registry::ActionContext,
    registration: crate::action_registry::ActionHandlerRegistration,
    focused_context: crate::action_registry::ActionContextId,
}

fn modal_records(
    content_context: crate::action_registry::ActionContextId,
    options: &super::Options,
) -> Option<ModalRecords> {
    debug_assert!(!super::HANDLER_MODAL_DISMISS.is_empty());
    debug_assert!(!super::ACTION_DISMISS_MODAL.is_empty());
    let modal = options.modal.as_ref()?;
    let modal_context = ids::modal_context_id(&modal.id);
    let boundary = if modal.allows_application_escape {
        crate::action_registry::ActionContextModalBoundary::allowing_application_escape()
    } else {
        crate::action_registry::ActionContextModalBoundary::blocking()
    };
    let context = crate::action_registry::ActionContext::child(
        modal_context.clone(),
        content_context,
        crate::action_registry::ActionContextKind::Modal,
        options.generation,
    )
    .with_modal(boundary);
    let dismiss = crate::action_registry::ActionHandlerRegistration::new(
        super::HANDLER_MODAL_DISMISS,
        super::ACTION_DISMISS_MODAL,
        modal_context.clone(),
        options.generation,
    );
    let registration = if modal.dismiss_enabled {
        dismiss
    } else {
        dismiss.disabled("modal cannot currently be dismissed")
    };
    Some(ModalRecords {
        context,
        registration,
        focused_context: modal_context,
    })
}

fn content_context_record(
    content_context: &crate::action_registry::ActionContextId,
    pane_context: &crate::action_registry::ActionContextId,
    options: &super::Options,
) -> crate::action_registry::ActionContext {
    let context = crate::action_registry::ActionContext::child(
        content_context.clone(),
        pane_context.clone(),
        options.content_kind.context_kind(),
        options.generation,
    )
    .with_fact(super::FACT_CONTENT_KIND, options.content_kind.fact_value())
    .with_fact(super::FACT_SELECTION_AVAILABLE, options.selection.is_enabled());
    if options.content.is_enabled() {
        context
    } else {
        context.disabled("focused content is disabled")
    }
}

fn registrations(
    generation: u64,
    workspace_context: &crate::action_registry::ActionContextId,
    pane_context: &crate::action_registry::ActionContextId,
    content_context: &crate::action_registry::ActionContextId,
    options: &super::Options,
) -> Vec<crate::action_registry::ActionHandlerRegistration> {
    debug_assert!(!workspace_context.as_str().is_empty());
    debug_assert!(!pane_context.as_str().is_empty());
    let workspace_refresh = crate::action_registry::ActionHandlerRegistration::new(
        super::HANDLER_WORKSPACE_REFRESH,
        super::ACTION_REFRESH,
        workspace_context.clone(),
        generation,
    )
    .with_priority(10)
    .with_propagation(crate::action_registry::ActionPropagation::Bubble);
    let pane_close = crate::action_registry::ActionHandlerRegistration::new(
        super::HANDLER_PANE_CLOSE,
        super::ACTION_CLOSE_PANE,
        pane_context.clone(),
        generation,
    );
    vec![
        crate::action_registry::ActionHandlerRegistration::new(
            super::HANDLER_APPLICATION_FIND,
            super::ACTION_FIND,
            super::CONTEXT_APPLICATION,
            generation,
        )
        .with_propagation(crate::action_registry::ActionPropagation::RootOnly),
        crate::action_registry::ActionHandlerRegistration::new(
            super::HANDLER_APPLICATION_REFRESH,
            super::ACTION_REFRESH,
            super::CONTEXT_APPLICATION,
            generation,
        )
        .with_propagation(crate::action_registry::ActionPropagation::RootOnly),
        crate::action_registry::ActionHandlerRegistration::new(
            super::HANDLER_APPLICATION_PALETTE,
            super::ACTION_OPEN_PALETTE,
            super::CONTEXT_APPLICATION,
            generation,
        )
        .with_propagation(crate::action_registry::ActionPropagation::RootOnly)
        .allowing_modal_escape(),
        if options.workspace_refresh.is_enabled() {
            workspace_refresh
        } else {
            workspace_refresh.disabled("workspace refresh is unavailable")
        },
        if options.close.is_enabled() {
            pane_close
        } else {
            pane_close.disabled("focused pane cannot be closed")
        },
        crate::action_registry::ActionHandlerRegistration::new(
            super::HANDLER_CONTENT_FIND,
            super::ACTION_FIND,
            content_context.clone(),
            generation,
        )
        .with_priority(10)
        .with_propagation(crate::action_registry::ActionPropagation::Bubble),
        select_all_registration(generation, content_context),
    ]
}

fn select_all_registration(
    generation: u64,
    content_context: &crate::action_registry::ActionContextId,
) -> crate::action_registry::ActionHandlerRegistration {
    crate::action_registry::ActionHandlerRegistration::new(
        super::HANDLER_CONTENT_SELECT_ALL,
        super::ACTION_SELECT_ALL,
        content_context.clone(),
        generation,
    )
    .with_predicate(crate::action_registry::ActionContextPredicate::FactEquals {
        fact_id: crate::action_registry::ActionContextFactId::new(super::FACT_SELECTION_AVAILABLE),
        value: crate::action_registry::ActionContextFactValue::Boolean(true),
    })
}

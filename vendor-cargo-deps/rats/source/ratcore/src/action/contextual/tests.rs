use super::*;

const ACTION_FIND: &str = "workspace.find";
const ACTION_SAVE: &str = "workspace.save";
const ACTION_PALETTE: &str = "workspace.palette";
const ACTION_UNHANDLED: &str = "workspace.unhandled";
const CONTEXT_APP: &str = "context.application";
const CONTEXT_WORKSPACE: &str = "context.workspace";
const CONTEXT_PANE: &str = "context.pane";
const CONTEXT_EDITOR: &str = "context.editor";
const FACT_KIND: &str = "content.kind";
const FACT_SELECTION: &str = "selection.available";
const HANDLER_APP_FIND: &str = "handler.application.find";
const HANDLER_EDITOR_FIND: &str = "handler.editor.find";
const HANDLER_EDITOR_SAVE: &str = "handler.editor.save";

#[path = "tests/predicates.rs"]
mod predicates;
#[path = "tests/resolution.rs"]
mod resolution;
#[path = "tests/validation.rs"]
mod validation;

fn registry() -> crate::nexus_core::ActionRegistrySnapshot {
    crate::nexus_core::build_registry(vec![
        crate::nexus_core::ActionDescriptor::new(ACTION_FIND, "Find", crate::nexus_core::ActionSource::new("test"))
            .with_shortcut(crate::nexus_core::ActionShortcut::global("f", vec!["Ctrl".to_string()])),
        crate::nexus_core::ActionDescriptor::new(ACTION_SAVE, "Save", crate::nexus_core::ActionSource::new("test"))
            .with_shortcut(crate::nexus_core::ActionShortcut::global("s", vec!["Ctrl".to_string()])),
        crate::nexus_core::ActionDescriptor::new(
            ACTION_PALETTE,
            "Palette",
            crate::nexus_core::ActionSource::new("test"),
        )
        .with_shortcut(crate::nexus_core::ActionShortcut::global("p", vec!["Ctrl".to_string()])),
        crate::nexus_core::ActionDescriptor::new(
            ACTION_UNHANDLED,
            "Unhandled",
            crate::nexus_core::ActionSource::new("test"),
        ),
    ])
    .snapshot
}

fn contexts(generation: u64) -> Vec<ActionContext> {
    vec![
        ActionContext::root(CONTEXT_APP, ActionContextKind::Application, generation),
        ActionContext::child(CONTEXT_WORKSPACE, CONTEXT_APP, ActionContextKind::Workspace, generation),
        ActionContext::child(CONTEXT_PANE, CONTEXT_WORKSPACE, ActionContextKind::Pane, generation),
        ActionContext::child(CONTEXT_EDITOR, CONTEXT_PANE, ActionContextKind::Editor, generation)
            .with_fact(FACT_KIND, "editor")
            .with_fact(FACT_SELECTION, true),
    ]
}

fn definitions() -> Vec<ActionContextFactDefinition> {
    vec![
        ActionContextFactDefinition::new(FACT_KIND, ActionContextFactValueKind::Text),
        ActionContextFactDefinition::new(FACT_SELECTION, ActionContextFactValueKind::Boolean),
    ]
}

fn registrations(generation: u64) -> Vec<ActionHandlerRegistration> {
    vec![
        ActionHandlerRegistration::new(HANDLER_APP_FIND, ACTION_FIND, CONTEXT_APP, generation)
            .with_propagation(ActionPropagation::RootOnly),
        ActionHandlerRegistration::new(HANDLER_EDITOR_FIND, ACTION_FIND, CONTEXT_EDITOR, generation)
            .with_priority(10)
            .with_propagation(ActionPropagation::Bubble)
            .with_predicate(ActionContextPredicate::All(vec![
                ActionContextPredicate::ContextKind(ActionContextKind::Editor),
                ActionContextPredicate::FactEquals {
                    fact_id: ActionContextFactId::new(FACT_KIND),
                    value: ActionContextFactValue::Text("editor".to_string()),
                },
            ])),
        ActionHandlerRegistration::new(HANDLER_EDITOR_SAVE, ACTION_SAVE, CONTEXT_EDITOR, generation),
        ActionHandlerRegistration::new("handler.application.palette", ACTION_PALETTE, CONTEXT_APP, generation)
            .with_propagation(ActionPropagation::RootOnly)
            .allowing_modal_escape(),
    ]
}

fn candidate(generation: u64) -> ActionRoutingCandidate {
    ActionRoutingCandidate::new(generation, CONTEXT_EDITOR, contexts(generation), registrations(generation))
        .with_fact_definitions(definitions())
}

fn accepted_snapshot() -> ActionRoutingSnapshot {
    let build = accept_action_snapshot(candidate(1), &registry(), None, None);
    assert!(build.candidate_accepted, "{:?}", build.diagnostics);
    build.snapshot.expect("valid candidate produces a snapshot")
}

fn diagnostic_code(probe: &ActionRouteProbe) -> Option<RoutingDiagnosticCode> {
    probe.outcome.diagnostic().map(|diagnostic| diagnostic.code)
}

fn action_id(value: &str) -> crate::nexus_core::ActionId {
    crate::nexus_core::ActionId::new(value)
}

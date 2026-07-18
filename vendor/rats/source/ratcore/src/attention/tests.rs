use super::*;

const ROOT: &str = "root";
const DIALOG: &str = "dialog";
const SAVE: &str = "save";
const OPEN: &str = "open";
const CANCEL: &str = "cancel";
const HIDDEN: &str = "hidden";
const MISSING_SCOPE: &str = "missing";
const SAVE_LABEL: &str = "Save";
const OPEN_LABEL: &str = "Open";
const CANCEL_LABEL: &str = "Cancel";
const CTRL_S: &str = "Ctrl+S";

fn graph() -> FocusGraph {
    FocusGraph::new()
        .with_scope(FocusScope::root(ROOT, "Application"))
        .with_scope(FocusScope::child(DIALOG, ROOT, "Dialog").modal().roving())
        .with_node(
            FocusNode::for_widget(FocusWidgetKind::CommandPalette, SAVE, ROOT, SAVE_LABEL)
                .with_shortcuts(vec![CTRL_S.to_string()]),
        )
        .with_node(FocusNode::new(OPEN, ROOT, FocusRole::Button, OPEN_LABEL).disabled())
        .with_node(FocusNode::new(HIDDEN, ROOT, FocusRole::Button, "Hidden").hidden())
        .with_node(FocusNode::for_widget(
            FocusWidgetKind::Dialog,
            CANCEL,
            DIALOG,
            CANCEL_LABEL,
        ))
}

#[test]
fn graph_records_nested_scopes_roles_labels_and_active_focus() {
    let graph = graph().activate_first().unwrap();
    assert_eq!(graph.active_id(), Some(SAVE));
    assert_eq!(graph.scope(DIALOG).unwrap().parent.as_deref(), Some(ROOT));
    let save = graph.node(SAVE).unwrap();
    assert_eq!(save.role, FocusRole::MenuItem);
    assert_eq!(save.label, SAVE_LABEL);
    assert_eq!(save.shortcuts, vec![CTRL_S.to_string()]);
}

#[test]
fn traversal_skips_disabled_and_hidden_nodes() {
    let graph = graph().activate_first().unwrap();
    let (graph, transition) = graph.move_next().unwrap();
    assert_eq!(transition.previous.as_deref(), Some(SAVE));
    assert_eq!(transition.active.as_deref(), Some(CANCEL));
    assert_eq!(graph.active_id(), Some(CANCEL));
}

#[test]
fn modal_trap_keeps_focus_inside_scope_and_restores() {
    let graph = graph().activate_first().unwrap();
    let (graph, transition) = graph.enter_modal(DIALOG).unwrap();
    assert_eq!(transition.active.as_deref(), Some(CANCEL));
    assert_eq!(graph.modal_scope(), Some(DIALOG));
    assert_eq!(
        graph.clone().focus_node(SAVE).unwrap_err().code,
        FocusErrorCode::ModalEscape
    );
    let (graph, transition) = graph.leave_modal().unwrap();
    assert_eq!(transition.active.as_deref(), Some(SAVE));
    assert_eq!(graph.modal_scope(), None);
}

#[test]
fn renderer_adapters_project_shared_semantics() {
    let graph = graph().activate_first().unwrap();
    let aria = graph.aria_projection().unwrap();
    assert_eq!(aria.active_id, SAVE);
    assert_eq!(aria.role, FocusRole::MenuItem.label());
    assert_eq!(aria.label, SAVE_LABEL);
    assert_eq!(aria.live_region, "Menu item: Save");
    let tui = graph.tui_projection().unwrap();
    assert_eq!(tui.active_id, SAVE);
    assert_eq!(tui.status_text, "Menu item: Save");
    assert_eq!(tui.style_state, STYLE_FOCUSED);
    assert_eq!(tui.shortcuts, vec![CTRL_S.to_string()]);
}

#[test]
fn missing_label_is_rejected() {
    let graph = FocusGraph::new()
        .with_scope(FocusScope::root(ROOT, "Application"))
        .with_node(FocusNode::new(SAVE, ROOT, FocusRole::Button, ""));
    assert_eq!(
        graph.validate().unwrap_err().code,
        FocusErrorCode::MissingLabel
    );
}

#[test]
fn unknown_scope_is_rejected() {
    let graph = FocusGraph::new()
        .with_scope(FocusScope::root(ROOT, "Application"))
        .with_node(FocusNode::new(
            SAVE,
            MISSING_SCOPE,
            FocusRole::Button,
            SAVE_LABEL,
        ));
    assert_eq!(
        graph.validate().unwrap_err().code,
        FocusErrorCode::UnknownScope
    );
}

#[test]
fn disabled_active_node_is_rejected() {
    let mut graph = FocusGraph::new()
        .with_scope(FocusScope::root(ROOT, "Application"))
        .with_node(FocusNode::new(SAVE, ROOT, FocusRole::Button, SAVE_LABEL).disabled());
    graph.active = Some(SAVE.to_string());
    assert_eq!(
        graph.validate().unwrap_err().code,
        FocusErrorCode::ActiveDisabled
    );
}

#[test]
fn hidden_active_node_is_rejected() {
    let mut graph = FocusGraph::new()
        .with_scope(FocusScope::root(ROOT, "Application"))
        .with_node(FocusNode::new(SAVE, ROOT, FocusRole::Button, SAVE_LABEL).hidden());
    graph.active = Some(SAVE.to_string());
    assert_eq!(
        graph.validate().unwrap_err().code,
        FocusErrorCode::ActiveHidden
    );
}

#[test]
fn no_eligible_target_is_rejected() {
    let graph = FocusGraph::new()
        .with_scope(FocusScope::root(ROOT, "Application"))
        .with_node(FocusNode::new(OPEN, ROOT, FocusRole::Button, OPEN_LABEL).disabled());
    assert_eq!(
        graph.activate_first().unwrap_err().code,
        FocusErrorCode::NoEligibleTarget
    );
}

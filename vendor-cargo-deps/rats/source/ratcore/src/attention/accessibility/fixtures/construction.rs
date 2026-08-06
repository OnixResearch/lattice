const DIALOG_SURFACE: &str = "surface.command-dialog";
const DIALOG_ROOT: &str = "semantic.command-dialog";

/// Builds the bounded menu/dialog semantic fixture.
#[must_use]
pub fn menu_dialog_semantic_fixture() -> super::super::SemanticSnapshot {
    let mut nodes = dialog_shell_nodes();
    nodes.extend(command_nodes());
    nodes.extend(dialog_status_nodes());
    let mut snapshot = super::super::SemanticSnapshot::new(
        super::super::SemanticGeneration::new(1),
        vec![super::super::SemanticSurface::new(DIALOG_SURFACE, DIALOG_ROOT)],
        nodes,
    );
    snapshot.focus_links = dialog_focus_links();
    snapshot
}

fn dialog_focus_links() -> Vec<super::super::SemanticFocusLink> {
    vec![
        super::super::SemanticFocusLink {
            focus: super::super::super::FocusNodeId::new("focus.command-open"),
            semantic: "semantic.command-open".into(),
        },
        super::super::SemanticFocusLink {
            focus: super::super::super::FocusNodeId::new("focus.command-delete"),
            semantic: "semantic.command-delete".into(),
        },
    ]
}

fn dialog_shell_nodes() -> Vec<super::super::SemanticNode> {
    vec![
        dialog_node(),
        super::super::SemanticNode::new(
            "semantic.dialog-title",
            DIALOG_SURFACE,
            super::super::SemanticRole::Heading,
            "Commands",
        )
        .child_of(DIALOG_ROOT, 0),
        super::super::SemanticNode::new(
            "semantic.dialog-description",
            DIALOG_SURFACE,
            super::super::SemanticRole::Description,
            "Command help",
        )
        .child_of(DIALOG_ROOT, 1),
        super::super::SemanticNode::new(
            "semantic.command-menu",
            DIALOG_SURFACE,
            super::super::SemanticRole::Menu,
            "Available commands",
        )
        .child_of(DIALOG_ROOT, 2),
    ]
}

fn dialog_node() -> super::super::SemanticNode {
    let mut dialog =
        super::super::SemanticNode::new(DIALOG_ROOT, DIALOG_SURFACE, super::super::SemanticRole::Dialog, "")
            .with_relation(super::super::SemanticRelationKind::LabelledBy, "semantic.dialog-title")
            .with_relation(super::super::SemanticRelationKind::DescribedBy, "semantic.dialog-description")
            .with_action(super::super::SemanticAction::Dismiss, crate::nexus_core::ActionId::new("dialog.dismiss"));
    dialog.description = Some("Choose an available command".to_string());
    debug_assert_eq!(dialog.role, super::super::SemanticRole::Dialog);
    debug_assert!(dialog.description.is_some());
    dialog
}

fn command_nodes() -> Vec<super::super::SemanticNode> {
    vec![open_command_node(), delete_command_node()]
}

fn open_command_node() -> super::super::SemanticNode {
    let mut node = super::super::SemanticNode::new(
        "semantic.command-open",
        DIALOG_SURFACE,
        super::super::SemanticRole::MenuItem,
        "Open file",
    )
    .child_of("semantic.command-menu", 0)
    .with_action(super::super::SemanticAction::Activate, crate::nexus_core::ActionId::new("file.open"))
    .with_action(super::super::SemanticAction::Focus, crate::nexus_core::ActionId::new("focus.command-open"));
    node.state.selected = Some(true);
    node.collection = Some(super::super::SemanticCollection {
        level: 1,
        position: 1,
        set_size: 2,
    });
    debug_assert_eq!(node.role, super::super::SemanticRole::MenuItem);
    debug_assert_eq!(node.state.selected, Some(true));
    node
}

fn delete_command_node() -> super::super::SemanticNode {
    let mut node = super::super::SemanticNode::new(
        "semantic.command-delete",
        DIALOG_SURFACE,
        super::super::SemanticRole::MenuItem,
        "Delete file",
    )
    .child_of("semantic.command-menu", 1)
    .with_action(super::super::SemanticAction::Activate, crate::nexus_core::ActionId::new("file.delete"))
    .with_action(super::super::SemanticAction::Focus, crate::nexus_core::ActionId::new("focus.command-delete"));
    node.state = node.state.with_flag(super::super::SemanticStateKind::Disabled);
    node.state.selected = Some(false);
    node.collection = Some(super::super::SemanticCollection {
        level: 1,
        position: 2,
        set_size: 2,
    });
    debug_assert_eq!(node.role, super::super::SemanticRole::MenuItem);
    debug_assert!(node.state.is_disabled());
    node
}

fn dialog_status_nodes() -> Vec<super::super::SemanticNode> {
    vec![progress_node(), status_node()]
}

fn progress_node() -> super::super::SemanticNode {
    let mut node = super::super::SemanticNode::new(
        "semantic.command-progress",
        DIALOG_SURFACE,
        super::super::SemanticRole::Progress,
        "Indexing progress",
    )
    .child_of(DIALOG_ROOT, 3)
    .with_action(super::super::SemanticAction::Increment, crate::nexus_core::ActionId::new("index.increment"))
    .with_action(super::super::SemanticAction::Decrement, crate::nexus_core::ActionId::new("index.decrement"))
    .with_action(super::super::SemanticAction::SetValue, crate::nexus_core::ActionId::new("index.set"));
    node.value = Some(super::super::SemanticValue::Range(super::super::SemanticRange {
        minimum: 0,
        maximum: 100,
        current: 40,
        step: 10,
    }));
    debug_assert_eq!(node.role, super::super::SemanticRole::Progress);
    debug_assert!(node.value.is_some());
    node
}

fn status_node() -> super::super::SemanticNode {
    let mut node = super::super::SemanticNode::new(
        "semantic.command-status",
        DIALOG_SURFACE,
        super::super::SemanticRole::Status,
        "Command diagnostics",
    )
    .child_of(DIALOG_ROOT, 4);
    node.announcement = Some(super::super::SemanticAnnouncement {
        politeness: super::super::SemanticPoliteness::Polite,
        message: "One command is unavailable".to_string(),
    });
    node
}

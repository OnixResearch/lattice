const TREE_SURFACE: &str = "surface.project-tree";
const TREE_ROOT: &str = "semantic.project-tree";

/// Builds the bounded hierarchical tree semantic fixture.
#[must_use]
pub fn tree_semantic_fixture() -> super::super::SemanticSnapshot {
    let mut snapshot = super::super::SemanticSnapshot::new(
        super::super::SemanticGeneration::new(7),
        vec![super::super::SemanticSurface::new(TREE_SURFACE, TREE_ROOT)],
        tree_nodes(),
    );
    snapshot.focus_links = tree_focus_links();
    snapshot
}

fn tree_nodes() -> Vec<super::super::SemanticNode> {
    let root =
        super::super::SemanticNode::new(TREE_ROOT, TREE_SURFACE, super::super::SemanticRole::Tree, "Project files");
    vec![root, source_tree_item(), library_tree_item(), archive_tree_item()]
}

fn source_tree_item() -> super::super::SemanticNode {
    let mut node = tree_item(TreeItemFixture {
        id: "semantic.tree-source",
        name: "Source",
        order: 0,
        position: 1,
        is_expanded: true,
    })
    .with_action(super::super::SemanticAction::Collapse, crate::nexus_core::ActionId::new("tree.collapse-source"));
    node.state.selected = Some(true);
    node
}

fn library_tree_item() -> super::super::SemanticNode {
    let mut node = super::super::SemanticNode::new(
        "semantic.tree-library",
        TREE_SURFACE,
        super::super::SemanticRole::TreeItem,
        "Library source",
    )
    .child_of("semantic.tree-source", 0)
    .with_action(super::super::SemanticAction::Activate, crate::nexus_core::ActionId::new("tree.open-library"))
    .with_action(super::super::SemanticAction::Select, crate::nexus_core::ActionId::new("tree.select-library"))
    .with_action(super::super::SemanticAction::Focus, crate::nexus_core::ActionId::new("focus.tree-library"));
    node.collection = Some(super::super::SemanticCollection {
        level: 2,
        position: 1,
        set_size: 1,
    });
    debug_assert_eq!(node.role, super::super::SemanticRole::TreeItem);
    debug_assert!(node.collection.is_some());
    node
}

fn archive_tree_item() -> super::super::SemanticNode {
    tree_item(TreeItemFixture {
        id: "semantic.tree-archive",
        name: "Archive",
        order: 1,
        position: 2,
        is_expanded: false,
    })
    .with_action(super::super::SemanticAction::Expand, crate::nexus_core::ActionId::new("tree.expand-archive"))
}

fn tree_focus_links() -> Vec<super::super::SemanticFocusLink> {
    ["source", "library", "archive"]
        .into_iter()
        .map(|name| super::super::SemanticFocusLink {
            focus: super::super::super::FocusNodeId::new(format!("focus.tree-{name}")),
            semantic: format!("semantic.tree-{name}").into(),
        })
        .collect()
}

struct TreeItemFixture<'a> {
    id: &'a str,
    name: &'a str,
    order: u64,
    position: u64,
    is_expanded: bool,
}

fn tree_item(fixture: TreeItemFixture<'_>) -> super::super::SemanticNode {
    let mut node =
        super::super::SemanticNode::new(fixture.id, TREE_SURFACE, super::super::SemanticRole::TreeItem, fixture.name)
            .child_of(TREE_ROOT, fixture.order)
            .with_action(
                super::super::SemanticAction::Select,
                crate::nexus_core::ActionId::new(format!("tree.select-{}", fixture.position)),
            )
            .with_action(
                super::super::SemanticAction::Focus,
                crate::nexus_core::ActionId::new(format!("focus.tree-{}", fixture.position)),
            );
    node.state = super::super::SemanticState {
        selected: Some(false),
        expanded: Some(fixture.is_expanded),
        ..super::super::SemanticState::default()
    };
    node.collection = Some(super::super::SemanticCollection {
        level: 1,
        position: fixture.position,
        set_size: 2,
    });
    debug_assert_eq!(node.role, super::super::SemanticRole::TreeItem);
    debug_assert_eq!(node.state.expanded, Some(fixture.is_expanded));
    node
}

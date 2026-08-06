const ROOT_SCOPE: &str = "focus.root";

mod action;
mod rendering;
mod update;
mod validation;

fn menu_focus_graph() -> super::super::FocusGraph {
    super::super::FocusGraph::new()
        .with_scope(super::super::FocusScope::root(ROOT_SCOPE, "Command dialog"))
        .with_node(super::super::FocusNode::new(
            "focus.command-open",
            ROOT_SCOPE,
            super::super::FocusRole::MenuItem,
            "Open file",
        ))
        .with_node(
            super::super::FocusNode::new(
                "focus.command-delete",
                ROOT_SCOPE,
                super::super::FocusRole::MenuItem,
                "Delete file",
            )
            .disabled(),
        )
}

fn tree_focus_graph() -> super::super::FocusGraph {
    super::super::FocusGraph::new()
        .with_scope(super::super::FocusScope::root(ROOT_SCOPE, "Project tree"))
        .with_node(super::super::FocusNode::new(
            "focus.tree-source",
            ROOT_SCOPE,
            super::super::FocusRole::TreeItem,
            "Source",
        ))
        .with_node(super::super::FocusNode::new(
            "focus.tree-library",
            ROOT_SCOPE,
            super::super::FocusRole::TreeItem,
            "Library source",
        ))
        .with_node(super::super::FocusNode::new(
            "focus.tree-archive",
            ROOT_SCOPE,
            super::super::FocusRole::TreeItem,
            "Archive",
        ))
}

fn issue_codes(issues: &[super::SemanticIssue]) -> Vec<super::SemanticIssueCode> {
    issues.iter().map(|issue| issue.code).collect()
}

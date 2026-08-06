#[test]
fn menu_dialog_summary_preserves_roles_states_actions_and_announcements() {
    let projection = super::project_terminal_semantics(&ratcore::focus::menu_dialog_semantic_fixture());
    assert!(projection.issues.is_empty());
    assert!(projection.limitation.contains("not a native accessibility tree"));
    let disabled = projection.nodes.iter().find(|node| node.id == "semantic.command-delete").unwrap();
    assert_eq!(disabled.role, "menu-item");
    assert!(disabled.states.contains(&"disabled".to_string()));
    assert!(!disabled.actions.is_empty());
    assert_eq!(projection.announcements, vec!["One command is unavailable"]);
}

#[test]
fn tree_summary_preserves_collection_role_and_expansion_state() {
    let projection = super::project_terminal_semantics(&ratcore::focus::tree_semantic_fixture());
    let source = projection.nodes.iter().find(|node| node.id == "semantic.tree-source").unwrap();
    assert_eq!(source.role, "tree-item");
    assert!(source.states.contains(&"expanded=true".to_string()));
    assert_eq!(source.name, "Source");
}

#[test]
fn malformed_snapshot_has_diagnostics_and_no_partial_summary() {
    let mut snapshot = ratcore::focus::tree_semantic_fixture();
    snapshot.nodes[1].parent = Some("semantic.missing".into());
    let projection = super::project_terminal_semantics(&snapshot);
    assert!(projection.nodes.is_empty());
    assert!(projection.issues.iter().any(|issue| issue.code == ratcore::focus::SemanticIssueCode::MissingParent));
}

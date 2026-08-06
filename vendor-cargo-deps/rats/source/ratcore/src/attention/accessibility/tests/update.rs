fn added_status() -> super::super::SemanticNode {
    super::super::SemanticNode::new(
        "semantic.tree-status",
        "surface.project-tree",
        super::super::SemanticRole::Status,
        "Tree updated",
    )
    .child_of("semantic.project-tree", 2)
}

#[test]
fn incremental_add_converges_with_full_reconstruction() {
    let initial = super::super::tree_semantic_fixture();
    let mut tree = super::super::SemanticTree::new(initial.clone(), Some(&super::tree_focus_graph())).unwrap();
    let status = added_status();
    let update = super::super::SemanticUpdate {
        base_generation: initial.generation,
        generation: super::super::SemanticGeneration::new(8),
        operations: vec![super::super::SemanticUpdateOperation::Add(status.clone())],
    };
    let change = tree.apply(&update, Some(&super::tree_focus_graph()));
    assert!(change.accepted);

    let mut rebuilt = initial;
    rebuilt.generation = super::super::SemanticGeneration::new(8);
    rebuilt.nodes.push(status);
    assert_eq!(tree.snapshot(), &rebuilt);
    assert!(super::super::validate_semantic_snapshot(tree.snapshot(), Some(&super::tree_focus_graph())).is_valid());
}

#[test]
fn stale_and_duplicate_updates_roll_back_atomically() {
    let initial = super::super::tree_semantic_fixture();
    let mut tree = super::super::SemanticTree::new(initial.clone(), Some(&super::tree_focus_graph())).unwrap();
    let stale = super::super::SemanticUpdate {
        base_generation: super::super::SemanticGeneration::new(6),
        generation: super::super::SemanticGeneration::new(8),
        operations: vec![super::super::SemanticUpdateOperation::Add(added_status())],
    };
    let stale_change = tree.apply(&stale, Some(&super::tree_focus_graph()));
    assert_eq!(stale_change.issues[0].code, super::super::SemanticIssueCode::StaleGeneration);
    assert_eq!(tree.snapshot(), &initial);

    let duplicate = super::super::SemanticUpdate {
        base_generation: initial.generation,
        generation: super::super::SemanticGeneration::new(8),
        operations: vec![
            super::super::SemanticUpdateOperation::Remove("semantic.tree-archive".into()),
            super::super::SemanticUpdateOperation::Remove("semantic.tree-archive".into()),
        ],
    };
    let duplicate_change = tree.apply(&duplicate, Some(&super::tree_focus_graph()));
    assert!(
        duplicate_change
            .issues
            .iter()
            .any(|issue| issue.code == super::super::SemanticIssueCode::DuplicateOperation)
    );
    assert_eq!(tree.snapshot(), &initial);
}

#[test]
fn dangling_removal_and_cyclic_reorder_preserve_state() {
    let initial = super::super::tree_semantic_fixture();
    let mut tree = super::super::SemanticTree::new(initial.clone(), Some(&super::tree_focus_graph())).unwrap();
    let dangling = super::super::SemanticUpdate {
        base_generation: initial.generation,
        generation: super::super::SemanticGeneration::new(8),
        operations: vec![super::super::SemanticUpdateOperation::Remove(
            "semantic.tree-source".into(),
        )],
    };
    let dangling_change = tree.apply(&dangling, Some(&super::tree_focus_graph()));
    assert!(
        dangling_change
            .issues
            .iter()
            .any(|issue| issue.code == super::super::SemanticIssueCode::MissingParent)
    );
    assert_eq!(tree.snapshot(), &initial);

    let cycle = super::super::SemanticUpdate {
        base_generation: initial.generation,
        generation: super::super::SemanticGeneration::new(8),
        operations: vec![super::super::SemanticUpdateOperation::Reorder {
            node: "semantic.tree-source".into(),
            parent: Some("semantic.tree-library".into()),
            order: 0,
        }],
    };
    let cycle_change = tree.apply(&cycle, Some(&super::tree_focus_graph()));
    assert!(cycle_change.issues.iter().any(|issue| issue.code == super::super::SemanticIssueCode::ParentCycle));
    assert_eq!(tree.snapshot(), &initial);
}

#[test]
fn replace_and_reorder_operations_accept_valid_current_generation() {
    let initial = super::super::tree_semantic_fixture();
    let mut tree = super::super::SemanticTree::new(initial.clone(), Some(&super::tree_focus_graph())).unwrap();
    let mut archive = initial.node(&super::super::SemanticNodeId::new("semantic.tree-archive")).unwrap().clone();
    archive.name = "Archived files".to_string();
    let update = super::super::SemanticUpdate {
        base_generation: initial.generation,
        generation: super::super::SemanticGeneration::new(8),
        operations: vec![super::super::SemanticUpdateOperation::Replace(archive)],
    };
    let change = tree.apply(&update, Some(&super::tree_focus_graph()));
    assert!(change.accepted);
    assert_eq!(
        tree.snapshot()
            .node(&super::super::SemanticNodeId::new("semantic.tree-archive"))
            .map(|node| node.name.as_str()),
        Some("Archived files")
    );
}

#[test]
fn representative_fixtures_include_focusable_and_descriptive_semantics() {
    let menu = super::super::menu_dialog_semantic_fixture();
    let menu_validation = super::super::validate_semantic_snapshot(&menu, Some(&super::menu_focus_graph()));
    assert!(menu_validation.is_valid());
    assert!(menu.nodes.iter().any(|node| node.role == super::super::SemanticRole::Description));
    assert!(menu.nodes.iter().any(|node| node.value.is_some()));

    let tree = super::super::tree_semantic_fixture();
    let tree_validation = super::super::validate_semantic_snapshot(&tree, Some(&super::tree_focus_graph()));
    assert!(tree_validation.is_valid());
    let source = tree.node(&super::super::SemanticNodeId::new("semantic.tree-source"));
    assert_eq!(source.and_then(|node| node.collection).map(|item| item.position), Some(1));
    assert_eq!(source.and_then(|node| node.state.expanded), Some(true));
}

#[test]
fn malformed_vocabulary_and_graph_return_stable_ordered_codes() {
    let mut candidate = super::super::menu_dialog_semantic_fixture();
    let title = candidate.nodes.iter_mut().find(|node| node.id == "semantic.dialog-title").unwrap();
    title.state.selected = Some(true);
    title.actions.insert(super::super::SemanticAction::Activate, "title.activate".into());

    let progress = candidate.nodes.iter_mut().find(|node| node.id == "semantic.command-progress").unwrap();
    progress.value = Some(super::super::SemanticValue::Range(super::super::SemanticRange {
        minimum: 0,
        maximum: 100,
        current: 101,
        step: 0,
    }));

    let duplicate = candidate.nodes[0].clone();
    candidate.nodes.push(duplicate);
    let validation = super::super::validate_semantic_snapshot(&candidate, Some(&super::menu_focus_graph()));
    let codes = super::issue_codes(&validation.issues);
    assert!(codes.contains(&super::super::SemanticIssueCode::DuplicateNode));
    assert!(codes.contains(&super::super::SemanticIssueCode::IncompatibleState));
    assert!(codes.contains(&super::super::SemanticIssueCode::IncompatibleAction));
    assert!(codes.contains(&super::super::SemanticIssueCode::InvalidValue));
    assert!(validation.issues.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn converging_acyclic_relationships_are_not_cycles() {
    let mut candidate = super::super::menu_dialog_semantic_fixture();
    for id in ["semantic.dialog-title", "semantic.dialog-description"] {
        candidate
            .nodes
            .iter_mut()
            .find(|node| node.id == id)
            .unwrap()
            .relations
            .push(super::super::SemanticRelation {
                kind: super::super::SemanticRelationKind::Controls,
                target: "semantic.command-status".into(),
            });
    }
    let validation = super::super::validate_semantic_snapshot(&candidate, Some(&super::menu_focus_graph()));
    assert!(validation.is_valid());
}

#[test]
fn parent_relation_and_focus_failures_are_diagnostic() {
    let mut candidate = super::super::menu_dialog_semantic_fixture();
    let description = candidate.nodes.iter_mut().find(|node| node.id == "semantic.dialog-description").unwrap();
    description.parent = Some("semantic.dialog-description".into());
    description.relations.push(super::super::SemanticRelation {
        kind: super::super::SemanticRelationKind::DescribedBy,
        target: "semantic.command-dialog".into(),
    });
    candidate.focus_links[0].focus = "focus.missing".into();

    let validation = super::super::validate_semantic_snapshot(&candidate, Some(&super::menu_focus_graph()));
    let codes = super::issue_codes(&validation.issues);
    assert!(codes.contains(&super::super::SemanticIssueCode::ParentCycle));
    assert!(codes.contains(&super::super::SemanticIssueCode::RelationCycle));
    assert!(codes.contains(&super::super::SemanticIssueCode::MissingFocusNode));
}

#[test]
fn invalid_full_replacement_preserves_accepted_snapshot() {
    let initial = super::super::menu_dialog_semantic_fixture();
    let mut tree = super::super::SemanticTree::new(initial.clone(), Some(&super::menu_focus_graph())).unwrap();
    let mut candidate = initial.clone();
    candidate.generation = super::super::SemanticGeneration::new(2);
    let duplicate = candidate.nodes[0].clone();
    candidate.nodes.push(duplicate);

    let change = tree.replace(candidate, Some(&super::menu_focus_graph()));
    assert!(!change.accepted);
    assert_eq!(tree.snapshot(), &initial);
    assert!(change.issues.iter().any(|issue| issue.code == super::super::SemanticIssueCode::DuplicateNode));
}

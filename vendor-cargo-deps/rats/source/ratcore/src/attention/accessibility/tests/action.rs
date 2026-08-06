fn request(node: &str, action: super::super::SemanticAction) -> super::super::SemanticActionRequest {
    super::super::SemanticActionRequest {
        generation: super::super::SemanticGeneration::new(1),
        node: node.into(),
        action,
        value: None,
    }
}

#[test]
fn current_enabled_actions_route_to_typed_application_intents() {
    let snapshot = super::super::menu_dialog_semantic_fixture();
    let activation = super::super::route_semantic_action(
        &snapshot,
        &request("semantic.command-open", super::super::SemanticAction::Activate),
    );
    let intent = activation.intent.unwrap();
    assert_eq!(intent.node, "semantic.command-open");
    assert_eq!(intent.action, super::super::SemanticAction::Activate);
    assert_eq!(intent.target.as_str(), "file.open");
    assert!(activation.issues.is_empty());

    let mut set_value = request("semantic.command-progress", super::super::SemanticAction::SetValue);
    set_value.value = Some(super::super::SemanticActionValue::Integer(70));
    let value_outcome = super::super::route_semantic_action(&snapshot, &set_value);
    assert_eq!(
        value_outcome.intent.and_then(|accepted| accepted.value),
        Some(super::super::SemanticActionValue::Integer(70))
    );
}

#[test]
fn disabled_hidden_stale_and_unsupported_requests_fail_closed() {
    let snapshot = super::super::menu_dialog_semantic_fixture();
    let disabled = super::super::route_semantic_action(
        &snapshot,
        &request("semantic.command-delete", super::super::SemanticAction::Activate),
    );
    assert_eq!(disabled.issues[0].code, super::super::SemanticIssueCode::DisabledNode);

    let mut hidden_snapshot = snapshot.clone();
    let hidden_node = hidden_snapshot.nodes.iter_mut().find(|node| node.id == "semantic.command-open").unwrap();
    hidden_node.state = hidden_node.state.clone().with_flag(super::super::SemanticStateKind::Hidden);
    let hidden = super::super::route_semantic_action(
        &hidden_snapshot,
        &request("semantic.command-open", super::super::SemanticAction::Activate),
    );
    assert_eq!(hidden.issues[0].code, super::super::SemanticIssueCode::HiddenNode);

    let mut stale_request = request("semantic.command-open", super::super::SemanticAction::Activate);
    stale_request.generation = super::super::SemanticGeneration::new(0);
    let stale = super::super::route_semantic_action(&snapshot, &stale_request);
    assert_eq!(stale.issues[0].code, super::super::SemanticIssueCode::StaleGeneration);

    let dismissed = super::super::route_semantic_action(
        &snapshot,
        &request("semantic.command-open", super::super::SemanticAction::Dismiss),
    );
    assert_eq!(dismissed.issues[0].code, super::super::SemanticIssueCode::UnsupportedAction);
    assert!(disabled.intent.is_none());
    assert!(hidden.intent.is_none());
    assert!(stale.intent.is_none());
}

#[test]
fn malformed_value_and_missing_node_return_diagnostics() {
    let snapshot = super::super::menu_dialog_semantic_fixture();
    let missing_value = super::super::route_semantic_action(
        &snapshot,
        &request("semantic.command-progress", super::super::SemanticAction::SetValue),
    );
    assert_eq!(missing_value.issues[0].code, super::super::SemanticIssueCode::InvalidActionValue);

    let mut outside = request("semantic.command-progress", super::super::SemanticAction::SetValue);
    outside.value = Some(super::super::SemanticActionValue::Integer(101));
    let outside_outcome = super::super::route_semantic_action(&snapshot, &outside);
    assert_eq!(outside_outcome.issues[0].code, super::super::SemanticIssueCode::InvalidActionValue);

    let unknown = super::super::route_semantic_action(
        &snapshot,
        &request("semantic.missing", super::super::SemanticAction::Activate),
    );
    assert_eq!(unknown.issues[0].code, super::super::SemanticIssueCode::UnknownNode);
}

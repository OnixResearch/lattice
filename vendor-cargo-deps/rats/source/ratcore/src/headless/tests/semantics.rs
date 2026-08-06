use super::super::*;

#[test]
fn projection_keeps_named_parts_slots_orientation_and_focus_distinct() {
    let state = canonical_tabs_state(TabsActivationPolicy::Manual).expect("fixture state");
    let projection = project_tabs(&state);

    assert!(projection.semantic_issues.is_empty(), "{:?}", projection.semantic_issues);
    assert_eq!(projection.list.orientation, TabsOrientation::Horizontal);
    assert_eq!(projection.list.activation, TabsActivationPolicy::Manual);
    assert_eq!(projection.tabs.len(), 3);
    assert_eq!(projection.panels.len(), 3);
    assert_eq!(projection.tabs[0].position, Some(1));
    assert_eq!(projection.tabs[0].set_size, 3);
    assert!(projection.tabs[0].is_selected());
    assert!(projection.tabs[0].is_focused());
    assert_eq!(projection.panels[0].slot_id.as_str(), "slot.overview");
    assert!(projection.panels[0].is_active);
    assert!(!projection.panels[0].is_hidden);
}

#[test]
fn exposes_tab_roles_associations_selection_collection_and_actions() {
    let state = canonical_tabs_state(TabsActivationPolicy::Automatic).expect("fixture state");
    let projection = project_tabs(&state);
    let roles = projection.semantics.nodes.iter().map(|node| node.role).collect::<Vec<_>>();

    assert_eq!(roles[0], crate::attention_core::SemanticRole::TabList);
    assert_eq!(roles.iter().filter(|role| **role == crate::attention_core::SemanticRole::Tab).count(), 3);
    assert_eq!(roles.iter().filter(|role| **role == crate::attention_core::SemanticRole::TabPanel).count(), 3);
    let overview = projection
        .semantics
        .nodes
        .iter()
        .find(|node| node.id.as_str().ends_with("tab.overview"))
        .expect("overview semantic trigger");
    assert_eq!(overview.state.selected, Some(true));
    assert_eq!(overview.collection.expect("collection").position, 1);
    assert!(
        overview
            .relations
            .iter()
            .any(|relation| relation.kind == crate::attention_core::SemanticRelationKind::Controls)
    );
    assert!(overview.actions.contains_key(&crate::attention_core::SemanticAction::Focus));
    assert!(overview.actions.contains_key(&crate::attention_core::SemanticAction::Select));
}

#[test]
fn hidden_and_disabled_projection_keeps_truth_and_removes_actions() {
    let mut catalog = canonical_tabs_state(TabsActivationPolicy::Automatic).expect("fixture state").catalog().clone();
    catalog.tabs[1].is_disabled = true;
    catalog.tabs[2].is_hidden = true;
    let state = TabsState::new(
        catalog,
        TabId::new("tab.overview"),
        TabId::new("tab.overview"),
        TabsOrientation::Vertical,
        TabsActivationPolicy::Automatic,
    )
    .expect("overview remains eligible");
    let projection = project_tabs(&state);

    assert!(projection.semantic_issues.is_empty(), "{:?}", projection.semantic_issues);
    assert!(projection.tabs[1].is_disabled());
    assert!(projection.tabs[1].actions.is_empty());
    assert!(projection.tabs[2].is_hidden());
    assert_eq!(projection.tabs[2].position, None);
    assert!(projection.tabs[2].actions.is_empty());
    assert_eq!(projection.tabs[0].set_size, 2);
    assert_eq!(projection.list.orientation, TabsOrientation::Vertical);
}

#[test]
fn adapter_script_rejects_messages_beyond_its_fixed_bound() {
    let mut state = canonical_tabs_state(TabsActivationPolicy::Automatic).expect("fixture state");
    let messages = vec![TabsMessage::MoveNext; 65];
    let receipt = run_tabs_adapter_script(&mut state, messages);

    assert!(receipt.diagnostics.iter().any(|issue| issue.code == TabsDiagnosticCode::ScriptTooLarge));
    assert!(receipt.events.len() <= 192);
}

#[test]
fn fixture_evidence_covers_required_states_and_rejects_visual_claims() {
    let evidence = headless_tabs_fixture_evidence();

    for fixture in [
        "normal-v1",
        "disabled-v1",
        "hidden-v1",
        "empty-v1",
        "replacement-v1",
        "narrow-v1",
        "manual-v1",
        "automatic-v1",
        "invalid-v1",
    ] {
        assert!(evidence.contains(fixture), "missing {fixture}");
    }
    assert!(evidence.contains("evidence_class=semantic-behavior-not-visual"));
    assert!(evidence.contains("not_claimed=markup-parity,pixel-parity"));
}

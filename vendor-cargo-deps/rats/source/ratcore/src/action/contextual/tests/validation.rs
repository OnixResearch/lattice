use super::*;

#[test]
fn valid_snapshot_preserves_graph_facts_and_stable_chain() {
    let snapshot = accepted_snapshot();

    assert_eq!(snapshot.generation(), 1);
    assert_eq!(snapshot.active_chain().iter().map(ActionContextId::as_str).collect::<Vec<_>>(), vec![
        CONTEXT_EDITOR,
        CONTEXT_PANE,
        CONTEXT_WORKSPACE,
        CONTEXT_APP,
    ]);
    assert_eq!(snapshot.registrations().len(), 4);
    assert_eq!(
        snapshot
            .context(&ActionContextId::new(CONTEXT_EDITOR))
            .and_then(|context| context.facts.get(FACT_KIND)),
        Some(&ActionContextFactValue::Text("editor".to_string()))
    );
}

#[test]
fn invalid_candidate_preserves_previous_snapshot_atomically() {
    let previous = accepted_snapshot();
    let mut invalid = candidate(2);
    invalid.contexts.push(invalid.contexts[0].clone());
    invalid.registrations.push(invalid.registrations[0].clone());
    invalid.fact_definitions.push(invalid.fact_definitions[0].clone());

    let build = accept_action_snapshot(invalid, &registry(), None, Some(&previous));

    assert!(!build.candidate_accepted);
    assert_eq!(build.snapshot.as_ref(), Some(&previous));
    let codes = build.diagnostics.iter().map(|diagnostic| diagnostic.code).collect::<Vec<_>>();
    assert!(codes.contains(&RoutingDiagnosticCode::DuplicateContextId));
    assert!(codes.contains(&RoutingDiagnosticCode::DuplicateRegistrationId));
    assert!(codes.contains(&RoutingDiagnosticCode::DuplicateFactDefinition));
}

#[test]
fn missing_parents_cycles_and_invalid_roots_are_rejected() {
    let mut missing = candidate(1);
    missing.contexts[3].parent = Some(ActionContextId::new("context.missing"));
    let missing_build = accept_action_snapshot(missing, &registry(), None, None);

    let mut cyclic = candidate(1);
    cyclic.contexts[2].parent = Some(ActionContextId::new(CONTEXT_EDITOR));
    let cycle_build = accept_action_snapshot(cyclic, &registry(), None, None);

    assert!(
        missing_build
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == RoutingDiagnosticCode::MissingParent })
    );
    assert!(
        cycle_build
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == RoutingDiagnosticCode::ContextCycle })
    );
}

#[test]
fn stale_generations_unknown_actions_and_invalid_propagation_fail() {
    let mut stale = candidate(1);
    stale.registrations[0].owner_generation = 2;
    stale.registrations[1].action_id = action_id("action.unknown");
    stale.registrations[2].propagation = ActionPropagation::RootOnly;

    let build = accept_action_snapshot(stale, &registry(), None, None);
    let codes = build.diagnostics.iter().map(|diagnostic| diagnostic.code).collect::<Vec<_>>();

    assert!(codes.contains(&RoutingDiagnosticCode::StaleContextGeneration));
    assert!(codes.contains(&RoutingDiagnosticCode::UnknownAction));
    assert!(codes.contains(&RoutingDiagnosticCode::InvalidPropagation));
}

#[test]
fn malformed_unknown_and_wrong_kind_facts_fail_closed() {
    let mut invalid = candidate(1);
    invalid.contexts[3] = invalid.contexts[3]
        .clone()
        .with_fact("fact.unknown", true)
        .with_fact(FACT_SELECTION, "yes")
        .with_fact(FACT_KIND, "x".repeat(257));

    let build = accept_action_snapshot(invalid, &registry(), None, None);
    let codes = build.diagnostics.iter().map(|diagnostic| diagnostic.code).collect::<Vec<_>>();

    assert!(codes.contains(&RoutingDiagnosticCode::UnknownFact));
    assert!(codes.contains(&RoutingDiagnosticCode::FactTypeMismatch));
    assert!(codes.contains(&RoutingDiagnosticCode::MalformedFact));
}

#[test]
fn stale_focus_links_and_contradictory_modal_roots_are_rejected() {
    let focus = crate::focus::FocusGraph::new()
        .with_scope(crate::focus::FocusScope::root("focus.root", "Application"))
        .with_node(crate::focus::FocusNode::new(
            "focus.editor",
            "focus.root",
            crate::focus::FocusRole::FormField,
            "Editor",
        ))
        .activate_first()
        .expect("valid focus graph");
    let mut stale_focus = candidate(1);
    stale_focus.contexts[3] =
        stale_focus.contexts[3].clone().with_focus_scope("focus.root").with_focus_node("focus.missing");
    let stale_build = accept_action_snapshot(stale_focus, &registry(), Some(&focus), None);

    let mut contradictory = candidate(1);
    contradictory.contexts.push(
        ActionContext::child("context.orphan-modal", CONTEXT_PANE, ActionContextKind::Modal, 1)
            .with_order(1)
            .with_modal(ActionContextModalBoundary::blocking()),
    );
    let modal_build = accept_action_snapshot(contradictory, &registry(), None, None);

    assert!(
        stale_build
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == RoutingDiagnosticCode::StaleFocusLink })
    );
    assert!(
        modal_build
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == RoutingDiagnosticCode::ContradictoryModalRoot })
    );
}

#[test]
fn non_monotonic_generation_rejects_replacement() {
    let previous = accepted_snapshot();

    let build = accept_action_snapshot(candidate(1), &registry(), None, Some(&previous));

    assert!(!build.candidate_accepted);
    assert_eq!(build.snapshot, Some(previous));
    assert!(
        build
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == RoutingDiagnosticCode::NonMonotonicGeneration })
    );
}

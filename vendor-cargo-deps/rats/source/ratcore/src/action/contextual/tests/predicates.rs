use super::*;

#[test]
fn logical_fact_kind_focus_and_action_state_checks_are_pure() {
    let snapshot = accepted_snapshot();
    let predicate = ActionContextPredicate::All(vec![
        ActionContextPredicate::FactPresent(ActionContextFactId::new(FACT_SELECTION)),
        ActionContextPredicate::FactEquals {
            fact_id: ActionContextFactId::new(FACT_SELECTION),
            value: ActionContextFactValue::Boolean(true),
        },
        ActionContextPredicate::ContextKind(ActionContextKind::Editor),
        ActionContextPredicate::Focused,
        ActionContextPredicate::ActionState(ActionStatePredicate::Enabled),
        ActionContextPredicate::Not(Box::new(ActionContextPredicate::Any(vec![
            ActionContextPredicate::FactEquals {
                fact_id: ActionContextFactId::new(FACT_KIND),
                value: ActionContextFactValue::Text("tree".to_string()),
            },
            ActionContextPredicate::ActionState(ActionStatePredicate::Disabled),
        ]))),
    ]);

    let evaluation = evaluate_action_context_predicate(
        &snapshot,
        &predicate,
        &ActionContextId::new(CONTEXT_EDITOR),
        &action_id(ACTION_FIND),
    );

    assert!(evaluation.matched);
    assert!(evaluation.diagnostics.is_empty());
    assert_eq!(evaluation.inspected_fact_contexts, vec![ActionContextId::new(CONTEXT_EDITOR)]);
}

#[test]
fn ancestor_and_inherited_fact_checks_use_the_focus_to_owner_chain() {
    let snapshot = accepted_snapshot();
    let predicate =
        ActionContextPredicate::All(vec![ActionContextPredicate::Ancestor, ActionContextPredicate::FactEquals {
            fact_id: ActionContextFactId::new(FACT_KIND),
            value: ActionContextFactValue::Text("editor".to_string()),
        }]);

    let evaluation = evaluate_action_context_predicate(
        &snapshot,
        &predicate,
        &ActionContextId::new(CONTEXT_APP),
        &action_id(ACTION_FIND),
    );

    assert!(evaluation.matched);
    assert_eq!(evaluation.inspected_fact_contexts, vec![ActionContextId::new(CONTEXT_EDITOR)]);
}

#[test]
fn malformed_unknown_and_wrong_kind_checks_return_diagnostics() {
    let snapshot = accepted_snapshot();
    let malformed = ActionContextPredicate::All(Vec::new());
    let unknown = ActionContextPredicate::FactPresent(ActionContextFactId::new("fact.unknown"));
    let wrong_kind = ActionContextPredicate::FactEquals {
        fact_id: ActionContextFactId::new(FACT_SELECTION),
        value: ActionContextFactValue::Text("true".to_string()),
    };

    let evaluations = [malformed, unknown, wrong_kind]
        .iter()
        .map(|predicate| {
            evaluate_action_context_predicate(
                &snapshot,
                predicate,
                &ActionContextId::new(CONTEXT_EDITOR),
                &action_id(ACTION_FIND),
            )
        })
        .collect::<Vec<_>>();

    assert!(evaluations.iter().all(|evaluation| !evaluation.matched));
    assert_eq!(evaluations.iter().map(|evaluation| evaluation.diagnostics[0].code).collect::<Vec<_>>(), vec![
        RoutingDiagnosticCode::MalformedPredicate,
        RoutingDiagnosticCode::UnknownFact,
        RoutingDiagnosticCode::FactTypeMismatch,
    ]);
}

#[test]
fn excessive_predicate_depth_and_nodes_reject_the_catalog() {
    let mut deep = ActionContextPredicate::Always;
    for _ in 0..9 {
        deep = ActionContextPredicate::Not(Box::new(deep));
    }
    let mut depth_candidate = candidate(1);
    depth_candidate.registrations[0].predicate = deep;
    let depth_build = accept_action_snapshot(depth_candidate, &registry(), None, None);

    let mut nodes_candidate = candidate(1).with_predicate_limits(ActionContextPredicateLimits::new(8, 2));
    nodes_candidate.registrations[0].predicate =
        ActionContextPredicate::All(vec![ActionContextPredicate::Always, ActionContextPredicate::Always]);
    let nodes_build = accept_action_snapshot(nodes_candidate, &registry(), None, None);

    assert!(
        depth_build
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == RoutingDiagnosticCode::PredicateDepthExceeded })
    );
    assert!(
        nodes_build
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == RoutingDiagnosticCode::PredicateNodeLimitExceeded })
    );
}

#[test]
fn disabled_action_state_is_observed_without_invoking_behavior() {
    let disabled_registry = crate::nexus_core::build_registry(vec![
        crate::nexus_core::ActionDescriptor::new(ACTION_FIND, "Find", crate::nexus_core::ActionSource::new("test")),
        crate::nexus_core::ActionDescriptor::new(ACTION_SAVE, "Save", crate::nexus_core::ActionSource::new("test"))
            .with_enablement(crate::nexus_core::ActionEnablement::Disabled {
                reason: "read only".to_string(),
            }),
        crate::nexus_core::ActionDescriptor::new(
            ACTION_PALETTE,
            "Palette",
            crate::nexus_core::ActionSource::new("test"),
        ),
        crate::nexus_core::ActionDescriptor::new(
            ACTION_UNHANDLED,
            "Unhandled",
            crate::nexus_core::ActionSource::new("test"),
        ),
    ])
    .snapshot;
    let build = accept_action_snapshot(candidate(1), &disabled_registry, None, None);
    let snapshot = build.snapshot.expect("catalog remains structurally valid");

    let evaluation = evaluate_action_context_predicate(
        &snapshot,
        &ActionContextPredicate::ActionState(ActionStatePredicate::Disabled),
        &ActionContextId::new(CONTEXT_EDITOR),
        &action_id(ACTION_SAVE),
    );

    assert!(evaluation.matched);
    assert!(evaluation.diagnostics.is_empty());
}

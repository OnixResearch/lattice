//! Profile facts and instance observations are separate, bounded and never
//! report idle, running, healthy or available.

// r[verify lattice.execution_binding.profile_facts]
// r[verify lattice.execution_binding.profile_facts.scenario.projection]
// r[verify lattice.execution_binding.instances]
// r[verify lattice.execution_binding.instances.scenario.unknown]
// r[verify lattice.execution_binding.deferred.scenario.serving]
// r[verify lattice.execution_binding.deferred]

use lattice::execution_binding::core::admission::bound::*;
use lattice::execution_binding::core::admission::*;
use lattice::execution_binding::core::projection::inventory::*;
use lattice::execution_binding::core::projection::retained::*;
use lattice::execution_binding::core::projection::*;
use lattice::execution_binding::core::request::*;

const ROUTE: &str = "blake3:9999999999999999999999999999999999999999999999999999999999999999";

fn routes() -> RouteDeclaration {
    RouteDeclaration::Routes {
        routes: vec![NamedRoute {
            name: "primary".into(),
            route_identity: ROUTE.into(),
        }],
    }
}

fn profile_input<'a>(routes: &'a RouteDeclaration, compute: &'a ComputeRequirement) -> ProfileInput<'a> {
    ProfileInput {
        owner: ProfileOwner::ImplementationAttempt,
        revision: "4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f",
        qualification_ref: None,
        adapter: Offered::Declared("omp-jsonl"),
        protocol_revision: Offered::Declared("e325ffc73ab86e03bdd20a91b77d8e2b3964c906"),
        executable_identity: Offered::Declared("eeee"),
        inputs: Offered::Declared("workspace-snapshot"),
        outputs: Offered::Declared("candidate-receipt"),
        sessions: Offered::Declared("single-shot-no-session"),
        checkpoint: Offered::Unavailable,
        workspace: Offered::Declared("admitted-read-write-paths"),
        network: Offered::Declared("attempt-local-model-broker"),
        isolation: Offered::Declared("required-sandbox"),
        deadline_secs: Offered::Declared(60),
        max_output_bytes: Offered::Declared(4096),
        model_routes: Offered::Declared(routes),
        compute: Offered::Declared(compute),
    }
}

#[test]
fn attempt_profile_projects_declared_facts_and_unknown_bounds() {
    let routes = routes();
    let compute = ComputeRequirement::ProviderExecution {
        direct_accelerator: None,
    };
    let facts = project_profile(&profile_input(&routes, &compute)).expect("profile facts");
    assert_eq!(facts.schema, PROFILE_FACTS_SCHEMA);
    assert_eq!(facts.revision, "4f".repeat(32));
    assert_eq!(facts.compute.support, SupportLevel::Declared);
    assert_eq!(facts.compute.value, Some(compute.clone()));
    assert_eq!(facts.model_routes.value, Some(routes.clone()));
    assert_eq!(facts.checkpoint.support, SupportLevel::Unavailable);
    for bound in [&facts.cpu_millis, &facts.memory_bytes, &facts.storage_bytes] {
        assert_eq!((bound.support, bound.value), (SupportLevel::Unknown, None));
    }
    assert_eq!(facts.serving_readiness.support, SupportLevel::Unknown);
    assert_eq!(facts.serving_readiness.owner, "animus#17");
    let text = serde_json::to_string(&facts).expect("facts json");
    assert!(!text.contains("qualified"), "declared facts are never qualified without evidence");
}

#[test]
fn qualified_requires_canonical_evidence_and_bounds_are_enforced() {
    let routes = routes();
    let compute = ComputeRequirement::HostLocal {};
    let mut input = profile_input(&routes, &compute);
    input.qualification_ref = Some("not-evidence");
    assert_eq!(project_profile(&input).expect("facts").adapter.support, SupportLevel::Declared);
    input.qualification_ref = Some("blake3:abababababababababababababababababababababababababababababababab");
    let qualified = project_profile(&input).expect("facts");
    assert_eq!(qualified.adapter.support, SupportLevel::Qualified);
    assert_eq!(qualified.cpu_millis.support, SupportLevel::Unknown, "unknown never becomes qualified");
    let many = RouteDeclaration::Routes {
        routes: (0..9)
            .map(|index| NamedRoute {
                name: format!("route-{index}"),
                route_identity: ROUTE.into(),
            })
            .collect(),
    };
    assert_eq!(project_profile(&profile_input(&many, &compute)), Err(FactsDefect::Routes));
    let mut invalid = profile_input(&routes, &compute);
    invalid.revision = "";
    assert_eq!(project_profile(&invalid), Err(FactsDefect::Revision));
    invalid = profile_input(&routes, &compute);
    invalid.adapter = Offered::Declared("line\nbreak");
    assert_eq!(project_profile(&invalid), Err(FactsDefect::Token { field: "adapter" }));
}

fn bound() -> Admitted {
    let declaration = Declaration {
        schema: DECLARATION_SCHEMA.into(),
        compute: ComputeDeclaration::HostLocal {},
        model_routes: routes(),
    };
    let requirement = WorkerRequirement {
        compute: ComputeRequirement::HostLocal {},
        model: ModelRequirement::Routes {
            names: vec!["primary".into()],
        },
    };
    let attempt = AttemptContext {
        work_id: "work-24",
        attempt_id: "attempt-24",
        fence: 7,
        envelope_identity: "envelope-24",
        profile_revision: "profile-24",
    };
    match admit(&declaration, &requirement, attempt, ProviderReading::NotRead) {
        Decision::Admitted(admitted) => *admitted,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn state(admitted: Option<&Admitted>) -> AttemptState<'_> {
    AttemptState {
        admitted,
        request_identity: "request-24",
        has_intent: false,
        has_refusal: false,
        terminal_disposition: None,
        has_cancel_request: false,
        artifacts: &[],
    }
}

#[test]
fn instance_lifecycle_comes_only_from_retained_artifacts() {
    let admitted = bound();
    let lifecycle =
        |state: AttemptState<'_>| project_instance(&state, 1_000).expect("projection").expect("instance").lifecycle;
    assert_eq!(lifecycle(state(Some(&admitted))), Lifecycle::AdmittedNotDispatched);
    let dispatched = AttemptState {
        has_intent: true,
        ..state(Some(&admitted))
    };
    assert_eq!(lifecycle(dispatched), Lifecycle::DispatchRecordedOutcomeUnknown);
    assert_eq!(
        lifecycle(AttemptState {
            has_refusal: true,
            ..dispatched
        }),
        Lifecycle::BindingRejectedBeforeDispatch
    );
    let terminal = AttemptState {
        terminal_disposition: Some("failed"),
        ..dispatched
    };
    assert_eq!(lifecycle(terminal), Lifecycle::TerminalObserved {
        disposition: "failed".into()
    });
    let instance = project_instance(&dispatched, 1_000).expect("projection").expect("instance");
    assert_eq!(instance.binding_identity, admitted.binding_identity);
    assert_eq!((instance.incarnation.as_str(), instance.fence), ("request-24", 7));
    for unknown in [
        &instance.host_reachability,
        &instance.model_readiness,
        &instance.allocation_liveness,
    ] {
        assert_eq!(unknown.support, SupportLevel::Unknown);
    }
    assert_eq!(instance.model_readiness.owner, "animus#17");
    let text = serde_json::to_string(&instance).expect("instance json");
    for forbidden in ["\"idle\"", "\"running\"", "\"healthy\"", "\"available\""] {
        assert!(!text.contains(forbidden), "{forbidden}");
    }
}

#[test]
fn missing_or_contradictory_state_is_unknown_or_an_error_never_inferred() {
    let admitted = bound();
    assert_eq!(project_instance(&state(None), 1), Ok(None), "no retained execution means no instance");
    let intent_only = AttemptState {
        has_intent: true,
        ..state(None)
    };
    assert_eq!(project_instance(&intent_only, 1), Err(StateDefect::IntentWithoutBinding));
    let refusal = AttemptState {
        has_refusal: true,
        ..state(Some(&admitted))
    };
    assert_eq!(project_instance(&refusal, 1), Err(StateDefect::RefusalWithoutIntent));
    let mut tampered = admitted.clone();
    tampered.attempt.fence = 8;
    assert_eq!(project_instance(&state(Some(&tampered)), 1), Err(StateDefect::BindingNotIntact));
    let cancellation = |requested, disposition| {
        let state = AttemptState {
            has_intent: true,
            has_cancel_request: requested,
            terminal_disposition: disposition,
            ..state(Some(&admitted))
        };
        project_instance(&state, 1).expect("projection").expect("instance").cancellation
    };
    assert_eq!(cancellation(false, None), Cancellation::NotRequested);
    assert_eq!(cancellation(true, None), Cancellation::RequestedUnobserved);
    assert_eq!(cancellation(true, Some("cancelled")), Cancellation::ObservedTerminated);
}

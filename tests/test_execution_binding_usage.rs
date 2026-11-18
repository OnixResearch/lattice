//! Attempt usage observations carry frozen binding provenance through the
//! existing immutable store path. Fixtures are synthetic.

// r[verify lattice.execution_binding.usage_provenance]
// r[verify lattice.execution_binding.usage_provenance.scenario.bound]
// r[verify lattice.execution_binding.usage_provenance.scenario.shared_allocation]
// r[verify lattice.execution_binding.usage_provenance.scenario.replay]

use lattice::execution_binding::core::admission::*;
use lattice::execution_binding::core::model::*;
use lattice::execution_binding::core::projection::accounting::*;
use lattice::execution_binding::core::request::*;

fn observation(name: &str) -> ComputeAllocationObservation {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive")
        .join(format!("{name}.json"));
    lattice::execution_binding::core::validation::decode_observation(&std::fs::read(path).expect("fixture"))
        .expect("valid synthetic fixture")
}

fn route_declaration(has_route: bool) -> RouteDeclaration {
    if has_route {
        RouteDeclaration::Routes {
            routes: vec![NamedRoute {
                name: "primary".into(),
                route_identity: "blake3:9999999999999999999999999999999999999999999999999999999999999999".into(),
            }],
        }
    } else {
        RouteDeclaration::NoModel {}
    }
}

fn bind(
    compute: ComputeDeclaration,
    routes: RouteDeclaration,
    observed: Option<&ComputeAllocationObservation>,
) -> lattice::execution_binding::core::admission::bound::Admitted {
    let required = WorkerRequirement {
        compute: match compute {
            ComputeDeclaration::HostLocal {} => ComputeRequirement::HostLocal {},
            ComputeDeclaration::ProviderAllocation { .. } => ComputeRequirement::ProviderExecution {
                direct_accelerator: None,
            },
        },
        model: match &routes {
            RouteDeclaration::NoModel {} => ModelRequirement::NoModel {},
            RouteDeclaration::Routes { .. } => ModelRequirement::Routes {
                names: vec!["primary".into()],
            },
        },
    };
    let attempt_id = observed.map_or("attempt-hosted", |observation| observation.allocation.attempt_id.as_str());
    let declaration = Declaration {
        schema: DECLARATION_SCHEMA.into(),
        compute,
        model_routes: routes,
    };
    let attempt = AttemptContext {
        work_id: "work-24",
        attempt_id,
        fence: 7,
        envelope_identity: "envelope-24",
        profile_revision: "profile-24",
    };
    let reading = observed.map_or(ProviderReading::NotRead, ProviderReading::Observed);
    match admit(&declaration, &required, attempt, reading) {
        Decision::Admitted(admitted) => *admitted,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn provider(name: &str) -> lattice::execution_binding::core::admission::bound::Admitted {
    let observed = observation(name);
    let compute = ComputeDeclaration::ProviderAllocation {
        allocation_id: observed.allocation.id.clone(),
        host_id: observed.allocation.host_id.clone(),
        consumer_id: observed.allocation.consumer_id.clone(),
        owner_generation: observed.allocation.owner_generation,
    };
    bind(compute, route_declaration(true), Some(&observed))
}

fn usage(
    admitted: &lattice::execution_binding::core::admission::bound::Admitted,
) -> lattice::orchestration::CommandWorkerObservation {
    lattice::orchestration::CommandWorkerObservation {
        schema: lattice::orchestration::USAGE_OBSERVATION_SCHEMA.into(),
        scope: lattice::orchestration::Scope::Attempt,
        workflow_revision: admitted.attempt.envelope_identity.clone(),
        instance_id: admitted.attempt.work_id.clone(),
        dispatch_id: "request-24".into(),
        run_id: 0,
        node_id: 0,
        invocation_id: "request-24".into(),
        worker_profile: admitted.profile_revision.clone(),
        attempt_id: admitted.attempt.attempt_id.clone(),
        agent_invocations: None,
        model_requests: None,
        subagent_calls: None,
        tokens: None,
        cost_microunits: None,
        duration_millis: Some(42),
        provenance: lattice::orchestration::UsageProvenance::HostMeasured,
        complete: false,
        accounting: None,
        execution_binding: Some(provenance(admitted, "request-24", WorkerCoverage::HostMeasured)),
    }
}

#[test]
fn bound_usage_carries_exact_identity_placement_and_reserved_claims() {
    let admitted = provider("agent-a-1-active");
    let observation = usage(&admitted);
    observation.validate().expect("bound attempt receipt");
    let binding = observation.execution_binding.as_ref().expect("provenance");
    assert_eq!(binding.binding_identity, admitted.binding_identity);
    let ProvenanceCompute::ProviderAllocation(allocation) = &binding.compute else {
        panic!("provider provenance expected");
    };
    assert_eq!((allocation.owner_generation, allocation.charge_cursor), (1, 2));
    assert_eq!(allocation.placement.topology_generation, 1);
    assert!(allocation.reserved_claims.iter().all(|claim| claim.label == ClaimLabel::Reserved));
    assert_eq!(binding.coverage.model_requests, UnknownCoverage::Unknown);
    assert_eq!(binding.coverage.device_utilization, UnknownCoverage::Unknown);
    let text = observation.to_json().expect("json");
    assert!(!text.contains("\"tokens\":0"));
    assert!(!text.contains("\"model_requests\":0"));
}

#[test]
fn shared_hosted_and_no_model_usage_is_never_charged_per_worker() {
    for name in ["agent-a-1-active", "agent-a-2-active"] {
        let text = usage(&provider(name)).to_json().expect("shared usage");
        assert!(text.contains("\"shared_serving_ref\":\"serving-b\""));
        assert!(!text.contains("host-b"), "serving-deployment claims are not charged per worker");
    }
    let hosted = usage(&bind(ComputeDeclaration::HostLocal {}, route_declaration(true), None));
    hosted.validate().expect("hosted-API receipt");
    let hosted_binding = hosted.execution_binding.expect("provenance");
    assert_eq!(hosted_binding.compute, ProvenanceCompute::HostLocal {});
    let no_model = usage(&bind(ComputeDeclaration::HostLocal {}, route_declaration(false), None));
    no_model.validate().expect("no-model receipt");
    assert_eq!(no_model.execution_binding.expect("provenance").model_routes, RouteDeclaration::NoModel {});
}

#[test]
fn attempt_receipts_replay_idempotently_and_refuse_conflicting_bindings() {
    let directory = tempfile::tempdir().expect("store directory");
    let path = directory.path().join("usage.redb");
    let store = lattice::store::Store::new(&path).expect("store");
    let first = usage(&provider("agent-a-1-active"));
    store.save_command_worker_usage(&first).expect("first save");
    store.save_command_worker_usage(&first).expect("identical replay");
    drop(store);
    let store = lattice::store::Store::new(&path).expect("restart");
    store.save_command_worker_usage(&first).expect("restart repeats persistence once");
    assert_eq!(store.load_command_worker_usage(&first.attempt_id).expect("load"), Some(first.clone()));
    assert_eq!(store.load_command_worker_usage_for_run(lattice::engine::RunId(0)).expect("no run index"), None);
    let mut conflicting = first.clone();
    if let Some(binding) = conflicting.execution_binding.as_mut()
        && let ProvenanceCompute::ProviderAllocation(allocation) = &mut binding.compute
    {
        allocation.placement.topology_generation = 2;
    }
    assert!(store.save_command_worker_usage(&conflicting).is_err(), "relocation cannot rebind usage");
    let mut other_binding = first.clone();
    if let Some(binding) = other_binding.execution_binding.as_mut() {
        binding.binding_identity = format!("blake3:{}", "1".repeat(64));
    }
    assert!(store.save_command_worker_usage(&other_binding).is_err());
    assert_eq!(store.load_command_worker_usage(&first.attempt_id).expect("load"), Some(first));
}

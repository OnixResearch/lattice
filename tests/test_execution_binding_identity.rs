//! Fixed vectors for `lattice-execution-binding-v1`, `lattice-model-route-v1`
//! and `lattice-worker-definition-v1`.

// r[verify lattice.execution_binding.identity]
// r[verify lattice.execution_binding.identity.scenario.stable]
// r[verify lattice.execution_binding.identity.scenario.drift]

use lattice::content_identity::*;
use lattice::execution_binding::core::admission::*;
use lattice::execution_binding::core::model::*;
use lattice::execution_binding::core::request::*;

const HOST_LOCAL_VECTOR: &str = "blake3:8b10b32df47976538796d8f83e99bf8c7fea5e9b1ebef7497ffcb4a63ae374c3";
const PROVIDER_VECTOR: &str = "blake3:cbe086cec074dfec82032a440374eb804c8ffd2028981cc74dda52a2fb8cad94";
const DEVICE_FIXTURE_VECTOR: &str = "blake3:0ce7ce5decc45b238a33e81a5beab7463b59cd90ec437926e07a180b8590d98b";
const MODEL_ROUTE_VECTOR: &str = "blake3:de3129027952ff48140d442f8ddde339fe681735501958ce08665b3d5c748ec5";
const WORKER_DEFINITION_VECTOR: &str = "blake3:367783379a41fd2e834f26af4e7504906c1f3967bd9f21bba979441a1f0789c1";
const ROUTE: &str = "blake3:9999999999999999999999999999999999999999999999999999999999999999";

const CLAIMS: [ClaimInput<'static>; 2] = [
    ClaimInput {
        pool_id: "host-a-device-memory",
        quantity: 50,
    },
    ClaimInput {
        pool_id: "host-a-chip",
        quantity: 1,
    },
];
const ROUTES: [RouteInput<'static>; 2] = [
    RouteInput {
        name: "primary",
        route_identity: ROUTE,
    },
    RouteInput {
        name: "embedding",
        route_identity: "blake3:8888888888888888888888888888888888888888888888888888888888888888",
    },
];

fn allocation<'a>(claims: &'a [ClaimInput<'a>]) -> AllocationInput<'a> {
    AllocationInput {
        allocation_id: "agent-a-device",
        consumer_id: "agent-device",
        owner_generation: 1,
        host_id: "host-a",
        host_incarnation: "synthetic-host-generation-1",
        device_incarnation: "synthetic-device-generation-1",
        topology_generation: 1,
        charge_cursor: 4,
        charge_at: 20,
        kind: "execution",
        view_id: "host-a-device",
        granularity: "mmio-bundle",
        claims,
        shared_serving_ref: Some("serving-b"),
        port_id: "synthetic-fixture-port",
    }
}

fn input<'a>(
    allocation: Option<&'a AllocationInput<'a>>,
    routes: Option<&'a [RouteInput<'a>]>,
) -> ExecutionBindingInput<'a> {
    ExecutionBindingInput {
        profile_revision: "profile-24",
        work_id: "work-24",
        attempt_id: "attempt-agent-a-device",
        fence: 7,
        envelope_identity: "envelope-24",
        serving: None,
        allocation,
        routes,
    }
}

fn identity(input: ExecutionBindingInput<'_>) -> String {
    execution_binding(input).expect("bounded identity")
}

#[test]
fn pinned_vectors_frame_host_local_provider_and_model_route_identities() {
    let provider = allocation(&CLAIMS);
    assert_eq!(identity(input(None, None)), HOST_LOCAL_VECTOR);
    assert_eq!(identity(input(Some(&provider), Some(&ROUTES))), PROVIDER_VECTOR);
    let route = model_route(ModelRouteInput {
        schema: "lattice.candidate-model-broker.v1",
        endpoint: "https://model.invalid/v1/chat/completions",
        model: "model",
        credential_provider: Some("env"),
        credential_key: Some("MODEL_TOKEN"),
        credential_field: None,
    })
    .expect("route identity");
    assert_eq!(route, MODEL_ROUTE_VECTOR);
    assert_eq!(EXECUTION_BINDING_DOMAIN, "lattice-execution-binding-v1");
    assert_eq!(MODEL_ROUTE_DOMAIN, "lattice-model-route-v1");
}

#[test]
fn reordered_claims_and_routes_keep_the_pinned_identity() {
    let reversed_claims = [CLAIMS[1], CLAIMS[0]];
    let reversed_routes = [ROUTES[1], ROUTES[0]];
    let provider = allocation(&reversed_claims);
    assert_eq!(identity(input(Some(&provider), Some(&reversed_routes))), PROVIDER_VECTOR);
}

fn observation(name: &str) -> ComputeAllocationObservation {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive")
        .join(format!("{name}.json"));
    lattice::execution_binding::core::validation::decode_observation(&std::fs::read(path).expect("fixture"))
        .expect("valid synthetic fixture")
}

fn admitted_identity(observation: &ComputeAllocationObservation) -> String {
    let declaration = Declaration {
        schema: DECLARATION_SCHEMA.into(),
        serving: None,
        compute: ComputeDeclaration::ProviderAllocation {
            allocation_id: observation.allocation.id.clone(),
            host_id: observation.allocation.host_id.clone(),
            consumer_id: observation.allocation.consumer_id.clone(),
            owner_generation: observation.allocation.owner_generation,
        },
        model_routes: RouteDeclaration::Routes {
            routes: vec![NamedRoute {
                name: "primary".into(),
                route_identity: ROUTE.into(),
            }],
        },
    };
    let requirement = WorkerRequirement {
        compute: ComputeRequirement::ProviderExecution {
            direct_accelerator: None,
        },
        model: ModelRequirement::Routes {
            names: vec!["primary".into()],
        },
    };
    let attempt = AttemptContext {
        work_id: "work-24",
        attempt_id: &observation.allocation.attempt_id,
        fence: 7,
        envelope_identity: "envelope-24",
        profile_revision: "profile-24",
    };
    match admit(&declaration, &requirement, attempt, ProviderReading::Observed(observation)) {
        Decision::Admitted(admitted) => admitted.binding_identity,
        other => panic!("expected admission, got {other:?}"),
    }
}

#[test]
fn re_observation_with_volatile_changes_preserves_the_pinned_identity() {
    let first = observation("device-active");
    assert_eq!(admitted_identity(&first), DEVICE_FIXTURE_VECTOR);
    let mut again = first.clone();
    again.port.observation_sequence = 9;
    again.port.provider_logical_now = Some(70);
    again.readback.revision = 3;
    again.allocation.record_ref = format!("blake3:{}", "a".repeat(64));
    again.allocation.enforcement = Enforcement::BackendFenced;
    again.readback.row.enforcement = Enforcement::BackendFenced;
    again.offer = None;
    assert_eq!(admitted_identity(&again), DEVICE_FIXTURE_VECTOR);
}

#[test]
fn semantic_drift_in_any_covered_field_changes_the_identity() {
    type Drift = for<'a> fn(&mut ExecutionBindingInput<'a>, &mut AllocationInput<'a>);
    let drifts: [Drift; 12] = [
        |input, _| input.profile_revision = "profile-25",
        |input, _| input.attempt_id = "attempt-other",
        |input, _| input.fence = 8,
        |input, _| input.envelope_identity = "envelope-25",
        |_, allocation| allocation.owner_generation = 2,
        |_, allocation| allocation.topology_generation = 2,
        |_, allocation| allocation.charge_cursor = 5,
        |_, allocation| allocation.charge_at = 21,
        |_, allocation| allocation.shared_serving_ref = None,
        |_, allocation| allocation.host_incarnation = "synthetic-host-generation-2",
        |_, allocation| allocation.port_id = "other-port",
        |_, allocation| allocation.claims = &CLAIMS[..1],
    ];
    for drift in drifts {
        let mut provider = allocation(&CLAIMS);
        let mut changed = input(None, Some(&ROUTES));
        drift(&mut changed, &mut provider);
        changed.allocation = Some(&provider);
        assert_ne!(identity(changed), PROVIDER_VECTOR);
    }
    let other_route = [RouteInput {
        name: "primary",
        route_identity: "blake3:7777777777777777777777777777777777777777777777777777777777777777",
    }];
    let provider = allocation(&CLAIMS);
    assert_ne!(identity(input(Some(&provider), Some(&other_route))), PROVIDER_VECTOR);
    assert_ne!(identity(input(Some(&provider), None)), PROVIDER_VECTOR);
    assert_ne!(identity(input(None, Some(&ROUTES))), identity(input(None, None)));
}

// r[verify lattice.worker_inventory.definition]
#[test]
fn worker_definition_identity_is_pinned_and_framed_by_basis() {
    let declared = worker_definition(WorkerDefinitionInput {
        owner: "implementation_attempt",
        basis: "declared",
        name: "rust-implementer",
    })
    .expect("definition identity");
    assert_eq!(declared, WORKER_DEFINITION_VECTOR);
    let default = worker_definition(WorkerDefinitionInput {
        owner: "implementation_attempt",
        basis: "owner_default",
        name: "rust-implementer",
    })
    .expect("default identity");
    assert_ne!(default, declared, "the basis is framed");
    assert_eq!(WORKER_DEFINITION_DOMAIN, "lattice-worker-definition-v1");
}

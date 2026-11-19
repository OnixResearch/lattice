//! Pure admission and revalidation over synthetic fixtures. No port, clock or
//! provider is involved.

// r[verify lattice.execution_binding.admission]
// r[verify lattice.execution_binding.admission.scenario.cpu_remote_model]
// r[verify lattice.execution_binding.admission.scenario.shared_serving]
// r[verify lattice.execution_binding.admission.scenario.hosted_and_no_model]
// r[verify lattice.execution_binding.admission.scenario.direct_device]
// r[verify lattice.execution_binding.admission.scenario.crossed]
// r[verify lattice.execution_binding.persistence.scenario.revalidation]

use lattice::execution_binding::core::admission::bound::*;
use lattice::execution_binding::core::admission::*;
use lattice::execution_binding::core::model::*;
use lattice::execution_binding::core::request::*;
use lattice::execution_binding::core::revalidation::*;

const ROUTE: &str = "blake3:9999999999999999999999999999999999999999999999999999999999999999";

fn observation(name: &str) -> ComputeAllocationObservation {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive")
        .join(format!("{name}.json"));
    lattice::execution_binding::core::validation::decode_observation(&std::fs::read(path).expect("fixture"))
        .expect("valid synthetic fixture")
}

fn routes() -> RouteDeclaration {
    RouteDeclaration::Routes {
        routes: vec![NamedRoute {
            name: "primary".into(),
            route_identity: ROUTE.into(),
        }],
    }
}

fn expected(observation: &ComputeAllocationObservation) -> Declaration {
    Declaration {
        schema: DECLARATION_SCHEMA.into(),
        serving: None,
        compute: ComputeDeclaration::ProviderAllocation {
            allocation_id: observation.allocation.id.clone(),
            host_id: observation.allocation.host_id.clone(),
            consumer_id: observation.allocation.consumer_id.clone(),
            owner_generation: observation.allocation.owner_generation,
        },
        model_routes: routes(),
    }
}

fn requirement(compute: ComputeRequirement) -> WorkerRequirement {
    WorkerRequirement {
        compute,
        model: ModelRequirement::Routes {
            names: vec!["primary".into()],
        },
    }
}

fn cpu() -> WorkerRequirement {
    requirement(ComputeRequirement::ProviderExecution {
        direct_accelerator: None,
    })
}

fn attempt(attempt_id: &str) -> AttemptContext<'_> {
    AttemptContext {
        work_id: "work-24",
        attempt_id,
        fence: 7,
        envelope_identity: "envelope-24",
        profile_revision: "profile-24",
    }
}

fn decide_with(observation: &ComputeAllocationObservation, required: &WorkerRequirement) -> Decision {
    admit(
        &expected(observation),
        required,
        attempt(&observation.allocation.attempt_id),
        ProviderReading::Observed(observation),
    )
}

fn admitted(decision: Decision) -> Admitted {
    match decision {
        Decision::Admitted(admitted) => *admitted,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn provenance(admitted: &Admitted) -> &AllocationProvenance {
    match &admitted.compute {
        AdmittedCompute::ProviderAllocation(provenance) => provenance,
        AdmittedCompute::HostLocal {} => panic!("expected a provider allocation"),
    }
}

fn with_status(
    mut observation: ComputeAllocationObservation,
    status: ReservationStatus,
) -> ComputeAllocationObservation {
    observation.allocation.status = status;
    observation.readback.row.status = status;
    observation.readback.row.is_logically_fenced = status != ReservationStatus::Released;
    if status == ReservationStatus::Intent {
        observation.allocation.activation_backend_ref = None;
    }
    observation
}

fn accelerator(granularity: Granularity, minimum_enforcement: Enforcement) -> WorkerRequirement {
    requirement(ComputeRequirement::ProviderExecution {
        direct_accelerator: Some(DirectAccelerator {
            min_chips: 1,
            accepted_granularities: vec![granularity],
            backend: Some("synthetic-backend".into()),
            runtime: Some("cuda".into()),
            architecture: None,
            minimum_enforcement,
        }),
    })
}

#[test]
fn cpu_worker_binds_allocation_and_remote_model_route_with_opaque_serving_reference() {
    let observation = observation("agent-a-1-active");
    let bound = admitted(decide_with(&observation, &cpu()));
    assert!(is_intact(&bound));
    assert_eq!(bound.model_routes, routes());
    let allocation = provenance(&bound);
    assert_eq!(allocation.shared_serving_ref.as_deref(), Some("serving-b"));
    assert_eq!(allocation.claims, observation.allocation.claims);
    assert_eq!(allocation.placement.host_id, "host-a");
    let encoded = serde_json::to_string(&bound).expect("binding json");
    assert!(!encoded.contains("host-b"));
    assert!(!encoded.contains("synthetic-backend-agent-a-1"), "activation evidence is never projected");
    assert!(!encoded.contains("2222222222"), "grant references are never projected");
}

#[test]
fn two_workers_share_serving_b_without_a_second_serving_charge() {
    let first = admitted(decide_with(&observation("agent-a-1-active"), &cpu()));
    let second = admitted(decide_with(&observation("agent-a-2-active"), &cpu()));
    assert_ne!(first.binding_identity, second.binding_identity);
    for bound in [&first, &second] {
        let allocation = provenance(bound);
        assert_eq!(allocation.shared_serving_ref.as_deref(), Some("serving-b"));
        assert!(allocation.claims.iter().all(|claim| claim.pool_id == "host-a-memory"));
    }
}

#[test]
fn hosted_api_and_no_model_workers_bind_without_leases() {
    let host_local = Declaration {
        schema: DECLARATION_SCHEMA.into(),
        serving: None,
        compute: ComputeDeclaration::HostLocal {},
        model_routes: routes(),
    };
    let hosted = requirement(ComputeRequirement::HostLocal {});
    let bound = admitted(admit(&host_local, &hosted, attempt("attempt-hosted"), ProviderReading::NotRead));
    assert_eq!(bound.compute, AdmittedCompute::HostLocal {});
    let no_model = Declaration {
        model_routes: RouteDeclaration::NoModel {},
        ..host_local.clone()
    };
    let no_model_requirement = WorkerRequirement {
        compute: ComputeRequirement::HostLocal {},
        model: ModelRequirement::NoModel {},
    };
    let bound = admitted(admit(&no_model, &no_model_requirement, attempt("attempt-none"), ProviderReading::NotRead));
    assert_eq!((bound.compute, bound.model_routes), (AdmittedCompute::HostLocal {}, RouteDeclaration::NoModel {}));
    let observed = observation("agent-a-1-active");
    for (declaration, required) in [(&host_local, &hosted), (&no_model, &no_model_requirement)] {
        let decision = admit(declaration, required, attempt("attempt-x"), ProviderReading::Observed(&observed));
        assert_eq!(decision, Decision::Rejected(Rejection::RequirementMismatch));
    }
    let routed = admit(&no_model, &hosted, attempt("attempt-y"), ProviderReading::NotRead);
    assert_eq!(routed, Decision::Rejected(Rejection::ModelBindingMismatch));
}

#[test]
fn direct_accelerator_needs_own_chip_claims_and_is_never_satisfied_by_cpu_or_serving() {
    let device = observation("device-active");
    let required = accelerator(Granularity::MmioBundle, Enforcement::BackendAcknowledged);
    assert!(matches!(decide_with(&device, &required), Decision::Admitted(_)));
    let cpu_only =
        decide_with(&observation("agent-a-1-active"), &accelerator(Granularity::Alias, Enforcement::LogicalFenceOnly));
    assert_eq!(cpu_only, Decision::Rejected(Rejection::InsufficientDevice), "serving-b chips never count");
    let unknown =
        decide_with(&observation("agent-a-2-active"), &accelerator(Granularity::Alias, Enforcement::LogicalFenceOnly));
    assert_eq!(unknown, Decision::Blocked(Block::DeviceEvidenceUnknown));
    let granularity = decide_with(&device, &accelerator(Granularity::Chip, Enforcement::BackendAcknowledged));
    assert_eq!(granularity, Decision::Rejected(Rejection::GranularityNotAccepted));
    let enforcement = decide_with(&device, &accelerator(Granularity::MmioBundle, Enforcement::BackendFenced));
    assert_eq!(enforcement, Decision::Rejected(Rejection::EnforcementTooWeak));
    let mut runtime = accelerator(Granularity::MmioBundle, Enforcement::BackendAcknowledged);
    if let ComputeRequirement::ProviderExecution {
        direct_accelerator: Some(direct),
    } = &mut runtime.compute
    {
        direct.runtime = Some("unified".into());
    }
    assert_eq!(
        decide_with(&device, &runtime),
        Decision::Rejected(Rejection::IncompatibleRuntime { field: "runtime" })
    );
    let serving = observation("serving-b-deployment");
    assert_eq!(decide_with(&serving, &required), Decision::Rejected(Rejection::ServingDeploymentAsExecution));
}

#[test]
fn crossed_released_or_expired_allocations_are_rejected() {
    let base = observation("agent-a-1-active");
    let crossed_attempt = admit(&expected(&base), &cpu(), attempt("attempt-other"), ProviderReading::Observed(&base));
    assert_eq!(crossed_attempt, Decision::Rejected(Rejection::CrossedAttempt));
    type Expectation = fn(&mut Declaration);
    let crossed: [(Expectation, &str); 4] = [
        (|d| set_expected(d, |id, _, _, _| *id = "agent-a-2".into()), "allocation_id"),
        (|d| set_expected(d, |_, host, _, _| *host = "host-b".into()), "host_id"),
        (|d| set_expected(d, |_, _, consumer, _| *consumer = "agent-2".into()), "consumer_id"),
        (|d| set_expected(d, |_, _, _, generation| *generation = 2), "owner_generation"),
    ];
    for (mutate, field) in crossed {
        let mut declaration = expected(&base);
        mutate(&mut declaration);
        let decision = admit(&declaration, &cpu(), attempt("attempt-agent-a-1"), ProviderReading::Observed(&base));
        assert_eq!(decision, Decision::Rejected(Rejection::CrossedOwner { field }));
    }
    let serving = observation("serving-b-deployment");
    assert_eq!(decide_with(&serving, &cpu()), Decision::Rejected(Rejection::ServingDeploymentAsExecution));
    let released = with_status(base.clone(), ReservationStatus::Released);
    assert_eq!(decide_with(&released, &cpu()), Decision::Rejected(Rejection::Released));
    let mut expired = base.clone();
    expired.port.provider_logical_now = Some(80);
    assert_eq!(decide_with(&expired, &cpu()), Decision::Rejected(Rejection::Expired));
    let reading = |reading| admit(&expected(&base), &cpu(), attempt("attempt-agent-a-1"), reading);
    assert_eq!(reading(ProviderReading::NotFound), Decision::Rejected(Rejection::AllocationNotFound));
    let mut malformed = base.clone();
    malformed.producer.time_domain = "wall-clock".into();
    assert!(matches!(decide_with(&malformed, &cpu()), Decision::Rejected(Rejection::Malformed { .. })));
}

fn set_expected(declaration: &mut Declaration, change: fn(&mut String, &mut String, &mut String, &mut u64)) {
    if let ComputeDeclaration::ProviderAllocation {
        allocation_id,
        host_id,
        consumer_id,
        owner_generation,
    } = &mut declaration.compute
    {
        change(allocation_id, host_id, consumer_id, owner_generation);
    }
}

#[test]
fn not_current_or_unknown_allocations_are_blocked_and_never_rebound() {
    let base = observation("agent-a-1-active");
    let intent = with_status(base.clone(), ReservationStatus::Intent);
    assert_eq!(decide_with(&intent, &cpu()), Decision::Blocked(Block::NotActive));
    let quarantined = with_status(base.clone(), ReservationStatus::Quarantined);
    assert_eq!(decide_with(&quarantined, &cpu()), Decision::Blocked(Block::Quarantined));
    let mut stale = base.clone();
    stale.readback.is_fresh = false;
    assert_eq!(decide_with(&stale, &cpu()), Decision::Blocked(Block::ReadbackStale));
    let mut unhealthy = base.clone();
    unhealthy.readback.health = Health::Degraded;
    assert_eq!(decide_with(&unhealthy, &cpu()), Decision::Blocked(Block::HostUnhealthy));
    let mut unknown_time = base.clone();
    unknown_time.port.provider_logical_now = None;
    assert_eq!(decide_with(&unknown_time, &cpu()), Decision::Blocked(Block::ExpiryUnknown));
    let reading = |reading| admit(&expected(&base), &cpu(), attempt("attempt-agent-a-1"), reading);
    assert_eq!(reading(ProviderReading::Unavailable), Decision::Blocked(Block::ProviderUnavailable));
    assert_eq!(reading(ProviderReading::NotRead), Decision::Blocked(Block::ProviderUnavailable));
    assert_eq!(reading(ProviderReading::PortIssue), Decision::Blocked(Block::PortIssue));
}

#[test]
fn unsupported_declarations_are_rejected_before_observation() {
    let base = observation("agent-a-1-active");
    let mut schema = expected(&base);
    schema.schema = "lattice.execution-binding.v2".into();
    let decision = admit(&schema, &cpu(), attempt("attempt-agent-a-1"), ProviderReading::Observed(&base));
    assert_eq!(
        decision,
        Decision::Rejected(Rejection::UnsupportedDeclaration {
            defect: DeclarationDefect::Schema
        })
    );
    let nine = (0..9).map(|index| format!("route-{index}")).collect::<Vec<_>>();
    let too_many = requirement(ComputeRequirement::ProviderExecution {
        direct_accelerator: None,
    });
    let too_many = WorkerRequirement {
        model: ModelRequirement::Routes { names: nine },
        ..too_many
    };
    let decision = admit(&expected(&base), &too_many, attempt("attempt-agent-a-1"), ProviderReading::Observed(&base));
    assert_eq!(
        decision,
        Decision::Rejected(Rejection::UnsupportedDeclaration {
            defect: DeclarationDefect::Routes
        })
    );
    let mut zero = accelerator(Granularity::Chip, Enforcement::BackendFenced);
    if let ComputeRequirement::ProviderExecution {
        direct_accelerator: Some(direct),
    } = &mut zero.compute
    {
        direct.min_chips = 0;
    }
    let zero_chips = DeclarationDefect::Zero { field: "min_chips" };
    assert_eq!(
        decide_with(&base, &zero),
        Decision::Rejected(Rejection::UnsupportedDeclaration { defect: zero_chips })
    );
}

fn next(observation: &ComputeAllocationObservation) -> ComputeAllocationObservation {
    let mut next = observation.clone();
    next.port.observation_sequence = 2;
    next.port.provider_logical_now = Some(40);
    next.readback.revision = 2;
    next.offer = None;
    next
}

#[test]
fn revalidation_requires_unchanged_placement_reservation_and_advancing_sequence() {
    let base = observation("agent-a-1-active");
    let bound = admitted(decide_with(&base, &cpu()));
    assert_eq!(revalidate(&bound, ProviderReading::Observed(&next(&base))), Recheck::Current);
    type Mutation = fn(&mut ComputeAllocationObservation);
    let rejected: [(Mutation, Rejection); 6] = [
        (|o| o.allocation.topology_generation = 2, Rejection::WrongPlacementGeneration {
            field: "topology_generation",
        }),
        (|o| o.allocation.device_incarnation = "device-2".into(), Rejection::WrongPlacementGeneration {
            field: "device_incarnation",
        }),
        (|o| o.allocation.charge_cursor = 9, Rejection::ReservationReplaced { field: "charge_cursor" }),
        (
            |o| {
                o.allocation.owner_generation = 2;
                o.readback.row.owner_generation = 2;
            },
            Rejection::CrossedOwner {
                field: "owner_generation",
            },
        ),
        (|o| o.port.observation_sequence = 1, Rejection::StaleObservationSequence),
        (|o| o.port.provider_logical_now = Some(80), Rejection::Expired),
    ];
    for (mutate, rejection) in rejected {
        let mut current = next(&base);
        mutate(&mut current);
        assert_eq!(revalidate(&bound, ProviderReading::Observed(&current)), Recheck::Rejected(rejection));
    }
    let quarantined = with_status(next(&base), ReservationStatus::Quarantined);
    assert_eq!(revalidate(&bound, ProviderReading::Observed(&quarantined)), Recheck::Blocked(Block::Quarantined));
    let released = with_status(next(&base), ReservationStatus::Released);
    assert_eq!(revalidate(&bound, ProviderReading::Observed(&released)), Recheck::Rejected(Rejection::Released));
    assert_eq!(revalidate(&bound, ProviderReading::PortIssue), Recheck::Blocked(Block::PortIssue));
}

#[test]
fn revalidation_rejects_enforcement_downgrade_and_accepts_upgrade() {
    let device = observation("device-active");
    let bound = admitted(decide_with(&device, &cpu()));
    assert_eq!(provenance(&bound).enforcement_at_bind, Enforcement::BackendAcknowledged);
    for (enforcement, expected) in [
        (Enforcement::LogicalFenceOnly, Recheck::Rejected(Rejection::EnforcementDowngraded)),
        (Enforcement::BackendFenced, Recheck::Current),
    ] {
        let mut current = next(&device);
        current.allocation.enforcement = enforcement;
        current.readback.row.enforcement = enforcement;
        assert_eq!(revalidate(&bound, ProviderReading::Observed(&current)), expected);
    }
    let host_local = admitted(admit(
        &Declaration {
            schema: DECLARATION_SCHEMA.into(),
            serving: None,
            compute: ComputeDeclaration::HostLocal {},
            model_routes: routes(),
        },
        &requirement(ComputeRequirement::HostLocal {}),
        attempt("attempt-hosted"),
        ProviderReading::NotRead,
    ));
    assert_eq!(revalidate(&host_local, ProviderReading::NotRead), Recheck::Current);
    let observed = revalidate(&host_local, ProviderReading::Observed(&device));
    assert_eq!(observed, Recheck::Rejected(Rejection::RequirementMismatch));
}

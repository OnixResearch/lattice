//! Shell binding steps over synthetic fixtures and a recording fixture port.

// r[verify lattice.execution_binding.persistence]
// r[verify lattice.execution_binding.persistence.scenario.persisted_first]
// r[verify lattice.execution_binding.persistence.scenario.revalidation]
// r[verify lattice.execution_binding.port_boundary.scenario.unavailable]
// r[verify lattice.execution_binding.port_boundary.scenario.read_only]
// r[verify lattice.execution_binding.recovery]
// r[verify lattice.execution_binding.recovery.scenario.lost_acknowledgement]
// r[verify lattice.execution_binding.recovery.scenario.cancellation]
// r[verify lattice.execution_binding.usage_provenance.scenario.shared_allocation]
// r[verify lattice.execution_binding.usage_provenance.scenario.bound]

mod brokered;
mod serving;

type Observation = lattice_core::execution_binding::model::ComputeAllocationObservation;

fn observed(name: &str) -> Observation {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive")
        .join(format!("{name}.json"));
    lattice_core::execution_binding::validation::decode_observation(&std::fs::read(path).expect("fixture bytes"))
        .expect("valid synthetic fixture")
}

fn provider(observation: &Observation) -> super::super::Request {
    super::super::tests::request(
        &observation.allocation.attempt_id,
        lattice_core::execution_binding::request::ComputeRequirement::ProviderExecution {
            direct_accelerator: None,
        },
        lattice_core::execution_binding::request::ComputeDeclaration::ProviderAllocation {
            allocation_id: observation.allocation.id.clone(),
            host_id: observation.allocation.host_id.clone(),
            consumer_id: observation.allocation.consumer_id.clone(),
            owner_generation: observation.allocation.owner_generation,
        },
    )
}

fn reply(
    observation: Observation,
) -> Result<crate::execution_binding::AllocationReply, crate::execution_binding::ComputePortIssue> {
    Ok(crate::execution_binding::AllocationReply::Observed(Box::new(observation)))
}

fn later(mut observation: Observation) -> Observation {
    observation.port.observation_sequence = 2;
    observation.offer = None;
    observation
}

fn admitted(observation: &Observation) -> lattice_core::execution_binding::admission::bound::Admitted {
    let request = provider(observation);
    let mut port = crate::execution_binding::FixtureComputePort::new(vec![reply(observation.clone())]);
    super::admit(&request, &mut port).expect("admitted").0
}

#[test]
fn binding_is_durable_before_intent_and_replay_requires_the_same_identity() {
    let root = tempfile::tempdir().expect("attempt directory");
    let first = observed("agent-a-1-active");
    let request = provider(&first);
    let mut port = crate::execution_binding::FixtureComputePort::new(vec![
        reply(first.clone()),
        reply(later(first.clone())),
        reply({
            let mut readmitted = later(first);
            readmitted.allocation.charge_cursor = 9;
            readmitted
        }),
    ]);
    let original = super::admit_and_persist(&request, root.path(), &mut port).expect("persisted binding");
    let path = root.path().join(super::ADMITTED_FILE);
    let persisted = std::fs::read(&path).expect("binding bytes");
    assert!(!root.path().join("intent.json").exists());
    // Interrupted before the intent: a repeated run re-admits the same identity.
    let replayed = super::admit_and_persist(&request, root.path(), &mut port).expect("identical identity");
    assert_eq!(replayed.binding_identity, original.binding_identity);
    let error = super::admit_and_persist(&request, root.path(), &mut port).expect_err("changed reservation refused");
    assert!(error.contains("different execution binding"), "{error}");
    assert_eq!(std::fs::read(&path).expect("binding bytes"), persisted);
    assert_eq!(super::load(root.path(), &request).expect("load"), Some(original));
    assert_eq!(port.calls().len(), 3);
}

#[test]
fn pre_spawn_revalidation_refuses_changed_allocations_without_spawn() {
    let base = observed("agent-a-1-active");
    let bound = admitted(&base);
    type Mutation = fn(&mut Observation);
    let mutations: &[(Mutation, &str)] = &[
        (|o| o.allocation.topology_generation = 2, "wrong_placement_generation"),
        (
            |o| o.allocation.host_incarnation = "synthetic-host-generation-2".into(),
            "wrong_placement_generation",
        ),
        (|o| o.allocation.charge_cursor = 9, "reservation_replaced"),
        (
            |o| {
                o.allocation.owner_generation = 2;
                o.readback.row.owner_generation = 2;
            },
            "crossed_owner",
        ),
        (|o| o.port.observation_sequence = 1, "stale_observation_sequence"),
        (
            |o| {
                o.allocation.status = lattice_core::execution_binding::model::ReservationStatus::Quarantined;
                o.readback.row.status = lattice_core::execution_binding::model::ReservationStatus::Quarantined;
            },
            "quarantined",
        ),
        (
            |o| {
                o.allocation.status = lattice_core::execution_binding::model::ReservationStatus::Released;
                o.readback.row.status = lattice_core::execution_binding::model::ReservationStatus::Released;
                o.readback.row.is_logically_fenced = false;
            },
            "released",
        ),
        (|o| o.port.provider_logical_now = Some(80), "expired"),
    ];
    for (mutate, reason) in mutations {
        let root = tempfile::tempdir().expect("attempt directory");
        let mut current = later(base.clone());
        mutate(&mut current);
        let mut port = crate::execution_binding::FixtureComputePort::new(vec![reply(current.clone()), reply(current)]);
        assert!(super::revalidate(root.path(), &bound, &mut port).is_err(), "{reason}");
        let refusal = super::refusal(root.path()).expect("readable").expect("persisted once");
        assert_eq!(refusal["decision"]["reason"], *reason);
        assert!(super::revalidate(root.path(), &bound, &mut port).is_err(), "replay is still refused");
        let request = provider(&base);
        let report =
            super::super::recovery::not_dispatched(&request, root.path(), &bound, &refusal).expect("not dispatched");
        assert_eq!(report["status"], "not_dispatched_binding_rejected");
        assert_eq!(report["envelope"], serde_json::to_value(&request.envelope).expect("envelope"));
        assert_eq!(report["request_identity"], request.admitted_envelope.as_str());
    }
    let root = tempfile::tempdir().expect("attempt directory");
    let mut port = crate::execution_binding::FixtureComputePort::new(vec![reply(later(base))]);
    super::revalidate(root.path(), &bound, &mut port).expect("unchanged allocation is current");
    assert!(super::refusal(root.path()).expect("readable").is_none());
}

#[test]
fn host_local_bindings_never_call_the_port() {
    let root = tempfile::tempdir().expect("attempt directory");
    let request = super::super::tests::host_local();
    let mut port = crate::execution_binding::FixtureComputePort::new(Vec::new());
    let bound = super::admit_and_persist(&request, root.path(), &mut port).expect("host-local binding");
    super::revalidate(root.path(), &bound, &mut port).expect("host-local is current");
    assert!(port.calls().is_empty());
    assert_eq!(bound.compute, lattice_core::execution_binding::admission::bound::AdmittedCompute::HostLocal {});
}

#[tokio::test]
async fn standalone_port_blocks_provider_allocation_before_any_effect() {
    let root = tempfile::tempdir().expect("scratch");
    let workspace = root.path().join("workspace");
    std::fs::create_dir_all(workspace.join("src")).expect("workspace");
    let mut request = provider(&observed("agent-a-1-active"));
    request.workspace_root = workspace;
    let request_path = root.path().join("request.json");
    std::fs::write(&request_path, serde_json::to_vec(&request).expect("request bytes")).expect("request");
    let state = root.path().join("state");
    let error = super::super::execution::run(
        &request_path,
        &state,
        Some(super::super::Ports {
            compute: &mut crate::execution_binding::UnavailableComputePort,
            serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
            repair: None,
        }),
        super::super::session::revision::Store(None),
    )
    .await
    .expect_err("provider unavailable");
    assert!(error.contains("not admitted"), "{error}");
    let directory = super::super::artifacts::directory(&state, &request);
    for name in [super::ADMITTED_FILE, "intent.json", "candidate.redb"] {
        assert!(!directory.join(name).exists(), "{name} must not exist");
    }
}

fn dispatched(root: &std::path::Path) -> (super::super::Request, std::path::PathBuf, String) {
    let base = observed("agent-a-1-active");
    let request = provider(&base);
    let request_path = root.join("request.json");
    let bytes = serde_json::to_vec(&request).expect("request bytes");
    std::fs::write(&request_path, &bytes).expect("request");
    let state = root.join("state");
    super::super::artifacts::private_directory(&state).expect("state");
    let directory = super::super::artifacts::directory(&state, &request);
    super::super::artifacts::private_directory(&directory).expect("attempt directory");
    let mut port = crate::execution_binding::FixtureComputePort::new(vec![reply(base)]);
    let bound = super::admit_and_persist(&request, &directory, &mut port).expect("binding");
    let identity = blake3::hash(&bytes).to_hex().to_string();
    super::super::artifacts::write_once(&directory.join("intent.json"), identity.as_bytes()).expect("intent");
    assert_eq!(port.calls().len(), 1);
    (request, request_path, bound.binding_identity)
}

#[test]
fn lost_acknowledgement_reconciles_without_rebinding_or_port_calls() {
    let root = tempfile::tempdir().expect("scratch");
    let (request, request_path, binding_identity) = dispatched(root.path());
    let state = root.path().join("state");
    let directory = super::super::artifacts::directory(&state, &request);
    let identity = blake3::hash(&std::fs::read(&request_path).expect("bytes")).to_hex().to_string();
    let error = super::super::execution::recover_existing(
        &request,
        &state,
        &directory,
        &identity,
        super::super::session::revision::Store(None),
    )
    .expect_err("unknown");
    assert!(error.contains("outcome unknown"), "{error}");
    assert!(error.contains(&binding_identity), "{error}");
    assert_eq!(
        super::load(&directory, &request).expect("load").expect("binding").binding_identity,
        binding_identity
    );
}

#[test]
fn cancellation_is_requested_not_observed_and_releases_nothing() {
    let root = tempfile::tempdir().expect("scratch");
    let (request, request_path, binding_identity) = dispatched(root.path());
    let state = root.path().join("state");
    super::super::cancellation::request(&request_path, &state).expect("cancellation requested");
    let directory = super::super::artifacts::directory(&state, &request);
    assert!(directory.join("cancel.request").exists());
    assert!(!directory.join(super::REFUSAL_FILE).exists());
    assert_eq!(
        super::load(&directory, &request).expect("load").expect("binding").binding_identity,
        binding_identity
    );
}

#[test]
fn attempt_usage_keeps_reserved_claims_and_unknown_dimensions() {
    let first = admitted(&observed("agent-a-1-active"));
    let second = admitted(&observed("agent-a-2-active"));
    let both = super::usage::WorkerWindow {
        started_epoch_ms: Some(10),
        finished_epoch_ms: Some(52),
    };
    let receipt = super::usage::WorkerReceipt::new(first.clone(), "request-identity".into());
    let unserved = || lattice_core::execution_binding::serving::usage::ModelRequestCoverage::Unknown;
    let usage = receipt.observation(both, unserved());
    usage.validate().expect("bound attempt usage");
    assert_eq!(usage.duration_millis, Some(42));
    assert_eq!(usage.provenance, crate::orchestration::UsageProvenance::HostMeasured);
    let missing = receipt.observation(
        super::usage::WorkerWindow {
            started_epoch_ms: Some(10),
            finished_epoch_ms: None,
        },
        unserved(),
    );
    missing.validate().expect("unknown duration");
    assert_eq!(missing.duration_millis, None);
    assert_eq!(missing.provenance, crate::orchestration::UsageProvenance::Unknown);
    for bound in [&first, &second] {
        let text = serde_json::to_string(
            &super::usage::WorkerReceipt::new(bound.clone(), "r".into()).observation(both, unserved()),
        )
        .expect("usage json");
        assert!(text.contains("\"label\":\"reserved\""));
        assert!(text.contains("\"shared_serving_ref\":\"serving-b\""));
        assert!(!text.contains("host-b"), "no serving-deployment claim is charged per worker");
        assert!(!text.contains("\"tokens\":0"));
    }
    let hosted =
        super::admit(&super::super::tests::host_local(), &mut crate::execution_binding::UnavailableComputePort)
            .expect("hosted-API binding")
            .0;
    let hosted_usage = super::usage::WorkerReceipt::new(hosted, "r".into()).observation(both, unserved());
    let provenance = hosted_usage.execution_binding.expect("provenance");
    assert_eq!(
        provenance.compute,
        lattice_core::execution_binding::projection::accounting::ProvenanceCompute::HostLocal {}
    );
    let store_root = tempfile::tempdir().expect("store");
    let store = crate::store::Store::new(store_root.path().join("usage.redb")).expect("store");
    let directory = store_root.path();
    receipt.persist(&store, both, directory).expect("persisted before result");
    receipt.persist(&store, both, directory).expect("replay is idempotent");
    assert!(
        receipt
            .persist(
                &store,
                super::usage::WorkerWindow {
                    started_epoch_ms: None,
                    finished_epoch_ms: None
                },
                directory
            )
            .is_err()
    );
}

/// Request files are write-once, read back in order, and feed the attempt
/// usage; a different second write for the same request is refused.
// r[verify lattice.serving_binding.request_observation]
// r[verify lattice.serving_binding.usage.scenario.settled]
#[test]
fn retained_request_files_feed_attempt_usage_and_refuse_rewrites() {
    let root = tempfile::tempdir().expect("attempt directory");
    let mut request = super::super::tests::host_local();
    let lattice_core::execution_binding::request::RouteDeclaration::Routes { routes } =
        &request.execution_binding.model_routes
    else {
        panic!("primary route");
    };
    let route_identity = routes[0].route_identity.clone();
    request.execution_binding.serving = Some(serving_declaration(&request.omp.model_broker.model));
    super::validate_request(&request).expect("served primary route validates");
    let admitted = super::admit(&request, &mut crate::execution_binding::UnavailableComputePort).expect("binding").0;
    assert!(admitted.serving.is_some());
    let mut files = super::requests::RequestFiles::new(root.path()).expect("request files");
    let record = refused_record(&admitted, &route_identity);
    crate::execution_binding::serving::Sink::write_record(&mut files, &record).expect("first write");
    crate::execution_binding::serving::Sink::write_record(&mut files, &record).expect("identical replay");
    let mut changed = record.clone();
    changed.outcome.label = "other".into();
    assert!(crate::execution_binding::serving::Sink::write_record(&mut files, &changed).is_err());
    let coverage = super::requests::coverage(&admitted, root.path()).expect("coverage");
    let lattice_core::execution_binding::serving::usage::ModelRequestCoverage::Served(summary) = coverage else {
        panic!("served coverage");
    };
    assert_eq!(summary.services[0].not_dispatched_requests, 1);
    let mut wrong_model = request.clone();
    if let Some(serving) = &mut wrong_model.execution_binding.serving {
        serving.routes[0].model_id = "another-model".into();
    }
    assert!(super::validate_request(&wrong_model).is_err(), "served model must equal the broker model");
}

pub(in super::super) fn serving_declaration(model: &str) -> lattice_core::execution_binding::serving::Declaration {
    lattice_core::execution_binding::serving::Declaration {
        schema: lattice_core::execution_binding::serving::DECLARATION_SCHEMA.into(),
        profile: lattice_core::execution_binding::serving::Profile {
            profile_id: "capacity-aware-observed-v1".into(),
            identity_rule: lattice_core::execution_binding::serving::IdentityRule::ObservedRequired,
            capacity_rule: lattice_core::execution_binding::serving::CapacityRule::KnownRequired,
            replica_rule: lattice_core::execution_binding::serving::ReplicaRule::SameModelAllowed,
            max_snapshot_age_ticks: 30,
        },
        privacy: lattice_core::execution_binding::serving::Privacy::RemoteAllowed,
        attribution: lattice_core::execution_binding::serving::Attribution::NotApportioned {},
        routes: vec![lattice_core::execution_binding::serving::Route {
            name: super::PRIMARY_ROUTE.into(),
            service_id: "svc-shared".into(),
            endpoint_identity: "endpoint:svc-shared".into(),
            model_id: model.into(),
            configuration_ref: format!("blake3:{}", "a".repeat(64)),
            requirements: lattice_core::execution_binding::serving::Requirements {
                needs_tools: false,
                needs_streaming: false,
                needs_structured_output: false,
                needs_developer_role: false,
                input_tokens: 1_000,
                output_tokens: 100,
            },
        }],
        stop_on_overrun: false,
    }
}

fn refused_record(
    admitted: &lattice_core::execution_binding::admission::bound::Admitted,
    route_identity: &str,
) -> lattice_core::execution_binding::serving::record::Observation {
    lattice_core::execution_binding::serving::record::Observation {
        schema: lattice_core::execution_binding::serving::record::OBSERVATION_SCHEMA.into(),
        binding_identity: admitted.binding_identity.clone(),
        profile_revision: admitted.profile_revision.clone(),
        worker_incarnation: "incarnation".into(),
        attempt_id: admitted.attempt.attempt_id.clone(),
        fence: admitted.attempt.fence,
        route_name: super::PRIMARY_ROUTE.into(),
        route_identity: route_identity.into(),
        service_id: "svc-shared".into(),
        request_id: format!("blake3:{}", "c".repeat(64)),
        ordinal: 1,
        served: None,
        snapshot: None,
        admission: lattice_core::execution_binding::serving::record::Admission {
            label: "not-requested".into(),
            grant_ref: None,
            admission_epoch: None,
        },
        outcome: lattice_core::execution_binding::serving::record::Outcome {
            class: lattice_core::execution_binding::serving::record::OutcomeClass::NotDispatched,
            label: "snapshot-unavailable".into(),
            detail: None,
        },
        usage: lattice_core::execution_binding::serving::record::Usage::UNKNOWN,
        evidence: lattice_core::execution_binding::serving::record::Evidence::Fixture,
        receipt_identity: None,
        provider_receipt_identity: None,
    }
}

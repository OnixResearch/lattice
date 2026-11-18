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
        let report = super::super::recovery::not_dispatched(root.path(), &bound, &refusal).expect("not dispatched");
        assert_eq!(report["status"], "not_dispatched_binding_rejected");
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
        Some(&mut crate::execution_binding::UnavailableComputePort),
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
    let error = super::super::execution::recover_existing(&request, &directory, &identity).expect_err("unknown");
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
    super::super::cancel(&request_path, &state).expect("cancellation requested");
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
    let usage = receipt.observation(both);
    usage.validate().expect("bound attempt usage");
    assert_eq!(usage.duration_millis, Some(42));
    assert_eq!(usage.provenance, crate::orchestration::UsageProvenance::HostMeasured);
    let missing = receipt.observation(super::usage::WorkerWindow {
        started_epoch_ms: Some(10),
        finished_epoch_ms: None,
    });
    missing.validate().expect("unknown duration");
    assert_eq!(missing.duration_millis, None);
    assert_eq!(missing.provenance, crate::orchestration::UsageProvenance::Unknown);
    for bound in [&first, &second] {
        let text =
            serde_json::to_string(&super::usage::WorkerReceipt::new(bound.clone(), "r".into()).observation(both))
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
    let hosted_usage = super::usage::WorkerReceipt::new(hosted, "r".into()).observation(both);
    let provenance = hosted_usage.execution_binding.expect("provenance");
    assert_eq!(
        provenance.compute,
        lattice_core::execution_binding::projection::accounting::ProvenanceCompute::HostLocal {}
    );
    let store_root = tempfile::tempdir().expect("store");
    let store = crate::store::Store::new(store_root.path().join("usage.redb")).expect("store");
    receipt.persist(&store, both).expect("persisted before result");
    receipt.persist(&store, both).expect("replay is idempotent");
    assert!(
        receipt
            .persist(&store, super::usage::WorkerWindow {
                started_epoch_ms: None,
                finished_epoch_ms: None
            })
            .is_err()
    );
}

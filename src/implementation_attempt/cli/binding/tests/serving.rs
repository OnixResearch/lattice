//! `inspect` over a served attempt: the actual readiness cause without a
//! serving port, the injected port's latest snapshot otherwise, and the
//! budget-exhausted refusal recorded once. Synthetic fixtures only; no live
//! serving owner, endpoint or provider is exercised.

// r[verify lattice.serving_binding.readiness]
// r[verify lattice.serving_binding.readiness.scenario.no_port]
// r[verify lattice.serving_binding.readiness.scenario.injected]
// r[verify lattice.serving_binding.overrun]
// r[verify lattice.serving_binding.overrun.scenario.budget_exhausted]

use animus_adapters::serving as producer;

const CAPABILITIES: producer::CapabilitySet = producer::CapabilitySet {
    text: animus_adapters::mesh::Capability::Supported,
    tools: animus_adapters::mesh::Capability::Supported,
    streaming: animus_adapters::mesh::Capability::Supported,
    structured_output: animus_adapters::mesh::Capability::Supported,
    developer_role: animus_adapters::mesh::Capability::Supported,
};

struct Served {
    _root: tempfile::TempDir,
    request: super::super::super::Request,
    request_path: std::path::PathBuf,
    state: std::path::PathBuf,
    directory: std::path::PathBuf,
}

/// A served host-local attempt whose binding is persisted; no worker ran.
fn served() -> Served {
    let root = tempfile::tempdir().expect("scratch");
    let mut request = super::super::super::tests::host_local();
    request.execution_binding.serving = Some(super::serving_declaration(&request.omp.model_broker.model));
    let request_path = root.path().join("request.json");
    std::fs::write(&request_path, serde_json::to_vec(&request).expect("request bytes")).expect("request");
    let state = root.path().join("state");
    super::super::super::artifacts::private_directory(&state).expect("state");
    let directory = super::super::super::artifacts::directory(&state, &request);
    super::super::super::artifacts::private_directory(&directory).expect("attempt directory");
    super::super::admit_and_persist(&request, &directory, &mut crate::execution_binding::UnavailableComputePort)
        .expect("binding");
    Served {
        _root: root,
        request,
        request_path,
        state,
        directory,
    }
}

fn ready_snapshot(model: &str) -> crate::execution_binding::serving::SnapshotReply {
    let configuration = format!("blake3:{}", "a".repeat(64));
    let service = producer::ServiceDraft {
        service_id: "svc-shared",
        endpoint_identity: "endpoint:svc-shared",
        deployment_id: "deploy-svc-shared",
        deployment_generation: 1,
        locality: producer::Locality::PrivateRemote,
        model: Some(producer::ModelDraft {
            model_id: model,
            configuration_ref: &configuration,
            attestation: producer::Attestation::Observed,
        }),
        readiness: producer::Readiness::Ready,
        capabilities: CAPABILITIES,
        envelope: None,
        capacity: &[],
        admission: producer::AdmissionOffer::OwnerAdmission,
        replica: None,
        resource_consumer_ref: None,
        rollout: None,
    };
    let snapshot = producer::AvailabilitySnapshot::new(&producer::SnapshotDraft {
        source_id: "host-observer",
        source_generation: 1,
        sequence: 3,
        observed_tick: 100,
        valid_until_tick: 160,
        services: std::slice::from_ref(&service),
    })
    .expect("fixture snapshot");
    crate::execution_binding::serving::SnapshotReply::Observed {
        snapshot: Box::new(snapshot),
        now_tick: Some(105),
    }
}

/// A served attempt's evidence directory proves a count only while it is
/// retained: pre-created and empty it is exactly zero, missing it is
/// `unknown`, and the draw then charges the reservation.
// r[verify lattice.serving_binding.brokered_evidence.scenario.charged]
// r[verify lattice.implementation_attempt.observed_charge.scenario.incomplete]
#[test]
fn missing_served_evidence_is_unknown_and_an_empty_retained_directory_is_zero() {
    let served = served();
    let admitted = super::super::load(&served.directory, &served.request).expect("load").expect("binding");
    let mut request = served.request.clone();
    request.lineage = Some(crate::implementation_attempt::lineage::tests::binding(4, 4));
    let frame = super::super::super::repair::SingleShotDraw::of(&request).expect("lineage");
    let ceiling = request.omp.model_broker.max_requests;
    let coverage =
        |directory: &std::path::Path| super::super::requests::coverage(&admitted, directory).expect("coverage");
    let missing = coverage(&served.directory);
    assert_eq!(missing, lattice_core::execution_binding::serving::usage::ModelRequestCoverage::Unknown);
    let mut files = super::super::requests::RequestFiles::new(&served.directory).expect("pre-created evidence");
    let empty = coverage(&served.directory);
    assert_eq!(empty.observed_requests(None), Some(0), "a retained empty directory proves no request");
    assert_eq!(frame.draw(Some(&empty))["usage"][0]["charged"]["model_requests"], 0);
    // A request is recorded, then the directory is lost before the draw.
    let mut port = crate::execution_binding::serving::FixtureOwner::default();
    {
        let session =
            crate::execution_binding::serving::gate::Session::new(&admitted, "incarnation-a", &mut port, &mut files)
                .expect("session");
        let mut gate = crate::candidate_worker::broker::OwnerGate {
            session,
            route_name: super::super::PRIMARY_ROUTE.into(),
        };
        crate::candidate_worker::broker::ModelRequestGate::exhausted(&mut gate).expect("recorded");
    }
    std::fs::remove_dir_all(served.directory.join("model-requests")).expect("lose the evidence");
    let lost = coverage(&served.directory);
    assert_eq!(lost, lattice_core::execution_binding::serving::usage::ModelRequestCoverage::Unknown);
    assert_eq!(lost.observed_requests(None), None, "lost evidence is never zero");
    let draw = frame.draw(Some(&lost));
    assert_eq!(draw["usage"][0]["provenance"], "unknown");
    assert_eq!(draw["usage"][0]["model_requests"], serde_json::Value::Null);
    assert_eq!(draw["usage"][0]["charged"]["model_requests"], ceiling, "the reservation is charged");
}

/// Two sessions of one binding and worker incarnation over the same retained
/// files: the second continues after the first's ordinals, so its records
/// never collapse into the first's, and a read-only replay counts both.
// r[verify lattice.serving_binding.brokered_evidence.scenario.conflict]
#[test]
fn a_restarted_session_over_retained_files_continues_after_their_ordinals() {
    let served = served();
    let admitted = super::super::load(&served.directory, &served.request).expect("load").expect("binding");
    let mut files = super::super::requests::RequestFiles::new(&served.directory).expect("request files");
    for _ in 0..2 {
        let mut port = crate::execution_binding::serving::FixtureOwner::default();
        let session =
            crate::execution_binding::serving::gate::Session::new(&admitted, "incarnation-a", &mut port, &mut files)
                .expect("session over retained files");
        let mut gate = crate::candidate_worker::broker::OwnerGate {
            session,
            route_name: super::super::PRIMARY_ROUTE.into(),
        };
        crate::candidate_worker::broker::ModelRequestGate::exhausted(&mut gate).expect("recorded");
    }
    let (_, records) = super::super::requests::load::<
        lattice_core::execution_binding::serving::record::DispatchIntent,
        lattice_core::execution_binding::serving::record::Observation,
    >(&served.directory)
    .expect("retained evidence")
    .expect("evidence directory");
    let ordinals: std::collections::BTreeSet<u64> = records.iter().map(|record| record.ordinal).collect();
    assert_eq!(ordinals, [1, 2].into(), "the second session continued after the first");
    let coverage = super::super::requests::coverage(&admitted, &served.directory).expect("replay");
    assert_eq!(coverage, super::super::requests::coverage(&admitted, &served.directory).expect("replay again"));
    let lattice_core::execution_binding::serving::usage::ModelRequestCoverage::Served(summary) = coverage else {
        panic!("served coverage");
    };
    assert_eq!(summary.services[0].not_dispatched_requests, 2);
}

#[test]
fn inspect_names_the_missing_port_and_shows_the_budget_refusal_once() {
    let served = served();
    let admitted = super::super::load(&served.directory, &served.request).expect("load").expect("binding");
    let mut port = crate::execution_binding::serving::FixtureOwner::default();
    let mut files = super::super::requests::RequestFiles::new(&served.directory).expect("request files");
    {
        let session =
            crate::execution_binding::serving::gate::Session::new(&admitted, "incarnation-a", &mut port, &mut files)
                .expect("session");
        let mut gate = crate::candidate_worker::broker::OwnerGate {
            session,
            route_name: super::super::PRIMARY_ROUTE.into(),
        };
        crate::candidate_worker::broker::ModelRequestGate::exhausted(&mut gate).expect("recorded");
        crate::candidate_worker::broker::ModelRequestGate::exhausted(&mut gate).expect("no second record");
    }
    assert!(port.calls().is_empty(), "budget exhaustion never calls the serving port");
    let (intents, records) = super::super::requests::load::<
        lattice_core::execution_binding::serving::record::DispatchIntent,
        lattice_core::execution_binding::serving::record::Observation,
    >(&served.directory)
    .expect("retained evidence")
    .expect("evidence directory");
    assert_eq!((intents.len(), records.len()), (0, 1), "the refusal is recorded once and never dispatched");
    crate::execution_binding::serving::Sink::write_record(&mut files, &records[0]).expect("identical replay");
    let output = super::super::super::inspect::project(
        &served.request_path,
        &served.state,
        &mut crate::execution_binding::serving::UnavailableOwner,
        unmanaged(),
    )
    .expect("inventory");
    assert_eq!(output["profile"]["serving_readiness"]["state"], "no_serving_port");
    assert_eq!(output["instance"]["model_readiness"]["state"], "no_serving_port");
    let summary = &output["model_requests"]["served"];
    assert_eq!(summary["overrun"]["refused_requests"], 1);
    assert_eq!(summary["services"][0]["not_dispatched_requests"], 1);
    assert_eq!(summary["services"][0]["requests"][0]["outcome"], "budget-exhausted");
    assert_eq!(summary["services"][0]["requests"][0]["class"], "not_dispatched");
}

#[test]
fn inspect_reports_the_injected_ports_latest_snapshot_readiness_and_freshness() {
    let served = served();
    let mut port = crate::execution_binding::serving::FixtureOwner::new(
        vec![ready_snapshot(&served.request.omp.model_broker.model)],
        vec![],
        vec![],
    );
    let output = super::super::super::inspect::project(&served.request_path, &served.state, &mut port, unmanaged())
        .expect("inventory");
    let readiness = &output["instance"]["model_readiness"];
    assert_eq!(readiness["state"], "observed");
    assert_eq!(output["profile"]["serving_readiness"], *readiness);
    let route = &readiness["routes"][0];
    assert_eq!((route["route_name"].as_str(), route["service_id"].as_str()), (Some("primary"), Some("svc-shared")));
    assert_eq!(route["snapshot"]["reading"], "observed");
    assert_eq!(route["snapshot"]["readiness"], "ready");
    assert_eq!(route["snapshot"]["freshness"], "current");
    assert_eq!(route["snapshot"]["sequence"], 3);
    let calls = port.calls();
    assert_eq!(calls.len(), 1, "one snapshot read per served route");
    assert!(
        calls.iter().all(|call| matches!(call, crate::execution_binding::serving::Call::Snapshot(_))),
        "inspect never admits or rechecks a request"
    );
    let route_reasons = |section: &serde_json::Value| {
        section["wait_reasons"]
            .as_array()
            .expect("reasons")
            .iter()
            .filter(|reason| reason["subject"]["kind"] == "route")
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(route_reasons(&output["profile"]), Vec::<serde_json::Value>::new(), "a ready, current route waits");
    let mut silent = crate::execution_binding::serving::FixtureOwner::default();
    let output = super::super::super::inspect::project(&served.request_path, &served.state, &mut silent, unmanaged())
        .expect("inventory");
    assert_eq!(output["instance"]["model_readiness"]["routes"][0]["snapshot"]["reading"], "unavailable");
    let expected = serde_json::json!([{
        "code": "serving_snapshot_unavailable",
        "subject": {"kind": "route", "name": "primary"},
        "owner": "animus#17",
    }]);
    assert_eq!(serde_json::Value::from(route_reasons(&output["instance"])), expected);
    assert_eq!(serde_json::Value::from(route_reasons(&output["profile"])), expected);
    assert!(
        !output["instance"]["wait_reasons"].to_string().contains("svc-shared"),
        "service ids stay in readiness"
    );
}

/// Inspection without a revision store.
const fn unmanaged() -> super::super::super::session::revision::Store<'static> {
    super::super::super::session::revision::Store(None)
}

/// Writes the served v3 captures Kiln uses as fixtures, only when
/// `LATTICE_INVENTORY_CAPTURE_DIR` names an existing directory.
// r[verify lattice.worker_inventory.captures.scenario.fixture]
// r[verify lattice.worker_inventory.wait_reasons.scenario.causes]
#[test]
fn capture_served_worker_inventory_v3() {
    let Some(dir) = std::env::var_os("LATTICE_INVENTORY_CAPTURE_DIR") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let served = served();
    let mut port = crate::execution_binding::serving::FixtureOwner::new(
        vec![ready_snapshot(&served.request.omp.model_broker.model)],
        vec![],
        vec![],
    );
    let output = super::super::super::inspect::project(&served.request_path, &served.state, &mut port, unmanaged())
        .expect("inventory");
    std::fs::write(dir.join("host-local-served-ready.json"), serde_json::to_vec_pretty(&output).expect("json"))
        .expect("write capture");
    let mut silent = crate::execution_binding::serving::FixtureOwner::default();
    let output = super::super::super::inspect::project(&served.request_path, &served.state, &mut silent, unmanaged())
        .expect("inventory");
    std::fs::write(dir.join("host-local-served-unavailable.json"), serde_json::to_vec_pretty(&output).expect("json"))
        .expect("write capture");
}

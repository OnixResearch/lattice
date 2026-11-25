//! Local fixture only: prepare -> one injected host dispatch -> exact native inspect.

use super::{Host, HostObservation, HostRequest, Lineage, Edge};
use crate::implementation_attempt::cli::{self, Command, CommandArgs, Ports};

struct FixtureHost {
    identity: String,
    emit: Box<dyn Fn(&HostRequest<'_>) -> Option<Lineage> + Send>,
}

impl Host for FixtureHost {
    fn adapter_identity(&self) -> &str { &self.identity }

    fn run(
        &mut self,
        request: &HostRequest<'_>,
        worker: &crate::candidate_worker::Request,
        _cancellation: &std::sync::atomic::AtomicBool,
        _services: crate::candidate_worker::broker::Services<'_>,
    ) -> Result<HostObservation, String> {
        let before = crate::candidate_capture::workspace(&worker.boundary.root).map_err(|error| error.to_string())?;
        std::fs::write(worker.boundary.root.join("src.rs"), b"implemented")
            .map_err(|error| error.to_string())?;
        let after = crate::candidate_capture::workspace(&worker.boundary.root).map_err(|error| error.to_string())?;
        let fence = worker.fence.as_str();
        let receipt = crate::candidate::produced_receipt_from_snapshots(
            crate::candidate::Bindings {
                work_id: &worker.work_id,
                attempt_id: &worker.attempt_id,
                fence,
                workspace_id: &worker.workspace_id,
            },
            &before, &after,
        ).map_err(|error| error.to_string())?;
        let changed_files = after.into_iter()
            .filter(|(path, _)| receipt.changed_paths.contains(path))
            .collect();
        Ok(HostObservation {
            outcome: crate::candidate_worker::Outcome::Produced {
                receipt: Box::new(receipt), changed_files, stdout: Vec::new(), stderr: Vec::new(),
            },
            lineage: (self.emit)(request),
        })
    }
}

fn fixture(root: &std::path::Path) -> cli::Request {
    std::fs::write(root.join("src.rs"), b"base").expect("base file");
    let mut request = super::super::tests::host_local();
    let model_broker = request.omp.take().expect("legacy fixture").model_broker;
    request.envelope.work_id = "a".repeat(64);
    request.envelope.attempt_id = "b".repeat(64);
    request.workspace_root = root.to_path_buf();
    request.read_paths = vec!["src.rs".into()];
    request.write_paths = vec!["src.rs".into()];
    request.program.clear();
    request.sandbox_program.clear();
    request.animus_implementation = Some(super::Profile {
        adapter_revision: "c".repeat(64),
        adapter_identity: "d".repeat(64),
        base_identity: "0".repeat(64),
        prompt: "implement the admitted task".into(),
        model_broker,
    });
    request.envelope.worker_profile = request.worker_profile_identity().expect("Animus revision");
    request.admitted_envelope = request.envelope.identity();
    request.execution_binding.model_routes = lattice_core::execution_binding::request::RouteDeclaration::Routes {
        routes: vec![super::super::binding::primary_route(&request).expect("model route")],
    };
    request
}

fn lineage(request: &HostRequest<'_>) -> Lineage {
    Lineage {
        schema: super::LINEAGE_SCHEMA.into(),
        work_identity: request.work_identity.into(),
        attempt_identity: request.attempt_identity.into(),
        fence: request.fence,
        request_identity: request.request_identity.into(),
        worker_incarnation: request.worker_incarnation.into(),
        profile_revision: request.profile_revision.into(),
        root_id: "1".repeat(64),
        coverage: "complete".into(),
        edges: vec![Edge { parent_id: "1".repeat(64), child_id: "2".repeat(64), state: "completed".into() }],
    }
}

/// Optional independent Animus integration: the executable is an already
/// compiled local fixture, supplied only to this test. No sibling path enters
/// the library or product request.
fn emitted_lineage(request: &HostRequest<'_>, root: &std::path::Path) -> Lineage {
    let Some(executable) = std::env::var_os("LATTICE_ANIMUS_LINEAGE_FIXTURE_EXECUTABLE") else {
        return lineage(request);
    };
    let binding = root.join("animus-binding.json");
    let output = root.join("animus-receipt.json");
    let encoded = serde_json::json!({
        "work_identity": request.work_identity,
        "attempt_identity": request.attempt_identity,
        "fence": request.fence,
        "request_identity": request.request_identity,
        "worker_incarnation": request.worker_incarnation,
        "profile_revision": request.profile_revision,
    });
    std::fs::write(&binding, serde_json::to_vec(&encoded).expect("fixture binding"))
        .expect("fixture binding file");
    let status = std::process::Command::new(executable)
        .arg("--exact")
        .arg("parent_steel_tool_dispatches_real_child_and_captures_native_lineage")
        .env("ANIMUS_LINEAGE_FIXTURE_BINDING", &binding)
        .env("ANIMUS_LINEAGE_FIXTURE_OUTPUT", &output)
        .status().expect("native Animus fixture executable");
    assert!(status.success(), "native Animus child fixture must succeed");
    serde_json::from_slice(&std::fs::read(output).expect("native child receipt"))
        .expect("native child lineage wire")
}

#[tokio::test]
async fn prepared_native_host_dispatch_retains_exact_lineage_and_candidate() {
    let root = tempfile::tempdir().expect("fixture root");
    let workspace = root.path().join("workspace");
    std::fs::create_dir(&workspace).expect("workspace");
    let request = fixture(&workspace);
    let raw = root.path().join("raw.json");
    let prepared = root.path().join("prepared.json");
    let state = root.path().join("state");
    std::fs::write(&raw, serde_json::to_vec(&request).expect("request")).expect("raw request");
    cli::artifacts::prepare(&raw, &prepared).expect("prepare");
    let admitted: cli::Request = serde_json::from_slice(&std::fs::read(&prepared).expect("prepared bytes"))
        .expect("prepared request");
    let adapter_identity = admitted.animus_implementation.as_ref().expect("opt in").adapter_identity.clone();
    let args = CommandArgs { command: Command::Run {
        request: prepared.clone(), state: state.clone(), revision_store: None, repair_checks: None,
    }};
    let mut compute = crate::execution_binding::UnavailableComputePort;
    let ports = Ports {
        compute: &mut compute,
        serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
        repair: None,
    };
    let fixture_root = root.path().to_path_buf();
    cli::run_command_with_animus_host(&args, ports, Box::new(FixtureHost {
        identity: adapter_identity,
        emit: Box::new(move |request| Some(emitted_lineage(request, &fixture_root))),
    })).await.expect("one bound dispatch");
    let mut owner = crate::execution_binding::serving::UnavailableOwner;
    let inventory = cli::inspect::project(&prepared, &state, &mut owner, cli::session::revision::Store(None))
        .expect("inspect retained state");
    assert_eq!(inventory["profile"]["adapter"]["value"], "animus-implementation");
    if std::env::var_os("LATTICE_ANIMUS_LINEAGE_FIXTURE_EXECUTABLE").is_some() {
        let native: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.path().join("animus-receipt.json")).expect("producer wire"))
            .expect("producer JSON");
        assert_eq!(inventory["agent_lineage"], native);
    } else {
        assert_eq!(inventory["agent_lineage"]["root_id"], "1".repeat(64));
        assert_eq!(inventory["agent_lineage"]["edges"][0]["child_id"], "2".repeat(64));
    }
    assert_eq!(inventory["agent_lineage"]["request_identity"],
        blake3::hash(&std::fs::read(&prepared).expect("prepared bytes")).to_hex().to_string());
    assert!(inventory["instance"].is_object(), "candidate attempt remains projected");
}

#[tokio::test]
async fn absent_native_host_refuses_before_execution_intent() {
    let root = tempfile::tempdir().expect("fixture root");
    let workspace = root.path().join("workspace");
    std::fs::create_dir(&workspace).expect("workspace");
    let request = fixture(&workspace);
    let raw = root.path().join("raw.json");
    let prepared = root.path().join("prepared.json");
    let state = root.path().join("state");
    std::fs::write(&raw, serde_json::to_vec(&request).expect("request")).expect("raw");
    cli::artifacts::prepare(&raw, &prepared).expect("prepare");
    let mut compute = crate::execution_binding::UnavailableComputePort;
    let ports = Ports { compute: &mut compute,
        serving: Box::new(crate::execution_binding::serving::UnavailableOwner), repair: None };
    let args = CommandArgs { command: Command::Run {
        request: prepared.clone(), state: state.clone(), revision_store: None, repair_checks: None,
    }};
    let error = cli::run_command_with_ports(&args, ports).await.expect_err("host absent");
    assert!(error.contains("pinned Animus host is unavailable"), "{error}");
    let admitted: cli::Request = serde_json::from_slice(&std::fs::read(&prepared).expect("prepared"))
        .expect("request");
    let directory = cli::artifacts::directory(&state, &admitted);
    assert!(!directory.join("candidate.redb").exists(), "no candidate or worker launch");
    assert!(!directory.join("animus-child-lineage.json").exists());
}

#[test]
fn forged_host_candidate_is_never_published_as_a_candidate() {
    let root = tempfile::tempdir().expect("fixture root");
    let workspace = root.path().join("workspace");
    std::fs::create_dir(&workspace).expect("workspace");
    let request = fixture(&workspace);
    let before = crate::candidate_capture::workspace(&workspace).expect("before");
    let modes = crate::candidate_capture::snapshot_modes(&workspace, &before).expect("modes");
    std::fs::write(workspace.join("src.rs"), b"candidate").expect("candidate");
    let after = crate::candidate_capture::workspace(&workspace).expect("after");
    let fence = request.envelope.fence.to_string();
    let receipt = crate::candidate::produced_receipt_from_snapshots(crate::candidate::Bindings {
        work_id: &request.envelope.work_id, attempt_id: &request.envelope.attempt_id,
        fence: &fence, workspace_id: &request.envelope.workspace_id,
    }, &before, &after).expect("receipt");
    let forged = crate::candidate_worker::Outcome::Produced {
        receipt: Box::new(receipt), changed_files: Default::default(),
        stdout: Vec::new(), stderr: Vec::new(),
    };
    let verified = super::verify_candidate(&request, &workspace, &before, &modes, forged);
    assert!(matches!(verified, crate::candidate_worker::Outcome::Unknown { .. }));
}

#[test]
fn missing_crossed_and_malformed_lineage_are_unobserved() {
    let root = tempfile::tempdir().expect("fixture root");
    let workspace = root.path().join("workspace");
    std::fs::create_dir(&workspace).expect("workspace");
    let request = fixture(&workspace);
    let directory = root.path().join("attempt");
    std::fs::create_dir(&directory).expect("attempt");
    let request_identity = "e".repeat(64);
    assert!(super::retained(&request, &directory, &request_identity).is_none());
    let worker_incarnation = super::incarnation(&request_identity);
    let expected = HostRequest {
        work_identity: &request.envelope.work_id,
        attempt_identity: &request.envelope.attempt_id,
        fence: request.envelope.fence,
        request_identity: &request_identity,
        worker_incarnation: &worker_incarnation,
        profile_revision: &request.envelope.worker_profile,
        profile: request.animus_implementation.as_ref().expect("profile"),
    };
    let mut crossed = lineage(&expected);
    crossed.request_identity = "f".repeat(64);
    let path = directory.join("animus-child-lineage.json");
    std::fs::write(&path, serde_json::to_vec(&crossed).expect("crossed")).expect("receipt");
    assert!(super::retained(&request, &directory, &request_identity).is_none());
    std::fs::write(&path, b"not-json").expect("malformed receipt");
    assert!(super::retained(&request, &directory, &request_identity).is_none());
}

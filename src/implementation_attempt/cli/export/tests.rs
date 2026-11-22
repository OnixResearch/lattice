use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::implementation_attempt::AttemptBudget;
use crate::implementation_attempt::ImplementationAttemptEnvelope;
use crate::implementation_attempt::SourceKind;

/// Export without a revision store: the attempt is unmanaged.
fn run(request: &std::path::Path, state: &std::path::Path, output: &std::path::Path) -> Result<(), String> {
    super::run(request, state, output, super::super::session::revision::Store(None))
}

fn execute(
    request: &std::path::Path,
    state: &std::path::Path,
    output: &std::path::Path,
) -> Result<serde_json::Value, String> {
    super::execute(request, state, output, super::super::session::revision::Store(None))
}

struct Fixture {
    _root: tempfile::TempDir,
    request: PathBuf,
    state: PathBuf,
    output: PathBuf,
    workspace: PathBuf,
    request_value: super::super::Request,
    receipt: crate::candidate::CandidateReceipt,
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().expect("fixture root");
    let base = root.path().join("base");
    let workspace = root.path().join("workspace");
    std::fs::create_dir(&base).expect("base dir");
    std::fs::create_dir(&workspace).expect("workspace dir");
    std::fs::write(base.join("source.txt"), b"base").expect("base file");
    std::fs::write(workspace.join("source.txt"), b"candidate").expect("candidate file");
    let base_map = crate::candidate_capture::workspace(&base).expect("capture base");
    let candidate = crate::candidate_capture::workspace(&workspace).expect("capture candidate");
    let base_identity = crate::candidate::snapshot_identity(&base_map).expect("base identity");
    let mut envelope = ImplementationAttemptEnvelope {
        schema: crate::implementation_attempt::IMPLEMENTATION_ATTEMPT_ENVELOPE_SCHEMA.into(),
        work_id: "work-export".into(),
        work_version: 1,
        source_kind: SourceKind::CairnTask,
        source_id: "task-export".into(),
        repository: "OnixResearch/lattice".into(),
        base: "a".repeat(40),
        readiness_snapshot: "b".repeat(64),
        task_spec_digest: "c".repeat(64),
        verification_profile: "f".repeat(64),
        attempt_id: "attempt-export".into(),
        fence: 7,
        workspace_id: "workspace-export".into(),
        worker_profile: String::new(),
        budget: AttemptBudget {
            max_turns: 1,
            max_seconds: 60,
            max_cost_micros: 1,
        },
        policy_digest: "d".repeat(64),
    };
    let mut request_value = super::super::Request {
        schema: super::super::REQUEST_SCHEMA.into(),
        envelope: envelope.clone(),
        admitted_envelope: String::new(),
        expires_at_ms: u64::MAX,
        workspace_root: workspace.clone(),
        read_paths: vec![".".into()],
        write_paths: vec![".".into()],
        program: "/usr/bin/worker".into(),
        sandbox_program: "/usr/bin/bwrap".into(),
        omp: crate::candidate_worker::omp::Profile {
            revision: crate::candidate_worker::omp::OMP_REVISION.into(),
            executable_blake3: "e".repeat(64),
            base_identity,
            model: "lattice/model".into(),
            prompt: "make the candidate".into(),
            model_broker: crate::candidate_worker::broker::Profile {
                schema: crate::candidate_worker::broker::SCHEMA.into(),
                endpoint: "https://model.invalid/v1/chat/completions".into(),
                model: "model".into(),
                credential: None,
                relay: crate::candidate_worker::broker::Executable {
                    path: "/nix/store/relay".into(),
                    blake3: "a".repeat(64),
                },
                shell: crate::candidate_worker::broker::Executable {
                    path: "/nix/store/shell".into(),
                    blake3: "b".repeat(64),
                },
                sandbox_blake3: "c".repeat(64),
                expires_at_ms: u64::MAX,
                max_requests: 1,
                forwarded: None,
            },
        },
        output_limit_bytes: 4096,
        allow_unchanged: false,
        worker_compute: lattice_core::execution_binding::request::ComputeRequirement::HostLocal {},
        execution_binding: lattice_core::execution_binding::request::Declaration {
            schema: lattice_core::execution_binding::request::DECLARATION_SCHEMA.into(),
            serving: None,
            compute: lattice_core::execution_binding::request::ComputeDeclaration::HostLocal {},
            model_routes: lattice_core::execution_binding::request::RouteDeclaration::NoModel {},
        },
        worker_capability: None,
        network: None,
        dispatch_plan: None,
        repair: None,
        lineage: None,
        workflow_agent: None,
        vm_binding: None,
        vm_profile: None,
    };
    request_value.execution_binding.model_routes = lattice_core::execution_binding::request::RouteDeclaration::Routes {
        routes: vec![super::super::binding::primary_route(&request_value).expect("primary route")],
    };
    envelope.worker_profile = request_value.worker_profile_identity().expect("profile identity");
    request_value.envelope = envelope;
    request_value.admitted_envelope = request_value.envelope.identity();
    let receipt = crate::candidate::produced_receipt_from_snapshots(
        crate::candidate::Bindings {
            work_id: "work-export",
            attempt_id: "attempt-export",
            fence: "7",
            workspace_id: "workspace-export",
        },
        &base_map,
        &candidate,
    )
    .expect("receipt");
    let state = root.path().join("state");
    std::fs::create_dir(&state).expect("state");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&state, std::fs::Permissions::from_mode(0o700)).expect("state mode");
    }
    let output = root.path().join("output");
    std::fs::create_dir(&output).expect("output parent");
    let request = root.path().join("request.json");
    std::fs::write(&request, serde_json::to_vec(&request_value).expect("request bytes")).expect("request file");
    Fixture {
        _root: root,
        request,
        state,
        output,
        workspace,
        request_value,
        receipt,
    }
}

fn retain_completed(f: &Fixture) {
    let dir = super::super::artifacts::directory(&f.state, &f.request_value);
    std::fs::create_dir(&dir).expect("attempt dir");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).expect("attempt mode");
    }
    let request_bytes = std::fs::read(&f.request).expect("request bytes");
    let (bound, observation) =
        super::super::binding::admit(&f.request_value, &mut crate::execution_binding::UnavailableComputePort)
            .expect("host-local binding");
    super::super::binding::persist(&dir, &bound, observation).expect("binding before intent");
    super::super::artifacts::write_once(&dir.join("intent.json"), blake3::hash(&request_bytes).to_hex().as_bytes())
        .expect("intent");
    let raw = concat!(
        "{\"type\":\"session\",\"version\":3,\"id\":\"owned-session\",\"cwd\":\"/workspace\"}\n",
        "{\"type\":\"agent_start\"}\n",
        "{\"type\":\"message_end\",\"message\":{\"role\":\"assistant\",\"stopReason\":\"stop\"}}\n",
        "{\"type\":\"agent_end\",\"messages\":[{\"role\":\"assistant\",\"stopReason\":\"stop\"}]}\n"
    )
    .as_bytes();
    super::super::artifacts::write_once(&dir.join("raw-events.jsonl"), raw).expect("raw events");
    let normalized = crate::candidate_worker::omp::normalize_jsonl(raw, 4096, "/workspace");
    super::super::artifacts::write_once(
        &dir.join("omp-observation.json"),
        &serde_json::to_vec(&normalized).expect("normalized"),
    )
    .expect("observation");
    let store = crate::store::Store::new(dir.join("candidate.redb")).expect("native store");
    store.save_candidate_receipt(&f.receipt).expect("native receipt");
    let result = crate::candidate_worker::AttemptResult {
        schema: crate::candidate_worker::CANDIDATE_ATTEMPT_RESULT_SCHEMA.into(),
        disposition: crate::candidate::CandidateDisposition::Produced,
        receipt: Some(f.receipt.clone()),
        published_artifacts: f.receipt.changed_paths.clone(),
        stdout_blake3: blake3::hash(raw).to_hex().to_string(),
        stderr_blake3: blake3::hash(b"").to_hex().to_string(),
        blocker: None,
        compatibility: None,
        process_failure: None,
        observed_artifacts: BTreeMap::new(),
    };
    store.save_candidate_attempt_result(&f.receipt.attempt_id, &result).expect("native result");
    let candidate_map = crate::candidate_capture::workspace(&f.workspace).expect("captured candidate");
    for path in &f.receipt.changed_paths {
        store
            .save_candidate_artifact(
                crate::store::ArtifactKey {
                    attempt_id: &f.receipt.attempt_id,
                    relative_path: path,
                },
                candidate_map.get(path).expect("changed path present"),
            )
            .expect("native candidate artifact");
    }
}

#[test]
fn exports_actual_retained_candidate_and_replays_exactly() {
    let f = fixture();
    retain_completed(&f);
    run(&f.request, &f.state, &f.output).expect("export");
    let root = f.output.join("attempt-export-7");
    assert_eq!(std::fs::read(root.join("source.txt")).expect("candidate file"), b"candidate");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&root).expect("root metadata").permissions().mode() & 0o777, 0o750);
        assert_eq!(
            std::fs::metadata(root.join("source.txt")).expect("file metadata").permissions().mode() & 0o777,
            0o640
        );
    }
    assert_eq!(
        crate::candidate::CandidateReceipt::from_json(
            &std::fs::read_to_string(root.join("host-receipt.json")).expect("receipt")
        )
        .expect("validated receipt"),
        f.receipt
    );
    assert_eq!(execute(&f.request, &f.state, &f.output).expect("exact replay")["accepted"], false);
}

#[test]
fn refuses_missing_native_result_and_substituted_output() {
    let f = fixture();
    assert!(run(&f.request, &f.state, &f.output).is_err());
    retain_completed(&f);
    run(&f.request, &f.state, &f.output).expect("initial export");
    std::fs::write(f.output.join("attempt-export-7/source.txt"), b"substituted").expect("substitute");
    assert!(run(&f.request, &f.state, &f.output).is_err());
    #[cfg(unix)]
    {
        let file = f.output.join("attempt-export-7/source.txt");
        std::fs::remove_file(&file).expect("remove substituted output");
        std::os::unix::fs::symlink(f.workspace.join("source.txt"), &file).expect("symlink substituted output");
        assert!(run(&f.request, &f.state, &f.output).is_err());
    }
}

#[test]
fn nested_publication_replays_exactly_and_rejects_extra_or_missing_members() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("publication root");
    let root = directory.path().join("candidate");
    let payload = super::content::Payload {
        snapshot: BTreeMap::from([
            ("a/b/run.sh".into(), b"candidate".to_vec()),
            ("empty".into(), Vec::new()),
        ]),
        modes: BTreeMap::from([("a/b/run.sh".into(), 0o755)]),
        receipt: b"receipt".to_vec(),
    };
    let observation = serde_json::json!({"retained": true});
    super::publication::write(&root, &payload, &observation).expect("nested publication");
    super::publication::write(&root, &payload, &observation).expect("exact replay");
    assert_eq!(std::fs::read(root.join("a/b/run.sh")).expect("nested bytes"), b"candidate");
    assert_eq!(std::fs::metadata(root.join("a/b/run.sh")).expect("mode").permissions().mode() & 0o777, 0o751);
    assert_eq!(std::fs::read(root.join("empty")).expect("empty file"), b"");
    std::fs::create_dir(root.join("extra")).expect("unexpected directory");
    std::fs::set_permissions(root.join("extra"), std::fs::Permissions::from_mode(0o750)).expect("directory mode");
    assert!(super::publication::write(&root, &payload, &observation).is_err());
    std::fs::remove_dir(root.join("extra")).expect("remove unexpected directory");
    std::fs::remove_file(root.join("a/b/run.sh")).expect("remove expected member");
    assert!(super::publication::write(&root, &payload, &observation).is_err());
}

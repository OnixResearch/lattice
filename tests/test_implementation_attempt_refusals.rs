//! Typed refusals of the `lattice implementation-attempt` binary state whether
//! the refused attempt can have executed: `dispatched` is `true` whenever its
//! directory holds an execution intent, and `false` only when no worker effect
//! of that attempt can have started.

// r[verify lattice.implementation_attempt.consumer.scenario.refusal]

use std::os::unix::fs::DirBuilderExt;

use lattice::implementation_attempt::cli::Request;

fn request(attempt_id: &str) -> Request {
    let envelope = lattice::implementation_attempt::ImplementationAttemptEnvelope {
        schema: lattice::implementation_attempt::IMPLEMENTATION_ATTEMPT_ENVELOPE_SCHEMA.into(),
        work_id: "work-6".into(),
        work_version: 1,
        source_kind: lattice::implementation_attempt::SourceKind::CairnTask,
        source_id: "task-6".into(),
        repository: "OnixResearch/lattice".into(),
        base: "a".repeat(40),
        readiness_snapshot: "b".repeat(64),
        task_spec_digest: "c".repeat(64),
        verification_profile: "f".repeat(64),
        attempt_id: attempt_id.into(),
        fence: 7,
        workspace_id: "workspace-6".into(),
        worker_profile: String::new(),
        budget: lattice::implementation_attempt::AttemptBudget {
            max_turns: 1,
            max_seconds: 60,
            max_cost_micros: 1,
        },
        policy_digest: "d".repeat(64),
    };
    let mut request = Request {
        schema: lattice::implementation_attempt::cli::REQUEST_SCHEMA.into(),
        admitted_envelope: String::new(),
        envelope,
        expires_at_ms: u64::MAX,
        workspace_root: "/workspace".into(),
        read_paths: vec!["src".into()],
        write_paths: vec!["src".into()],
        program: "/usr/bin/worker".into(),
        sandbox_program: "/usr/bin/bwrap".into(),
        omp: lattice::candidate_worker::omp::Profile {
            revision: lattice::candidate_worker::omp::OMP_REVISION.into(),
            executable_blake3: "e".repeat(64),
            base_identity: "0".repeat(64),
            model: "lattice/model".into(),
            prompt: "make the candidate".into(),
            model_broker: lattice::candidate_worker::broker::Profile {
                schema: lattice::candidate_worker::broker::SCHEMA.into(),
                endpoint: "https://model.invalid/v1/chat/completions".into(),
                model: "model".into(),
                credential: Some(lattice::credentials::SecretRef::new("env", "MODEL_TOKEN", None)),
                relay: lattice::candidate_worker::broker::Executable {
                    path: "/nix/store/relay".into(),
                    blake3: "a".repeat(64),
                },
                shell: lattice::candidate_worker::broker::Executable {
                    path: "/nix/store/shell".into(),
                    blake3: "b".repeat(64),
                },
                sandbox_blake3: "c".repeat(64),
                expires_at_ms: u64::MAX,
                max_requests: 1,
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
    };
    request.envelope.worker_profile = request.worker_profile_identity().expect("revision");
    request.admitted_envelope = request.envelope.identity();
    request
}

/// A request retained under an earlier profile framing.
fn superseded(attempt_id: &str) -> Request {
    let mut request = request(attempt_id);
    request.envelope.worker_profile = "e".repeat(64);
    request.admitted_envelope = request.envelope.identity();
    request
}

struct Fixture {
    _root: tempfile::TempDir,
    root: std::path::PathBuf,
    state: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("scratch");
        let canonical = root.path().canonicalize().expect("canonical scratch");
        let state = canonical.join("state");
        std::fs::DirBuilder::new().mode(0o700).create(&state).expect("private state");
        Self {
            _root: root,
            root: canonical,
            state,
        }
    }

    /// Write `request` and an intent with `content(identity)` into its attempt
    /// directory; returns the request path.
    fn retained(&self, name: &str, request: &Request, content: impl FnOnce(&str) -> String) -> std::path::PathBuf {
        let bytes = serde_json::to_vec(request).expect("request bytes");
        let path = self.root.join(name);
        std::fs::write(&path, &bytes).expect("request");
        let key = self.state.join(format!("{}-{}", request.envelope.attempt_id, request.envelope.fence));
        let directory = self.state.join(blake3::hash(key.as_os_str().as_encoded_bytes()).to_hex().as_str());
        std::fs::DirBuilder::new().mode(0o700).create(&directory).expect("attempt directory");
        let identity = blake3::hash(&bytes).to_hex().to_string();
        std::fs::write(directory.join("intent.json"), content(&identity)).expect("intent");
        path
    }

    /// Run `lattice implementation-attempt COMMAND` and return its exit status
    /// and printed refusal record.
    fn refusal(&self, command: &str, request: &std::path::Path) -> (bool, serde_json::Value) {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_lattice"))
            .args(["implementation-attempt", command, "--request"])
            .arg(request)
            .arg("--state")
            .arg(&self.state)
            .output()
            .expect("lattice runs");
        let stdout = String::from_utf8(output.stdout).expect("UTF-8 output");
        let line = stdout.lines().next().expect("one refusal record");
        (output.status.success(), serde_json::from_str(line).expect("refusal record"))
    }
}

#[test]
fn a_superseded_attempt_with_a_retained_intent_may_have_executed() {
    let fixture = Fixture::new();
    let request = superseded("attempt-superseded");
    let path = fixture.retained("superseded.json", &request, str::to_owned);
    for command in ["run", "recover"] {
        let (is_success, record) = fixture.refusal(command, &path);
        assert!(!is_success, "{command} exits non-zero");
        assert_eq!(record["schema"], "lattice.implementation-attempt-refusal.v1");
        assert_eq!(record["reason"], "profile_revision_superseded", "{command}");
        assert_eq!(record["dispatched"], true, "{command}: the retained intent may have executed");
        assert_eq!(record["attempt_id"], "attempt-superseded");
    }
}

#[test]
fn a_superseded_attempt_closed_before_execution_never_dispatched() {
    let fixture = Fixture::new();
    let request = superseded("attempt-closed");
    let path = fixture.retained("closed.json", &request, |identity| format!("closed-before-execution:{identity}"));
    for command in ["run", "recover"] {
        let (is_success, record) = fixture.refusal(command, &path);
        assert!(!is_success, "{command} exits non-zero");
        assert_eq!(record["reason"], "profile_revision_superseded", "{command}");
        assert_eq!(record["dispatched"], false, "{command}: only the closure marker is retained");
    }
    let fresh = superseded("attempt-without-directory");
    let bytes = serde_json::to_vec(&fresh).expect("request bytes");
    let fresh_path = fixture.root.join("fresh.json");
    std::fs::write(&fresh_path, bytes).expect("request");
    let (_, record) = fixture.refusal("run", &fresh_path);
    assert_eq!(record["dispatched"], false, "no attempt directory, no intent");
}

#[test]
fn a_substituted_request_reports_that_the_retained_intent_may_have_executed() {
    let fixture = Fixture::new();
    let original = request("attempt-substituted");
    let original_bytes = serde_json::to_vec(&original).expect("request bytes");
    let mut substituted = original.clone();
    substituted.omp.prompt = "a different task under the same attempt".into();
    let path =
        fixture.retained("substituted.json", &substituted, |_| blake3::hash(&original_bytes).to_hex().to_string());
    for command in ["run", "recover"] {
        let (is_success, record) = fixture.refusal(command, &path);
        assert!(!is_success, "{command} exits non-zero");
        assert_eq!(record["reason"], "substituted_request", "{command}");
        assert_eq!(record["dispatched"], true, "{command}: another request's intent may have executed");
        assert_eq!(record["attempt_id"], "attempt-substituted");
    }
}

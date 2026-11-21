//! Coordinator transport fixtures over the real run/recover/cancel shell.
//! No worker process is started: every case stops before dispatch.

// r[verify lattice.implementation_attempt.consumer]
// r[verify lattice.implementation_attempt.consumer.scenario.recovery]
// r[verify lattice.implementation_attempt.consumer.scenario.refusal]

struct Fixture {
    _root: tempfile::TempDir,
    root: std::path::PathBuf,
    state: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("scratch");
        let canonical = root.path().canonicalize().expect("canonical scratch");
        std::fs::create_dir_all(canonical.join("workspace/src")).expect("workspace");
        Self {
            state: canonical.join("state"),
            root: canonical,
            _root: root,
        }
    }

    fn request(&self, attempt_id: &str, fence: u64) -> super::super::Request {
        let mut request = super::request(
            attempt_id,
            lattice_core::execution_binding::request::ComputeRequirement::HostLocal {},
            lattice_core::execution_binding::request::ComputeDeclaration::HostLocal {},
        );
        request.workspace_root = self.root.join("workspace");
        request.envelope.fence = fence;
        request.admitted_envelope = request.envelope.identity();
        request
    }

    fn write(&self, name: &str, request: &super::super::Request) -> (std::path::PathBuf, String) {
        let bytes = serde_json::to_vec(request).expect("request bytes");
        let path = self.root.join(name);
        std::fs::write(&path, &bytes).expect("request file");
        (path, blake3::hash(&bytes).to_hex().to_string())
    }

    fn directory(&self, request: &super::super::Request) -> std::path::PathBuf {
        super::super::artifacts::directory(&self.state, request)
    }
}

async fn run(path: &std::path::Path, state: &std::path::Path) -> Result<(), String> {
    super::super::execution::run(
        path,
        state,
        Some(super::super::Ports {
            compute: &mut crate::execution_binding::UnavailableComputePort,
            serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
        }),
        super::super::session::revision::Store(None),
    )
    .await
}

async fn recover(path: &std::path::Path, state: &std::path::Path) -> Result<(), String> {
    super::super::execution::run(path, state, None, super::super::session::revision::Store(None)).await
}

fn assert_never_bound(directory: &std::path::Path) {
    for name in [
        super::super::binding::ADMITTED_FILE,
        "candidate.redb",
        "raw-events.jsonl",
        "omp-observation.json",
    ] {
        assert!(!directory.join(name).exists(), "{name} must not exist");
    }
}

#[tokio::test]
async fn recovery_without_intent_closes_the_attempt_so_no_run_can_dispatch() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-closed", 7);
    let (path, identity) = fixture.write("request.json", &request);
    recover(&path, &fixture.state).await.expect("controller loss before intent reconciles");
    let directory = fixture.directory(&request);
    let marker = super::super::recovery::closed_marker(&identity);
    assert_eq!(std::fs::read(directory.join("intent.json")).expect("closed intent"), marker.as_bytes());
    recover(&path, &fixture.state).await.expect("exact replay reports the same closure");
    run(&path, &fixture.state).await.expect("a duplicate dispatch reconciles instead of running");
    assert_never_bound(&directory);
    super::super::cancellation::request(&path, &fixture.state).expect("cancelling a closed attempt is a no-op");
    assert!(!directory.join("cancel.request").exists());
    let observation = super::super::recovery::not_started(&request);
    assert_eq!(observation["status"], "not_started_closed_by_recovery");
    assert_eq!(observation["envelope"], serde_json::to_value(&request.envelope).expect("envelope"));
    assert_eq!(observation["request_identity"], request.admitted_envelope.as_str());
    assert!(observation["result"].is_null());
}

#[tokio::test]
async fn substituted_request_bytes_are_refused_without_touching_the_intent() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-substituted", 7);
    let (_, identity) = fixture.write("request.json", &request);
    let directory = fixture.directory(&request);
    super::super::artifacts::private_directory(&fixture.state).expect("state");
    super::super::artifacts::private_directory(&directory).expect("attempt directory");
    super::super::artifacts::write_once(&directory.join("intent.json"), identity.as_bytes()).expect("intent");
    let mut substituted = request.clone();
    substituted.omp.prompt = "a different task under the same attempt".into();
    assert_eq!(substituted.worker_profile_identity().expect("revision"), request.envelope.worker_profile);
    let (path, _) = fixture.write("substituted.json", &substituted);
    for error in [recover(&path, &fixture.state).await, run(&path, &fixture.state).await] {
        assert_eq!(error, Err(super::super::refusal::Reason::SubstitutedRequest.message().to_owned()));
    }
    assert_eq!(std::fs::read(directory.join("intent.json")).expect("intent"), identity.as_bytes());
    assert_never_bound(&directory);
}

#[tokio::test]
async fn stale_fences_and_crossed_attempts_are_refused_before_any_effect() {
    let fixture = Fixture::new();
    super::super::artifacts::private_directory(&fixture.state).expect("state");
    let successor = fixture.request("attempt-successor", 8);
    super::super::session::fencing::admit(&fixture.state, &successor.envelope).expect("successor recorded");
    let stale = fixture.request("attempt-stale", 7);
    let (path, _) = fixture.write("stale.json", &stale);
    let error = run(&path, &fixture.state).await;
    assert_eq!(error, Err(super::super::refusal::Reason::StaleFence.message().to_owned()));
    assert_never_bound(&fixture.directory(&stale));
    assert!(!fixture.directory(&stale).join("intent.json").exists());

    let recorded = fixture.request("attempt-recorded", 9);
    super::super::session::fencing::admit(&fixture.state, &recorded.envelope).expect("recorded");
    let crossed = fixture.request("attempt-crossed", 9);
    let (path, _) = fixture.write("crossed.json", &crossed);
    let error = run(&path, &fixture.state).await;
    assert_eq!(error, Err(super::super::refusal::Reason::CrossedAttempt.message().to_owned()));
    assert_eq!(recover(&path, &fixture.state).await, error, "recovery refuses the crossed attempt too");
    assert_never_bound(&fixture.directory(&crossed));
    assert!(!fixture.directory(&crossed).join("intent.json").exists());
}

#[tokio::test]
async fn attempts_retained_under_the_superseded_profile_framing_never_recover_or_run() {
    let fixture = Fixture::new();
    let mut legacy = fixture.request("attempt-legacy", 7);
    legacy.envelope.worker_profile = "e".repeat(64);
    legacy.admitted_envelope = legacy.envelope.identity();
    let (path, identity) = fixture.write("legacy.json", &legacy);
    let directory = fixture.directory(&legacy);
    super::super::artifacts::private_directory(&fixture.state).expect("state");
    super::super::artifacts::private_directory(&directory).expect("attempt directory");
    super::super::artifacts::write_once(&directory.join("intent.json"), identity.as_bytes()).expect("retained intent");
    let refused = Err(super::super::refusal::Reason::ProfileRevisionSuperseded.message().to_owned());
    assert_eq!(recover(&path, &fixture.state).await, refused);
    assert_eq!(run(&path, &fixture.state).await, refused);
    assert_eq!(std::fs::read(directory.join("intent.json")).expect("intent"), identity.as_bytes());
    assert_never_bound(&directory);
}

#[test]
fn a_worker_blocked_before_any_event_stays_recoverable_without_an_invented_terminal() {
    let root = tempfile::tempdir().expect("attempt directory");
    let directory = root.path();
    let blocker = serde_json::to_vec(&serde_json::json!({"Err": "private model socket is unavailable"})).expect("json");
    super::super::artifacts::write_once(&directory.join("raw-events.jsonl"), b"").expect("raw");
    super::super::artifacts::write_once(&directory.join("omp-observation.json"), &blocker).expect("observation");
    let empty = blake3::hash(b"").to_hex().to_string();
    super::super::artifacts::verify_events(directory, 4096, &empty).expect("blocker retained for empty output");
    let other = tempfile::tempdir().expect("attempt directory");
    let raw = b"{\"type\":\"session\",\"version\":3,\"id\":\"s\",\"cwd\":\"/workspace\"}\n";
    super::super::artifacts::write_once(&other.path().join("raw-events.jsonl"), raw).expect("raw");
    super::super::artifacts::write_once(&other.path().join("omp-observation.json"), &blocker).expect("observation");
    let digest = blake3::hash(raw).to_hex().to_string();
    assert!(
        super::super::artifacts::verify_events(other.path(), 4096, &digest).is_err(),
        "a blocker cannot replace the normalization of actual events"
    );
}

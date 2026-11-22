//! Bounded same-attempt repair over the real run/recover shell with a
//! fixture worker port, a fixture host check port and a fixture clock.
//! No OMP process, sandbox or model endpoint is involved.

// r[verify lattice.implementation_attempt.repair]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
// r[verify lattice.implementation_attempt.repair.scenario.bypass]
// r[verify lattice.implementation_attempt.repair.scenario.kiln_owned]
// r[verify lattice.implementation_attempt.repair.scenario.absent]
// r[verify lattice.implementation_attempt.lineage.scenario.exhausted]
// r[verify lattice.implementation_attempt.lineage.scenario.usage]

use crate::implementation_attempt::repair::checks::Invocation;
use crate::implementation_attempt::repair::checks::Observation;
use crate::implementation_attempt::repair::checks::Outcome;
use crate::implementation_attempt::repair::step::Report as StepReport;
use crate::implementation_attempt::repair::step::Request as StepRequest;
use crate::implementation_attempt::repair::step::WorkerOutcome;

mod export;
mod forged;
mod operator;

const EXECUTOR: &str = "7777777777777777777777777777777777777777777777777777777777777777";
const UNIT_CONFIG: &[u8] = b"unit-config";
const START_EPOCH_MS: i64 = 1_000_000;

type Script<'a> = Box<dyn FnMut(&StepRequest, &std::path::Path, &dyn Fn() -> bool) -> StepReport + 'a>;

/// Replays one scripted step per call and records every request and deadline.
struct FixtureWorker<'a> {
    script: std::collections::VecDeque<Script<'a>>,
    requests: Vec<StepRequest>,
    deadlines: Vec<std::time::Duration>,
}

impl<'a> FixtureWorker<'a> {
    fn new(script: Vec<Script<'a>>) -> Self {
        Self {
            script: script.into(),
            requests: Vec::new(),
            deadlines: Vec::new(),
        }
    }
}

impl super::super::repair::WorkerPort for FixtureWorker<'_> {
    fn step(
        &mut self,
        request: &StepRequest,
        workspace: &std::path::Path,
        deadline: std::time::Duration,
        is_cancel_requested: &dyn Fn() -> bool,
    ) -> StepReport {
        self.requests.push(request.clone());
        self.deadlines.push(deadline);
        let mut step = self.script.pop_front().expect("a scripted step for every spawn");
        step(request, workspace, is_cancel_requested)
    }
}

/// One check's view of its candidate copy.
struct Seen {
    candidate: std::path::PathBuf,
    is_read_only: bool,
    deadline: std::time::Duration,
}

/// Passes a check exactly when `src/lib.txt` reads `fixed`, and reports the
/// configuration it actually read from `checks/unit.conf`.
struct FixtureChecks {
    executor: String,
    forced: Option<Outcome>,
    /// A formatter-style `fmt` check that rewrites its candidate to `fixed`.
    is_rewriting: bool,
    /// A slow check that advances the clock by this much.
    slow: Option<(FixtureClock, i64)>,
    /// A check that only changes the mode of this path of its copy.
    chmod: Option<(&'static str, u32)>,
    invocations: Vec<Invocation>,
    seen: Vec<Seen>,
}

impl FixtureChecks {
    fn admitted() -> Self {
        Self {
            executor: EXECUTOR.into(),
            forced: None,
            is_rewriting: false,
            slow: None,
            chmod: None,
            invocations: Vec::new(),
            seen: Vec::new(),
        }
    }
}

impl super::super::repair::CheckPort for FixtureChecks {
    fn run(
        &mut self,
        invocation: &Invocation,
        candidate: &std::path::Path,
        deadline: std::time::Duration,
    ) -> Observation {
        self.invocations.push(invocation.clone());
        let source_path = candidate.join("src/lib.txt");
        let is_read_only = std::fs::metadata(&source_path).expect("candidate source").permissions().readonly();
        self.seen.push(Seen {
            candidate: candidate.to_path_buf(),
            is_read_only,
            deadline,
        });
        if self.is_rewriting && invocation.check == "fmt" {
            make_writable(candidate);
            std::fs::write(&source_path, b"fixed").expect("the formatter rewrites the candidate");
        }
        if let Some((path, mode)) = self.chmod {
            use std::os::unix::fs::PermissionsExt;
            let target = candidate.join(path);
            std::fs::set_permissions(&target, std::fs::Permissions::from_mode(mode)).expect("chmod the copy");
        }
        if let Some((clock, delay_ms)) = &self.slow {
            clock.advance(*delay_ms);
        }
        let source = std::fs::read(&source_path).expect("candidate source");
        let configuration = std::fs::read(candidate.join("checks/unit.conf")).expect("check configuration");
        let outcome = self.forced.clone().unwrap_or(if source == b"fixed" {
            Outcome::Passed
        } else {
            Outcome::Failed { class: "test".into() }
        });
        Observation {
            check: invocation.check.clone(),
            configuration: blake3::hash(&configuration).to_hex().to_string(),
            revision: invocation.revision.clone(),
            executor: self.executor.clone(),
            outcome,
        }
    }
}

/// What a misbehaving check would do to get write access to its copy.
fn make_writable(candidate: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    for path in [
        candidate.to_path_buf(),
        candidate.join("src"),
        candidate.join("src/lib.txt"),
    ] {
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).expect("chmod");
    }
}

/// What happens to a fixture clock at one point of a run.
#[derive(Debug, Clone, Copy)]
enum Shift {
    /// The clock jumps this far ahead.
    Advance(i64),
    /// The clock becomes unreadable.
    Fail,
}

/// Advances one second per observation and counts them; scripts may move it
/// further or make it unreadable. With `on_copy`, every observation while a
/// check copy exists in that records directory applies the shift first.
#[derive(Clone)]
struct FixtureClock {
    now: std::sync::Arc<std::sync::atomic::AtomicI64>,
    observations: std::sync::Arc<std::sync::atomic::AtomicU64>,
    on_copy: Option<(std::path::PathBuf, Shift)>,
}

impl FixtureClock {
    fn new() -> Self {
        Self {
            now: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(START_EPOCH_MS)),
            observations: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            on_copy: None,
        }
    }

    fn advance(&self, delay_ms: i64) {
        self.now.fetch_add(delay_ms, std::sync::atomic::Ordering::SeqCst);
    }

    /// Every later observation is before the epoch, which the shell cannot read.
    fn fail(&self) {
        self.now.store(-START_EPOCH_MS, std::sync::atomic::Ordering::SeqCst);
    }

    fn shift(&self, shift: Shift) {
        match shift {
            Shift::Advance(delay_ms) => self.advance(delay_ms),
            Shift::Fail => self.fail(),
        }
    }

    fn observations(&self) -> u64 {
        self.observations.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn is_copy_present(records: &std::path::Path) -> bool {
        std::fs::read_dir(records).is_ok_and(|entries| {
            entries.flatten().any(|entry| entry.file_name().to_string_lossy().starts_with("check-"))
        })
    }
}

impl crate::operation::shell::Clock for FixtureClock {
    fn observe_epoch_millis(&self) -> i64 {
        if let Some((records, shift)) = &self.on_copy
            && Self::is_copy_present(records)
        {
            self.shift(*shift);
        }
        self.observations.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.now.fetch_add(1_000, std::sync::atomic::Ordering::SeqCst)
    }
}

fn write(text: &'static str) -> Script<'static> {
    Box::new(move |_, workspace, _| {
        std::fs::write(workspace.join("src/lib.txt"), text).expect("edit");
        exited()
    })
}

const fn exited() -> StepReport {
    StepReport {
        outcome: WorkerOutcome::Exited { code: 0 },
        model_requests: None,
        tool_invocations: None,
        proposal: None,
    }
}

fn policy() -> crate::implementation_attempt::repair::Policy {
    crate::implementation_attempt::repair::Policy {
        schema: crate::implementation_attempt::repair::POLICY_SCHEMA.into(),
        failure_classes: vec!["test".into()],
        max_iterations: 2,
        max_elapsed_ms: 60_000,
        max_model_requests: 3,
        max_tool_invocations: 6,
        step_tool_invocations: 1,
        check_executor: EXECUTOR.into(),
        checks: vec![crate::implementation_attempt::repair::Check {
            name: "unit".into(),
            configuration: blake3::hash(UNIT_CONFIG).to_hex().to_string(),
        }],
    }
}

/// [`policy`] with a formatter check before the unit check.
fn formatted_policy() -> crate::implementation_attempt::repair::Policy {
    let mut formatted = policy();
    formatted.checks.insert(0, crate::implementation_attempt::repair::Check {
        name: "fmt".into(),
        configuration: blake3::hash(UNIT_CONFIG).to_hex().to_string(),
    });
    formatted
}

/// The broken source and check configuration of every fixture workspace.
fn seed(workspace: &std::path::Path) {
    std::fs::create_dir_all(workspace.join("src")).expect("workspace");
    std::fs::create_dir_all(workspace.join("checks")).expect("checks");
    std::fs::write(workspace.join("src/lib.txt"), b"broken").expect("source");
    std::fs::write(workspace.join("checks/unit.conf"), UNIT_CONFIG).expect("check configuration");
}

/// Bind `request` to `workspace`, `policy` and `lineage` and re-admit it as
/// a dispatcher would export it.
fn readmit(
    mut request: super::super::Request,
    workspace: &std::path::Path,
    policy: Option<crate::implementation_attempt::repair::Policy>,
    lineage: Option<crate::implementation_attempt::lineage::Binding>,
) -> super::super::Request {
    request.envelope.budget.max_turns = 4;
    request.workspace_root = workspace.to_path_buf();
    let base = crate::candidate_capture::workspace(workspace).expect("base");
    request.omp.base_identity = crate::candidate::snapshot_identity(&base).expect("base identity");
    request.repair = policy;
    request.lineage = lineage;
    request.envelope.worker_profile = request.worker_profile_identity().expect("revision");
    request.admitted_envelope = request.envelope.identity();
    request
}

struct Fixture {
    _root: tempfile::TempDir,
    root: std::path::PathBuf,
    state: std::path::PathBuf,
    workspace: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("scratch");
        let canonical = root.path().canonicalize().expect("canonical scratch");
        let workspace = canonical.join("workspace");
        seed(&workspace);
        Self {
            state: canonical.join("state"),
            workspace,
            root: canonical,
            _root: root,
        }
    }

    /// A host-local request of the current workspace, re-admitted with
    /// `policy` and `lineage` as a dispatcher would export it.
    fn request(
        &self,
        attempt_id: &str,
        policy: Option<crate::implementation_attempt::repair::Policy>,
        lineage: Option<crate::implementation_attempt::lineage::Binding>,
    ) -> super::super::Request {
        let mut request = super::host_local();
        request.envelope.attempt_id = attempt_id.into();
        // One work item per attempt: attempts of one work share its fence record.
        request.envelope.work_id = format!("work-{attempt_id}");
        readmit(request, &self.workspace, policy, lineage)
    }

    fn write(&self, request: &super::super::Request) -> std::path::PathBuf {
        let path = self.root.join(format!("{}.json", request.envelope.attempt_id));
        std::fs::write(&path, serde_json::to_vec(request).expect("request bytes")).expect("request file");
        path
    }

    fn directory(&self, request: &super::super::Request) -> std::path::PathBuf {
        super::super::artifacts::directory(&self.state, request)
    }

    fn disposition(&self, request: &super::super::Request) -> serde_json::Value {
        disposition(&self.directory(request))
    }
}

fn disposition(directory: &std::path::Path) -> serde_json::Value {
    let path = directory.join("repair/disposition.json");
    serde_json::from_slice(&std::fs::read(path).expect("retained disposition")).expect("disposition json")
}

/// Everything but the repair ports of one run.
struct Host<'a> {
    clock: FixtureClock,
    compute: &'a mut dyn crate::execution_binding::ComputeAllocationPort,
    revisions: super::super::session::revision::Store<'a>,
}

async fn run_with(
    path: &std::path::Path,
    state: &std::path::Path,
    worker: &mut FixtureWorker<'_>,
    checks: &mut FixtureChecks,
    host: Host<'_>,
) -> Result<(), String> {
    super::super::execution::run(
        path,
        state,
        Some(super::super::Ports {
            compute: host.compute,
            serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
            repair: Some(super::super::repair::Ports {
                worker,
                checks,
                clock: &host.clock,
            }),
        }),
        host.revisions,
    )
    .await
}

async fn run(
    path: &std::path::Path,
    state: &std::path::Path,
    worker: &mut FixtureWorker<'_>,
    checks: &mut FixtureChecks,
) -> Result<(), String> {
    let host = Host {
        clock: FixtureClock::new(),
        compute: &mut crate::execution_binding::UnavailableComputePort,
        revisions: super::super::session::revision::Store(None),
    };
    run_with(path, state, worker, checks, host).await
}

async fn recover(path: &std::path::Path, state: &std::path::Path) -> Result<(), String> {
    super::super::execution::run(path, state, None, super::super::session::revision::Store(None)).await
}

/// Run `script` with admitted checks; returns the retained disposition,
/// the worker's requests and the check invocations.
async fn scenario(
    fixture: &Fixture,
    request: &super::super::Request,
    script: Vec<Script<'_>>,
    checks: FixtureChecks,
) -> (serde_json::Value, Vec<StepRequest>, Vec<Invocation>) {
    let path = fixture.write(request);
    let mut worker = FixtureWorker::new(script);
    let mut checks = checks;
    run(&path, &fixture.state, &mut worker, &mut checks).await.expect("the loop reaches a disposition");
    assert!(worker.script.is_empty(), "every scripted step ran");
    (fixture.disposition(request), worker.requests, checks.invocations)
}

#[tokio::test]
async fn a_first_candidate_that_passes_is_correct_and_recovery_never_reruns() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-correct", Some(policy()), None);
    let (observation, requests, invocations) =
        scenario(&fixture, &request, vec![write("fixed")], FixtureChecks::admitted()).await;
    assert_eq!(observation["disposition"]["kind"], "correct_candidate");
    assert_eq!(observation["steps"], 1);
    assert_eq!((observation["accepted"].clone(), observation["integrated"].clone()), (false.into(), false.into()));
    let revision = observation["revisions"][0]["revision"].as_str().expect("revision").to_owned();
    assert_eq!(observation["disposition"]["revision"], revision.as_str());
    assert_eq!(invocations[0].revision, revision, "the check is bound to the exact revision");
    let receipt = &observation["revisions"][0]["receipt"];
    assert_eq!((receipt["attempt_id"].clone(), receipt["fence"].clone()), ("attempt-correct".into(), "7".into()));
    assert_eq!(receipt["changed_paths"], serde_json::json!(["src/lib.txt"]));
    assert_eq!(requests[0].phase, crate::implementation_attempt::repair::step::Phase::Edit);
    let phases: Vec<_> = observation["usage"].as_array().expect("usage").iter().map(|u| u["phase"].clone()).collect();
    assert_eq!(phases, ["preparation", "edit", "check"]);
    assert_eq!(observation["usage"][1]["provenance"], "unknown", "unknown worker usage stays unknown");
    assert_eq!(observation["repair_policy"], policy().identity().as_str());
    let directory = fixture.directory(&request);
    for name in ["repair/step-1.intent", "repair/step-1.json", "intent.json"] {
        assert!(directory.join(name).exists(), "{name}");
    }
    let store = crate::store::Store::new(directory.join("candidate.redb")).expect("store");
    let usage = store.load_command_worker_usage("attempt-correct").expect("load").expect("attempt usage retained");
    usage.validate().expect("existing attempt usage provenance");
    assert_eq!(usage.scope, crate::orchestration::Scope::Attempt);
    drop(store);
    let path = fixture.write(&request);
    recover(&path, &fixture.state).await.expect("recovery reports the retained disposition");
    let inventory = super::super::inspect::project(
        &path,
        &fixture.state,
        &mut crate::execution_binding::serving::UnavailableOwner,
        super::super::session::revision::Store(None),
    )
    .expect("inventory");
    // r[verify lattice.implementation_attempt.repair_export.scenario.inventory]
    // Inventory v3 reports the retained disposition as the terminal result.
    let instance = &inventory["instance"];
    assert_eq!(
        instance["lifecycle"],
        serde_json::json!({"state": "terminal_observed", "disposition": "correct_candidate"})
    );
    assert_eq!(
        (instance["liveness"]["state"].clone(), instance["liveness"]["unknown_cause"].clone()),
        ("exited".into(), serde_json::Value::Null)
    );
    assert_eq!(instance["wait_reasons"], serde_json::json!([]), "{inventory}");
    let mut idle = FixtureWorker::new(Vec::new());
    let mut checks = FixtureChecks::admitted();
    run(&path, &fixture.state, &mut idle, &mut checks).await.expect("a duplicate run reconciles");
    assert!(idle.requests.is_empty() && checks.invocations.is_empty(), "nothing runs again");
}

#[tokio::test]
async fn a_bounded_repair_records_a_new_revision_and_rebinds_its_checks() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-repaired", Some(policy()), None);
    let script = vec![write("still broken"), write("fixed")];
    let (observation, requests, invocations) = scenario(&fixture, &request, script, FixtureChecks::admitted()).await;
    assert_eq!(observation["disposition"]["kind"], "repaired_candidate");
    assert_eq!(observation["disposition"]["repairs"], 1);
    let revisions = observation["revisions"].as_array().expect("revisions");
    assert_eq!(revisions.len(), 2);
    let first = revisions[0]["revision"].as_str().expect("first");
    assert_eq!(revisions[1]["parent"], first, "the repair names its parent revision");
    assert_ne!(revisions[1]["revision"], revisions[0]["revision"]);
    assert_eq!(revisions[1]["receipt"]["base_identity"], revisions[0]["receipt"]["base_identity"]);
    assert_eq!(requests[1].phase, crate::implementation_attempt::repair::step::Phase::Repair);
    assert_eq!(requests[1].parent.as_deref(), Some(first));
    assert_eq!(requests[1].failures, vec!["test".to_owned()], "only admitted classes, no transcript");
    assert_eq!(requests[0].binding, requests[1].binding, "the same attempt and fence");
    assert_eq!(invocations.len(), 2);
    assert_eq!(invocations[1].revision, revisions[1]["revision"].as_str().expect("second"), "checks rebound");
    assert_eq!(observation["drawn"], serde_json::json!({"model_requests": 2, "tool_invocations": 4}));
}

#[tokio::test]
async fn exhausted_iterations_and_lineage_stop_before_another_spawn() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-iterations", Some(policy()), None);
    let script = vec![write("broken 1"), write("broken 2"), write("broken 3")];
    let (observation, requests, _) = scenario(&fixture, &request, script, FixtureChecks::admitted()).await;
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "budget_exhausted", "bound": {"kind": "iterations"}})
    );
    assert_eq!(requests.len(), 3, "the first step plus two repairs");
    let lineage = crate::implementation_attempt::lineage::tests::binding(2, 100);
    let request = fixture.request("attempt-lineage", Some(policy()), Some(lineage.clone()));
    let (observation, requests, _) =
        scenario(&fixture, &request, vec![write("broken 4"), write("broken 5")], FixtureChecks::admitted()).await;
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "budget_exhausted", "bound": {"kind": "lineage", "dimension": "model_requests"}})
    );
    assert_eq!(requests.len(), 2, "every iteration draws from the one lineage account");
    assert_eq!(observation["budget_lineage"], lineage.identity().as_str());
    assert_eq!(observation["unaccounted_dimensions"], serde_json::json!([]));
}

#[tokio::test]
// r[verify lattice.implementation_attempt.consumer.scenario.fence_lock]
async fn a_newer_fence_stops_the_loop_before_its_next_step_intent() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-superseded", Some(policy()), None);
    let state = fixture.state.clone();
    let mut successor = request.envelope.clone();
    successor.attempt_id = "attempt-successor".into();
    successor.fence += 1;
    // The first step fails a repairable check while Kiln's next fence of the
    // same work is recorded in this state root.
    let first: Script<'_> = Box::new(move |_, workspace, _| {
        std::fs::write(workspace.join("src/lib.txt"), "still broken").expect("edit");
        let directory = state.join("successor");
        drop(super::super::session::fencing::admit(&state, &successor, &directory).expect("fence 8 recorded"));
        exited()
    });
    let (observation, requests, invocations) =
        scenario(&fixture, &request, vec![first], FixtureChecks::admitted()).await;
    // A post-dispatch outcome: the retained repair observation of a step that
    // ran, never a refusal record and never `dispatched:false`.
    assert_eq!(observation["schema"], "lattice.implementation-attempt-repair-observation.v2");
    assert_eq!(observation["disposition"], serde_json::json!({"kind": "fence_superseded"}));
    assert!(observation.get("dispatched").is_none() && observation.get("reason").is_none());
    assert_eq!((observation["steps"].clone(), requests.len()), (1.into(), 1), "the repair never spawned");
    assert_eq!(observation["revisions"].as_array().map(Vec::len), Some(1), "the first step's candidate");
    assert_eq!(invocations.len(), 1);
    let directory = fixture.directory(&request);
    assert!(directory.join("repair/step-1.json").exists());
    assert!(!directory.join("repair/step-2.intent").exists(), "no intent for the superseded step");
    let path = fixture.write(&request);
    recover(&path, &fixture.state)
        .await
        .expect("recovery reports the retained post-dispatch disposition");
    // A later fence-7 refusal of this attempt reports that it may have run.
    let guard =
        super::super::session::fencing::Guard::acquire(&fixture.state, &request.envelope.work_id).expect("fence lock");
    let refused = super::super::session::fencing::Refused {
        reason: super::super::refusal::Reason::StaleFence,
        dispatched: true,
    };
    assert_eq!(
        guard.record(&request.envelope, &directory),
        Err(super::super::session::fencing::Error::Refused(refused))
    );
}

#[tokio::test]
async fn cancellation_during_and_between_steps_is_its_own_disposition() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-cancelled", Some(policy()), None);
    let path = fixture.write(&request);
    let state = fixture.state.clone();
    let cancel = move |_: &StepRequest, _: &std::path::Path, is_cancel_requested: &dyn Fn() -> bool| {
        super::super::cancellation::request(&path, &state).expect("cancel the running attempt");
        assert!(is_cancel_requested(), "the worker observes the persisted request");
        StepReport {
            outcome: WorkerOutcome::Cancelled,
            ..exited()
        }
    };
    let (observation, _, invocations) =
        scenario(&fixture, &request, vec![Box::new(cancel)], FixtureChecks::admitted()).await;
    assert_eq!(observation["disposition"]["kind"], "cancelled");
    assert!(invocations.is_empty(), "a cancelled step is not checked");
    let between = fixture.request("attempt-cancelled-between", Some(policy()), None);
    let path = fixture.write(&between);
    let state = fixture.state.clone();
    let failing_then_cancelled = move |_: &StepRequest, workspace: &std::path::Path, _: &dyn Fn() -> bool| {
        std::fs::write(workspace.join("src/lib.txt"), "still broken").expect("edit");
        super::super::cancellation::request(&path, &state).expect("cancel");
        exited()
    };
    let (observation, requests, _) =
        scenario(&fixture, &between, vec![Box::new(failing_then_cancelled)], FixtureChecks::admitted()).await;
    assert_eq!(observation["disposition"]["kind"], "cancelled");
    assert_eq!(requests.len(), 1, "no repair spawns after cancellation");
}

#[tokio::test]
async fn uncertain_worker_and_check_outcomes_are_distinct_from_failure() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-uncertain-worker", Some(policy()), None);
    let unknown: Script<'_> = Box::new(|_, _, _| StepReport {
        outcome: WorkerOutcome::Unknown,
        ..exited()
    });
    let (observation, _, invocations) = scenario(&fixture, &request, vec![unknown], FixtureChecks::admitted()).await;
    assert_eq!(observation["disposition"], serde_json::json!({"kind": "uncertain", "cause": "worker_outcome"}));
    assert!(invocations.is_empty());
    let request = fixture.request("attempt-uncertain-check", Some(policy()), None);
    let checks = FixtureChecks {
        forced: Some(Outcome::Unknown),
        ..FixtureChecks::admitted()
    };
    let (observation, _, _) = scenario(&fixture, &request, vec![write("fixed")], checks).await;
    assert_eq!(observation["disposition"], serde_json::json!({"kind": "uncertain", "cause": "check_outcome"}));
}

#[tokio::test]
// r[verify lattice.implementation_attempt.repair.scenario.bypass]
async fn exit_zero_fabricated_success_skips_reconfiguration_and_self_approval_cannot_pass() {
    let fixture = Fixture::new();
    let unchanged: Script<'_> = Box::new(|_, _, _| exited());
    let request = fixture.request("attempt-exit-zero", Some(policy()), None);
    let (observation, _, invocations) = scenario(&fixture, &request, vec![unchanged], FixtureChecks::admitted()).await;
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "not_repairable", "classes": ["no_candidate"]})
    );
    assert!(invocations.is_empty());
    let claim = |_: &StepRequest, workspace: &std::path::Path, _: &dyn Fn() -> bool| {
        std::fs::write(workspace.join("src/CHECKS_PASSED"), "all required checks passed").expect("claim");
        StepReport {
            model_requests: Some(0),
            tool_invocations: Some(0),
            ..exited()
        }
    };
    let request = fixture.request("attempt-fabricated", Some(policy()), None);
    let script: Vec<Script<'_>> = vec![Box::new(claim), write("broken 1"), write("broken 2")];
    let (observation, _, _) = scenario(&fixture, &request, script, FixtureChecks::admitted()).await;
    assert_eq!(observation["disposition"]["kind"], "budget_exhausted", "a claimed pass is not a check");
    let reconfigure = |_: &StepRequest, workspace: &std::path::Path, _: &dyn Fn() -> bool| {
        std::fs::write(workspace.join("src/lib.txt"), "fixed").expect("edit");
        std::fs::write(workspace.join("checks/unit.conf"), "unit-config --skip-all").expect("reconfigure");
        exited()
    };
    let rejected =
        |defect: &str| serde_json::json!({"kind": "checks_rejected", "defect": {"code": defect, "check": "unit"}});
    let fresh = Fixture::new();
    let request = fresh.request("attempt-reconfigured", Some(policy()), None);
    let (observation, _, _) = scenario(&fresh, &request, vec![Box::new(reconfigure)], FixtureChecks::admitted()).await;
    assert_eq!(observation["disposition"], rejected("configuration_changed"));
    let cases = [
        (
            FixtureChecks {
                forced: Some(Outcome::Skipped),
                ..FixtureChecks::admitted()
            },
            "skipped_check",
        ),
        (FixtureChecks::admitted(), "self_approval"),
    ];
    for (index, (mut checks, defect)) in cases.into_iter().enumerate() {
        let fresh = Fixture::new();
        let request = fresh.request(&format!("attempt-bypass-{index}"), Some(policy()), None);
        if defect == "self_approval" {
            checks.executor = request.envelope.worker_profile.clone();
        }
        let (observation, _, _) = scenario(&fresh, &request, vec![write("fixed")], checks).await;
        assert_eq!(observation["disposition"], rejected(defect), "{defect}");
    }
}

#[tokio::test]
// r[verify lattice.implementation_attempt.repair.scenario.kiln_owned]
async fn a_proposed_change_of_work_or_new_attempt_stays_with_kiln() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-kiln-owned", Some(policy()), None);
    let propose: Script<'_> = Box::new(|_, workspace, _| {
        std::fs::write(workspace.join("src/lib.txt"), "a different task").expect("edit");
        StepReport {
            proposal: Some(crate::implementation_attempt::repair::KilnOwned::ChangedWork),
            ..exited()
        }
    });
    let (observation, requests, invocations) =
        scenario(&fixture, &request, vec![propose], FixtureChecks::admitted()).await;
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "kiln_owned", "continuation": "changed_work"})
    );
    assert_eq!(requests.len(), 1);
    assert!(invocations.is_empty(), "no repair or check follows a Kiln-owned proposal");
}

/// Every path under `root`, sorted.
fn entries(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("state directory") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path.clone());
            }
            paths.push(path);
        }
    }
    paths.sort();
    paths
}

/// The closure path a pre-intent refusal of `request` writes: the per-work
/// lock directory and the attempt directory with its closure marker.
fn closure_path(state: &std::path::Path, request: &super::super::Request) -> Vec<std::path::PathBuf> {
    let works = state.join("works");
    let directory = super::super::artifacts::directory(state, request);
    vec![
        works.join(blake3::hash(request.envelope.work_id.as_bytes()).to_hex().as_str()),
        works,
        directory.join(super::super::session::intent::FILE),
        directory,
    ]
}

/// The first line printed by a refusal is not captured here; the typed
/// reason is the returned diagnostic, and the only write is the closure
/// marker.
fn assert_closed(directory: &std::path::Path) {
    let intent = std::fs::read(directory.join(super::super::session::intent::FILE)).expect("closure marker");
    assert!(super::super::recovery::is_closed_marker(&intent), "the refusal closed the attempt-fence");
    let names = std::fs::read_dir(directory).expect("attempt directory").count();
    assert_eq!(names, 1, "the closure marker is the attempt directory's only entry");
}

#[tokio::test]
// r[verify lattice.implementation_attempt.lineage.scenario.exhausted]
// r[verify lattice.implementation_attempt.consumer.scenario.closure]
async fn lineage_and_missing_ports_refusals_write_only_the_closure_marker() {
    let fixture = Fixture::new();
    let mut overloaded = crate::implementation_attempt::lineage::tests::binding(10, 10);
    overloaded.backpressure = crate::implementation_attempt::lineage::Backpressure::Overloaded;
    let cases = [
        (
            crate::implementation_attempt::lineage::tests::binding(0, 10),
            super::super::refusal::Reason::LineageExhausted,
        ),
        (
            crate::implementation_attempt::lineage::tests::binding(1, 0),
            super::super::refusal::Reason::LineageExhausted,
        ),
        (overloaded, super::super::refusal::Reason::LineageBlocked),
    ];
    let mut expected = Vec::new();
    for (index, (lineage, reason)) in cases.into_iter().enumerate() {
        for policy in [Some(policy()), None] {
            let attempt = format!("attempt-lineage-{index}-{}", policy.is_some());
            let request = fixture.request(&attempt, policy.clone(), Some(lineage.clone()));
            let path = fixture.write(&request);
            let mut worker = FixtureWorker::new(Vec::new());
            let mut checks = FixtureChecks::admitted();
            let refused = run(&path, &fixture.state, &mut worker, &mut checks).await;
            assert_eq!(refused, Err(reason.message().to_owned()), "{attempt}");
            assert!(worker.requests.is_empty());
            assert_closed(&fixture.directory(&request));
            expected.extend(closure_path(&fixture.state, &request));
            // A valid request for the same attempt-fence can never create an
            // intent once the refusal reported `dispatched: false`.
            let valid = fixture.request(&attempt, policy, None);
            let path = fixture.write(&valid);
            let mut worker = FixtureWorker::new(vec![write("fixed")]);
            let closed = run(&path, &fixture.state, &mut worker, &mut checks).await;
            assert_eq!(closed, Err(super::super::refusal::Reason::AttemptClosed.message().to_owned()), "{attempt}");
            assert!(worker.requests.is_empty(), "{attempt}: no step starts");
            assert_closed(&fixture.directory(&request));
        }
    }
    let request = fixture.request("attempt-standalone", Some(policy()), None);
    let path = fixture.write(&request);
    let ports = super::super::Ports {
        compute: &mut crate::execution_binding::UnavailableComputePort,
        serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
        repair: None,
    };
    let refused =
        super::super::execution::run(&path, &fixture.state, Some(ports), super::super::session::revision::Store(None))
            .await;
    assert_eq!(refused, Err(super::super::refusal::Reason::RepairPortsUnavailable.message().to_owned()));
    assert_closed(&fixture.directory(&request));
    expected.extend(closure_path(&fixture.state, &request));
    expected.sort();
    expected.dedup();
    // No fence record, binding, candidate store or repair record anywhere.
    assert_eq!(entries(&fixture.state), expected, "only the closure paths exist in the state root");
    let record = super::super::refusal::record(&request.envelope, super::super::refusal::Reason::LineageExhausted);
    assert_eq!((record["reason"].clone(), record["dispatched"].clone()), ("lineage_exhausted".into(), false.into()));
    // An existing state root gains exactly the refused attempt's closure path.
    let fixture = Fixture::new();
    let admitted = fixture.request("attempt-admitted-first", Some(policy()), None);
    let path = fixture.write(&admitted);
    let mut worker = FixtureWorker::new(vec![write("fixed")]);
    let mut checks = FixtureChecks::admitted();
    run(&path, &fixture.state, &mut worker, &mut checks).await.expect("a first attempt runs");
    let before = entries(&fixture.state);
    let exhausted = crate::implementation_attempt::lineage::tests::binding(0, 10);
    let request = fixture.request("attempt-lineage-later", Some(policy()), Some(exhausted));
    let path = fixture.write(&request);
    let mut worker = FixtureWorker::new(Vec::new());
    let refused = run(&path, &fixture.state, &mut worker, &mut checks).await;
    assert_eq!(refused, Err(super::super::refusal::Reason::LineageExhausted.message().to_owned()));
    let mut expected = before;
    expected.extend(closure_path(&fixture.state, &request));
    expected.sort();
    expected.dedup();
    assert_eq!(entries(&fixture.state), expected, "only the closure marker path is new");
}

/// A panicking worker step stands in for a controller lost mid-step.
#[test]
fn a_lost_controller_without_a_disposition_is_unknown_and_never_reruns() {
    let runtime = || tokio::runtime::Builder::new_current_thread().enable_all().build().expect("runtime");
    let fixture = Fixture::new();
    let request = fixture.request("attempt-lost", Some(policy()), None);
    let path = fixture.write(&request);
    let lost: Script<'_> = Box::new(|_, _, _| panic!("controller lost during the step"));
    let mut worker = FixtureWorker::new(vec![lost]);
    let mut checks = FixtureChecks::admitted();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime().block_on(run(&path, &fixture.state, &mut worker, &mut checks))
    }));
    assert!(outcome.is_err(), "the controller was lost mid-step");
    let directory = fixture.directory(&request);
    assert!(directory.join("repair/step-1.intent").exists(), "the step intent preceded the worker effect");
    assert!(!directory.join("repair/disposition.json").exists());
    let error = runtime().block_on(recover(&path, &fixture.state)).expect_err("no retained disposition");
    assert!(error.contains("outcome unknown"), "{error}");
    let mut idle = FixtureWorker::new(Vec::new());
    let mut checks = FixtureChecks::admitted();
    assert!(runtime().block_on(run(&path, &fixture.state, &mut idle, &mut checks)).is_err());
    assert!(idle.requests.is_empty() && checks.invocations.is_empty(), "never rerun");
}

#[tokio::test]
async fn a_drifted_workspace_is_refused_before_the_first_step() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-drifted", Some(policy()), None);
    std::fs::write(fixture.workspace.join("src/lib.txt"), "drifted").expect("drift");
    let path = fixture.write(&request);
    let mut worker = FixtureWorker::new(Vec::new());
    let mut checks = FixtureChecks::admitted();
    let error = run(&path, &fixture.state, &mut worker, &mut checks).await.expect_err("drift");
    assert!(error.contains("admitted base snapshot"), "{error}");
    assert!(worker.requests.is_empty());
}

#[test]
// r[verify lattice.implementation_attempt.repair.scenario.absent]
fn absent_policy_and_lineage_keep_request_bytes_and_revision_and_a_policy_moves_the_revision() {
    let plain = super::host_local();
    let bytes = serde_json::to_vec(&plain).expect("bytes");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert!(value.get("repair").is_none() && value.get("lineage").is_none(), "absent fields stay absent");
    let decoded: super::super::Request = serde_json::from_slice(&bytes).expect("decodes");
    assert_eq!(serde_json::to_vec(&decoded).expect("re-encode"), bytes, "byte-identical round trip");
    let mut repaired = plain.clone();
    repaired.repair = Some(policy());
    repaired.envelope.budget.max_turns = 4;
    let revision = repaired.worker_profile_identity().expect("revision");
    let mut single = repaired.clone();
    single.repair = None;
    assert_ne!(revision, single.worker_profile_identity().expect("revision"), "a policy is a new revision");
    let mut lineage = repaired.clone();
    lineage.lineage = Some(crate::implementation_attempt::lineage::tests::binding(4, 4));
    assert_eq!(lineage.worker_profile_identity().expect("revision"), revision, "lineage is per-attempt");
    let mut served = repaired.clone();
    served.execution_binding.serving =
        Some(super::super::binding::tests::serving_declaration(&served.omp.model_broker.model));
    assert!(super::super::repair::validate(&served).expect_err("served").contains("served"));
    let mut malformed = plain;
    malformed.lineage = Some(crate::implementation_attempt::lineage::tests::binding(4, 4));
    if let Some(binding) = &mut malformed.lineage {
        binding.depth = 0;
    }
    assert!(super::super::repair::validate(&malformed).is_err());
}

#[test]
fn a_single_shot_attempt_with_a_lineage_reports_its_conservative_draw() {
    let mut request = super::host_local();
    assert!(super::super::repair::single_shot_draw(&request).is_none(), "no lineage, no draw");
    let lineage = crate::implementation_attempt::lineage::tests::binding(4, 4);
    request.lineage = Some(lineage.clone());
    let draw = super::super::repair::single_shot_draw(&request).expect("draw");
    assert_eq!(draw["binding"], lineage.identity().as_str());
    assert_eq!(draw["usage"][0]["charged"], serde_json::json!({"model_requests": 1, "tool_invocations": 0}));
    assert_eq!(draw["usage"][0]["model_requests"], serde_json::Value::Null, "unknown, never zero");
    assert_eq!(draw["unaccounted_dimensions"], serde_json::json!(["tool_invocations"]));
}

#[tokio::test]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
async fn a_step_that_overruns_the_tool_budget_is_exhausted_even_though_its_check_would_pass() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-overrun", Some(policy()), None);
    // The aggregate tool budget is six; the step reports nine and leaves a
    // candidate the unit check would pass.
    let overrun: Script<'_> = Box::new(|_, workspace, _| {
        std::fs::write(workspace.join("src/lib.txt"), "fixed").expect("edit");
        StepReport {
            model_requests: Some(1),
            tool_invocations: Some(9),
            ..exited()
        }
    });
    let (observation, _, invocations) = scenario(&fixture, &request, vec![overrun], FixtureChecks::admitted()).await;
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "budget_exhausted", "bound": {"kind": "tool_invocations"}})
    );
    assert!(invocations.is_empty(), "an overrun step is never checked");
    assert_eq!(observation["drawn"]["tool_invocations"], 9);
    assert_eq!(observation["revisions"].as_array().map(Vec::len), Some(1), "its candidate is retained, not passed");
}

#[tokio::test]
// r[verify lattice.implementation_attempt.repair.scenario.bypass]
async fn every_check_runs_against_its_own_read_only_copy_and_drift_is_rejected() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-copies", Some(formatted_policy()), None);
    let path = fixture.write(&request);
    let mut worker = FixtureWorker::new(vec![write("fixed")]);
    let mut checks = FixtureChecks::admitted();
    run(&path, &fixture.state, &mut worker, &mut checks).await.expect("a disposition");
    assert_eq!(fixture.disposition(&request)["disposition"]["kind"], "correct_candidate");
    let records = fixture.directory(&request).join("repair");
    assert_eq!(checks.seen.len(), 2);
    for seen in &checks.seen {
        assert!(seen.candidate.starts_with(&records), "a copy under the attempt, never the workspace");
        assert!(seen.is_read_only, "the copy is read-only");
        assert!(!seen.candidate.exists(), "the copy is removed after its check");
        assert!(seen.deadline > std::time::Duration::ZERO, "bounded by the time left");
        assert!(seen.deadline < std::time::Duration::from_millis(60_000), "never past the elapsed bound");
    }
    assert_ne!(checks.seen[0].candidate, checks.seen[1].candidate, "no check shares another's copy");
    // A formatter that rewrites its copy of a broken candidate to `fixed`.
    let fixture = Fixture::new();
    let request = fixture.request("attempt-drift", Some(formatted_policy()), None);
    let path = fixture.write(&request);
    let mut worker = FixtureWorker::new(vec![write("still broken")]);
    let mut checks = FixtureChecks {
        is_rewriting: true,
        ..FixtureChecks::admitted()
    };
    run(&path, &fixture.state, &mut worker, &mut checks).await.expect("a disposition");
    let observation = fixture.disposition(&request);
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "checks_rejected", "defect": {"code": "candidate_drift", "check": "fmt"}})
    );
    assert_eq!(checks.invocations.len(), 1, "no check runs after drift");
    assert!(!checks.seen[0].candidate.exists(), "the drifted copy is removed");
    let source = std::fs::read(fixture.workspace.join("src/lib.txt")).expect("workspace source");
    assert_eq!(source, b"still broken", "the worker workspace is untouched");
    assert_eq!(observation["drawn"]["tool_invocations"], 2, "the step and the one check that ran");
}

/// Run one scripted step of a fresh workspace under `policy` against
/// `clock`; returns the disposition, the worker's deadlines and the check port.
async fn clocked(
    attempt: &str,
    policy: crate::implementation_attempt::repair::Policy,
    step: Script<'_>,
    mut checks: FixtureChecks,
    clock: FixtureClock,
) -> (serde_json::Value, Vec<std::time::Duration>, FixtureChecks) {
    let fixture = Fixture::new();
    let request = fixture.request(attempt, Some(policy), None);
    let path = fixture.write(&request);
    let mut worker = FixtureWorker::new(vec![step]);
    let host = Host {
        clock,
        compute: &mut crate::execution_binding::UnavailableComputePort,
        revisions: super::super::session::revision::Store(None),
    };
    run_with(&path, &fixture.state, &mut worker, &mut checks, host).await.expect("a disposition");
    (fixture.disposition(&request)["disposition"].clone(), worker.deadlines, checks)
}

#[tokio::test]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
async fn the_elapsed_bound_and_the_clock_hold_through_the_terminal_decision() {
    let elapsed = serde_json::json!({"kind": "budget_exhausted", "bound": {"kind": "elapsed"}});
    // A worker that returns past the bound is never checked.
    let clock = FixtureClock::new();
    let worker_clock = clock.clone();
    let slow: Script<'_> = Box::new(move |_, workspace, _| {
        std::fs::write(workspace.join("src/lib.txt"), "fixed").expect("edit");
        worker_clock.advance(120_000);
        exited()
    });
    let (disposition, deadlines, checks) =
        clocked("attempt-slow-worker", policy(), slow, FixtureChecks::admitted(), clock).await;
    assert_eq!(disposition, elapsed);
    assert!(checks.invocations.is_empty());
    assert!(deadlines[0] <= std::time::Duration::from_millis(60_000), "the worker is bounded by the time left");
    // A clock that becomes unreadable during the step is uncertain.
    let clock = FixtureClock::new();
    let worker_clock = clock.clone();
    let unreadable: Script<'_> = Box::new(move |_, workspace, _| {
        std::fs::write(workspace.join("src/lib.txt"), "fixed").expect("edit");
        worker_clock.fail();
        exited()
    });
    let (disposition, _, checks) =
        clocked("attempt-lost-clock", policy(), unreadable, FixtureChecks::admitted(), clock).await;
    assert_eq!(disposition, serde_json::json!({"kind": "uncertain", "cause": "clock"}));
    assert!(checks.invocations.is_empty());
    // A passing check that returns past the bound does not make a candidate.
    let clock = FixtureClock::new();
    let slow_checks = FixtureChecks {
        slow: Some((clock.clone(), 120_000)),
        ..FixtureChecks::admitted()
    };
    let (disposition, _, checks) = clocked("attempt-slow-check", policy(), write("fixed"), slow_checks, clock).await;
    assert_eq!(disposition, elapsed);
    assert_eq!(checks.invocations.len(), 1, "the check ran and passed");
    // A check that uses up the time left stops the loop before the next one.
    let clock = FixtureClock::new();
    let slow_checks = FixtureChecks {
        slow: Some((clock.clone(), 120_000)),
        ..FixtureChecks::admitted()
    };
    let (disposition, _, checks) =
        clocked("attempt-slow-first-check", formatted_policy(), write("fixed"), slow_checks, clock).await;
    assert_eq!(disposition, elapsed);
    assert_eq!(checks.invocations.len(), 1, "the unit check never ran");
    // Inside the bound the same candidate passes.
    let (disposition, _, _) =
        clocked("attempt-in-time", policy(), write("fixed"), FixtureChecks::admitted(), FixtureClock::new()).await;
    assert_eq!(disposition["kind"], "correct_candidate");
}

#[tokio::test]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
async fn a_withdrawn_execution_binding_ends_the_loop_after_dispatch() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive/agent-a-1-active.json");
    let observation =
        lattice_core::execution_binding::validation::decode_observation(&std::fs::read(path).expect("fixture"))
            .expect("valid fixture");
    let later = |charge_cursor: Option<u64>| {
        let mut later = observation.clone();
        later.port.observation_sequence = 2;
        later.offer = None;
        if let Some(charge_cursor) = charge_cursor {
            later.allocation.charge_cursor = charge_cursor;
        }
        Ok(crate::execution_binding::AllocationReply::Observed(Box::new(later)))
    };
    let admitted = || Ok(crate::execution_binding::AllocationReply::Observed(Box::new(observation.clone())));
    // The third observation is the recheck before the repair step.
    let cases = [
        (later(None), vec![write("still broken"), write("fixed")]),
        (later(Some(9)), vec![write("still broken")]),
    ];
    for (third, script) in cases {
        let is_withdrawn = script.len() == 1;
        let fixture = Fixture::new();
        let request = super::request(
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
        );
        let request = readmit(request, &fixture.workspace, Some(policy()), None);
        let path = fixture.write(&request);
        let mut port = crate::execution_binding::FixtureComputePort::new(vec![admitted(), later(None), third]);
        let mut worker = FixtureWorker::new(script);
        let mut checks = FixtureChecks::admitted();
        let host = Host {
            clock: FixtureClock::new(),
            compute: &mut port,
            revisions: super::super::session::revision::Store(None),
        };
        run_with(&path, &fixture.state, &mut worker, &mut checks, host).await.expect("a disposition");
        assert!(worker.script.is_empty(), "every scripted step ran");
        let directory = fixture.directory(&request);
        let observation = disposition(&directory);
        if !is_withdrawn {
            assert_eq!(observation["disposition"]["kind"], "repaired_candidate", "a current binding admits the repair");
            assert!(observation.get("authority_withdrawn").is_none());
            continue;
        }
        // A post-dispatch outcome: retained like any disposition, never a
        // pre-spawn refusal record and never `dispatched:false`.
        assert_eq!(
            observation["disposition"],
            serde_json::json!({"kind": "authority_withdrawn", "cause": "execution_binding"})
        );
        assert_eq!(observation["authority_withdrawn"]["iteration"], 2);
        assert_eq!(observation["authority_withdrawn"]["evidence"]["decision"]["reason"], "reservation_replaced");
        assert!(observation.get("dispatched").is_none() && observation.get("reason").is_none());
        assert_eq!((observation["steps"].clone(), worker.requests.len()), (1.into(), 1), "the repair never spawned");
        assert!(!directory.join("repair/step-2.intent").exists(), "no intent for the withdrawn step");
        assert!(!directory.join(super::super::binding::REFUSAL_FILE).exists(), "no pre-dispatch refusal record");
        recover(&fixture.write(&request), &fixture.state)
            .await
            .expect("recovery reports the retained disposition");
    }
}

/// What happens to the managed revision during the first repair step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Revocation {
    Kept,
    Revoked,
    /// The revision store cannot be opened when the repair step rechecks it.
    Unreadable,
}

#[tokio::test]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
async fn a_revoked_or_unreadable_managed_revision_ends_the_loop_after_dispatch() {
    for case in [Revocation::Kept, Revocation::Revoked, Revocation::Unreadable] {
        let fixture = super::revisions::Fixture::new();
        let workspace = fixture.root.join("workspace");
        seed(&workspace);
        let request = readmit(fixture.request("attempt-managed-repair", 7), &workspace, Some(policy()), None);
        let (path, _) = fixture.write("request.json", &request);
        let (definition, revision) = fixture.prepared(&request, &path);
        fixture.promoted(&definition, &revision);
        let session = super::super::session::revision::identifier(&request);
        let moved = fixture.root.join("revisions.moved");
        let first: Script<'_> = Box::new(|_, workspace, _| {
            std::fs::write(workspace.join("src/lib.txt"), "still broken").expect("edit");
            match case {
                Revocation::Kept => {}
                Revocation::Revoked => fixture
                    .revision(&["revoke", "--definition", &definition, "--session", &session])
                    .expect("revoke the session during the first step"),
                Revocation::Unreadable => std::fs::rename(&fixture.store, &moved).expect("move the store away"),
            }
            exited()
        });
        let script = if case == Revocation::Kept {
            vec![first, write("fixed")]
        } else {
            vec![first]
        };
        let mut worker = FixtureWorker::new(script);
        let mut checks = FixtureChecks::admitted();
        let host = Host {
            clock: FixtureClock::new(),
            compute: &mut crate::execution_binding::UnavailableComputePort,
            revisions: super::super::session::revision::Store(Some(&fixture.store)),
        };
        let ran = run_with(&path, &fixture.state, &mut worker, &mut checks, host).await;
        assert!(worker.script.is_empty(), "every scripted step ran");
        let directory = fixture.directory(&request);
        let observation = disposition(&directory);
        if case == Revocation::Unreadable {
            // The disposition is retained before the session close, which
            // needs the store back.
            let error = ran.expect_err("the session cannot be closed without its store");
            assert!(error.contains("agent revision store is unavailable"), "{error}");
            assert_eq!(observation["authority_withdrawn"]["evidence"]["error"], "agent revision store is unavailable");
            std::fs::rename(&moved, &fixture.store).expect("restore the store");
        } else {
            ran.expect("a disposition");
        }
        if case == Revocation::Kept {
            assert_eq!(
                observation["disposition"]["kind"], "repaired_candidate",
                "a permitted revision admits the repair"
            );
            assert!(fixture.lifecycle(&definition).sessions.is_empty(), "the session is closed");
            continue;
        }
        assert_eq!(
            observation["disposition"],
            serde_json::json!({"kind": "authority_withdrawn", "cause": "managed_revision"}),
            "{case:?}"
        );
        if case == Revocation::Revoked {
            assert_eq!(observation["authority_withdrawn"]["evidence"]["refusal"]["code"], "session_revoked");
        }
        assert_eq!(worker.requests.len(), 1, "the repair never spawned");
        assert!(!directory.join("repair/step-2.intent").exists());
        let block = directory.join(super::super::session::revision::BLOCK_FILE);
        assert!(!block.exists(), "no pre-dispatch revision block");
        fixture.attempt("recover", &path, true).await.expect("recovery reports the retained disposition");
        assert!(fixture.lifecycle(&definition).sessions.is_empty(), "the session is closed");
    }
}

#[tokio::test]
// r[verify lattice.implementation_attempt.repair.scenario.bypass]
async fn a_check_that_only_changes_a_mode_of_its_copy_drifts() {
    let rejected = |check: &str| serde_json::json!({"kind": "checks_rejected", "defect": {"code": "candidate_drift", "check": check}});
    // A newly executable source file, a writable directory and a writable
    // copy root leave every byte unchanged.
    for (index, (path, mode)) in [("src/lib.txt", 0o555), ("src", 0o755), ("", 0o755)].into_iter().enumerate() {
        let fixture = Fixture::new();
        let request = fixture.request(&format!("attempt-chmod-{index}"), Some(formatted_policy()), None);
        let path_file = fixture.write(&request);
        let mut worker = FixtureWorker::new(vec![write("fixed")]);
        let mut checks = FixtureChecks {
            chmod: Some((path, mode)),
            ..FixtureChecks::admitted()
        };
        run(&path_file, &fixture.state, &mut worker, &mut checks).await.expect("a disposition");
        assert_eq!(fixture.disposition(&request)["disposition"], rejected("fmt"), "{path:?}");
        assert_eq!(checks.invocations.len(), 1, "no check runs after drift");
        assert!(!checks.seen[0].candidate.exists(), "the drifted copy is removed");
    }
    // The sealed modes themselves pass.
    let fixture = Fixture::new();
    let request = fixture.request("attempt-chmod-unchanged", Some(formatted_policy()), None);
    let path_file = fixture.write(&request);
    let mut worker = FixtureWorker::new(vec![write("fixed")]);
    let mut checks = FixtureChecks {
        chmod: Some(("src/lib.txt", 0o444)),
        ..FixtureChecks::admitted()
    };
    run(&path_file, &fixture.state, &mut worker, &mut checks).await.expect("a disposition");
    assert_eq!(fixture.disposition(&request)["disposition"]["kind"], "correct_candidate");
    assert_eq!(checks.invocations.len(), 2);
}

#[tokio::test]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
async fn time_spent_materializing_a_copy_is_charged_before_its_check_is_called() {
    let elapsed = serde_json::json!({"kind": "budget_exhausted", "bound": {"kind": "elapsed"}});
    let clock_lost = serde_json::json!({"kind": "uncertain", "cause": "clock"});
    for (shift, expected) in [(Shift::Advance(120_000), elapsed), (Shift::Fail, clock_lost)] {
        let fixture = Fixture::new();
        let request = fixture.request("attempt-slow-copy", Some(policy()), None);
        let path = fixture.write(&request);
        let records = fixture.directory(&request).join("repair");
        // The bound passes, or the clock is lost, while the copy is written:
        // the clock reads past it only once the copy exists.
        let clock = FixtureClock {
            on_copy: Some((records.clone(), shift)),
            ..FixtureClock::new()
        };
        let mut worker = FixtureWorker::new(vec![write("fixed")]);
        let mut checks = FixtureChecks::admitted();
        let host = Host {
            clock,
            compute: &mut crate::execution_binding::UnavailableComputePort,
            revisions: super::super::session::revision::Store(None),
        };
        run_with(&path, &fixture.state, &mut worker, &mut checks, host).await.expect("a disposition");
        assert_eq!(fixture.disposition(&request)["disposition"], expected, "{shift:?}");
        assert!(checks.invocations.is_empty(), "the check port is never called: {shift:?}");
        assert!(!FixtureClock::is_copy_present(&records), "the unused copy is removed");
    }
}

#[tokio::test]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
async fn time_spent_waiting_for_the_step_lock_is_charged_before_the_worker_is_called() {
    let elapsed = serde_json::json!({"kind": "budget_exhausted", "bound": {"kind": "elapsed"}});
    let clock_lost = serde_json::json!({"kind": "uncertain", "cause": "clock"});
    for (shift, expected) in [(Shift::Advance(120_000), elapsed), (Shift::Fail, clock_lost)] {
        let fixture = Fixture::new();
        let request = fixture.request("attempt-slow-lock", Some(policy()), None);
        let path = fixture.write(&request);
        let clock = FixtureClock::new();
        let holder = std::cell::RefCell::new(None);
        let (state, work_id, holder_clock) = (fixture.state.clone(), request.envelope.work_id.clone(), clock.clone());
        // During the first step another holder takes the work's fence lock.
        // It keeps it until the run has asked for the repair step (four
        // more clock readings: after the step, before the check, after the
        // check and before the repair), so the run waits on the lock; then
        // the time passes, or the clock is lost, and the lock is released.
        let first: Script<'_> = Box::new(|_, workspace, _| {
            let (acquired, is_acquired) = std::sync::mpsc::channel();
            let (state, work_id, clock) = (state.clone(), work_id.clone(), holder_clock.clone());
            let start = clock.observations();
            *holder.borrow_mut() = Some(std::thread::spawn(move || {
                let guard =
                    super::super::session::fencing::Guard::acquire(&state, &work_id).expect("hold the fence lock");
                acquired.send(()).expect("signal");
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                while clock.observations() < start + 4 {
                    assert!(std::time::Instant::now() < deadline, "the run never reached the repair step");
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
                clock.shift(shift);
                drop(guard);
            }));
            is_acquired.recv().expect("the lock is held");
            std::fs::write(workspace.join("src/lib.txt"), "still broken").expect("edit");
            exited()
        });
        let mut worker = FixtureWorker::new(vec![first]);
        let mut checks = FixtureChecks::admitted();
        let host = Host {
            clock,
            compute: &mut crate::execution_binding::UnavailableComputePort,
            revisions: super::super::session::revision::Store(None),
        };
        run_with(&path, &fixture.state, &mut worker, &mut checks, host).await.expect("a disposition");
        holder.borrow_mut().take().expect("the holder ran").join().expect("the holder finished");
        let observation = fixture.disposition(&request);
        assert_eq!(observation["disposition"], expected, "{shift:?}");
        assert_eq!((observation["steps"].clone(), worker.requests.len()), (1.into(), 1), "the worker is never called");
        // The repair step got past the lock: its intent exists, its worker
        // was never called.
        let directory = fixture.directory(&request);
        assert!(directory.join("repair/step-2.intent").exists(), "{shift:?}");
        assert!(!directory.join("repair/step-2.json").exists());
    }
}

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

const EXECUTOR: &str = "7777777777777777777777777777777777777777777777777777777777777777";
const UNIT_CONFIG: &[u8] = b"unit-config";

type Script = Box<dyn FnMut(&StepRequest, &std::path::Path, &dyn Fn() -> bool) -> StepReport>;

/// Replays one scripted step per call and records every request.
struct FixtureWorker {
    script: std::collections::VecDeque<Script>,
    requests: Vec<StepRequest>,
}

impl FixtureWorker {
    fn new(script: Vec<Script>) -> Self {
        Self {
            script: script.into(),
            requests: Vec::new(),
        }
    }
}

impl super::super::repair::WorkerPort for FixtureWorker {
    fn step(
        &mut self,
        request: &StepRequest,
        workspace: &std::path::Path,
        is_cancel_requested: &dyn Fn() -> bool,
    ) -> StepReport {
        self.requests.push(request.clone());
        let mut step = self.script.pop_front().expect("a scripted step for every spawn");
        step(request, workspace, is_cancel_requested)
    }
}

/// Passes `unit` exactly when `src/lib.txt` reads `fixed`, and reports the
/// configuration it actually read from `checks/unit.conf`.
struct FixtureChecks {
    executor: String,
    forced: Option<Outcome>,
    invocations: Vec<Invocation>,
}

impl FixtureChecks {
    fn admitted() -> Self {
        Self {
            executor: EXECUTOR.into(),
            forced: None,
            invocations: Vec::new(),
        }
    }
}

impl super::super::repair::CheckPort for FixtureChecks {
    fn run(&mut self, invocation: &Invocation, workspace: &std::path::Path) -> Observation {
        self.invocations.push(invocation.clone());
        let source = std::fs::read(workspace.join("src/lib.txt")).expect("candidate source");
        let configuration = std::fs::read(workspace.join("checks/unit.conf")).expect("check configuration");
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

/// Advances one second per observation.
struct FixtureClock(std::cell::Cell<i64>);

impl crate::operation::shell::Clock for FixtureClock {
    fn observe_epoch_millis(&self) -> i64 {
        let now = self.0.get();
        self.0.set(now + 1_000);
        now
    }
}

fn write(text: &'static str) -> Script {
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
        std::fs::create_dir_all(workspace.join("src")).expect("workspace");
        std::fs::create_dir_all(workspace.join("checks")).expect("checks");
        std::fs::write(workspace.join("src/lib.txt"), b"broken").expect("source");
        std::fs::write(workspace.join("checks/unit.conf"), UNIT_CONFIG).expect("check configuration");
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
        request.envelope.budget.max_turns = 4;
        request.workspace_root = self.workspace.clone();
        let base = crate::candidate_capture::workspace(&self.workspace).expect("base");
        request.omp.base_identity = crate::candidate::snapshot_identity(&base).expect("base identity");
        request.repair = policy;
        request.lineage = lineage;
        request.envelope.worker_profile = request.worker_profile_identity().expect("revision");
        request.admitted_envelope = request.envelope.identity();
        request
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
        let path = self.directory(request).join("repair/disposition.json");
        serde_json::from_slice(&std::fs::read(path).expect("retained disposition")).expect("disposition json")
    }
}

async fn run(
    path: &std::path::Path,
    state: &std::path::Path,
    worker: &mut FixtureWorker,
    checks: &mut FixtureChecks,
) -> Result<(), String> {
    let clock = FixtureClock(std::cell::Cell::new(1_000_000));
    super::super::execution::run(
        path,
        state,
        Some(super::super::Ports {
            compute: &mut crate::execution_binding::UnavailableComputePort,
            serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
            repair: Some(super::super::repair::Ports {
                worker,
                checks,
                clock: &clock,
            }),
        }),
        super::super::session::revision::Store(None),
    )
    .await
}

async fn recover(path: &std::path::Path, state: &std::path::Path) -> Result<(), String> {
    super::super::execution::run(path, state, None, super::super::session::revision::Store(None)).await
}

/// Run `script` with admitted checks; returns the retained disposition,
/// the worker's requests and the check invocations.
async fn scenario(
    fixture: &Fixture,
    request: &super::super::Request,
    script: Vec<Script>,
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
    // Inventory v3 does not read repair records: the instance stays conservative.
    let lifecycle = &inventory["instance"]["lifecycle"]["state"];
    assert_eq!(lifecycle, "dispatch_recorded_outcome_unknown", "{inventory}");
    assert!(
        inventory["instance"]["wait_reasons"].to_string().contains("dispatch_outcome_unknown"),
        "{inventory}"
    );
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
    let first: Script = Box::new(move |_, workspace, _| {
        std::fs::write(workspace.join("src/lib.txt"), "still broken").expect("edit");
        let directory = state.join("successor");
        drop(super::super::session::fencing::admit(&state, &successor, &directory).expect("fence 8 recorded"));
        exited()
    });
    let (observation, requests, invocations) =
        scenario(&fixture, &request, vec![first], FixtureChecks::admitted()).await;
    // A post-dispatch outcome: the retained repair observation of a step that
    // ran, never a refusal record and never `dispatched:false`.
    assert_eq!(observation["schema"], "lattice.implementation-attempt-repair-observation.v1");
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
    let unknown: Script = Box::new(|_, _, _| StepReport {
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
    let unchanged: Script = Box::new(|_, _, _| exited());
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
    let script: Vec<Script> = vec![Box::new(claim), write("broken 1"), write("broken 2")];
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
    let propose: Script = Box::new(|_, workspace, _| {
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

/// The first line printed by a refusal is not captured here; the typed
/// reason is the returned diagnostic and nothing is written.
fn assert_nothing_written(directory: &std::path::Path) {
    for name in [
        super::super::binding::ADMITTED_FILE,
        super::super::session::intent::FILE,
        "candidate.redb",
        "repair",
    ] {
        assert!(!directory.join(name).exists(), "{name} must not exist");
    }
}

#[tokio::test]
// r[verify lattice.implementation_attempt.lineage.scenario.exhausted]
async fn lineage_and_missing_ports_refuse_before_any_write() {
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
    for (index, (lineage, reason)) in cases.into_iter().enumerate() {
        for policy in [Some(policy()), None] {
            let attempt = format!("attempt-lineage-{index}-{}", policy.is_some());
            let request = fixture.request(&attempt, policy, Some(lineage.clone()));
            let path = fixture.write(&request);
            let mut worker = FixtureWorker::new(Vec::new());
            let mut checks = FixtureChecks::admitted();
            let refused = run(&path, &fixture.state, &mut worker, &mut checks).await;
            assert_eq!(refused, Err(reason.message().to_owned()), "{attempt}");
            assert!(worker.requests.is_empty());
            assert_nothing_written(&fixture.directory(&request));
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
    assert_nothing_written(&fixture.directory(&request));
    let record = super::super::refusal::record(&request.envelope, super::super::refusal::Reason::LineageExhausted);
    assert_eq!((record["reason"].clone(), record["dispatched"].clone()), ("lineage_exhausted".into(), false.into()));
}

/// A panicking worker step stands in for a controller lost mid-step.
#[test]
fn a_lost_controller_without_a_disposition_is_unknown_and_never_reruns() {
    let runtime = || tokio::runtime::Builder::new_current_thread().enable_all().build().expect("runtime");
    let fixture = Fixture::new();
    let request = fixture.request("attempt-lost", Some(policy()), None);
    let path = fixture.write(&request);
    let lost: Script = Box::new(|_, _, _| panic!("controller lost during the step"));
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

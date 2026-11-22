//! The repair loop's worker port over the pinned OMP launcher.
//!
//! Each step runs one OMP process in the attempt workspace through the
//! single-shot attempt's workspace boundary and launcher, from the parent
//! revision the workspace holds, within the loop's deadline. Its raw events
//! and launcher observation are retained next to the step intent before the
//! loop records the step; only an observed stop is reported as one.

// r[impl lattice.implementation_attempt.operator_repair.scenario.outcomes]
// r[impl lattice.implementation_attempt.operator_repair.scenario.repaired]
// r[impl lattice.implementation_attempt.operator_repair.scenario.recovery]

const SCHEMA: &str = "lattice.implementation-attempt-omp-step.v1";
/// How often a persisted cancellation request is polled during a step.
const CANCEL_POLL: std::time::Duration = std::time::Duration::from_millis(20);

type Report = crate::implementation_attempt::repair::step::Report;
type StepOutcome = crate::implementation_attempt::repair::step::WorkerOutcome;

/// The OMP worker port of one repair run.
pub(in crate::implementation_attempt::cli) struct OmpPort<'a, 'r> {
    request: &'r super::super::super::Request,
    /// The run's private repair records directory.
    records: std::path::PathBuf,
    launch: &'a mut dyn super::Launch,
}

impl<'a, 'r> OmpPort<'a, 'r> {
    pub(in crate::implementation_attempt::cli) fn new(
        request: &'r super::super::super::Request,
        directory: &std::path::Path,
        launch: &'a mut dyn super::Launch,
    ) -> Self {
        Self {
            request,
            records: super::super::records::Retained::root(directory),
            launch,
        }
    }

    /// The step's bounded worker request and its OMP profile: the parent
    /// revision the workspace holds as its base, the step's prompt, and the
    /// loop's deadline within the admission's own bound.
    fn prepare(
        &self,
        step: &crate::implementation_attempt::repair::step::Request,
        deadline: std::time::Duration,
    ) -> Result<(crate::candidate_worker::Request, crate::candidate_worker::omp::Profile), String> {
        let mut worker = super::super::super::session::launch::worker_request(self.request)?;
        let admitted_secs = worker.timeout_secs;
        worker.timeout_secs =
            crate::implementation_attempt::repair::step::operator::timeout_secs(deadline, admitted_secs);
        let parent = crate::candidate_capture::workspace(&worker.boundary.root).map_err(|error| error.to_string())?;
        let mut profile = self.request.omp.clone();
        profile.base_identity = crate::candidate::snapshot_identity(&parent).map_err(|error| error.to_string())?;
        profile.prompt = crate::implementation_attempt::repair::step::operator::prompt(&self.request.omp.prompt, step);
        Ok((worker, profile))
    }

    /// Retain the step's raw events and launcher observation once.
    fn retain(&self, iteration: u32, record: &serde_json::Value, raw_events: &[u8]) -> Result<(), String> {
        let bytes = serde_json::to_vec(record).map_err(|error| error.to_string())?;
        let events = self.records.join(format!("omp-step-{iteration}.jsonl"));
        super::super::super::artifacts::write_once(&events, raw_events)?;
        super::super::super::artifacts::write_once(&self.records.join(format!("omp-step-{iteration}.json")), &bytes)?;
        Ok(())
    }
}

impl super::super::WorkerPort for OmpPort<'_, '_> {
    fn step(
        &mut self,
        request: &crate::implementation_attempt::repair::step::Request,
        workspace: &std::path::Path,
        deadline: std::time::Duration,
        is_cancel_requested: &dyn Fn() -> bool,
    ) -> Report {
        debug_assert_eq!(workspace, self.request.workspace_root.as_path(), "the attempt workspace");
        let (outcome, record, raw_events) = match self.prepare(request, deadline) {
            Err(blocker) => {
                // Nothing was launched, but the loop has no not-started outcome:
                // the step is uncertain and never retried.
                let record = serde_json::json!({"schema": SCHEMA, "iteration": request.iteration,
                    "launched": false, "blocker": blocker});
                (StepOutcome::Unknown, record, Vec::new())
            }
            Ok((worker, profile)) => {
                let result = launch(&mut *self.launch, &worker, &profile, is_cancel_requested);
                observed(request.iteration, &worker, &profile, result)
            }
        };
        // A step whose evidence could not be retained is uncertain.
        let outcome = match self.retain(request.iteration, &record, &raw_events) {
            Ok(()) => outcome,
            Err(_) => StepOutcome::Unknown,
        };
        Report {
            outcome,
            // JSONL is not a complete request ledger: the reservation is charged.
            model_requests: None,
            tool_invocations: None,
            proposal: None,
        }
    }
}

/// The worker outcome, retained record and raw events of one launch.
fn observed(
    iteration: u32,
    worker: &crate::candidate_worker::Request,
    profile: &crate::candidate_worker::omp::Profile,
    result: Result<crate::candidate_worker::omp::RunResult, String>,
) -> (StepOutcome, serde_json::Value, Vec<u8>) {
    debug_assert!(iteration >= 1, "steps count from one");
    let prompt = blake3::hash(profile.prompt.as_bytes()).to_hex().to_string();
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            let record = serde_json::json!({"schema": SCHEMA, "iteration": iteration, "launched": true,
                "base_identity": profile.base_identity, "prompt": prompt, "timeout_secs": worker.timeout_secs,
                "error": error});
            return (StepOutcome::Unknown, record, Vec::new());
        }
    };
    let outcome = crate::implementation_attempt::repair::step::operator::outcome(&result.outcome);
    let (label, blocker) = crate::implementation_attempt::repair::step::operator::describe(&result.outcome);
    debug_assert!(label != "produced" || outcome == StepOutcome::Exited { code: 0 }, "a candidate is an exit");
    let (observation, observation_error) = match &result.observation {
        Ok(observation) => (Some(observation), None),
        Err(error) => (None, Some(error)),
    };
    let record = serde_json::json!({"schema": SCHEMA, "iteration": iteration, "launched": true,
        "base_identity": profile.base_identity, "prompt": prompt, "timeout_secs": worker.timeout_secs,
        "launcher_outcome": label, "blocker": blocker, "worker_outcome": outcome,
        "observation": observation, "observation_error": observation_error,
        "revision": result.revision, "executable_blake3": result.executable_blake3,
        "argv_blake3": result.argv_blake3, "compatibility": result.compatibility});
    (outcome, record, result.raw_events().to_vec())
}

/// Run one launch on its own thread while this thread polls the persisted
/// cancellation request. A launcher that panics is a lost controller: the
/// panic propagates, the step intent stays without a record and the outcome
/// stays unknown.
fn launch(
    launch: &mut dyn super::Launch,
    worker: &crate::candidate_worker::Request,
    profile: &crate::candidate_worker::omp::Profile,
    is_cancel_requested: &dyn Fn() -> bool,
) -> Result<crate::candidate_worker::omp::RunResult, String> {
    let cancellation = std::sync::atomic::AtomicBool::new(is_cancel_requested());
    std::thread::scope(|scope| {
        let running = scope.spawn(|| launch.launch(worker, profile, &cancellation));
        while !running.is_finished() {
            if is_cancel_requested() {
                cancellation.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            std::thread::sleep(CANCEL_POLL);
        }
        running.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}

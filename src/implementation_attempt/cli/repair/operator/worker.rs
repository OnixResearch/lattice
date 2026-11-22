//! The repair loop's worker port over the pinned OMP launcher.
//!
//! Each step runs one OMP process in the attempt workspace through the
//! single-shot attempt's workspace boundary and launcher, within the loop's
//! deadline. The launcher refuses a workspace that is not the expected base:
//! the admitted base for the first step, then the tree the previous exited
//! step left, so a workspace changed between steps is blocked and the step
//! is uncertain. Raw events and the launcher observation are retained next
//! to the step intent before the loop records the step; only an observed
//! stop is reported as one.

// r[impl lattice.implementation_attempt.operator_repair.scenario.outcomes]
// r[impl lattice.implementation_attempt.operator_repair.scenario.repaired]
// r[impl lattice.implementation_attempt.operator_repair.scenario.recovery]

const SCHEMA: &str = "lattice.implementation-attempt-omp-step.v1";
/// An expected base no workspace has, after a tree could not be captured.
const UNKNOWN_BASE: &str = "";

type Report = crate::implementation_attempt::repair::step::Report;
type StepOutcome = crate::implementation_attempt::repair::step::WorkerOutcome;

/// The OMP worker port of one repair run.
pub(in crate::implementation_attempt::cli) struct OmpPort<'a, 'r> {
    request: &'r super::super::super::Request,
    /// The run's private repair records directory.
    records: std::path::PathBuf,
    launch: &'a mut dyn super::Launch,
    /// The snapshot identity the next step's workspace must have.
    expected_base: String,
}

/// One prepared launch.
struct Prepared {
    worker: crate::candidate_worker::Request,
    profile: crate::candidate_worker::omp::Profile,
    /// The admission's bound capped the worker below the elapsed deadline.
    is_capped: bool,
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
            expected_base: request.omp.base_identity.clone(),
        }
    }

    /// The step's bounded worker request and its OMP profile: the expected
    /// base, the step's prompt, and the loop's deadline within the
    /// admission's own bound.
    fn prepare(
        &self,
        step: &crate::implementation_attempt::repair::step::Request,
        deadline: std::time::Duration,
    ) -> Result<Prepared, String> {
        let mut worker = super::super::super::session::launch::worker_request(self.request)?;
        let admitted_secs = worker.timeout_secs;
        worker.timeout_secs =
            crate::implementation_attempt::repair::step::operator::timeout_secs(deadline, admitted_secs);
        let mut profile = self.request.omp.clone();
        profile.base_identity.clone_from(&self.expected_base);
        profile.prompt = crate::implementation_attempt::repair::step::operator::prompt(&self.request.omp.prompt, step);
        let is_capped = crate::implementation_attempt::repair::step::operator::is_capped(deadline, admitted_secs);
        Ok(Prepared {
            worker,
            profile,
            is_capped,
        })
    }

    /// After an exited step, the tree it left is the next step's base.
    fn advance(&mut self) {
        let captured = crate::candidate_capture::workspace(&self.request.workspace_root)
            .ok()
            .and_then(|tree| crate::candidate::snapshot_identity(&tree).ok());
        self.expected_base = captured.unwrap_or_else(|| UNKNOWN_BASE.to_owned());
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
        let (outcome, record, result) = match self.prepare(request, deadline) {
            Err(blocker) => {
                // Nothing was launched, but the loop has no not-started outcome:
                // the step is uncertain and never retried.
                let record = serde_json::json!({"schema": SCHEMA, "iteration": request.iteration,
                    "launched": false, "blocker": blocker});
                (StepOutcome::Unknown, record, None)
            }
            Ok(prepared) => {
                let launch = &mut *self.launch;
                let result = super::polled(
                    |cancellation| launch.launch(&prepared.worker, &prepared.profile, cancellation),
                    is_cancel_requested,
                );
                observed(request.iteration, &prepared, result)
            }
        };
        if matches!(outcome, StepOutcome::Exited { .. }) {
            self.advance();
        }
        let raw_events = result.as_ref().map_or(&[][..], crate::candidate_worker::omp::RunResult::raw_events);
        // A step whose evidence could not be retained is uncertain.
        let outcome = match self.retain(request.iteration, &record, raw_events) {
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

/// The worker outcome and retained record of one launch, and its result.
fn observed(
    iteration: u32,
    prepared: &Prepared,
    result: Result<crate::candidate_worker::omp::RunResult, String>,
) -> (StepOutcome, serde_json::Value, Option<crate::candidate_worker::omp::RunResult>) {
    debug_assert!(iteration >= 1, "steps count from one");
    let profile = &prepared.profile;
    let prompt = blake3::hash(profile.prompt.as_bytes()).to_hex().to_string();
    let timeout_secs = prepared.worker.timeout_secs;
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            let record = serde_json::json!({"schema": SCHEMA, "iteration": iteration, "launched": true,
                "base_identity": profile.base_identity, "prompt": prompt, "timeout_secs": timeout_secs,
                "error": error});
            return (StepOutcome::Unknown, record, None);
        }
    };
    let outcome = crate::implementation_attempt::repair::step::operator::outcome(&result.outcome, prepared.is_capped);
    let (label, blocker) = crate::implementation_attempt::repair::step::operator::describe(&result.outcome);
    debug_assert!(label != "produced" || outcome == StepOutcome::Exited { code: 0 }, "a candidate is an exit");
    let (observation, observation_error) = match &result.observation {
        Ok(observation) => (Some(observation), None),
        Err(error) => (None, Some(error)),
    };
    let record = serde_json::json!({"schema": SCHEMA, "iteration": iteration, "launched": true,
        "base_identity": profile.base_identity, "prompt": prompt, "timeout_secs": timeout_secs,
        "deadline_capped": prepared.is_capped,
        "launcher_outcome": label, "blocker": blocker, "worker_outcome": outcome,
        "observation": observation, "observation_error": observation_error,
        "revision": result.revision, "executable_blake3": result.executable_blake3,
        "argv_blake3": result.argv_blake3, "compatibility": result.compatibility});
    (outcome, record, Some(result))
}

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
//!
//! Each step's broker retains every request it relays in the attempt's
//! `model-requests/` evidence under the step's own request-index range,
//! `[(n - 1) * max_requests, n * max_requests)` for step `n`. The step
//! reports the requests complete evidence in that range proves as observed;
//! otherwise its model requests stay unknown and the reservation is charged.

// r[impl lattice.implementation_attempt.operator_repair.scenario.outcomes]
// r[impl lattice.implementation_attempt.operator_repair.scenario.repaired]
// r[impl lattice.implementation_attempt.operator_repair.scenario.recovery]
// r[impl lattice.implementation_attempt.observed_charge.scenario.step_range]

const SCHEMA: &str = "lattice.implementation-attempt-omp-step.v1";
/// An expected base no workspace has, after a tree could not be captured.
const UNKNOWN_BASE: &str = "";

type Report = crate::implementation_attempt::repair::step::Report;
type StepOutcome = crate::implementation_attempt::repair::step::WorkerOutcome;

/// The OMP worker port of one repair run.
pub(in crate::implementation_attempt::cli) struct OmpPort<'a, 'r> {
    request: &'r super::super::super::Request,
    /// The attempt directory, which holds the request evidence.
    directory: std::path::PathBuf,
    /// The run's private repair records directory.
    records: std::path::PathBuf,
    admitted: lattice_core::execution_binding::admission::bound::Admitted,
    launch: &'a mut dyn super::Launch,
    /// The snapshot identity the next step's workspace must have.
    expected_base: String,
    /// The operator's clock, on which the admission's expiry is read.
    clock: &'a dyn crate::operation::shell::Clock,
}

/// One prepared launch.
struct Prepared {
    worker: crate::candidate_worker::Request,
    profile: crate::candidate_worker::omp::Profile,
    /// The admission's bound capped the worker below the elapsed deadline.
    is_capped: bool,
    /// The forwarded index of this step's first model request.
    first_request_index: u32,
    /// The attempt's request evidence, created before the broker runs.
    files: super::super::super::binding::requests::RequestFiles,
}

impl<'a, 'r> OmpPort<'a, 'r> {
    pub(in crate::implementation_attempt::cli) fn new(
        request: &'r super::super::super::Request,
        directory: &std::path::Path,
        admitted: lattice_core::execution_binding::admission::bound::Admitted,
        launch: &'a mut dyn super::Launch,
        clock: &'a dyn crate::operation::shell::Clock,
    ) -> Self {
        debug_assert!(admitted.serving.is_none(), "a repair policy is never combined with a served route");
        Self {
            request,
            directory: directory.to_path_buf(),
            records: super::super::records::Retained::root(directory),
            admitted,
            launch,
            expected_base: request.omp.as_ref().expect("OMP repair profile").base_identity.clone(),
            clock,
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
        let mut worker = super::super::super::session::launch::worker_request(self.request, self.clock)?;
        let admitted_secs = worker.timeout_secs;
        worker.timeout_secs =
            crate::implementation_attempt::repair::step::operator::timeout_secs(deadline, admitted_secs);
        let mut profile = self.request.omp.clone().ok_or("OMP repair profile is absent")?;
        profile.base_identity.clone_from(&self.expected_base);
        profile.prompt = crate::implementation_attempt::repair::step::operator::prompt(&profile.prompt, step);
        let is_capped = crate::implementation_attempt::repair::step::operator::is_capped(deadline, admitted_secs);
        let first_request_index = first_request_index(step, &profile.model_broker)?;
        let files = super::super::super::binding::requests::RequestFiles::new(&self.directory)?;
        Ok(Prepared {
            worker,
            profile,
            is_capped,
            first_request_index,
            files,
        })
    }

    /// The model requests the attempt's complete request evidence proves in
    /// the step's own index range; `None` when any intent there lacks a
    /// terminal record, a request ended uncertain or the evidence conflicts.
    fn observed_requests(&self, first_request_index: u32) -> Option<u64> {
        let first = u64::from(first_request_index);
        let indices = first..first.checked_add(u64::from(self.request.omp.as_ref()?.model_broker.max_requests))?;
        let coverage = super::super::super::binding::requests::coverage(&self.admitted, &self.directory).ok()?;
        coverage.observed_requests(Some(&indices))
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

/// The forwarded index of a step's first model request. Each broker run
/// serves at most the broker's `max_requests` requests, so step `n` (from 1)
/// owns the range `[(n - 1) * max_requests, n * max_requests)`: indices
/// strictly increase across the steps of one attempt and are derived from
/// the retained step iteration alone, so a restarted controller reproduces
/// them.
// r[impl lattice.vm_profile.forwarded_upstream.scenario.request_index]
fn first_request_index(
    step: &crate::implementation_attempt::repair::step::Request,
    broker: &crate::candidate_worker::broker::Profile,
) -> Result<u32, String> {
    step.iteration
        .checked_sub(1)
        .and_then(|earlier_steps| earlier_steps.checked_mul(broker.max_requests))
        .ok_or_else(|| "the step's forwarded request indices exceed their range".to_owned())
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
        let mut observed_model_requests = None;
        let (outcome, record, result) = match self.prepare(request, deadline) {
            Err(blocker) => {
                // Nothing was launched, but the loop has no not-started outcome:
                // the step is uncertain and never retried.
                let record = serde_json::json!({"schema": SCHEMA, "iteration": request.iteration,
                    "launched": false, "blocker": blocker});
                (StepOutcome::Unknown, record, None)
            }
            Ok(mut prepared) => {
                let launch = &mut *self.launch;
                let mut gate = super::super::super::binding::requests::recorded(&self.admitted, &mut prepared.files);
                let (worker, profile) = (&prepared.worker, &prepared.profile);
                let first_request_index = prepared.first_request_index;
                let result = super::polled(
                    |cancellation| {
                        let broker = super::Broker {
                            first_request_index,
                            gate: &mut gate,
                        };
                        launch.launch(worker, profile, broker, cancellation)
                    },
                    is_cancel_requested,
                );
                observed_model_requests = self.observed_requests(first_request_index);
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
            // JSONL is not a complete request ledger; the broker's request
            // evidence is, when it is complete.
            model_requests: None,
            tool_invocations: None,
            observed_model_requests,
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

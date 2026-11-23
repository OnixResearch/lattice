//! The repair loop over one attempt's injected ports.

// r[impl lattice.implementation_attempt.repair.scenario.dispositions]
// r[impl lattice.implementation_attempt.repair.scenario.bypass]
// r[impl lattice.implementation_attempt.lineage.scenario.usage]

/// Every iteration is a spawn, a check and a repair decision, plus the first
/// and last decisions: `3 * (MAX_REPAIR_ITERATIONS + 1) + 2`.
const MAX_TRANSITIONS: u32 = 29;

mod authority;
mod candidate;
mod checks;

type Authority = crate::implementation_attempt::repair::state::Authority;
type Disposition = crate::implementation_attempt::repair::state::Disposition;
type Next = crate::implementation_attempt::repair::state::Next;
type Observation = crate::implementation_attempt::repair::checks::Observation;
type StepRequest = crate::implementation_attempt::repair::step::Request;
type Verdict = crate::implementation_attempt::repair::checks::Verdict;

/// One host-derived candidate revision.
type Revision = crate::implementation_attempt::repair::revision::Record;

/// A candidate revision and the captured tree its checks copy.
struct Candidate {
    revision: Revision,
    tree: super::materialized::Tree,
}

/// The retained record of one step.
type StepRecord = crate::implementation_attempt::repair::retained::StepRecord;

/// The authority a later step found withdrawn before its intent.
type Withdrawal = crate::implementation_attempt::repair::retained::Withdrawal;

/// What one run's loop is bound to.
pub(super) struct Frame<'a, 'p> {
    pub(super) request: &'a super::super::Request,
    /// The state root whose fence lock guards every step intent.
    pub(super) state_root: &'a std::path::Path,
    pub(super) records: &'a super::records::Retained,
    pub(super) authority: super::Authority<'p>,
}

/// The loop over one attempt's ports.
pub(super) struct Driver<'a, 'p> {
    frame: Frame<'a, 'p>,
    policy: crate::implementation_attempt::repair::Policy,
    state: crate::implementation_attempt::repair::state::Loop,
    base: std::collections::BTreeMap<String, Vec<u8>>,
    parent_identity: String,
    usage: Vec<crate::implementation_attempt::repair::step::Usage>,
    revisions: Vec<Revision>,
    withdrawal: Option<Withdrawal>,
    started_epoch_ms: Option<u64>,
    pub(super) finished_epoch_ms: Option<u64>,
}

impl<'a, 'p> Driver<'a, 'p> {
    /// Re-observe the admitted base before any worker effect. Preparation is
    /// a host step without model or tool use.
    pub(super) fn prepare(
        frame: Frame<'a, 'p>,
        policy: crate::implementation_attempt::repair::Policy,
    ) -> Result<Self, String> {
        let request = frame.request;
        let base = crate::candidate_capture::workspace(&request.workspace_root).map_err(|error| error.to_string())?;
        let parent_identity = crate::candidate::snapshot_identity(&base).map_err(|error| error.to_string())?;
        if parent_identity != request.omp.base_identity {
            return Err("repair workspace differs from the admitted base snapshot; nothing was dispatched".into());
        }
        let binding = crate::implementation_attempt::repair::StepBinding::of(&request.envelope);
        let step_model_requests = u64::from(request.omp.model_broker.max_requests);
        let state = crate::implementation_attempt::repair::state::Loop::new(
            policy.clone(),
            binding,
            request.lineage.clone(),
            step_model_requests,
        )
        .map_err(|error| error.to_string())?;
        let step_count = usize::try_from(policy.max_iterations).map_err(|error| error.to_string())?.saturating_add(1);
        let mut usage = Vec::with_capacity(step_count.saturating_mul(2).saturating_add(1));
        usage.push(crate::implementation_attempt::repair::step::Usage {
            phase: crate::implementation_attempt::repair::step::Phase::Preparation,
            iteration: 0,
            model_requests: Some(0),
            tool_invocations: Some(0),
            provenance: crate::orchestration::UsageProvenance::HostMeasured,
            charged: crate::implementation_attempt::lineage::Draws::default(),
        });
        Ok(Self {
            frame,
            policy,
            state,
            base,
            parent_identity,
            usage,
            revisions: Vec::with_capacity(step_count),
            withdrawal: None,
            started_epoch_ms: None,
            finished_epoch_ms: None,
        })
    }

    fn elapsed_ms(&self, clock: &dyn crate::operation::shell::Clock) -> Option<u64> {
        super::observe(clock)?.checked_sub(self.started_epoch_ms?)
    }

    /// Run steps until the loop stops; bounded by [`MAX_TRANSITIONS`].
    pub(super) fn drive(
        &mut self,
        mut ports: super::Ports<'_>,
        started_epoch_ms: Option<u64>,
    ) -> Result<Disposition, String> {
        self.started_epoch_ms = started_epoch_ms;
        let records = self.frame.records;
        let mut next = self.state.next(self.elapsed_ms(ports.clock), records.is_cancel_requested());
        for _ in 0..MAX_TRANSITIONS {
            next = match next {
                Next::Stop(disposition) => {
                    self.finished_epoch_ms = super::observe(ports.clock);
                    return Ok(disposition);
                }
                Next::Repair => self.state.next(self.elapsed_ms(ports.clock), records.is_cancel_requested()),
                Next::Spawn(step) => self.spawn(&mut ports, &step)?,
                Next::Check { .. } => {
                    return Err("repair loop asked for checks outside a step".into());
                }
            };
        }
        Err("repair loop exceeded its transition bound".into())
    }

    fn spawn(&mut self, ports: &mut super::Ports<'_>, step: &StepRequest) -> Result<Next, String> {
        // r[impl lattice.implementation_attempt.consumer.scenario.fence_lock]
        // The step intent is created under the fence lock after a recheck; a
        // newer fence, or another attempt at this one, ends the loop before
        // this spawn. Every later step also rechecks the execution binding
        // and the managed revision under that lock, and with the standalone
        // operator's ports every step rechecks the operator's grant.
        let request = self.frame.request;
        let records = self.frame.records;
        let guard = super::super::session::fencing::Guard::acquire(self.frame.state_root, &request.envelope.work_id)?;
        if !guard.is_current(&request.envelope)? {
            return Ok(Next::Stop(Disposition::FenceSuperseded));
        }
        if let Some(withdrawal) = self.recheck_authority(step.iteration, ports.clock) {
            let cause = withdrawal.cause;
            self.withdrawal = Some(withdrawal);
            return Ok(Next::Stop(Disposition::AuthorityWithdrawn { cause }));
        }
        records.intend(step)?;
        drop(guard);
        // The worker is bounded by the time left now, after the possibly
        // blocking lock and the intent: a bound reached or a clock lost by
        // then ends the loop with this step's intent retained and its worker
        // never called.
        let remaining_ms = match self.state.remaining_ms(self.elapsed_ms(ports.clock)) {
            Ok(remaining_ms) => remaining_ms,
            Err(disposition) => return Ok(Next::Stop(disposition)),
        };
        let workspace = request.workspace_root.as_path();
        let deadline = std::time::Duration::from_millis(remaining_ms);
        let report = ports.worker.step(step, workspace, deadline, &|| records.is_cancel_requested());
        let charge = match self.state.start(step, &report) {
            Ok(charge) => charge,
            Err(continuation) => {
                return Ok(Next::Stop(Disposition::KilnOwned { continuation }));
            }
        };
        self.usage.push(charge);
        let candidate = self.candidate(step, &report)?;
        let revision = candidate.as_ref().map(|candidate| candidate.revision.revision.clone());
        let next = self.state.after_step(&report, revision, self.elapsed_ms(ports.clock));
        let (observations, copies, verdict, next) = match (next, &candidate) {
            (Next::Check { revision }, Some(candidate)) => {
                self.check(ports, step.iteration, &revision, &candidate.tree)?
            }
            (Next::Check { .. }, None) => return Err("repair loop asked for checks without a candidate".into()),
            (other, _) => (Vec::new(), Vec::new(), None, other),
        };
        records.retain(&format!("step-{}.json", step.iteration), &StepRecord {
            schema: crate::implementation_attempt::repair::retained::STEP_SCHEMA.to_owned(),
            step: step.clone(),
            report,
            candidate: candidate.as_ref().map(|candidate| candidate.revision.clone()),
            checks: observations,
            copies,
            verdict,
        })?;
        if let Some(candidate) = candidate {
            self.revisions.push(candidate.revision);
        }
        Ok(next)
    }

    /// The printed and retained observation of the run.
    pub(super) fn observation(
        &self,
        request: &super::super::Request,
        identity: &str,
        receipt: &super::super::binding::usage::WorkerReceipt,
        disposition: &Disposition,
    ) -> Result<serde_json::Value, String> {
        let lineage = request.lineage.as_ref();
        let observation = crate::implementation_attempt::repair::retained::Observation {
            schema: crate::implementation_attempt::repair::retained::OBSERVATION_SCHEMA.to_owned(),
            envelope: request.envelope.clone(),
            request_identity: request.admitted_envelope.clone(),
            worker_request: identity.to_owned(),
            execution_binding: receipt.admitted().binding_identity.clone(),
            repair_policy: self.policy.identity(),
            budget_lineage: lineage.map(crate::implementation_attempt::lineage::Binding::identity),
            disposition: disposition.clone(),
            steps: self.state.steps(),
            revisions: self.revisions.clone(),
            usage: self.usage.clone(),
            drawn: self.state.drawn().clone(),
            unaccounted_dimensions: lineage.map(crate::implementation_attempt::lineage::Binding::unaccounted),
            accepted: false,
            integrated: false,
            dispatch_plan: request.dispatch_plan.clone(),
            authority_withdrawn: self.withdrawal.clone(),
        };
        serde_json::to_value(&observation).map_err(|error| error.to_string())
    }
}

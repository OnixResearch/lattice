//! The repair loop over one attempt's injected ports.

// r[impl lattice.implementation_attempt.repair.scenario.dispositions]
// r[impl lattice.implementation_attempt.repair.scenario.bypass]
// r[impl lattice.implementation_attempt.lineage.scenario.usage]

/// Every iteration is a spawn, a check and a repair decision, plus the first
/// and last decisions: `3 * (MAX_REPAIR_ITERATIONS + 1) + 2`.
const MAX_TRANSITIONS: u32 = 29;

/// One host-derived candidate revision.
#[derive(Debug, Clone, serde::Serialize)]
struct Revision {
    iteration: u32,
    revision: String,
    parent: Option<String>,
    receipt: crate::candidate::CandidateReceipt,
}

/// The retained record of one step.
#[derive(serde::Serialize)]
struct StepRecord<'a> {
    schema: &'static str,
    step: &'a crate::implementation_attempt::repair::step::Request,
    report: &'a crate::implementation_attempt::repair::step::Report,
    candidate: Option<&'a Revision>,
    checks: &'a [crate::implementation_attempt::repair::checks::Observation],
    verdict: Option<&'a crate::implementation_attempt::repair::checks::Verdict>,
}

/// The loop over one attempt's ports.
pub(super) struct Driver<'a> {
    request: &'a super::super::Request,
    /// The state root whose fence lock guards every step intent.
    state_root: &'a std::path::Path,
    records: &'a super::records::Retained,
    policy: crate::implementation_attempt::repair::Policy,
    state: crate::implementation_attempt::repair::state::Loop,
    base: std::collections::BTreeMap<String, Vec<u8>>,
    parent_identity: String,
    usage: Vec<crate::implementation_attempt::repair::step::Usage>,
    revisions: Vec<Revision>,
    pub(super) finished_epoch_ms: Option<u64>,
}

impl<'a> Driver<'a> {
    /// Re-observe the admitted base before any worker effect. Preparation is
    /// a host step without model or tool use.
    pub(super) fn prepare(
        request: &'a super::super::Request,
        state_root: &'a std::path::Path,
        records: &'a super::records::Retained,
        policy: crate::implementation_attempt::repair::Policy,
    ) -> Result<Self, String> {
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
            request,
            state_root,
            records,
            policy,
            state,
            base,
            parent_identity,
            usage,
            revisions: Vec::with_capacity(step_count),
            finished_epoch_ms: None,
        })
    }

    fn elapsed_ms(clock: &dyn crate::operation::shell::Clock, started_epoch_ms: Option<u64>) -> Option<u64> {
        super::observe(clock)?.checked_sub(started_epoch_ms?)
    }

    /// Run steps until the loop stops; bounded by [`MAX_TRANSITIONS`].
    pub(super) fn drive(
        &mut self,
        mut ports: super::Ports<'_>,
        started_epoch_ms: Option<u64>,
    ) -> Result<crate::implementation_attempt::repair::state::Disposition, String> {
        let elapsed_ms = Self::elapsed_ms(ports.clock, started_epoch_ms);
        let mut next = self.state.next(elapsed_ms, self.records.is_cancel_requested());
        for _ in 0..MAX_TRANSITIONS {
            next = match next {
                crate::implementation_attempt::repair::state::Next::Stop(disposition) => {
                    self.finished_epoch_ms = super::observe(ports.clock);
                    return Ok(disposition);
                }
                crate::implementation_attempt::repair::state::Next::Repair => {
                    let elapsed_ms = Self::elapsed_ms(ports.clock, started_epoch_ms);
                    self.state.next(elapsed_ms, self.records.is_cancel_requested())
                }
                crate::implementation_attempt::repair::state::Next::Spawn(step) => self.spawn(&mut ports, &step)?,
                crate::implementation_attempt::repair::state::Next::Check { .. } => {
                    return Err("repair loop asked for checks outside a step".into());
                }
            };
        }
        Err("repair loop exceeded its transition bound".into())
    }

    fn spawn(
        &mut self,
        ports: &mut super::Ports<'_>,
        step: &crate::implementation_attempt::repair::step::Request,
    ) -> Result<crate::implementation_attempt::repair::state::Next, String> {
        // r[impl lattice.implementation_attempt.consumer.scenario.fence_lock]
        // The step intent is created under the fence lock after a recheck; a
        // newer fence, or another attempt at this one, ends the loop before
        // this spawn.
        let envelope = &self.request.envelope;
        let guard = super::super::session::fencing::Guard::acquire(self.state_root, &envelope.work_id)?;
        if !guard.is_current(envelope)? {
            let disposition = crate::implementation_attempt::repair::state::Disposition::FenceSuperseded;
            return Ok(crate::implementation_attempt::repair::state::Next::Stop(disposition));
        }
        self.records.intend(step)?;
        drop(guard);
        let records = self.records;
        let workspace = self.request.workspace_root.as_path();
        let report = ports.worker.step(step, workspace, &|| records.is_cancel_requested());
        let charge = match self.state.start(step, &report) {
            Ok(charge) => charge,
            Err(continuation) => {
                let disposition = crate::implementation_attempt::repair::state::Disposition::KilnOwned { continuation };
                return Ok(crate::implementation_attempt::repair::state::Next::Stop(disposition));
            }
        };
        self.usage.push(charge);
        let candidate = self.candidate(step, &report)?;
        let next = self.state.after_step(&report, candidate.as_ref().map(|revision| revision.revision.clone()));
        let (observations, verdict, next) = match next {
            crate::implementation_attempt::repair::state::Next::Check { revision } => {
                let invocations =
                    crate::implementation_attempt::repair::checks::invocations(&self.policy, &revision, step.iteration);
                let observations: Vec<_> =
                    invocations.iter().map(|invocation| ports.checks.run(invocation, workspace)).collect();
                self.check(step.iteration, &revision, observations)?
            }
            other => (Vec::new(), None, other),
        };
        self.records.retain(&format!("step-{}.json", step.iteration), &StepRecord {
            schema: super::records::STEP_SCHEMA,
            step,
            report: &report,
            candidate: candidate.as_ref(),
            checks: &observations,
            verdict: verdict.as_ref(),
        })?;
        if let Some(revision) = candidate {
            self.revisions.push(revision);
        }
        Ok(next)
    }

    fn check(
        &mut self,
        iteration: u32,
        revision: &str,
        observations: Vec<crate::implementation_attempt::repair::checks::Observation>,
    ) -> Result<
        (
            Vec<crate::implementation_attempt::repair::checks::Observation>,
            Option<crate::implementation_attempt::repair::checks::Verdict>,
            crate::implementation_attempt::repair::state::Next,
        ),
        String,
    > {
        let executions = u64::try_from(observations.len()).map_err(|error| error.to_string())?;
        self.usage.push(self.state.charge_checks(iteration, executions));
        let subject = crate::implementation_attempt::repair::checks::Subject {
            policy: &self.policy,
            revision,
            worker_profile: &self.request.envelope.worker_profile,
        };
        let verdict = crate::implementation_attempt::repair::checks::classify(subject, &observations);
        let next = self.state.after_checks(verdict.clone());
        Ok((observations, Some(verdict), next))
    }

    /// The host-derived candidate of an exited step: `None` when the
    /// workspace equals its parent or the base. A cancelled or uncertain
    /// step's workspace is retained, never captured as a candidate.
    fn candidate(
        &mut self,
        step: &crate::implementation_attempt::repair::step::Request,
        report: &crate::implementation_attempt::repair::step::Report,
    ) -> Result<Option<Revision>, String> {
        if !matches!(report.outcome, crate::implementation_attempt::repair::step::WorkerOutcome::Exited { .. }) {
            return Ok(None);
        }
        let snapshot =
            crate::candidate_capture::workspace(&self.request.workspace_root).map_err(|error| error.to_string())?;
        let identity = crate::candidate::snapshot_identity(&snapshot).map_err(|error| error.to_string())?;
        if identity == self.parent_identity {
            return Ok(None);
        }
        let envelope = &self.request.envelope;
        let fence = envelope.fence.to_string();
        let bindings = crate::candidate::Bindings {
            work_id: &envelope.work_id,
            attempt_id: &envelope.attempt_id,
            fence: &fence,
            workspace_id: &envelope.workspace_id,
        };
        let receipt = match crate::candidate::produced_receipt_from_snapshots(bindings, &self.base, &snapshot) {
            Ok(receipt) => receipt,
            Err(crate::candidate::CandidateReceiptError::NoCandidateChanges) => return Ok(None),
            Err(error) => return Err(error.to_string()),
        };
        self.parent_identity = identity;
        let parent = self.state.last_revision().map(str::to_owned);
        let revision = crate::implementation_attempt::repair::revision::identity(
            crate::implementation_attempt::repair::revision::Input {
                binding: &step.binding,
                iteration: step.iteration,
                parent: parent.as_deref(),
                candidate_identity: receipt.candidate_identity.as_deref().unwrap_or_default(),
                diff_identity: receipt.diff_identity.as_deref().unwrap_or_default(),
            },
        );
        Ok(Some(Revision {
            iteration: step.iteration,
            revision,
            parent,
            receipt,
        }))
    }

    /// The printed and retained observation of the run.
    pub(super) fn observation(
        &self,
        request: &super::super::Request,
        identity: &str,
        receipt: &super::super::binding::usage::WorkerReceipt,
        disposition: &crate::implementation_attempt::repair::state::Disposition,
    ) -> serde_json::Value {
        let lineage = request.lineage.as_ref();
        let mut observation = serde_json::json!({
            "schema": super::records::OBSERVATION_SCHEMA,
            "envelope": request.envelope,
            "request_identity": request.admitted_envelope,
            "worker_request": identity,
            "execution_binding": receipt.admitted().binding_identity,
            "repair_policy": self.policy.identity(),
            "budget_lineage": lineage.map(crate::implementation_attempt::lineage::Binding::identity),
            "disposition": disposition,
            "steps": self.state.steps(),
            "revisions": self.revisions,
            "usage": self.usage,
            "drawn": self.state.drawn(),
            "unaccounted_dimensions": lineage.map(crate::implementation_attempt::lineage::Binding::unaccounted),
            "accepted": false,
            "integrated": false,
        });
        if let (Some(plan), Some(fields)) = (&request.dispatch_plan, observation.as_object_mut()) {
            fields.insert("dispatch_plan".to_owned(), serde_json::Value::from(plan.as_str()));
        }
        observation
    }
}

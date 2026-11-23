//! Shell of the bounded same-attempt repair loop.
//!
//! A [`WorkerPort`] runs one bounded worker step in the attempt workspace, a
//! [`CheckPort`] runs one declared check, and a clock reads time. An
//! embedder supplies them, or the standalone CLI supplies its operator
//! ports (`operator`): the pinned OMP worker and the operator's check
//! catalog, used only while the operator's effect-authority profile grants
//! them. The shell owns everything else: the frozen policy and lineage, the
//! host-derived candidate of every step, the immutable copy of it each check
//! runs against, the deadlines, the check invocations, the charges and the
//! retained records. Without ports a request with a repair policy is refused
//! before any effect.

// r[impl lattice.implementation_attempt.repair.scenario.absent]
// r[impl lattice.implementation_attempt.lineage.scenario.exhausted]

pub(super) mod admission;
mod draw;
mod evidence;
mod materialized;
pub(crate) mod operator;
mod records;
mod steps;

/// Why no complete retained disposition of the exact request was read.
pub(super) type Unretained = records::Unretained;
/// One attempt's complete, request-bound retained repair records.
pub(super) type Evidence = evidence::RecordSet;

/// One bounded worker step of the attempt.
pub trait WorkerPort {
    /// Run `request` in `workspace` and stop it within `deadline`, the time
    /// left before the policy's elapsed bound. `is_cancel_requested` reports
    /// a persisted cancellation request; a failed observation reports `true`.
    /// The shell re-reads its clock afterwards: a step that returns past the
    /// bound is never checked.
    fn step(
        &mut self,
        request: &crate::implementation_attempt::repair::step::Request,
        workspace: &std::path::Path,
        deadline: std::time::Duration,
        is_cancel_requested: &dyn Fn() -> bool,
    ) -> crate::implementation_attempt::repair::step::Report;
}

/// The admitted host check executor.
pub trait CheckPort {
    /// Run one declared check against `candidate`, a read-only copy of the
    /// captured candidate revision that no other check sees, and stop it
    /// within `deadline`. The shell re-captures the copy afterwards and
    /// rejects the evidence as `candidate_drift` if it changed.
    fn run(
        &mut self,
        invocation: &crate::implementation_attempt::repair::checks::Invocation,
        candidate: &std::path::Path,
        deadline: std::time::Duration,
    ) -> crate::implementation_attempt::repair::checks::Observation;
}

/// Embedder-supplied repair ports.
pub struct Ports<'a> {
    pub worker: &'a mut dyn WorkerPort,
    pub checks: &'a mut dyn CheckPort,
    pub clock: &'a dyn crate::operation::shell::Clock,
}

/// The revocable authority every repair step after the first rechecks under
/// the fence lock, through the same binding and revision-seam checks that
/// ran before the first step.
pub(super) struct Authority<'a> {
    pub(super) compute: &'a mut dyn crate::execution_binding::ComputeAllocationPort,
    pub(super) admitted: lattice_core::execution_binding::admission::bound::Admitted,
    pub(super) revisions: super::session::revision::Store<'a>,
    pub(super) recorded: super::session::intent::Recorded,
    /// The standalone operator's grant, rechecked before every step's
    /// intent; `None` for embedder-supplied ports.
    pub(super) operator: Option<operator::Grant>,
}

/// Refuse a malformed policy or lineage, or a policy over served routes.
pub(super) fn validate(request: &super::Request) -> Result<(), String> {
    if let Some(lineage) = &request.lineage {
        lineage.validate().map_err(|error| error.to_string())?;
    }
    let Some(policy) = &request.repair else {
        return Ok(());
    };
    policy.validate(frame(request)).map_err(|error| error.to_string())?;
    if request.execution_binding.serving.is_some() {
        return Err("repair iterations over served model routes are unsupported".into());
    }
    Ok(())
}

fn frame(request: &super::Request) -> crate::implementation_attempt::repair::Frame<'_> {
    crate::implementation_attempt::repair::Frame {
        budget: &request.envelope.budget,
        step_model_requests: u64::from(request.omp.model_broker.max_requests),
        is_unchanged_allowed: request.allow_unchanged,
    }
}

/// The reservation of the first worker step: one broker ceiling of model
/// requests and, with a policy, its step and check tool invocations.
fn first_reservation(request: &super::Request) -> Result<crate::implementation_attempt::lineage::Draws, String> {
    let step_model_requests = u64::from(request.omp.model_broker.max_requests);
    match &request.repair {
        Some(policy) => policy.reservation(step_model_requests).map_err(|error| error.to_string()),
        None => Ok(crate::implementation_attempt::lineage::Draws {
            model_requests: step_model_requests,
            tool_invocations: 0,
        }),
    }
}

/// The single-shot attempt's lineage draw, when its request has a lineage.
pub(super) type SingleShotDraw = draw::SingleShot;

/// The frozen inputs of one repair run after its intent.
pub(super) struct Run<'a> {
    pub(super) request: &'a super::Request,
    /// The state root whose fence lock guards every step intent.
    pub(super) state: &'a std::path::Path,
    pub(super) directory: &'a std::path::Path,
    pub(super) identity: &'a str,
    pub(super) receipt: super::binding::usage::WorkerReceipt,
    pub(super) store: crate::store::Store,
    pub(super) authority: Authority<'a>,
}

/// Drive the loop to a disposition and retain it; returns the observation.
// r[impl lattice.implementation_attempt.repair]
pub(super) fn execute(run: Run<'_>, ports: Ports<'_>) -> Result<serde_json::Value, String> {
    let Some(policy) = run.request.repair.clone() else {
        return Err("repair execution requires an admitted repair policy".into());
    };
    let records = records::Retained::open(run.directory, run.identity)?;
    let started_epoch_ms = observe(ports.clock);
    let frame = steps::Frame {
        request: run.request,
        state_root: run.state,
        records: &records,
        authority: run.authority,
    };
    let mut driver = steps::Driver::prepare(frame, policy)?;
    let disposition = driver.drive(ports, started_epoch_ms)?;
    let window = super::binding::usage::WorkerWindow {
        started_epoch_ms,
        finished_epoch_ms: driver.finished_epoch_ms,
    };
    // The attempt usage observation precedes the retained disposition.
    debug_assert_eq!(run.receipt.request_identity(), run.identity, "the receipt binds this request");
    run.receipt.persist(&run.store, window, run.directory)?;
    let observation = driver.observation(run.request, run.identity, &run.receipt, &disposition)?;
    debug_assert!(observation.get("disposition").is_some(), "the observation names its disposition");
    records.retain_disposition(&observation)?;
    Ok(observation)
}

/// Reconcile a repair attempt from its retained disposition only; it never
/// spawns, checks or charges again.
pub(super) fn recover(
    request: &super::Request,
    directory: &std::path::Path,
    identity: &str,
    admitted: &lattice_core::execution_binding::admission::bound::Admitted,
) -> Result<serde_json::Value, String> {
    records::Retained::at(directory, identity).disposition(request, &admitted.binding_identity)
}

/// The complete, request-bound retained records of this exact request under
/// its persisted execution binding, with a typed reason when they are not.
/// It only reads.
pub(super) fn evidence(
    request: &super::Request,
    directory: &std::path::Path,
    identity: &str,
    admitted: &lattice_core::execution_binding::admission::bound::Admitted,
) -> Result<Evidence, Unretained> {
    evidence::read(request, directory, identity, admitted)
}

/// Run `decide` with what this request admits for replaying its retained
/// records; `None` without a repair policy.
pub(super) fn with_frame<T>(
    request: &super::Request,
    decide: impl FnOnce(&crate::implementation_attempt::repair::exportable::Frame<'_>) -> T,
) -> Option<T> {
    let policy = request.repair.as_ref()?;
    let fence = request.envelope.fence.to_string();
    let binding = crate::implementation_attempt::repair::StepBinding::of(&request.envelope);
    let frame = crate::implementation_attempt::repair::exportable::Frame {
        binding: &binding,
        receipt: crate::candidate::Bindings {
            work_id: &request.envelope.work_id,
            attempt_id: &request.envelope.attempt_id,
            fence: &fence,
            workspace_id: &request.envelope.workspace_id,
        },
        base_identity: &request.omp.base_identity,
        policy,
        worker_profile: &request.envelope.worker_profile,
    };
    Some(decide(&frame))
}

/// The disposition inventory v3 publishes for a repair attempt: the kind of
/// its retained disposition once the complete records agree with it and
/// prove every worker step it started has stopped, otherwise `None`, an
/// unknown outcome. A malformed, crossed or contradicted record is an error,
/// never a guessed state.
// r[impl lattice.implementation_attempt.repair_export.scenario.inventory]
pub(super) fn terminal_label(
    request: &super::Request,
    directory: &std::path::Path,
    identity: &str,
    admitted: Option<&lattice_core::execution_binding::admission::bound::Admitted>,
) -> Result<Option<String>, String> {
    debug_assert!(request.repair.is_some(), "only a repair attempt retains repair records");
    // Without a persisted execution binding nothing can bind the records.
    let Some(admitted) = admitted else {
        return Ok(None);
    };
    let evidence = match evidence::read(request, directory, identity, admitted) {
        Ok(evidence) => evidence,
        Err(Unretained::Missing | Unretained::Legacy) => return Ok(None),
        Err(unretained @ (Unretained::Malformed | Unretained::Crossed)) => return Err(unretained.message().into()),
    };
    let is_consistent = with_frame(request, |frame| {
        crate::implementation_attempt::repair::exportable::is_consistent(frame, evidence.view())
    });
    if is_consistent != Some(true) {
        return Err(Unretained::Malformed.message().into());
    }
    let disposition = &evidence.observation.disposition;
    if !crate::implementation_attempt::repair::retained::is_stopped(
        disposition,
        &evidence.steps,
        evidence.has_unrecorded_intent,
    ) {
        return Ok(None);
    }
    let value = serde_json::to_value(disposition).map_err(|error| error.to_string())?;
    let label = value.get("kind").and_then(serde_json::Value::as_str).ok_or(Unretained::Malformed.message())?;
    debug_assert!(!label.is_empty(), "a disposition names its kind");
    Ok(Some(label.to_owned()))
}

fn observe(clock: &dyn crate::operation::shell::Clock) -> Option<u64> {
    u64::try_from(clock.observe_epoch_millis()).ok()
}

//! Shell of the bounded same-attempt repair loop.
//!
//! The embedder supplies a [`WorkerPort`] that runs one bounded worker step in
//! the attempt workspace, a [`CheckPort`] that runs one declared check, and a
//! clock. The shell owns everything else: the frozen policy and lineage, the
//! host-derived candidate of every step, the immutable copy of it each check
//! runs against, the deadlines, the check invocations, the charges and the
//! retained records. The standalone CLI supplies no ports, so a request with
//! a repair policy is refused there before any effect.

// r[impl lattice.implementation_attempt.repair.scenario.absent]
// r[impl lattice.implementation_attempt.lineage.scenario.exhausted]

mod materialized;
mod records;
mod steps;

/// Why no retained disposition of the exact request was read.
pub(super) type Unretained = records::Unretained;

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

/// Before the fence record: the lineage must admit the first step, and a
/// repair policy needs embedder ports. Each refusal is typed; the caller
/// settles it under the per-work lock, which closes the attempt-fence.
pub(super) fn admit<'a>(
    request: &super::Request,
    ports: Option<Ports<'a>>,
) -> Result<Option<Ports<'a>>, super::refusal::deferred::Stop> {
    debug_assert!(request.validate_binding().is_ok(), "admission follows request validation");
    if let Some(lineage) = &request.lineage {
        let next = first_reservation(request)?;
        debug_assert!(next.model_requests > 0, "a step reserves at least one model request");
        let drawn = crate::implementation_attempt::lineage::Draws::default();
        if let Err(refusal) = lineage.gate(&drawn, &next) {
            let reason = match refusal {
                crate::implementation_attempt::lineage::Refusal::Exhausted { .. } => {
                    super::refusal::Reason::LineageExhausted
                }
                _ => super::refusal::Reason::LineageBlocked,
            };
            let pending = super::refusal::deferred::Pending::lineage(reason, &refusal);
            return Err(super::refusal::deferred::Stop::Refused(pending));
        }
    }
    match (&request.repair, ports) {
        (None, _) => Ok(None),
        (Some(_), Some(ports)) => Ok(Some(ports)),
        (Some(_), None) => Err(super::refusal::deferred::Stop::Refused(super::refusal::deferred::Pending::new(
            super::refusal::Reason::RepairPortsUnavailable,
        ))),
    }
}

/// The single-shot attempt's conservative lineage draw, when it has one.
pub(super) fn single_shot_draw(request: &super::Request) -> Option<serde_json::Value> {
    let lineage = request.lineage.as_ref()?;
    debug_assert!(request.repair.is_none(), "a repair attempt reports its own draws");
    let charged = first_reservation(request).ok()?;
    debug_assert_eq!(charged.tool_invocations, 0, "a single shot reserves no tool invocations");
    let usage = crate::implementation_attempt::repair::step::Usage {
        phase: crate::implementation_attempt::repair::step::Phase::Edit,
        iteration: 1,
        model_requests: None,
        tool_invocations: None,
        provenance: crate::orchestration::UsageProvenance::Unknown,
        charged,
    };
    let mut unaccounted = lineage.unaccounted();
    if lineage.remaining.contains_key(crate::implementation_attempt::lineage::TOOL_INVOCATIONS) {
        unaccounted.push(crate::implementation_attempt::lineage::TOOL_INVOCATIONS.to_owned());
    }
    Some(serde_json::json!({
        "binding": lineage.identity(),
        "usage": [usage],
        "unaccounted_dimensions": unaccounted,
    }))
}

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
    let observation = driver.observation(run.request, run.identity, &run.receipt, &disposition);
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
) -> Result<serde_json::Value, String> {
    records::Retained::at(directory, identity).disposition(request)
}

/// The retained disposition of this exact request, with a typed reason when
/// none is read. It only reads.
pub(super) fn retained(
    request: &super::Request,
    directory: &std::path::Path,
    identity: &str,
) -> Result<serde_json::Value, Unretained> {
    records::Retained::at(directory, identity).retained(request)
}

/// The retained record of step `iteration`, if it is readable JSON.
pub(super) fn step_record(directory: &std::path::Path, identity: &str, iteration: u32) -> Option<serde_json::Value> {
    records::Retained::at(directory, identity).step(iteration)
}

/// The disposition inventory v3 publishes for a repair attempt: the kind of
/// its retained disposition, or `None` while none is retained. A malformed or
/// crossed record is an error, never a guessed state.
// r[impl lattice.implementation_attempt.repair_export.scenario.inventory]
pub(super) fn terminal_label(
    request: &super::Request,
    directory: &std::path::Path,
    identity: &str,
) -> Result<Option<String>, String> {
    let observation = match retained(request, directory, identity) {
        Ok(observation) => observation,
        Err(Unretained::Missing) => return Ok(None),
        Err(unretained @ (Unretained::Malformed | Unretained::Crossed)) => return Err(unretained.message().into()),
    };
    let value = observation.get("disposition").ok_or(Unretained::Malformed.message())?;
    let _: crate::implementation_attempt::repair::state::Disposition =
        serde::Deserialize::deserialize(value).map_err(|_| Unretained::Malformed.message())?;
    let label = value.get("kind").and_then(serde_json::Value::as_str).ok_or(Unretained::Malformed.message())?;
    Ok(Some(label.to_owned()))
}

fn observe(clock: &dyn crate::operation::shell::Clock) -> Option<u64> {
    u64::try_from(clock.observe_epoch_millis()).ok()
}

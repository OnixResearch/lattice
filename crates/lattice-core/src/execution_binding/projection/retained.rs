//! `lattice.worker-instance-observation.v3`: one owned attempt's retained state.
//! Lifecycle and single-shot liveness come only from retained artifacts and
//! the shell's observation time; host reachability and allocation liveness
//! stay unknown, and model readiness is the injected port's latest snapshot or
//! why none was observed. The pinned revision of the attempt's session comes
//! from its recorded admission and the definition's lifecycle. Nothing here is
//! a claim; nothing reports idle or live.

// r[impl lattice.execution_binding.instances]
// r[impl lattice.execution_binding.instances.scenario.unknown]
// r[impl lattice.worker_inventory.liveness]
// r[impl lattice.worker_inventory.liveness.scenario.unknown]
// r[impl lattice.worker_inventory.liveness.scenario.single_shot]
// r[impl lattice.worker_inventory.revisions.scenario.pinned]

pub const INSTANCE_SCHEMA: &str = "lattice.worker-instance-observation.v3";
pub const STATE_SOURCE: &str = "lattice.implementation-attempt-state.v1";

type Code = super::wait::Code;
type Field = super::wait::FactField;
type Reason = super::wait::Reason;

/// Facts the shell read from one attempt directory. Reading failures never
/// reach this core; the shell reports them as typed errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttemptState<'a> {
    pub admitted: Option<&'a super::super::admission::bound::Admitted>,
    /// Identity of the exact request bytes (the worker incarnation).
    pub request_identity: &'a str,
    /// Definition identity of the profile the attempt was admitted under.
    pub definition_identity: &'a str,
    /// End of the request's admitted window; later missing data is stale.
    pub admitted_until_epoch_ms: u64,
    pub has_intent: bool,
    pub has_refusal: bool,
    /// A managed session was blocked by its revision lifecycle before spawn.
    pub has_revision_block: bool,
    pub terminal_disposition: Option<&'a str>,
    pub has_cancel_request: bool,
    /// `(artifact name, artifact identity)` for every present artifact.
    pub artifacts: &'a [(&'a str, &'a str)],
    /// Readiness from the injected serving-owner port, or why none was observed.
    pub serving_readiness: &'a super::ServingReadiness,
    /// The pinned revision of the attempt's session.
    pub pinned: &'a super::lineage::Slot<crate::agent_revision::projection::SessionPin>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Lifecycle {
    AdmittedNotDispatched,
    BindingRejectedBeforeDispatch,
    /// The session's pinned revision permitted no effect; nothing spawned.
    RevisionBlockedBeforeDispatch,
    DispatchRecordedOutcomeUnknown,
    TerminalObserved {
        disposition: String,
    },
}

/// Requested cancellation stays distinct from observed termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Cancellation {
    NotRequested,
    RequestedUnobserved,
    ObservedTerminated,
}

/// Lattice owns only single-shot attempt workers; none is reused or idles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeClass {
    SingleShot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LivenessState {
    NotStarted,
    /// A terminal result was observed.
    Exited,
    Unknown,
}

/// Why liveness is unknown: no owner observes it, or the admitted window passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownCause {
    Unobserved,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Liveness {
    pub runtime: RuntimeClass,
    pub state: LivenessState,
    pub unknown_cause: Option<UnknownCause>,
    /// Retained write-once artifacts plus the terminal result; never decreases.
    pub retained_generation: u64,
    pub observed_at_epoch_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct WorkerInstanceObservation {
    pub schema: &'static str,
    pub source: &'static str,
    pub definition_identity: String,
    pub profile_revision: String,
    pub work_id: String,
    pub attempt_id: String,
    pub incarnation: String,
    pub fence: u64,
    pub binding_identity: String,
    /// Admitted compute provenance; it never carries backend handles, grant
    /// references or producer record references.
    pub compute: super::super::admission::bound::AdmittedCompute,
    pub model_routes: super::super::request::RouteDeclaration,
    /// The pinned revision of this attempt's session.
    pub pinned: super::lineage::Slot<crate::agent_revision::projection::SessionPin>,
    pub lifecycle: Lifecycle,
    pub cancellation: Cancellation,
    pub liveness: Liveness,
    pub host_reachability: super::OwnedUnknown,
    pub model_readiness: super::ServingReadiness,
    pub allocation_liveness: super::OwnedUnknown,
    pub wait_reasons: Vec<Reason>,
    pub observed_at_epoch_ms: i64,
    pub state_digest: String,
}

/// Retained state that cannot describe one admitted execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StateDefect {
    IntentWithoutBinding,
    RefusalWithoutIntent,
    TerminalWithoutIntent,
    BindingNotIntact,
    InvalidToken,
    Digest,
    RefusalWithTerminal,
    Definition,
    WaitReasons,
}

/// Project retained state. `Ok(None)` means no retained execution exists.
pub fn project_instance(
    state: &AttemptState<'_>,
    observed_at_epoch_ms: i64,
) -> Result<Option<WorkerInstanceObservation>, StateDefect> {
    let Some(admitted) = state.admitted else {
        return if state.has_intent {
            Err(StateDefect::IntentWithoutBinding)
        } else {
            Ok(None)
        };
    };
    if !super::super::admission::bound::is_intact(admitted) {
        return Err(StateDefect::BindingNotIntact);
    }
    let is_bounded = super::super::is_token(state.request_identity)
        && state.terminal_disposition.is_none_or(super::super::is_token)
        && state.serving_readiness.is_bounded();
    if !is_bounded {
        return Err(StateDefect::InvalidToken);
    }
    if !super::super::is_reference(state.definition_identity) {
        return Err(StateDefect::Definition);
    }
    let lifecycle = super::attempt::lifecycle(state)?;
    let state_digest =
        crate::content_identity::worker_instance_state(state.artifacts).map_err(|_| StateDefect::Digest)?;
    let liveness = super::attempt::liveness(state, &lifecycle, observed_at_epoch_ms);
    let wait_reasons = instance_reasons(state, &lifecycle, &liveness)?;
    debug_assert!(super::super::admission::bound::is_intact(admitted));
    debug_assert!(super::super::is_token(state.request_identity));
    Ok(Some(observation(admitted, state, Derived {
        lifecycle,
        liveness,
        wait_reasons,
        state_digest,
    })))
}

/// Why a non-terminal instance is waiting; a finished one waits for nothing.
fn instance_reasons(
    state: &AttemptState<'_>,
    lifecycle: &Lifecycle,
    liveness: &Liveness,
) -> Result<Vec<Reason>, StateDefect> {
    let lifecycle_code = match lifecycle {
        Lifecycle::BindingRejectedBeforeDispatch
        | Lifecycle::RevisionBlockedBeforeDispatch
        | Lifecycle::TerminalObserved { .. } => return Ok(Vec::new()),
        Lifecycle::AdmittedNotDispatched if super::attempt::is_past_window(state, liveness.observed_at_epoch_ms) => {
            Code::AdmissionExpired
        }
        Lifecycle::AdmittedNotDispatched => Code::AwaitingDispatch,
        Lifecycle::DispatchRecordedOutcomeUnknown => Code::DispatchOutcomeUnknown,
    };
    let mut reasons = Vec::with_capacity(super::wait::MAX_WAIT_REASONS);
    reasons.push(Reason::fact(lifecycle_code, Field::Lifecycle));
    if super::attempt::cancellation(state) == Cancellation::RequestedUnobserved {
        reasons.push(Reason::fact(Code::CancellationUnobserved, Field::Cancellation));
    }
    match liveness.unknown_cause {
        Some(UnknownCause::Unobserved) => reasons.push(Reason::fact(Code::LivenessUnobserved, Field::Liveness)),
        Some(UnknownCause::Stale) => reasons.push(Reason::fact(Code::LivenessStale, Field::Liveness)),
        None => {}
    }
    reasons.push(Reason::fact(Code::HostReachabilityUnknown, Field::HostReachability));
    reasons.push(Reason::fact(Code::AllocationLivenessUnknown, Field::AllocationLiveness));
    debug_assert!(reasons.len() <= 5, "lifecycle, cancellation, liveness, host and allocation");
    debug_assert_ne!(liveness.state, LivenessState::Exited, "a finished instance waits for nothing");
    reasons.extend(super::wait::serving(state.serving_readiness));
    super::wait::finish(reasons).map_err(|_| StateDefect::WaitReasons)
}

/// Facts the core derived from one retained state.
struct Derived {
    lifecycle: Lifecycle,
    liveness: Liveness,
    wait_reasons: Vec<Reason>,
    state_digest: String,
}

fn observation(
    admitted: &super::super::admission::bound::Admitted,
    state: &AttemptState<'_>,
    derived: Derived,
) -> WorkerInstanceObservation {
    WorkerInstanceObservation {
        schema: INSTANCE_SCHEMA,
        source: STATE_SOURCE,
        definition_identity: state.definition_identity.to_owned(),
        profile_revision: admitted.profile_revision.clone(),
        work_id: admitted.attempt.work_id.clone(),
        attempt_id: admitted.attempt.attempt_id.clone(),
        incarnation: state.request_identity.to_owned(),
        fence: admitted.attempt.fence,
        binding_identity: admitted.binding_identity.clone(),
        compute: admitted.compute.clone(),
        model_routes: admitted.model_routes.clone(),
        pinned: state.pinned.clone(),
        lifecycle: derived.lifecycle,
        cancellation: super::attempt::cancellation(state),
        liveness: derived.liveness,
        host_reachability: super::OwnedUnknown::unknown(super::LIVE_HOST_OWNER),
        model_readiness: state.serving_readiness.clone(),
        allocation_liveness: super::OwnedUnknown::unknown(super::LIVE_HOST_OWNER),
        wait_reasons: derived.wait_reasons,
        observed_at_epoch_ms: derived.liveness.observed_at_epoch_ms,
        state_digest: derived.state_digest,
    }
}

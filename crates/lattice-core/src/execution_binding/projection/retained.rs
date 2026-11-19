//! `lattice.worker-instance-observation.v1`: one owned attempt's retained state.
//!
//! The lifecycle comes only from retained artifacts. Host reachability and
//! current allocation liveness stay unknown. Model readiness is the injected
//! serving-owner port's latest snapshot, or the actual reason none was
//! observed. Nothing here is a queue, claim or scheduling decision.

// r[impl lattice.execution_binding.instances]
// r[impl lattice.execution_binding.instances.scenario.unknown]

pub const INSTANCE_SCHEMA: &str = "lattice.worker-instance-observation.v1";
pub const STATE_SOURCE: &str = "lattice.implementation-attempt-state.v1";

/// Facts the shell read from one attempt directory. Reading failures never
/// reach this core; the shell reports them as typed errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttemptState<'a> {
    pub admitted: Option<&'a super::super::admission::bound::Admitted>,
    /// Identity of the exact request bytes (the worker incarnation).
    pub request_identity: &'a str,
    pub has_intent: bool,
    pub has_refusal: bool,
    pub terminal_disposition: Option<&'a str>,
    pub has_cancel_request: bool,
    /// `(artifact name, artifact identity)` for every present artifact.
    pub artifacts: &'a [(&'a str, &'a str)],
    /// Readiness from the injected serving-owner port, or why none was observed.
    pub serving_readiness: &'a super::ServingReadiness,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Lifecycle {
    AdmittedNotDispatched,
    BindingRejectedBeforeDispatch,
    DispatchRecordedOutcomeUnknown,
    TerminalObserved { disposition: String },
}

/// Requested cancellation stays distinct from observed termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Cancellation {
    NotRequested,
    RequestedUnobserved,
    ObservedTerminated,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct WorkerInstanceObservation {
    pub schema: &'static str,
    pub source: &'static str,
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
    pub lifecycle: Lifecycle,
    pub cancellation: Cancellation,
    pub host_reachability: super::OwnedUnknown,
    pub model_readiness: super::ServingReadiness,
    pub allocation_liveness: super::OwnedUnknown,
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
    let lifecycle = lifecycle(state)?;
    let state_digest =
        crate::content_identity::worker_instance_state(state.artifacts).map_err(|_| StateDefect::Digest)?;
    debug_assert!(super::super::admission::bound::is_intact(admitted));
    debug_assert!(super::super::is_token(state.request_identity));
    Ok(Some(observation(admitted, state, lifecycle, observed_at_epoch_ms, state_digest)))
}

fn lifecycle(state: &AttemptState<'_>) -> Result<Lifecycle, StateDefect> {
    match (state.has_intent, state.has_refusal, state.terminal_disposition) {
        (false, false, None) => Ok(Lifecycle::AdmittedNotDispatched),
        (false, true, None) => Err(StateDefect::RefusalWithoutIntent),
        (false, _, Some(_)) => Err(StateDefect::TerminalWithoutIntent),
        (true, true, Some(_)) => Err(StateDefect::RefusalWithTerminal),
        (true, true, None) => Ok(Lifecycle::BindingRejectedBeforeDispatch),
        (true, false, None) => Ok(Lifecycle::DispatchRecordedOutcomeUnknown),
        (true, false, Some(disposition)) => Ok(Lifecycle::TerminalObserved {
            disposition: disposition.to_owned(),
        }),
    }
}

fn cancellation(state: &AttemptState<'_>) -> Cancellation {
    if state.terminal_disposition == Some("cancelled") {
        Cancellation::ObservedTerminated
    } else if state.has_cancel_request {
        Cancellation::RequestedUnobserved
    } else {
        Cancellation::NotRequested
    }
}

fn observation(
    admitted: &super::super::admission::bound::Admitted,
    state: &AttemptState<'_>,
    lifecycle: Lifecycle,
    observed_at_epoch_ms: i64,
    state_digest: String,
) -> WorkerInstanceObservation {
    WorkerInstanceObservation {
        schema: INSTANCE_SCHEMA,
        source: STATE_SOURCE,
        profile_revision: admitted.profile_revision.clone(),
        work_id: admitted.attempt.work_id.clone(),
        attempt_id: admitted.attempt.attempt_id.clone(),
        incarnation: state.request_identity.to_owned(),
        fence: admitted.attempt.fence,
        binding_identity: admitted.binding_identity.clone(),
        compute: admitted.compute.clone(),
        model_routes: admitted.model_routes.clone(),
        lifecycle,
        cancellation: cancellation(state),
        host_reachability: super::OwnedUnknown::unknown(super::LIVE_HOST_OWNER),
        model_readiness: state.serving_readiness.clone(),
        allocation_liveness: super::OwnedUnknown::unknown(super::LIVE_HOST_OWNER),
        observed_at_epoch_ms,
        state_digest,
    }
}

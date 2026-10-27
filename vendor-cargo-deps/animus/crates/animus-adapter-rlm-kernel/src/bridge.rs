use alloc::{
    string::{String, ToString},
    vec::Vec,
};

const BLAKE3_HEX_CHARS: usize = 64;
const BRIDGE_CALL_IDENTITY_DOMAIN: &str = "animus.rlm-bridge-call.v1";
const BRIDGE_RESULT_IDENTITY_DOMAIN: &str = "animus.rlm-bridge-result.v1";
const HOST_REQUEST_IDENTITY_DOMAIN: &str = "animus.rlm-host-request.v1";

/// Process-local lifecycle state for one staged Steel bridge cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeState {
    /// Steel evaluation can produce a final value or one host request.
    Evaluating,
    /// The cell waits for the exact correlated host result.
    AwaitingHostResult,
    /// Steel evaluation will resume with all prior results replayed.
    Resuming,
    /// The staged cell committed successfully.
    Completed,
    /// The staged cell failed and cannot commit.
    Failed,
    /// Cancellation stopped the staged cell.
    Cancelled,
}

/// Stable failure class for a bridged cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeFailureClass {
    /// Current authority denied a host request.
    AuthorityDenied,
    /// The injected host executor failed.
    HostFailure,
    /// A result did not match the pending call.
    CorrelationMismatch,
    /// Cancellation stopped evaluation.
    Cancelled,
    /// The admitted wall-time bound expired.
    Timeout,
    /// A finite policy bound was exceeded.
    BoundExceeded,
    /// Steel evaluation failed.
    SteelFailure,
    /// A host result violated the bounded result contract.
    ResultInvalid,
}

impl BridgeFailureClass {
    /// Returns the stable failure label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AuthorityDenied => "authority_denied",
            Self::HostFailure => "host_failure",
            Self::CorrelationMismatch => "correlation_mismatch",
            Self::Cancelled => "cancelled",
            Self::Timeout => "timeout",
            Self::BoundExceeded => "bound_exceeded",
            Self::SteelFailure => "steel_failure",
            Self::ResultInvalid => "result_invalid",
        }
    }
}

/// Stable session and cell identities for one bridge run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCellIdentity {
    session_identity: String,
    cell_identity: String,
}

impl BridgeCellIdentity {
    /// Validates BLAKE3 session and cell identities.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when either identity is malformed.
    pub fn new(session_identity: &str, cell_identity: &str) -> Result<Self, crate::Error> {
        validate_identity("bridge_session_identity", session_identity)?;
        validate_identity("bridge_cell_identity", cell_identity)?;
        Ok(Self {
            session_identity: String::from(session_identity),
            cell_identity: String::from(cell_identity),
        })
    }

    /// Returns the session identity.
    #[must_use]
    pub fn session_identity(&self) -> &str {
        self.session_identity.as_str()
    }

    /// Returns the cell identity.
    #[must_use]
    pub fn cell_identity(&self) -> &str {
        self.cell_identity.as_str()
    }
}

/// Exact correlation for one ordered host call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCorrelation {
    cell: BridgeCellIdentity,
    sequence: u32,
    call_identity: String,
}

impl BridgeCorrelation {
    /// Returns the bridge cell identity.
    #[must_use]
    pub const fn cell(&self) -> &BridgeCellIdentity {
        &self.cell
    }

    /// Returns the zero-based call sequence.
    #[must_use]
    pub const fn sequence(&self) -> u32 {
        self.sequence
    }

    /// Returns the domain-separated call identity.
    #[must_use]
    pub fn call_identity(&self) -> &str {
        self.call_identity.as_str()
    }
}

/// One admitted host call that waits for execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgePendingCall {
    correlation: BridgeCorrelation,
    request: crate::HostRequest,
    admission: crate::HostRequestAdmission,
    request_identity: String,
}

impl BridgePendingCall {
    /// Returns the exact correlation.
    #[must_use]
    pub const fn correlation(&self) -> &BridgeCorrelation {
        &self.correlation
    }

    /// Returns the typed host request.
    #[must_use]
    pub const fn request(&self) -> &crate::HostRequest {
        &self.request
    }

    /// Returns the pure current admission.
    #[must_use]
    pub const fn admission(&self) -> &crate::HostRequestAdmission {
        &self.admission
    }

    /// Returns the request identity without exposing request content.
    #[must_use]
    pub fn request_identity(&self) -> &str {
        self.request_identity.as_str()
    }
}

/// Metadata-only observation for one completed bridge call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCallObservation {
    correlation: BridgeCorrelation,
    operation: crate::HostOperationClass,
    request_identity: String,
    result_identity: String,
    result_chars: u32,
}

impl BridgeCallObservation {
    /// Returns the exact safe correlation.
    #[must_use]
    pub const fn correlation(&self) -> &BridgeCorrelation {
        &self.correlation
    }

    /// Returns the operation class.
    #[must_use]
    pub const fn operation(&self) -> crate::HostOperationClass {
        self.operation
    }

    /// Returns the request identity.
    #[must_use]
    pub fn request_identity(&self) -> &str {
        self.request_identity.as_str()
    }

    /// Returns the result identity.
    #[must_use]
    pub fn result_identity(&self) -> &str {
        self.result_identity.as_str()
    }

    /// Returns the result character count.
    #[must_use]
    pub const fn result_chars(&self) -> u32 {
        self.result_chars
    }
}

/// Borrowed private durable replay parts for one completed bridge call.
#[derive(Clone, Copy, Debug)]
pub struct BridgeReplayCallDraft<'a> {
    /// Exact bridge cell identity.
    pub cell: &'a BridgeCellIdentity,
    /// Zero-based call sequence.
    pub sequence: u32,
    /// Domain-separated call identity.
    pub call_identity: &'a str,
    /// Typed host operation class.
    pub operation: crate::HostOperationClass,
    /// Domain-separated request identity.
    pub request_identity: &'a str,
    /// Private bounded result body.
    pub result_value: &'a str,
    /// Domain-separated result identity.
    pub result_identity: &'a str,
}

/// One completed private bridge result and its safe identities.
#[derive(Clone, Eq, PartialEq)]
pub struct BridgeCompletedCall {
    correlation: BridgeCorrelation,
    operation: crate::HostOperationClass,
    request_identity: String,
    result_value: String,
    result_identity: String,
    result_chars: u32,
}

impl core::fmt::Debug for BridgeCompletedCall {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("BridgeCompletedCall")
            .field("correlation", &self.correlation)
            .field("operation", &self.operation)
            .field("request_identity", &self.request_identity)
            .field("result_value", &"<private>")
            .field("result_identity", &self.result_identity)
            .field("result_chars", &self.result_chars)
            .finish()
    }
}

impl BridgeCompletedCall {
    /// Reconstructs one private completed call from a durable replay record.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when correlation, operation, request identity,
    /// result identity, sequence, or result bounds do not match current policy.
    pub fn from_replay_parts(
        draft: BridgeReplayCallDraft<'_>,
        policy: &crate::RlmKernelPolicy,
    ) -> Result<Self, crate::Error> {
        validate_identity("bridge_call_identity", draft.call_identity)?;
        validate_identity("bridge_request_identity", draft.request_identity)?;
        validate_identity("bridge_result_identity", draft.result_identity)?;
        let sequence_text = draft.sequence.to_string();
        let expected_call_identity = crate::hash_parts(
            BRIDGE_CALL_IDENTITY_DOMAIN,
            &[
                draft.cell.session_identity(),
                draft.cell.cell_identity(),
                sequence_text.as_str(),
                draft.request_identity,
                policy.identity(),
            ],
        );
        if expected_call_identity != draft.call_identity {
            return Err(crate::Error::BridgeCorrelationMismatch);
        }
        let result_value = crate::required_text(
            "bridge_host_result",
            draft.result_value,
            policy.bounds().max_host_result_chars,
        )?;
        let result_chars = crate::usize_to_u32(result_value.chars().count());
        let expected_result_identity = crate::hash_parts(
            BRIDGE_RESULT_IDENTITY_DOMAIN,
            &[draft.call_identity, result_value.as_str()],
        );
        if expected_result_identity != draft.result_identity {
            return Err(crate::Error::BridgeCorrelationMismatch);
        }
        Ok(Self {
            correlation: BridgeCorrelation {
                cell: draft.cell.clone(),
                sequence: draft.sequence,
                call_identity: String::from(draft.call_identity),
            },
            operation: draft.operation,
            request_identity: String::from(draft.request_identity),
            result_value,
            result_identity: String::from(draft.result_identity),
            result_chars,
        })
    }

    /// Returns the exact correlation.
    #[must_use]
    pub const fn correlation(&self) -> &BridgeCorrelation {
        &self.correlation
    }

    /// Returns the operation class.
    #[must_use]
    pub const fn operation(&self) -> crate::HostOperationClass {
        self.operation
    }

    /// Returns the request identity.
    #[must_use]
    pub fn request_identity(&self) -> &str {
        self.request_identity.as_str()
    }

    /// Returns the private bounded result for Steel replay.
    #[must_use]
    pub fn result_value(&self) -> &str {
        self.result_value.as_str()
    }

    /// Returns the result identity.
    #[must_use]
    pub fn result_identity(&self) -> &str {
        self.result_identity.as_str()
    }

    /// Returns the result character count.
    #[must_use]
    pub const fn result_chars(&self) -> u32 {
        self.result_chars
    }

    /// Projects a metadata-only observation without the private result value.
    #[must_use]
    pub fn observation(&self) -> BridgeCallObservation {
        BridgeCallObservation {
            correlation: self.correlation.clone(),
            operation: self.operation,
            request_identity: self.request_identity.clone(),
            result_identity: self.result_identity.clone(),
            result_chars: self.result_chars,
        }
    }
}

/// Pure state for one bounded bridge run.
#[derive(Clone, Eq, PartialEq)]
pub struct BridgeRunState {
    cell: BridgeCellIdentity,
    state: BridgeState,
    next_sequence: u32,
    call_count: u32,
    cumulative_result_chars: u32,
    consumed_evaluation_fuel: u64,
    nesting_depth: u32,
    pending: Option<BridgePendingCall>,
    completed_calls: Vec<BridgeCompletedCall>,
    failure: Option<BridgeFailureClass>,
}

impl core::fmt::Debug for BridgeRunState {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let pending = self.pending.as_ref().map(BridgePendingCall::correlation);
        let completed_calls = self
            .completed_calls
            .iter()
            .map(BridgeCompletedCall::observation)
            .collect::<Vec<_>>();
        formatter
            .debug_struct("BridgeRunState")
            .field("cell", &self.cell)
            .field("state", &self.state)
            .field("next_sequence", &self.next_sequence)
            .field("call_count", &self.call_count)
            .field("cumulative_result_chars", &self.cumulative_result_chars)
            .field("consumed_evaluation_fuel", &self.consumed_evaluation_fuel)
            .field("nesting_depth", &self.nesting_depth)
            .field("pending", &pending)
            .field("completed_calls", &completed_calls)
            .field("failure", &self.failure)
            .finish()
    }
}

impl BridgeRunState {
    /// Starts one staged bridge run under finite policy bounds.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] for malformed identities or excessive nesting.
    pub fn new(
        cell: BridgeCellIdentity,
        nesting_depth: u32,
        policy: &crate::RlmKernelPolicy,
    ) -> Result<Self, crate::Error> {
        crate::ensure_at_most(
            "bridge_nesting_depth",
            u64::from(nesting_depth),
            u64::from(policy.bounds().max_bridge_nesting),
        )?;
        Ok(Self {
            cell,
            state: BridgeState::Evaluating,
            next_sequence: 0,
            call_count: 0,
            cumulative_result_chars: 0,
            consumed_evaluation_fuel: 0,
            nesting_depth,
            pending: None,
            completed_calls: Vec::new(),
            failure: None,
        })
    }

    /// Returns the current bridge state.
    #[must_use]
    pub const fn state(&self) -> BridgeState {
        self.state
    }

    /// Returns the bridge cell identity.
    #[must_use]
    pub const fn cell(&self) -> &BridgeCellIdentity {
        &self.cell
    }

    /// Returns the completed call count.
    #[must_use]
    pub const fn call_count(&self) -> u32 {
        self.call_count
    }

    /// Returns cumulative private result characters.
    #[must_use]
    pub const fn cumulative_result_chars(&self) -> u32 {
        self.cumulative_result_chars
    }

    /// Returns cumulative deterministic evaluator fuel.
    #[must_use]
    pub const fn consumed_evaluation_fuel(&self) -> u64 {
        self.consumed_evaluation_fuel
    }

    /// Returns the bridge nesting depth.
    #[must_use]
    pub const fn nesting_depth(&self) -> u32 {
        self.nesting_depth
    }

    /// Returns the pending call, if present.
    #[must_use]
    pub const fn pending(&self) -> Option<&BridgePendingCall> {
        self.pending.as_ref()
    }

    /// Returns completed private replay calls in sequence order.
    #[must_use]
    pub fn completed_calls(&self) -> &[BridgeCompletedCall] {
        self.completed_calls.as_slice()
    }

    /// Returns the terminal failure class, if present.
    #[must_use]
    pub const fn failure(&self) -> Option<BridgeFailureClass> {
        self.failure
    }

    /// Records one deterministic evaluator attempt before Steel runs.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the state is terminal or cumulative fuel exceeds policy.
    pub fn record_evaluation_attempt(
        &mut self,
        fuel: u64,
        policy: &crate::RlmKernelPolicy,
    ) -> Result<(), crate::Error> {
        require_runnable(self.state)?;
        let next = self.consumed_evaluation_fuel.saturating_add(fuel);
        crate::ensure_at_most(
            "bridge_evaluation_fuel",
            next,
            policy.bounds().max_bridge_evaluation_fuel,
        )?;
        self.consumed_evaluation_fuel = next;
        self.state = if self.call_count == 0 {
            BridgeState::Evaluating
        } else {
            BridgeState::Resuming
        };
        Ok(())
    }

    /// Records one exact pending host request after pure admission.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] for stale admission, invalid state, or call-bound exhaustion.
    pub fn await_host_request(
        &mut self,
        request: &crate::HostRequest,
        supplied_admission: &crate::HostRequestAdmission,
        policy: &crate::RlmKernelPolicy,
    ) -> Result<BridgePendingCall, crate::Error> {
        require_runnable(self.state)?;
        if self.pending.is_some() {
            return Err(crate::Error::BridgeStateConflict);
        }
        let current = crate::admit_host_request(policy, request)?;
        if current != *supplied_admission {
            return Err(crate::Error::BridgeCorrelationMismatch);
        }
        crate::ensure_at_most(
            "bridge_host_calls",
            u64::from(self.call_count.saturating_add(1)),
            u64::from(policy.bounds().max_host_calls_per_cell),
        )?;
        let request_identity = host_request_identity(request);
        let sequence_text = self.next_sequence.to_string();
        let call_identity = crate::hash_parts(
            BRIDGE_CALL_IDENTITY_DOMAIN,
            &[
                self.cell.session_identity(),
                self.cell.cell_identity(),
                sequence_text.as_str(),
                request_identity.as_str(),
                policy.identity(),
            ],
        );
        let pending = BridgePendingCall {
            correlation: BridgeCorrelation {
                cell: self.cell.clone(),
                sequence: self.next_sequence,
                call_identity,
            },
            request: request.clone(),
            admission: current,
            request_identity,
        };
        self.pending = Some(pending.clone());
        self.state = BridgeState::AwaitingHostResult;
        Ok(pending)
    }

    /// Accepts one exact bounded host result and prepares the next resume.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] for stale correlation, invalid state, or result bounds.
    pub fn accept_host_result(
        &mut self,
        correlation: &BridgeCorrelation,
        value: &str,
        policy: &crate::RlmKernelPolicy,
    ) -> Result<BridgeCompletedCall, crate::Error> {
        if self.state != BridgeState::AwaitingHostResult {
            return Err(crate::Error::BridgeStateConflict);
        }
        let pending = self
            .pending
            .as_ref()
            .ok_or(crate::Error::BridgeStateConflict)?;
        if pending.correlation != *correlation {
            return Err(crate::Error::BridgeCorrelationMismatch);
        }
        let value = crate::required_text(
            "bridge_host_result",
            value,
            policy.bounds().max_host_result_chars,
        )?;
        let result_chars = crate::usize_to_u32(value.chars().count());
        let cumulative = self.cumulative_result_chars.saturating_add(result_chars);
        crate::ensure_at_most(
            "bridge_cumulative_result_chars",
            u64::from(cumulative),
            u64::from(policy.bounds().max_cumulative_host_result_chars),
        )?;
        let result_identity = crate::hash_parts(
            BRIDGE_RESULT_IDENTITY_DOMAIN,
            &[pending.correlation.call_identity(), value.as_str()],
        );
        let completed = BridgeCompletedCall {
            correlation: pending.correlation.clone(),
            operation: pending.request.operation_class(),
            request_identity: pending.request_identity.clone(),
            result_value: value,
            result_identity,
            result_chars,
        };
        self.pending = None;
        self.completed_calls.push(completed.clone());
        self.call_count = self.call_count.saturating_add(1);
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.cumulative_result_chars = cumulative;
        self.state = BridgeState::Resuming;
        Ok(completed)
    }

    /// Checks cancellation and elapsed wall time at one shell checkpoint.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] after moving the state to cancelled or failed.
    pub fn observe_control(
        &mut self,
        cancelled: bool,
        elapsed_ms: u64,
        policy: &crate::RlmKernelPolicy,
    ) -> Result<(), crate::Error> {
        require_runnable_or_waiting(self.state)?;
        if cancelled {
            self.state = BridgeState::Cancelled;
            self.failure = Some(BridgeFailureClass::Cancelled);
            return Err(crate::Error::BridgeCancelled);
        }
        if elapsed_ms > policy.bounds().max_bridge_wall_time_ms {
            self.state = BridgeState::Failed;
            self.failure = Some(BridgeFailureClass::Timeout);
            return Err(crate::Error::BridgeTimedOut);
        }
        Ok(())
    }

    /// Marks a successful final Steel value.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when a call is pending or the run is terminal.
    pub fn complete(&mut self) -> Result<(), crate::Error> {
        require_runnable(self.state)?;
        if self.pending.is_some() {
            return Err(crate::Error::BridgeStateConflict);
        }
        self.state = BridgeState::Completed;
        Ok(())
    }

    /// Marks a typed terminal failure without erasing completed calls.
    pub fn fail(&mut self, failure: BridgeFailureClass) {
        self.pending = None;
        self.failure = Some(failure);
        self.state = if failure == BridgeFailureClass::Cancelled {
            BridgeState::Cancelled
        } else {
            BridgeState::Failed
        };
    }
}

/// Returns a stable BLAKE3 identity for one typed host request.
#[must_use]
pub fn host_request_identity(request: &crate::HostRequest) -> String {
    let operation = request.operation_class().as_str();
    match request {
        crate::HostRequest::FileRead { target } => {
            crate::hash_parts(HOST_REQUEST_IDENTITY_DOMAIN, &[operation, target])
        }
        crate::HostRequest::FileEdit {
            target,
            replacement,
        } => crate::hash_parts(
            HOST_REQUEST_IDENTITY_DOMAIN,
            &[operation, target, replacement],
        ),
        crate::HostRequest::Shell { command } => {
            crate::hash_parts(HOST_REQUEST_IDENTITY_DOMAIN, &[operation, command])
        }
        crate::HostRequest::Tool { name, payload } => {
            crate::hash_parts(HOST_REQUEST_IDENTITY_DOMAIN, &[operation, name, payload])
        }
        crate::HostRequest::Subagent(request) => {
            let depth = request.depth().to_string();
            let fan_out = request.fan_out().to_string();
            let token_budget = request.token_budget().to_string();
            let wall_time_ms = request.wall_time_ms().to_string();
            let result_char_budget = request.result_char_budget().to_string();
            let ambient_discovery = request.ambient_discovery().to_string();
            let cross_session = request.cross_session().to_string();
            crate::hash_parts(
                HOST_REQUEST_IDENTITY_DOMAIN,
                &[
                    operation,
                    request.profile(),
                    request.context_slice(),
                    depth.as_str(),
                    fan_out.as_str(),
                    token_budget.as_str(),
                    wall_time_ms.as_str(),
                    result_char_budget.as_str(),
                    ambient_discovery.as_str(),
                    cross_session.as_str(),
                ],
            )
        }
        crate::HostRequest::ContextRead { key } => {
            crate::hash_parts(HOST_REQUEST_IDENTITY_DOMAIN, &[operation, key])
        }
        crate::HostRequest::KernelClear => {
            crate::hash_parts(HOST_REQUEST_IDENTITY_DOMAIN, &[operation])
        }
    }
}

fn validate_identity(field: &'static str, value: &str) -> Result<(), crate::Error> {
    if crate::is_lower_hex(value, BLAKE3_HEX_CHARS) {
        Ok(())
    } else {
        Err(crate::Error::InvalidBlake3Identity { field })
    }
}

fn require_runnable(state: BridgeState) -> Result<(), crate::Error> {
    if matches!(state, BridgeState::Evaluating | BridgeState::Resuming) {
        Ok(())
    } else {
        Err(crate::Error::BridgeStateConflict)
    }
}

fn require_runnable_or_waiting(state: BridgeState) -> Result<(), crate::Error> {
    if matches!(
        state,
        BridgeState::Evaluating | BridgeState::AwaitingHostResult | BridgeState::Resuming
    ) {
        Ok(())
    } else {
        Err(crate::Error::BridgeStateConflict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const ID_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const GIT_REVISION: &str = "c5991bc853d27754aed345c13ff4d2e05c40f1f4";
    const RESULT: &str = "bounded result";
    const SOURCE_LIMIT: u32 = 4_096;
    const ENVIRONMENT_LIMIT: u32 = 16_384;
    const COUNT_LIMIT: u32 = 16;
    const EVALUATION_FUEL_LIMIT: u64 = 100_000;
    const RECURSION_LIMIT: u32 = 64;
    const ALLOCATION_LIMIT: u64 = 1_048_576;
    const HOST_REQUEST_LIMIT: u32 = 8_192;
    const CHILD_DEPTH_LIMIT: u32 = 2;
    const CHILD_FAN_OUT_LIMIT: u32 = 4;
    const CHILD_TOKEN_LIMIT: u64 = 16_000;
    const CHILD_WALL_TIME_LIMIT_MS: u64 = 60_000;
    const CHILD_RESULT_LIMIT: u32 = 8_192;
    const HOST_CALL_LIMIT: u32 = 4;
    const HOST_RESULT_LIMIT: u32 = 32_768;
    const CUMULATIVE_HOST_RESULT_LIMIT: u32 = 65_536;
    const BRIDGE_NESTING_LIMIT: u32 = 4;
    const BRIDGE_EVALUATION_FUEL_LIMIT: u64 = 500_000;
    const BRIDGE_WALL_TIME_LIMIT_MS: u64 = 120_000;
    const FIRST_NESTING_DEPTH: u32 = 1;

    #[test]
    fn exact_result_advances_the_pure_bridge_state() {
        // r[verify agent_harness_runtime.steel_host_bridge.continuation.multiple]
        let policy = policy();
        let cell = BridgeCellIdentity::new(ID_A, ID_B).expect("identities must parse");
        let mut state =
            BridgeRunState::new(cell, FIRST_NESTING_DEPTH, &policy).expect("run must start");
        state
            .record_evaluation_attempt(FIRST_NESTING_DEPTH.into(), &policy)
            .expect("fuel must admit");
        let request = crate::HostRequest::context_read("task_context").expect("request must parse");
        let admission = crate::admit_host_request(&policy, &request).expect("request must admit");
        let pending = state
            .await_host_request(&request, &admission, &policy)
            .expect("call must wait");
        let completed = state
            .accept_host_result(pending.correlation(), RESULT, &policy)
            .expect("result must correlate");
        assert_eq!(completed.result_value(), RESULT);
        assert!(!alloc::format!("{completed:?}").contains(RESULT));
        assert!(!alloc::format!("{state:?}").contains(RESULT));
        assert_eq!(state.state(), BridgeState::Resuming);
        assert_eq!(state.call_count(), 1);
    }

    #[test]
    fn stale_result_and_bounds_fail_closed() {
        // r[verify agent_harness_runtime.steel_host_bridge.continuation.stale]
        // r[verify agent_harness_runtime.steel_host_bridge.bounds.exceeded]
        let policy = policy();
        let cell = BridgeCellIdentity::new(ID_A, ID_B).expect("identities must parse");
        let mut state = BridgeRunState::new(cell.clone(), FIRST_NESTING_DEPTH, &policy)
            .expect("run must start");
        let request = crate::HostRequest::context_read("task_context").expect("request must parse");
        let admission = crate::admit_host_request(&policy, &request).expect("request must admit");
        let pending = state
            .await_host_request(&request, &admission, &policy)
            .expect("call must wait");
        let stale_cell = BridgeCellIdentity::new(ID_B, ID_A).expect("identities must parse");
        let mismatched_correlation = BridgeCorrelation {
            cell: stale_cell,
            sequence: pending.correlation().sequence(),
            call_identity: String::from(pending.correlation().call_identity()),
        };
        assert_eq!(
            state.accept_host_result(&mismatched_correlation, RESULT, &policy),
            Err(crate::Error::BridgeCorrelationMismatch)
        );
        assert_eq!(state.state(), BridgeState::AwaitingHostResult);

        let nested = BridgeRunState::new(cell, BRIDGE_NESTING_LIMIT + 1, &policy);
        assert!(matches!(nested, Err(crate::Error::BoundExceeded { .. })));
    }

    fn policy() -> crate::RlmKernelPolicy {
        crate::RlmKernelPolicy::from_draft(&crate::RlmKernelPolicyDraft {
            schema: crate::RLM_KERNEL_POLICY_SCHEMA,
            cohort: "prime-agent-rlm-bridge",
            cohort_revision: GIT_REVISION,
            profile: "model-facing-steel",
            bounds: crate::ResourceBounds {
                max_source_chars: SOURCE_LIMIT,
                max_environment_chars: ENVIRONMENT_LIMIT,
                max_bindings: COUNT_LIMIT,
                max_definitions: COUNT_LIMIT,
                max_evaluation_fuel: EVALUATION_FUEL_LIMIT,
                max_recursion_depth: RECURSION_LIMIT,
                max_allocation_bytes: ALLOCATION_LIMIT,
                max_host_request_chars: HOST_REQUEST_LIMIT,
                max_child_depth: CHILD_DEPTH_LIMIT,
                max_child_fan_out: CHILD_FAN_OUT_LIMIT,
                max_child_tokens: CHILD_TOKEN_LIMIT,
                max_child_wall_time_ms: CHILD_WALL_TIME_LIMIT_MS,
                max_child_result_chars: CHILD_RESULT_LIMIT,
                max_host_calls_per_cell: HOST_CALL_LIMIT,
                max_host_result_chars: HOST_RESULT_LIMIT,
                max_cumulative_host_result_chars: CUMULATIVE_HOST_RESULT_LIMIT,
                max_bridge_nesting: BRIDGE_NESTING_LIMIT,
                max_bridge_evaluation_fuel: BRIDGE_EVALUATION_FUEL_LIMIT,
                max_bridge_wall_time_ms: BRIDGE_WALL_TIME_LIMIT_MS,
            },
            admitted_operations: &[crate::HostOperationClass::ContextRead],
            admitted_child_profiles: &["focused-child"],
            non_claims: crate::REQUIRED_NON_CLAIMS,
        })
        .expect("policy must parse")
    }
}

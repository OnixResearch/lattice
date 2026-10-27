//! Thin host-injected execution seam for admitted RLM kernel requests.
//!
//! The pure request and policy contracts live in `animus-adapter-rlm-kernel`.
//! This optional runtime module invokes exactly one caller-supplied executor
//! after revalidation. It does not implement files, commands, tools, context
//! stores, subagent lifecycle, credentials, clocks, or persistence.

use std::{string::String, vec::Vec};

/// Host implementation for already-admitted RLM kernel requests.
pub trait RlmKernelHostExecutor {
    /// Host-specific execution error.
    type Error;

    /// Executes one typed request after runtime admission.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the host cannot satisfy the request.
    fn execute(
        &mut self,
        request: &animus_adapter_rlm_kernel::HostRequest,
    ) -> Result<String, Self::Error>;
}

/// Bounded typed host result returned to the persistent kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RlmKernelHostResult {
    operation: animus_adapter_rlm_kernel::HostOperationClass,
    value: String,
    value_hash: String,
    receipt: animus_adapter_rlm_kernel::Receipt,
}

impl RlmKernelHostResult {
    /// Returns the completed operation class.
    #[must_use]
    pub const fn operation(&self) -> animus_adapter_rlm_kernel::HostOperationClass {
        self.operation
    }

    /// Returns bounded model-visible result text.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    /// Returns the domain-separated BLAKE3 result identity.
    #[must_use]
    pub fn value_hash(&self) -> &str {
        self.value_hash.as_str()
    }

    /// Returns the metadata-only execution receipt.
    #[must_use]
    pub const fn receipt(&self) -> &animus_adapter_rlm_kernel::Receipt {
        &self.receipt
    }
}

/// Runtime RLM kernel execution error.
#[derive(Debug)]
pub enum RlmKernelExecutionError<HostError> {
    /// Runtime revalidation denied the request before execution.
    Admission(animus_adapter_rlm_kernel::Error),
    /// The supplied admission does not match the current request or policy.
    AdmissionMismatch,
    /// The injected host executor failed.
    Host(HostError),
    /// Host result text violated the bounded result contract.
    Result(animus_adapter_rlm_kernel::Error),
}

impl<HostError: core::fmt::Display> core::fmt::Display for RlmKernelExecutionError<HostError> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Admission(error) => write!(formatter, "RLM request admission failed: {error}"),
            Self::AdmissionMismatch => formatter.write_str("RLM request admission mismatch"),
            Self::Host(error) => write!(formatter, "RLM host executor failed: {error}"),
            Self::Result(error) => write!(formatter, "RLM host result failed: {error}"),
        }
    }
}

/// Revalidates and executes one typed RLM kernel request through an injected host.
///
/// This function is the imperative shell. Pure policy admission runs again
/// before the exact supplied executor receives the request.
///
/// # Errors
///
/// Returns [`RlmKernelExecutionError`] before execution for stale or mismatched
/// admission. Host failures and bounded-result failures remain distinct.
pub fn execute_admitted_host_request<Executor: RlmKernelHostExecutor>(
    policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
    supplied_admission: &animus_adapter_rlm_kernel::HostRequestAdmission,
    request: &animus_adapter_rlm_kernel::HostRequest,
    executor: &mut Executor,
) -> Result<RlmKernelHostResult, RlmKernelExecutionError<Executor::Error>> {
    let current = animus_adapter_rlm_kernel::admit_host_request(policy, request)
        .map_err(RlmKernelExecutionError::Admission)?;
    if current != *supplied_admission {
        return Err(RlmKernelExecutionError::AdmissionMismatch);
    }
    let value = executor
        .execute(request)
        .map_err(RlmKernelExecutionError::Host)?;
    let value = bounded_result(value.as_str(), policy.bounds().max_host_result_chars)
        .map_err(RlmKernelExecutionError::Result)?;
    let value_hash = hash_result(value.as_str());
    let counts = [animus_adapter_rlm_kernel::ReceiptCount::new(
        "result_chars",
        u64::try_from(value.chars().count()).unwrap_or(u64::MAX),
    )
    .map_err(RlmKernelExecutionError::Result)?];
    let non_claims = policy
        .non_claims()
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let receipt =
        animus_adapter_rlm_kernel::Receipt::from_draft(&animus_adapter_rlm_kernel::ReceiptDraft {
            status: animus_adapter_rlm_kernel::RlmKernelStatus::Completed,
            policy_identity: policy.identity(),
            operation_classes: &[current.operation()],
            counts: &counts,
            child_lineage_hashes: &[],
            artifact_hashes: &[value_hash.as_str()],
            kernel_state_preserved: false,
            non_claims: non_claims.as_slice(),
            variable_value: None,
            code_body: None,
            child_transcript: None,
            credential: None,
            raw_path: None,
            claims: &["injected_host_execution"],
        })
        .map_err(RlmKernelExecutionError::Result)?;
    Ok(RlmKernelHostResult {
        operation: current.operation(),
        value,
        value_hash,
        receipt,
    })
}

/// Shell checkpoint for cancellation and elapsed-time observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RlmBridgeCheckpoint {
    /// Observe control before current runtime admission.
    Admission,
    /// Observe control before the injected executor receives a request.
    Execution,
    /// Observe control before Steel receives the correlated result.
    Resumption,
}

/// Host-supplied control facts at one bridge checkpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RlmBridgeControlObservation {
    /// Whether current cancellation is active.
    pub cancelled: bool,
    /// Elapsed wall time from the bridge start in milliseconds.
    pub elapsed_ms: u64,
}

/// Injected cancellation and elapsed-time observation service.
pub trait RlmBridgeControl {
    /// Observes current control facts without changing bridge policy.
    fn observe(&mut self, checkpoint: RlmBridgeCheckpoint) -> RlmBridgeControlObservation;
}

/// Borrowed request for one complete bridged Steel cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RlmBridgeExecutionRequest<'a> {
    /// Model-written Steel source.
    pub source: &'a str,
    /// BLAKE3 identity of the owning session.
    pub session_identity: &'a str,
    /// BLAKE3 identity of this exact cell.
    pub cell_identity: &'a str,
    /// Explicit nested bridge depth.
    pub nesting_depth: u32,
}

/// Successful complete bridge result.
#[derive(Clone, Eq, PartialEq)]
pub struct RlmBridgeCompleted {
    evaluation: animus_steel::SteelRlmEvaluation,
    completed_calls: Vec<animus_adapter_rlm_kernel::BridgeCompletedCall>,
}

impl RlmBridgeCompleted {
    /// Returns the committed Steel evaluation.
    #[must_use]
    pub const fn evaluation(&self) -> &animus_steel::SteelRlmEvaluation {
        &self.evaluation
    }

    /// Returns ordered metadata-only completed-call observations.
    #[must_use]
    pub fn observations(&self) -> Vec<animus_adapter_rlm_kernel::BridgeCallObservation> {
        bridge_observations(self.completed_calls.as_slice())
    }
}

impl core::fmt::Debug for RlmBridgeCompleted {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RlmBridgeCompleted")
            .field("evaluation", &self.evaluation)
            .field("completed_calls", &self.observations())
            .finish()
    }
}

/// Detailed bridge error while safe completed-call facts remain available.
#[derive(Debug)]
pub enum RlmBridgeErrorDetail<HostError> {
    /// The pure bridge contract rejected a transition or bound.
    Contract(animus_adapter_rlm_kernel::Error),
    /// Steel evaluation or commit failed.
    Steel(animus_steel::SteelRlmError),
    /// Runtime host-request execution failed.
    Execution(RlmKernelExecutionError<HostError>),
}

/// Terminal bridge error with completed observations preserved.
pub struct RlmBridgeExecutionError<HostError> {
    failure: animus_adapter_rlm_kernel::BridgeFailureClass,
    completed_calls: Vec<animus_adapter_rlm_kernel::BridgeCompletedCall>,
    detail: RlmBridgeErrorDetail<HostError>,
}

impl<HostError> RlmBridgeExecutionError<HostError> {
    /// Returns the stable terminal failure class.
    #[must_use]
    pub const fn failure(&self) -> animus_adapter_rlm_kernel::BridgeFailureClass {
        self.failure
    }

    /// Returns metadata-only observations for calls completed before the error.
    #[must_use]
    pub fn observations(&self) -> Vec<animus_adapter_rlm_kernel::BridgeCallObservation> {
        bridge_observations(self.completed_calls.as_slice())
    }

    /// Returns the detailed error category.
    #[must_use]
    pub const fn detail(&self) -> &RlmBridgeErrorDetail<HostError> {
        &self.detail
    }
}

impl<HostError> core::fmt::Debug for RlmBridgeExecutionError<HostError> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let detail_class = match &self.detail {
            RlmBridgeErrorDetail::Contract(_) => "contract",
            RlmBridgeErrorDetail::Steel(_) => "steel",
            RlmBridgeErrorDetail::Execution(_) => "execution",
        };
        formatter
            .debug_struct("RlmBridgeExecutionError")
            .field("failure", &self.failure)
            .field("completed_calls", &self.observations())
            .field("detail_class", &detail_class)
            .finish()
    }
}

/// Drives one complete bounded Steel cell through injected host services.
///
/// The driver checks cancellation and elapsed time before admission, execution,
/// and resumption. It revalidates every request before the exact executor runs.
///
/// # Errors
///
/// Returns [`RlmBridgeExecutionError`] without committing staged Steel state.
/// Completed host calls remain attached to the error as partial-effect facts.
pub fn execute_bridged_cell<Executor, Control>(
    kernel: &mut animus_steel::SteelRlmKernel,
    request: RlmBridgeExecutionRequest<'_>,
    executor: &mut Executor,
    control: &mut Control,
) -> Result<RlmBridgeCompleted, RlmBridgeExecutionError<Executor::Error>>
where
    Executor: RlmKernelHostExecutor,
    Control: RlmBridgeControl,
{
    let policy = kernel.policy().clone();
    let cell = animus_adapter_rlm_kernel::BridgeCellIdentity::new(
        request.session_identity,
        request.cell_identity,
    )
    .map_err(|error| {
        bridge_contract_error(
            animus_adapter_rlm_kernel::BridgeFailureClass::CorrelationMismatch,
            Vec::new(),
            error,
        )
    })?;
    let mut state =
        animus_adapter_rlm_kernel::BridgeRunState::new(cell, request.nesting_depth, &policy)
            .map_err(|error| {
                bridge_contract_error(
                    animus_adapter_rlm_kernel::BridgeFailureClass::BoundExceeded,
                    Vec::new(),
                    error,
                )
            })?;
    let evaluation_fuel = u64::try_from(request.source.chars().count()).unwrap_or(u64::MAX);

    loop {
        state
            .record_evaluation_attempt(evaluation_fuel, &policy)
            .map_err(|error| {
                state.fail(animus_adapter_rlm_kernel::BridgeFailureClass::BoundExceeded);
                bridge_contract_error(
                    animus_adapter_rlm_kernel::BridgeFailureClass::BoundExceeded,
                    state.completed_calls().to_vec(),
                    error,
                )
            })?;
        let step = kernel
            .evaluate_bridge_step(request.source, state.completed_calls())
            .map_err(|error| {
                let failure = classify_steel_failure(&error);
                state.fail(failure);
                RlmBridgeExecutionError {
                    failure,
                    completed_calls: state.completed_calls().to_vec(),
                    detail: RlmBridgeErrorDetail::Steel(error),
                }
            })?;
        if step.evaluation_fuel() != evaluation_fuel {
            state.fail(animus_adapter_rlm_kernel::BridgeFailureClass::SteelFailure);
            return Err(bridge_contract_error(
                animus_adapter_rlm_kernel::BridgeFailureClass::SteelFailure,
                state.completed_calls().to_vec(),
                animus_adapter_rlm_kernel::Error::BridgeStateConflict,
            ));
        }
        match step {
            animus_steel::SteelRlmBridgeStep::HostRequest {
                request: host_request,
                admission,
                ..
            } => execute_bridge_host_call(
                &mut state,
                BridgeHostCall {
                    request: &host_request,
                    admission: &admission,
                    policy: &policy,
                },
                executor,
                control,
            )?,
            animus_steel::SteelRlmBridgeStep::Completed { .. } => {
                state.complete().map_err(|error| {
                    state.fail(animus_adapter_rlm_kernel::BridgeFailureClass::SteelFailure);
                    bridge_contract_error(
                        animus_adapter_rlm_kernel::BridgeFailureClass::SteelFailure,
                        state.completed_calls().to_vec(),
                        error,
                    )
                })?;
                let evaluation = kernel
                    .commit_bridged(request.source, state.completed_calls())
                    .map_err(|error| RlmBridgeExecutionError {
                        failure: animus_adapter_rlm_kernel::BridgeFailureClass::SteelFailure,
                        completed_calls: state.completed_calls().to_vec(),
                        detail: RlmBridgeErrorDetail::Steel(error),
                    })?;
                return Ok(RlmBridgeCompleted {
                    evaluation,
                    completed_calls: state.completed_calls().to_vec(),
                });
            }
        }
    }
}

#[derive(Clone, Copy)]
struct BridgeHostCall<'a> {
    request: &'a animus_adapter_rlm_kernel::HostRequest,
    admission: &'a animus_adapter_rlm_kernel::HostRequestAdmission,
    policy: &'a animus_adapter_rlm_kernel::RlmKernelPolicy,
}

fn execute_bridge_host_call<Executor, Control>(
    state: &mut animus_adapter_rlm_kernel::BridgeRunState,
    call: BridgeHostCall<'_>,
    executor: &mut Executor,
    control: &mut Control,
) -> Result<(), RlmBridgeExecutionError<Executor::Error>>
where
    Executor: RlmKernelHostExecutor,
    Control: RlmBridgeControl,
{
    observe_bridge_control(state, control, RlmBridgeCheckpoint::Admission, call.policy)?;
    let pending = state
        .await_host_request(call.request, call.admission, call.policy)
        .map_err(|error| {
            let failure = classify_contract_failure(&error);
            state.fail(failure);
            bridge_contract_error(failure, state.completed_calls().to_vec(), error)
        })?;
    observe_bridge_control(state, control, RlmBridgeCheckpoint::Execution, call.policy)?;
    let result = execute_admitted_host_request(
        call.policy,
        pending.admission(),
        pending.request(),
        executor,
    )
    .map_err(|error| {
        let failure = classify_execution_failure(&error);
        state.fail(failure);
        RlmBridgeExecutionError {
            failure,
            completed_calls: state.completed_calls().to_vec(),
            detail: RlmBridgeErrorDetail::Execution(error),
        }
    })?;
    state
        .accept_host_result(pending.correlation(), result.value(), call.policy)
        .map_err(|error| {
            let failure = classify_contract_failure(&error);
            state.fail(failure);
            bridge_contract_error(failure, state.completed_calls().to_vec(), error)
        })?;
    observe_bridge_control(state, control, RlmBridgeCheckpoint::Resumption, call.policy)
}

fn classify_execution_failure<HostError>(
    error: &RlmKernelExecutionError<HostError>,
) -> animus_adapter_rlm_kernel::BridgeFailureClass {
    match error {
        RlmKernelExecutionError::Admission(_) | RlmKernelExecutionError::AdmissionMismatch => {
            animus_adapter_rlm_kernel::BridgeFailureClass::AuthorityDenied
        }
        RlmKernelExecutionError::Host(_) => {
            animus_adapter_rlm_kernel::BridgeFailureClass::HostFailure
        }
        RlmKernelExecutionError::Result(_) => {
            animus_adapter_rlm_kernel::BridgeFailureClass::ResultInvalid
        }
    }
}

fn observe_bridge_control<HostError, Control: RlmBridgeControl>(
    state: &mut animus_adapter_rlm_kernel::BridgeRunState,
    control: &mut Control,
    checkpoint: RlmBridgeCheckpoint,
    policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
) -> Result<(), RlmBridgeExecutionError<HostError>> {
    let observation = control.observe(checkpoint);
    state
        .observe_control(observation.cancelled, observation.elapsed_ms, policy)
        .map_err(|error| {
            let failure = classify_contract_failure(&error);
            bridge_contract_error(failure, state.completed_calls().to_vec(), error)
        })
}

fn classify_steel_failure(
    error: &animus_steel::SteelRlmError,
) -> animus_adapter_rlm_kernel::BridgeFailureClass {
    match error {
        animus_steel::SteelRlmError::Contract(contract) => classify_contract_failure(contract),
        animus_steel::SteelRlmError::ToolOutput { .. } => {
            animus_adapter_rlm_kernel::BridgeFailureClass::ResultInvalid
        }
        _other => animus_adapter_rlm_kernel::BridgeFailureClass::SteelFailure,
    }
}

fn classify_contract_failure(
    error: &animus_adapter_rlm_kernel::Error,
) -> animus_adapter_rlm_kernel::BridgeFailureClass {
    match error {
        animus_adapter_rlm_kernel::Error::OperationNotAdmitted { .. } => {
            animus_adapter_rlm_kernel::BridgeFailureClass::AuthorityDenied
        }
        animus_adapter_rlm_kernel::Error::BridgeCorrelationMismatch => {
            animus_adapter_rlm_kernel::BridgeFailureClass::CorrelationMismatch
        }
        animus_adapter_rlm_kernel::Error::BridgeCancelled => {
            animus_adapter_rlm_kernel::BridgeFailureClass::Cancelled
        }
        animus_adapter_rlm_kernel::Error::BridgeTimedOut => {
            animus_adapter_rlm_kernel::BridgeFailureClass::Timeout
        }
        animus_adapter_rlm_kernel::Error::TextTooLong { .. }
        | animus_adapter_rlm_kernel::Error::EmptyField { .. }
        | animus_adapter_rlm_kernel::Error::NulCharacter { .. } => {
            animus_adapter_rlm_kernel::BridgeFailureClass::ResultInvalid
        }
        _other => animus_adapter_rlm_kernel::BridgeFailureClass::BoundExceeded,
    }
}

fn bridge_observations(
    calls: &[animus_adapter_rlm_kernel::BridgeCompletedCall],
) -> Vec<animus_adapter_rlm_kernel::BridgeCallObservation> {
    calls
        .iter()
        .map(animus_adapter_rlm_kernel::BridgeCompletedCall::observation)
        .collect()
}

fn bridge_contract_error<HostError>(
    failure: animus_adapter_rlm_kernel::BridgeFailureClass,
    completed_calls: Vec<animus_adapter_rlm_kernel::BridgeCompletedCall>,
    error: animus_adapter_rlm_kernel::Error,
) -> RlmBridgeExecutionError<HostError> {
    RlmBridgeExecutionError {
        failure,
        completed_calls,
        detail: RlmBridgeErrorDetail::Contract(error),
    }
}

fn bounded_result(
    value: &str,
    max_result_chars: u32,
) -> Result<String, animus_adapter_rlm_kernel::Error> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(animus_adapter_rlm_kernel::Error::EmptyField {
            field: "host_result",
        });
    }
    if normalized.contains('\0') {
        return Err(animus_adapter_rlm_kernel::Error::NulCharacter {
            field: "host_result",
        });
    }
    let actual_chars = u32::try_from(normalized.chars().count()).unwrap_or(u32::MAX);
    if actual_chars > max_result_chars {
        return Err(animus_adapter_rlm_kernel::Error::TextTooLong {
            field: "host_result",
            max_chars: max_result_chars,
            actual_chars,
        });
    }
    Ok(String::from(normalized))
}

fn hash_result(value: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"animus.rlm-host-result.v1");
    hasher.update(&u64::try_from(value.len()).unwrap_or(u64::MAX).to_le_bytes());
    hasher.update(value.as_bytes());
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIME_AGENT_REVISION: &str = "c5991bc853d27754aed345c13ff4d2e05c40f1f4";
    const SOURCE_LIMIT: u32 = 4_096;
    const ENVIRONMENT_LIMIT: u32 = 16_384;
    const COUNT_LIMIT: u32 = 16;
    const FUEL_LIMIT: u64 = 100_000;
    const RECURSION_LIMIT: u32 = 64;
    const ALLOCATION_LIMIT: u64 = 1_048_576;
    const REQUEST_LIMIT: u32 = 8_192;
    const CHILD_DEPTH_LIMIT: u32 = 2;
    const CHILD_FAN_OUT_LIMIT: u32 = 4;
    const CHILD_TOKEN_LIMIT: u64 = 16_000;
    const CHILD_WALL_LIMIT_MS: u64 = 60_000;
    const CHILD_RESULT_LIMIT: u32 = 8_192;
    const HOST_CALL_LIMIT: u32 = 4;
    const HOST_RESULT_CHAR_LIMIT: u32 = 32_768;
    const CUMULATIVE_HOST_RESULT_CHAR_LIMIT: u32 = 65_536;
    const BRIDGE_NESTING_LIMIT: u32 = 4;
    const BRIDGE_FUEL_LIMIT: u64 = 500_000;
    const BRIDGE_WALL_TIME_LIMIT_MS: u64 = 120_000;
    const EXECUTOR_RESULT: &str = "bounded context value";
    const OVERSIZED_RESULT_CHARS: u32 = HOST_RESULT_CHAR_LIMIT + 1;
    const UNAVAILABLE_ERROR: &str = "unavailable";
    const CANCELLED_ERROR: &str = "cancelled";
    const SESSION_IDENTITY: &str =
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const CELL_IDENTITY: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const FIRST_RESULT: &str = "left";
    const SECOND_RESULT: &str = "right";
    const BRIDGED_VALUE: &str = "leftright";
    const FIRST_NESTING_DEPTH: u32 = 1;
    const EXPECTED_TWO_CALLS: usize = 2;
    const EXPECTED_TWO_EXECUTIONS: u32 = 2;

    #[derive(Default)]
    struct FakeExecutor {
        calls: u32,
    }

    impl RlmKernelHostExecutor for FakeExecutor {
        type Error = &'static str;

        fn execute(
            &mut self,
            _request: &animus_adapter_rlm_kernel::HostRequest,
        ) -> Result<String, Self::Error> {
            self.calls = self.calls.saturating_add(1);
            Ok(String::from(EXECUTOR_RESULT))
        }
    }

    struct ErrorExecutor {
        calls: u32,
        error: &'static str,
    }

    impl RlmKernelHostExecutor for ErrorExecutor {
        type Error = &'static str;

        fn execute(
            &mut self,
            _request: &animus_adapter_rlm_kernel::HostRequest,
        ) -> Result<String, Self::Error> {
            self.calls = self.calls.saturating_add(1);
            Err(self.error)
        }
    }

    #[derive(Default)]
    struct OversizedExecutor {
        calls: u32,
    }

    #[derive(Default)]
    struct EmptyExecutor {
        calls: u32,
    }

    impl RlmKernelHostExecutor for OversizedExecutor {
        type Error = &'static str;

        fn execute(
            &mut self,
            _request: &animus_adapter_rlm_kernel::HostRequest,
        ) -> Result<String, Self::Error> {
            self.calls = self.calls.saturating_add(1);
            Ok("x".repeat(usize::try_from(OVERSIZED_RESULT_CHARS).unwrap_or(usize::MAX)))
        }
    }

    impl RlmKernelHostExecutor for EmptyExecutor {
        type Error = &'static str;

        fn execute(
            &mut self,
            _request: &animus_adapter_rlm_kernel::HostRequest,
        ) -> Result<String, Self::Error> {
            self.calls = self.calls.saturating_add(1);
            Ok(String::new())
        }
    }

    #[derive(Default)]
    struct SequenceExecutor {
        calls: Vec<String>,
    }

    impl RlmKernelHostExecutor for SequenceExecutor {
        type Error = &'static str;

        fn execute(
            &mut self,
            request: &animus_adapter_rlm_kernel::HostRequest,
        ) -> Result<String, Self::Error> {
            let animus_adapter_rlm_kernel::HostRequest::ContextRead { key } = request else {
                return Err(UNAVAILABLE_ERROR);
            };
            self.calls.push(key.clone());
            match key.as_str() {
                "first" => Ok(String::from(FIRST_RESULT)),
                "second" => Ok(String::from(SECOND_RESULT)),
                _other => Err(UNAVAILABLE_ERROR),
            }
        }
    }

    #[derive(Default)]
    struct EchoExecutor {
        calls: Vec<String>,
    }

    impl RlmKernelHostExecutor for EchoExecutor {
        type Error = &'static str;

        fn execute(
            &mut self,
            request: &animus_adapter_rlm_kernel::HostRequest,
        ) -> Result<String, Self::Error> {
            let animus_adapter_rlm_kernel::HostRequest::ContextRead { key } = request else {
                return Err(UNAVAILABLE_ERROR);
            };
            self.calls.push(key.clone());
            Ok(key.clone())
        }
    }

    #[derive(Default)]
    struct FailSecondExecutor {
        calls: u32,
    }

    impl RlmKernelHostExecutor for FailSecondExecutor {
        type Error = &'static str;

        fn execute(
            &mut self,
            _request: &animus_adapter_rlm_kernel::HostRequest,
        ) -> Result<String, Self::Error> {
            self.calls = self.calls.saturating_add(1);
            if self.calls == 1 {
                Ok(String::from(FIRST_RESULT))
            } else {
                Err(UNAVAILABLE_ERROR)
            }
        }
    }

    #[derive(Default)]
    struct FakeControl {
        cancel_at: Option<RlmBridgeCheckpoint>,
        elapsed_ms: u64,
        checkpoints: Vec<RlmBridgeCheckpoint>,
    }

    impl RlmBridgeControl for FakeControl {
        fn observe(&mut self, checkpoint: RlmBridgeCheckpoint) -> RlmBridgeControlObservation {
            self.checkpoints.push(checkpoint);
            RlmBridgeControlObservation {
                cancelled: self.cancel_at == Some(checkpoint),
                elapsed_ms: self.elapsed_ms,
            }
        }
    }

    #[test]
    fn bridged_cell_uses_two_results_and_commits_only_the_final_value() {
        // r[verify agent_harness_runtime.steel_host_bridge.continuation.multiple]
        // r[verify agent_harness_runtime.steel_host_bridge.commit_boundary]
        let mut kernel =
            animus_steel::SteelRlmKernel::new(policy()).expect("kernel must initialize");
        let mut executor = SequenceExecutor::default();
        let mut control = FakeControl::default();
        let result = execute_bridged_cell(
            &mut kernel,
            RlmBridgeExecutionRequest {
                source: "(define bridged-value (string-append (animus-context-read \"first\") (animus-context-read \"second\"))) bridged-value",
                session_identity: SESSION_IDENTITY,
                cell_identity: CELL_IDENTITY,
                nesting_depth: FIRST_NESTING_DEPTH,
            },
            &mut executor,
            &mut control,
        )
        .expect("bridged cell must complete");
        assert_eq!(executor.calls, ["first", "second"]);
        assert_eq!(result.observations().len(), EXPECTED_TWO_CALLS);
        assert_ne!(
            result.observations()[0].correlation().call_identity(),
            result.observations()[1].correlation().call_identity()
        );
        assert!(matches!(
            result.evaluation(),
            animus_steel::SteelRlmEvaluation::Value { value, .. } if value == BRIDGED_VALUE
        ));
        let persisted = kernel
            .evaluate("bridged-value")
            .expect("committed bridge value must persist");
        assert!(matches!(
            persisted,
            animus_steel::SteelRlmEvaluation::Value { value, .. } if value == BRIDGED_VALUE
        ));
    }

    #[test]
    fn later_steel_failure_preserves_completed_effect_fact_without_committing_state() {
        // r[verify agent_harness_runtime.steel_host_bridge.commit_boundary.failure]
        // r[verify agent_harness_runtime.steel_host_bridge.partial_effects.recorded]
        let mut kernel =
            animus_steel::SteelRlmKernel::new(policy()).expect("kernel must initialize");
        let mut executor = SequenceExecutor::default();
        let mut control = FakeControl::default();
        let error = execute_bridged_cell(
            &mut kernel,
            RlmBridgeExecutionRequest {
                source: "(define staged-value (animus-context-read \"first\")) (+ staged-value 1)",
                session_identity: SESSION_IDENTITY,
                cell_identity: CELL_IDENTITY,
                nesting_depth: FIRST_NESTING_DEPTH,
            },
            &mut executor,
            &mut control,
        )
        .expect_err("later Steel failure must reject the cell");
        assert_eq!(
            error.failure(),
            animus_adapter_rlm_kernel::BridgeFailureClass::SteelFailure
        );
        assert_eq!(error.observations().len(), 1);
        assert!(!format!("{error:?}").contains(FIRST_RESULT));
        assert!(kernel.evaluate("staged-value").is_err());
    }

    #[test]
    fn cancellation_before_resumption_preserves_the_host_observation_and_rejects_commit() {
        // r[verify agent_harness_runtime.steel_host_bridge.verification]
        let mut kernel =
            animus_steel::SteelRlmKernel::new(policy()).expect("kernel must initialize");
        let mut executor = SequenceExecutor::default();
        let mut control = FakeControl {
            cancel_at: Some(RlmBridgeCheckpoint::Resumption),
            ..FakeControl::default()
        };
        let error = execute_bridged_cell(
            &mut kernel,
            RlmBridgeExecutionRequest {
                source: "(define cancelled-value (animus-context-read \"first\")) cancelled-value",
                session_identity: SESSION_IDENTITY,
                cell_identity: CELL_IDENTITY,
                nesting_depth: FIRST_NESTING_DEPTH,
            },
            &mut executor,
            &mut control,
        )
        .expect_err("cancellation must reject the staged cell");
        assert_eq!(
            error.failure(),
            animus_adapter_rlm_kernel::BridgeFailureClass::Cancelled
        );
        assert_eq!(error.observations().len(), 1);
        assert!(kernel.evaluate("cancelled-value").is_err());
    }

    #[test]
    fn authority_denial_timeout_and_call_bounds_stop_before_unadmitted_effects() {
        // r[verify agent_harness_runtime.steel_host_bridge.authority.denied]
        // r[verify agent_harness_runtime.steel_host_bridge.bounds.exceeded]
        let mut denied_kernel =
            animus_steel::SteelRlmKernel::new(policy()).expect("kernel must initialize");
        let mut denied_executor = FakeExecutor::default();
        let mut control = FakeControl::default();
        let denied = execute_bridged_cell(
            &mut denied_kernel,
            RlmBridgeExecutionRequest {
                source: "(animus-host-shell \"cargo test\")",
                session_identity: SESSION_IDENTITY,
                cell_identity: CELL_IDENTITY,
                nesting_depth: FIRST_NESTING_DEPTH,
            },
            &mut denied_executor,
            &mut control,
        )
        .expect_err("unadmitted operation must fail");
        assert_eq!(
            denied.failure(),
            animus_adapter_rlm_kernel::BridgeFailureClass::AuthorityDenied
        );
        assert_eq!(denied_executor.calls, 0);

        let mut timeout_kernel =
            animus_steel::SteelRlmKernel::new(policy()).expect("kernel must initialize");
        let mut timeout_executor = FakeExecutor::default();
        let mut timeout_control = FakeControl {
            elapsed_ms: BRIDGE_WALL_TIME_LIMIT_MS + 1,
            ..FakeControl::default()
        };
        let timeout = execute_bridged_cell(
            &mut timeout_kernel,
            RlmBridgeExecutionRequest {
                source: "(animus-context-read \"first\")",
                session_identity: SESSION_IDENTITY,
                cell_identity: CELL_IDENTITY,
                nesting_depth: FIRST_NESTING_DEPTH,
            },
            &mut timeout_executor,
            &mut timeout_control,
        )
        .expect_err("expired bridge must fail");
        assert_eq!(
            timeout.failure(),
            animus_adapter_rlm_kernel::BridgeFailureClass::Timeout
        );
        assert_eq!(timeout_executor.calls, 0);

        let mut bounded_kernel =
            animus_steel::SteelRlmKernel::new(policy()).expect("kernel must initialize");
        let mut bounded_executor = EchoExecutor::default();
        let mut bounded_control = FakeControl::default();
        let bounded = execute_bridged_cell(
            &mut bounded_kernel,
            RlmBridgeExecutionRequest {
                source: "(string-append (animus-context-read \"a\") (animus-context-read \"b\") (animus-context-read \"c\") (animus-context-read \"d\") (animus-context-read \"e\"))",
                session_identity: SESSION_IDENTITY,
                cell_identity: CELL_IDENTITY,
                nesting_depth: FIRST_NESTING_DEPTH,
            },
            &mut bounded_executor,
            &mut bounded_control,
        )
        .expect_err("host-call overflow must fail");
        assert_eq!(
            bounded.failure(),
            animus_adapter_rlm_kernel::BridgeFailureClass::BoundExceeded
        );
        assert_eq!(
            bounded_executor.calls.len(),
            usize::try_from(HOST_CALL_LIMIT).expect("host-call limit must fit usize")
        );
    }

    #[test]
    fn a_second_host_failure_retains_only_the_first_completed_call() {
        // r[verify agent_harness_runtime.steel_host_bridge.partial_effects.recorded]
        let mut kernel =
            animus_steel::SteelRlmKernel::new(policy()).expect("kernel must initialize");
        let mut executor = FailSecondExecutor::default();
        let mut control = FakeControl::default();
        let error = execute_bridged_cell(
            &mut kernel,
            RlmBridgeExecutionRequest {
                source: "(string-append (animus-context-read \"first\") (animus-context-read \"second\"))",
                session_identity: SESSION_IDENTITY,
                cell_identity: CELL_IDENTITY,
                nesting_depth: FIRST_NESTING_DEPTH,
            },
            &mut executor,
            &mut control,
        )
        .expect_err("second host failure must reject the cell");
        assert_eq!(
            error.failure(),
            animus_adapter_rlm_kernel::BridgeFailureClass::HostFailure
        );
        assert_eq!(error.observations().len(), 1);
        assert_eq!(executor.calls, EXPECTED_TWO_EXECUTIONS);
    }

    #[test]
    fn admitted_request_invokes_exact_injected_executor_and_returns_safe_receipt() {
        // r[verify agent_harness_runtime.rlm_kernel.host_requests.admitted]
        let policy = policy();
        let request = animus_adapter_rlm_kernel::HostRequest::context_read("task_context")
            .expect("request must parse");
        let admission = animus_adapter_rlm_kernel::admit_host_request(&policy, &request)
            .expect("request must admit");
        let mut executor = FakeExecutor::default();
        let result = execute_admitted_host_request(&policy, &admission, &request, &mut executor)
            .expect("injected execution must succeed");
        assert_eq!(executor.calls, 1);
        assert_eq!(result.value(), EXECUTOR_RESULT);
        assert_eq!(result.receipt().operation_classes().len(), 1);
    }

    #[test]
    fn stale_admission_and_oversized_result_fail_closed() {
        // r[verify agent_harness_runtime.rlm_kernel.host_requests.rejected]
        let policy = policy();
        let admitted_request = animus_adapter_rlm_kernel::HostRequest::context_read("task_context")
            .expect("request must parse");
        let stale = animus_adapter_rlm_kernel::admit_host_request(&policy, &admitted_request)
            .expect("request must admit");
        let other = animus_adapter_rlm_kernel::HostRequest::context_read("other_context")
            .expect("request must parse");
        let mut executor = FakeExecutor::default();
        assert!(matches!(
            execute_admitted_host_request(&policy, &stale, &other, &mut executor),
            Err(RlmKernelExecutionError::AdmissionMismatch)
        ));
        assert_eq!(executor.calls, 0);

        let mut oversized = OversizedExecutor::default();
        assert!(matches!(
            execute_admitted_host_request(&policy, &stale, &admitted_request, &mut oversized),
            Err(RlmKernelExecutionError::Result(
                animus_adapter_rlm_kernel::Error::TextTooLong {
                    field: "host_result",
                    ..
                }
            ))
        ));
        assert_eq!(oversized.calls, 1);

        let mut empty = EmptyExecutor::default();
        assert!(matches!(
            execute_admitted_host_request(&policy, &stale, &admitted_request, &mut empty),
            Err(RlmKernelExecutionError::Result(
                animus_adapter_rlm_kernel::Error::EmptyField {
                    field: "host_result"
                }
            ))
        ));
        assert_eq!(empty.calls, 1);
    }

    #[test]
    fn unavailable_and_cancelled_executors_fail_without_result() {
        // r[verify agent_harness_runtime.rlm_kernel.children.rejected]
        let policy = policy();
        let request = animus_adapter_rlm_kernel::HostRequest::context_read("task_context")
            .expect("request must parse");
        let admission = animus_adapter_rlm_kernel::admit_host_request(&policy, &request)
            .expect("request must admit");
        for expected in [UNAVAILABLE_ERROR, CANCELLED_ERROR] {
            let mut executor = ErrorExecutor {
                calls: 0,
                error: expected,
            };
            assert!(matches!(
                execute_admitted_host_request(&policy, &admission, &request, &mut executor),
                Err(RlmKernelExecutionError::Host(error)) if error == expected
            ));
            assert_eq!(executor.calls, 1);
        }
    }

    fn policy() -> animus_adapter_rlm_kernel::RlmKernelPolicy {
        animus_adapter_rlm_kernel::RlmKernelPolicy::from_draft(
            &animus_adapter_rlm_kernel::RlmKernelPolicyDraft {
                schema: animus_adapter_rlm_kernel::RLM_KERNEL_POLICY_SCHEMA,
                cohort: "prime-agent-rlm-2026-08-06",
                cohort_revision: PRIME_AGENT_REVISION,
                profile: "model-facing-steel",
                bounds: animus_adapter_rlm_kernel::ResourceBounds {
                    max_source_chars: SOURCE_LIMIT,
                    max_environment_chars: ENVIRONMENT_LIMIT,
                    max_bindings: COUNT_LIMIT,
                    max_definitions: COUNT_LIMIT,
                    max_evaluation_fuel: FUEL_LIMIT,
                    max_recursion_depth: RECURSION_LIMIT,
                    max_allocation_bytes: ALLOCATION_LIMIT,
                    max_host_request_chars: REQUEST_LIMIT,
                    max_child_depth: CHILD_DEPTH_LIMIT,
                    max_child_fan_out: CHILD_FAN_OUT_LIMIT,
                    max_child_tokens: CHILD_TOKEN_LIMIT,
                    max_child_wall_time_ms: CHILD_WALL_LIMIT_MS,
                    max_child_result_chars: CHILD_RESULT_LIMIT,
                    max_host_calls_per_cell: HOST_CALL_LIMIT,
                    max_host_result_chars: HOST_RESULT_CHAR_LIMIT,
                    max_cumulative_host_result_chars: CUMULATIVE_HOST_RESULT_CHAR_LIMIT,
                    max_bridge_nesting: BRIDGE_NESTING_LIMIT,
                    max_bridge_evaluation_fuel: BRIDGE_FUEL_LIMIT,
                    max_bridge_wall_time_ms: BRIDGE_WALL_TIME_LIMIT_MS,
                },
                admitted_operations: &[animus_adapter_rlm_kernel::HostOperationClass::ContextRead],
                admitted_child_profiles: &["focused-child"],
                non_claims: animus_adapter_rlm_kernel::REQUIRED_NON_CLAIMS,
            },
        )
        .expect("test policy must parse")
    }
}

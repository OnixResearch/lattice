//! Thin host-injected execution seam for admitted RLM kernel requests.
//!
//! The pure request and policy contracts live in `animus-adapter-rlm-kernel`.
//! This optional runtime module invokes exactly one caller-supplied executor
//! after revalidation. It does not implement files, commands, tools, context
//! stores, subagent lifecycle, credentials, clocks, or persistence.

use std::string::String;
use std::vec::Vec;

/// Maximum bounded result text returned from one host request.
const HOST_RESULT_CHAR_LIMIT: u32 = 32_768;

/// Host implementation for already-admitted RLM kernel requests.
pub trait RlmKernelHostExecutor {
    /// Host-specific execution error.
    type Error;

    /// Executes one typed request after runtime admission.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the host cannot satisfy the request.
    fn execute(&mut self, request: &animus_adapter_rlm_kernel::HostRequest) -> Result<String, Self::Error>;
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
    let current =
        animus_adapter_rlm_kernel::admit_host_request(policy, request).map_err(RlmKernelExecutionError::Admission)?;
    if current != *supplied_admission {
        return Err(RlmKernelExecutionError::AdmissionMismatch);
    }
    let value = executor.execute(request).map_err(RlmKernelExecutionError::Host)?;
    let value = bounded_result(value.as_str()).map_err(RlmKernelExecutionError::Result)?;
    let value_hash = hash_result(value.as_str());
    let counts = [animus_adapter_rlm_kernel::ReceiptCount::new(
        "result_chars",
        u64::try_from(value.chars().count()).unwrap_or(u64::MAX),
    )
    .map_err(RlmKernelExecutionError::Result)?];
    let non_claims = policy.non_claims().iter().map(String::as_str).collect::<Vec<_>>();
    let receipt = animus_adapter_rlm_kernel::Receipt::from_draft(&animus_adapter_rlm_kernel::ReceiptDraft {
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

fn bounded_result(value: &str) -> Result<String, animus_adapter_rlm_kernel::Error> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(animus_adapter_rlm_kernel::Error::EmptyField { field: "host_result" });
    }
    if normalized.contains('\0') {
        return Err(animus_adapter_rlm_kernel::Error::NulCharacter { field: "host_result" });
    }
    let actual_chars = u32::try_from(normalized.chars().count()).unwrap_or(u32::MAX);
    if actual_chars > HOST_RESULT_CHAR_LIMIT {
        return Err(animus_adapter_rlm_kernel::Error::TextTooLong {
            field: "host_result",
            max_chars: HOST_RESULT_CHAR_LIMIT,
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
    const EXECUTOR_RESULT: &str = "bounded context value";
    const OVERSIZED_RESULT_CHARS: u32 = HOST_RESULT_CHAR_LIMIT + 1;
    const UNAVAILABLE_ERROR: &str = "unavailable";
    const CANCELLED_ERROR: &str = "cancelled";

    #[derive(Default)]
    struct FakeExecutor {
        calls: u32,
    }

    impl RlmKernelHostExecutor for FakeExecutor {
        type Error = &'static str;

        fn execute(&mut self, _request: &animus_adapter_rlm_kernel::HostRequest) -> Result<String, Self::Error> {
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

        fn execute(&mut self, _request: &animus_adapter_rlm_kernel::HostRequest) -> Result<String, Self::Error> {
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

        fn execute(&mut self, _request: &animus_adapter_rlm_kernel::HostRequest) -> Result<String, Self::Error> {
            self.calls = self.calls.saturating_add(1);
            Ok("x".repeat(usize::try_from(OVERSIZED_RESULT_CHARS).unwrap_or(usize::MAX)))
        }
    }

    impl RlmKernelHostExecutor for EmptyExecutor {
        type Error = &'static str;

        fn execute(&mut self, _request: &animus_adapter_rlm_kernel::HostRequest) -> Result<String, Self::Error> {
            self.calls = self.calls.saturating_add(1);
            Ok(String::new())
        }
    }

    #[test]
    fn admitted_request_invokes_exact_injected_executor_and_returns_safe_receipt() {
        // r[verify agent_harness_runtime.rlm_kernel.host_requests.admitted]
        let policy = policy();
        let request = animus_adapter_rlm_kernel::HostRequest::context_read("task_context").expect("request must parse");
        let admission = animus_adapter_rlm_kernel::admit_host_request(&policy, &request).expect("request must admit");
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
        let admitted_request =
            animus_adapter_rlm_kernel::HostRequest::context_read("task_context").expect("request must parse");
        let stale =
            animus_adapter_rlm_kernel::admit_host_request(&policy, &admitted_request).expect("request must admit");
        let other = animus_adapter_rlm_kernel::HostRequest::context_read("other_context").expect("request must parse");
        let mut executor = FakeExecutor::default();
        assert!(matches!(
            execute_admitted_host_request(&policy, &stale, &other, &mut executor),
            Err(RlmKernelExecutionError::AdmissionMismatch)
        ));
        assert_eq!(executor.calls, 0);

        let mut oversized = OversizedExecutor::default();
        assert!(matches!(
            execute_admitted_host_request(&policy, &stale, &admitted_request, &mut oversized),
            Err(RlmKernelExecutionError::Result(animus_adapter_rlm_kernel::Error::TextTooLong {
                field: "host_result",
                ..
            }))
        ));
        assert_eq!(oversized.calls, 1);

        let mut empty = EmptyExecutor::default();
        assert!(matches!(
            execute_admitted_host_request(&policy, &stale, &admitted_request, &mut empty),
            Err(RlmKernelExecutionError::Result(animus_adapter_rlm_kernel::Error::EmptyField {
                field: "host_result"
            }))
        ));
        assert_eq!(empty.calls, 1);
    }

    #[test]
    fn unavailable_and_cancelled_executors_fail_without_result() {
        // r[verify agent_harness_runtime.rlm_kernel.children.rejected]
        let policy = policy();
        let request = animus_adapter_rlm_kernel::HostRequest::context_read("task_context").expect("request must parse");
        let admission = animus_adapter_rlm_kernel::admit_host_request(&policy, &request).expect("request must admit");
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
        animus_adapter_rlm_kernel::RlmKernelPolicy::from_draft(&animus_adapter_rlm_kernel::RlmKernelPolicyDraft {
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
            },
            admitted_operations: &[animus_adapter_rlm_kernel::HostOperationClass::ContextRead],
            admitted_child_profiles: &["focused-child"],
            non_claims: animus_adapter_rlm_kernel::REQUIRED_NON_CLAIMS,
        })
        .expect("test policy must parse")
    }
}

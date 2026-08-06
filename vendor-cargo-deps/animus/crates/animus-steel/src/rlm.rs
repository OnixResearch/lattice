use std::time::Duration;

use animus_adapters::ComponentAdapter;

const RLM_PRELUDE: &str = r#"
(define (animus-host-read target) (list 'animus-host-request "file_read" target))
(define (animus-host-edit target replacement) (list 'animus-host-request "file_edit" target replacement))
(define (animus-host-shell command) (list 'animus-host-request "shell" command))
(define (animus-host-tool name payload) (list 'animus-host-request "tool" name payload))
(define (animus-context-read key) (list 'animus-host-request "context_read" key))
(define (animus-kernel-clear) (list 'animus-host-request "kernel_clear"))
(define (rlm profile context depth fan-out token-budget wall-time-ms result-char-budget)
  (list 'animus-host-request "subagent" profile context depth fan-out token-budget wall-time-ms result-char-budget))
"#;
const HOST_REQUEST_TAG: &str = "animus-host-request";
const FILE_READ_TAG: &str = "file_read";
const FILE_EDIT_TAG: &str = "file_edit";
const SHELL_TAG: &str = "shell";
const TOOL_TAG: &str = "tool";
const SUBAGENT_TAG: &str = "subagent";
const CONTEXT_READ_TAG: &str = "context_read";
const KERNEL_CLEAR_TAG: &str = "kernel_clear";
const TAG_INDEX: usize = 0;
const OPERATION_INDEX: usize = 1;
const FIRST_ARGUMENT_INDEX: usize = 2;
const SECOND_ARGUMENT_INDEX: usize = 3;
const CHILD_DEPTH_INDEX: usize = 4;
const CHILD_FAN_OUT_INDEX: usize = 5;
const CHILD_TOKEN_BUDGET_INDEX: usize = 6;
const CHILD_WALL_TIME_INDEX: usize = 7;
const CHILD_RESULT_BUDGET_INDEX: usize = 8;
const TAGGED_REQUEST_ARITY: usize = 2;
const FILE_READ_ARITY: usize = 3;
const FILE_EDIT_ARITY: usize = 4;
const SHELL_ARITY: usize = 3;
const TOOL_ARITY: usize = 4;
const SUBAGENT_ARITY: usize = 9;
const CONTEXT_READ_ARITY: usize = 3;
const KERNEL_CLEAR_ARITY: usize = 2;
const EVALUATION_TIMEOUT_MS: u64 = 2_000;
const ESTIMATED_ALLOCATION_BYTES_PER_SOURCE_BYTE: u64 = 8;
const MAX_VISIBLE_VALUE_CHARS: u32 = 32_768;
const CODE_ARGUMENT: &str = "code";
const SAFE_RESULT_PREFIX: &str = "value:";
const HOST_REQUEST_RESULT_PREFIX: &str = "host_request:";

/// Error from model-facing Steel evaluation or tool adaptation.
#[derive(Debug)]
pub enum SteelRlmError {
    /// The no-std kernel contract rejected a request or transition.
    Contract(animus_adapter_rlm_kernel::Error),
    /// Steel rejected the prelude, replay history, context, or current cell.
    Evaluate {
        /// Bounded evaluator diagnostic.
        message: String,
    },
    /// Steel returned an unsupported host-request shape.
    InvalidHostRequest {
        /// Bounded shape diagnostic.
        message: String,
    },
    /// The generic tool request did not match the Steel REPL contract.
    ToolRequest {
        /// Bounded request diagnostic.
        message: String,
    },
    /// Bounded tool-output construction failed.
    ToolOutput {
        /// Bounded output diagnostic.
        message: String,
    },
}

impl core::fmt::Display for SteelRlmError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "RLM kernel contract denied: {error}"),
            Self::Evaluate { message } => write!(formatter, "Steel evaluation failed: {message}"),
            Self::InvalidHostRequest { message } => {
                write!(formatter, "Steel host request is invalid: {message}")
            }
            Self::ToolRequest { message } => {
                write!(formatter, "Steel tool request failed: {message}")
            }
            Self::ToolOutput { message } => {
                write!(formatter, "Steel tool output failed: {message}")
            }
        }
    }
}

impl From<animus_adapter_rlm_kernel::Error> for SteelRlmError {
    fn from(error: animus_adapter_rlm_kernel::Error) -> Self {
        Self::Contract(error)
    }
}

/// Successful model-facing Steel evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SteelRlmEvaluation {
    /// A bounded model-visible value completed without a host effect.
    Value {
        /// Bounded value text.
        value: String,
        /// Metadata-only evaluation receipt.
        receipt: animus_adapter_rlm_kernel::Receipt,
    },
    /// Model code returned a typed request for host admission and execution.
    HostRequest {
        /// Typed request. No effect has occurred.
        request: animus_adapter_rlm_kernel::HostRequest,
        /// Pure admission result bound to current policy.
        admission: animus_adapter_rlm_kernel::HostRequestAdmission,
        /// Metadata-only request receipt.
        receipt: animus_adapter_rlm_kernel::Receipt,
    },
}

impl SteelRlmEvaluation {
    /// Returns bounded model-visible tool text.
    #[must_use]
    pub fn model_visible_text(&self) -> String {
        match self {
            Self::Value { value, .. } => format!("{SAFE_RESULT_PREFIX}{value}"),
            Self::HostRequest { admission, .. } => {
                format!("{HOST_REQUEST_RESULT_PREFIX}{}:admitted", admission.operation().as_str())
            }
        }
    }

    /// Returns the metadata-only receipt.
    #[must_use]
    pub const fn receipt(&self) -> &animus_adapter_rlm_kernel::Receipt {
        match self {
            Self::Value { receipt, .. } | Self::HostRequest { receipt, .. } => receipt,
        }
    }
}

/// Persistent transactional model-facing Steel environment.
pub struct SteelRlmKernel {
    policy: animus_adapter_rlm_kernel::RlmKernelPolicy,
    environment: animus_adapter_rlm_kernel::EnvironmentState,
    committed_sources: Vec<String>,
    engine: steel::steel_vm::engine::Engine,
}

impl SteelRlmKernel {
    /// Builds an empty model-facing Steel environment from an admitted policy.
    ///
    /// # Errors
    ///
    /// Returns [`SteelRlmError`] when the restricted Steel prelude fails.
    pub fn new(policy: animus_adapter_rlm_kernel::RlmKernelPolicy) -> Result<Self, SteelRlmError> {
        let engine = build_engine(&[], &[])?;
        Ok(Self {
            policy,
            environment: animus_adapter_rlm_kernel::EnvironmentState::default(),
            committed_sources: Vec::new(),
            engine,
        })
    }

    /// Returns the admitted policy.
    #[must_use]
    pub const fn policy(&self) -> &animus_adapter_rlm_kernel::RlmKernelPolicy {
        &self.policy
    }

    /// Returns pure persistent environment metadata.
    #[must_use]
    pub const fn environment(&self) -> &animus_adapter_rlm_kernel::EnvironmentState {
        &self.environment
    }

    /// Atomically binds prompt context and reconstructs the restricted engine.
    ///
    /// # Errors
    ///
    /// Returns [`SteelRlmError`] without changing current state when validation
    /// or candidate-engine reconstruction fails.
    pub fn bind_prompt_context(
        &mut self,
        bindings: &[animus_adapter_rlm_kernel::ContextBinding],
    ) -> Result<(), SteelRlmError> {
        let candidate_environment =
            animus_adapter_rlm_kernel::bind_prompt_context(&self.environment, bindings, &self.policy)?;
        let candidate_engine = build_engine(candidate_environment.bindings(), self.committed_sources.as_slice())?;
        self.environment = candidate_environment;
        self.engine = candidate_engine;
        Ok(())
    }

    /// Evaluates one model-written Steel cell transactionally.
    ///
    /// Successful cells persist across later calls. A failed cell, bound
    /// violation, timeout, or malformed host request leaves committed state
    /// unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`SteelRlmError`] for contract denial, timeout, Steel failure, or
    /// malformed typed output.
    pub fn evaluate(&mut self, source: &str) -> Result<SteelRlmEvaluation, SteelRlmError> {
        let metrics = structural_metrics(source);
        let plan = animus_adapter_rlm_kernel::plan_evaluation(
            &self.environment,
            &animus_adapter_rlm_kernel::EvaluationRequest { source, metrics },
            &self.policy,
        )?;
        let mut candidate = build_engine(self.environment.bindings(), self.committed_sources.as_slice())?;
        let values = evaluate_with_timeout(&mut candidate, source)?;
        let last = values.last().cloned().unwrap_or(steel::SteelVal::Void);
        let parsed_request = parse_host_request(last.clone(), &self.policy)?;
        let candidate_environment =
            animus_adapter_rlm_kernel::commit_evaluation(&self.environment, &plan, &self.policy)?;
        let result = if let Some((request, admission)) = parsed_request {
            let receipt = evaluation_receipt(&self.policy, &plan, Some(admission.operation()), false)?;
            SteelRlmEvaluation::HostRequest {
                request,
                admission,
                receipt,
            }
        } else {
            let value = bounded_value_text(&last)?;
            let receipt = evaluation_receipt(&self.policy, &plan, None, false)?;
            SteelRlmEvaluation::Value { value, receipt }
        };
        self.environment = candidate_environment;
        self.committed_sources.push(String::from(source));
        self.engine = candidate;
        Ok(result)
    }

    /// Plans prompt compaction while preserving the persistent environment.
    ///
    /// # Errors
    ///
    /// Returns [`SteelRlmError`] if an implicit clear is requested.
    pub fn plan_compaction(&self) -> Result<animus_adapter_rlm_kernel::KernelCompactionPlan, SteelRlmError> {
        Ok(animus_adapter_rlm_kernel::plan_compaction(
            &self.environment,
            animus_adapter_rlm_kernel::CompactionRequest {
                cause: animus_adapter_rlm_kernel::CompactionCause::Compaction,
                clear_kernel: false,
            },
        )?)
    }

    /// Clears the kernel only through an explicit host action.
    ///
    /// # Errors
    ///
    /// Returns [`SteelRlmError`] when clear planning or fresh-engine creation fails.
    pub fn clear_explicit(&mut self) -> Result<animus_adapter_rlm_kernel::Receipt, SteelRlmError> {
        let clear = animus_adapter_rlm_kernel::plan_compaction(
            &self.environment,
            animus_adapter_rlm_kernel::CompactionRequest {
                cause: animus_adapter_rlm_kernel::CompactionCause::ExplicitHostClear,
                clear_kernel: true,
            },
        )?;
        let engine = build_engine(&[], &[])?;
        let counts = [
            animus_adapter_rlm_kernel::ReceiptCount::new("cleared_bindings", u64::from(clear.binding_count()))?,
            animus_adapter_rlm_kernel::ReceiptCount::new("cleared_definitions", u64::from(clear.definition_count()))?,
        ];
        let receipt = animus_adapter_rlm_kernel::Receipt::from_draft(&animus_adapter_rlm_kernel::ReceiptDraft {
            status: animus_adapter_rlm_kernel::RlmKernelStatus::Cleared,
            policy_identity: self.policy.identity(),
            operation_classes: &[animus_adapter_rlm_kernel::HostOperationClass::KernelClear],
            counts: &counts,
            child_lineage_hashes: &[],
            artifact_hashes: &[],
            kernel_state_preserved: false,
            non_claims: string_refs(self.policy.non_claims()).as_slice(),
            variable_value: None,
            code_body: None,
            child_transcript: None,
            credential: None,
            raw_path: None,
            claims: &["explicit_host_clear"],
        })?;
        self.environment = animus_adapter_rlm_kernel::EnvironmentState::default();
        self.committed_sources.clear();
        self.engine = engine;
        Ok(receipt)
    }
}

/// Concrete persistent Steel REPL adapter for the generic Animus tool port.
pub struct SteelRlmToolAdapter {
    kernel: SteelRlmKernel,
}

impl SteelRlmToolAdapter {
    /// Builds a persistent Steel REPL tool adapter.
    ///
    /// # Errors
    ///
    /// Returns [`SteelRlmError`] when kernel initialization fails.
    pub fn new(policy: animus_adapter_rlm_kernel::RlmKernelPolicy) -> Result<Self, SteelRlmError> {
        Ok(Self {
            kernel: SteelRlmKernel::new(policy)?,
        })
    }

    /// Returns the persistent model-facing kernel.
    #[must_use]
    pub const fn kernel(&self) -> &SteelRlmKernel {
        &self.kernel
    }

    /// Returns the mutable persistent model-facing kernel.
    #[must_use]
    pub const fn kernel_mut(&mut self) -> &mut SteelRlmKernel {
        &mut self.kernel
    }
}

impl ComponentAdapter<animus_adapters::ToolRequest, animus_adapters::ToolResponse> for SteelRlmToolAdapter {
    type Error = SteelRlmError;

    fn call(&mut self, request: &animus_adapters::ToolRequest) -> Result<animus_adapters::ToolResponse, Self::Error> {
        validate_tool_request(request)?;
        let source = request
            .arguments()
            .iter()
            .find(|field| field.name() == CODE_ARGUMENT)
            .map(animus_adapters::tool::ArgumentField::value)
            .ok_or_else(|| SteelRlmError::ToolRequest {
                message: String::from("required code argument is missing"),
            })?;
        let evaluation = self.kernel.evaluate(source)?;
        let output_text = evaluation.model_visible_text();
        let bounded = animus_adapters::tool::BoundedOutput::new(
            &animus_adapters::tool::OutputDraft {
                body: output_text.as_str(),
                full_output_reference: None,
                hash_requirement: animus_adapters::tool::HashRequirement::Required,
            },
            request.output_policy(),
        )
        .map_err(|error| SteelRlmError::ToolOutput {
            message: format!("{error:?}"),
        })?;
        Ok(animus_adapters::ToolResponse::from_bounded(
            bounded,
            animus_adapters::tool::OutputRedactionClass::Safe,
        ))
    }
}

fn build_engine(
    bindings: &[animus_adapter_rlm_kernel::ContextBinding],
    committed_sources: &[String],
) -> Result<steel::steel_vm::engine::Engine, SteelRlmError> {
    let mut engine = steel::steel_vm::engine::Engine::new_sandboxed();
    run_program(&mut engine, RLM_PRELUDE)?;
    for binding in bindings {
        let program = format!("(define {} {})", binding.name(), crate::quoting::scheme_string(binding.value()));
        run_program(&mut engine, program.as_str())?;
    }
    for source in committed_sources {
        run_program(&mut engine, source.as_str())?;
    }
    Ok(engine)
}

fn run_program(
    engine: &mut steel::steel_vm::engine::Engine,
    source: &str,
) -> Result<Vec<steel::SteelVal>, SteelRlmError> {
    engine.run(String::from(source)).map_err(|error| SteelRlmError::Evaluate {
        message: format!("{error}"),
    })
}

fn evaluate_with_timeout(
    engine: &mut steel::steel_vm::engine::Engine,
    source: &str,
) -> Result<Vec<steel::SteelVal>, SteelRlmError> {
    let handler =
        steel::steel_vm::interrupt::InterruptHandler::new(engine, Duration::from_millis(EVALUATION_TIMEOUT_MS));
    handler.run_with_timeout(|| run_program(engine, source))
}

fn structural_metrics(source: &str) -> animus_adapter_rlm_kernel::EvaluationMetrics {
    let mut current_depth = 0u32;
    let mut maximum_depth = 0u32;
    for character in source.chars() {
        match character {
            '(' => {
                current_depth = current_depth.saturating_add(1);
                maximum_depth = maximum_depth.max(current_depth);
            }
            ')' => current_depth = current_depth.saturating_sub(1),
            _other => {}
        }
    }
    animus_adapter_rlm_kernel::EvaluationMetrics {
        fuel: u64::try_from(source.chars().count()).unwrap_or(u64::MAX),
        recursion_depth: maximum_depth,
        allocation_bytes: u64::try_from(source.len())
            .unwrap_or(u64::MAX)
            .saturating_mul(ESTIMATED_ALLOCATION_BYTES_PER_SOURCE_BYTE),
    }
}

fn parse_host_request(
    value: steel::SteelVal,
    policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
) -> Result<
    Option<(animus_adapter_rlm_kernel::HostRequest, animus_adapter_rlm_kernel::HostRequestAdmission)>,
    SteelRlmError,
> {
    let steel::SteelVal::ListV(list) = value else {
        return Ok(None);
    };
    let items = list.iter().cloned().collect::<Vec<_>>();
    if items.len() < TAGGED_REQUEST_ARITY || symbol(&items, TAG_INDEX)? != HOST_REQUEST_TAG {
        return Ok(None);
    }
    let request = request_from_items(&items, policy)?;
    let admission = animus_adapter_rlm_kernel::admit_host_request(policy, &request)?;
    Ok(Some((request, admission)))
}

fn request_from_items(
    items: &[steel::SteelVal],
    policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
) -> Result<animus_adapter_rlm_kernel::HostRequest, SteelRlmError> {
    let operation = string(items, OPERATION_INDEX)?;
    let max_chars = policy.bounds().max_host_request_chars;
    match operation {
        FILE_READ_TAG => {
            expect_arity(items, FILE_READ_ARITY)?;
            Ok(animus_adapter_rlm_kernel::HostRequest::file_read(string(items, FIRST_ARGUMENT_INDEX)?)?)
        }
        FILE_EDIT_TAG => {
            expect_arity(items, FILE_EDIT_ARITY)?;
            Ok(animus_adapter_rlm_kernel::HostRequest::file_edit(
                string(items, FIRST_ARGUMENT_INDEX)?,
                string(items, SECOND_ARGUMENT_INDEX)?,
                max_chars,
            )?)
        }
        SHELL_TAG => {
            expect_arity(items, SHELL_ARITY)?;
            Ok(animus_adapter_rlm_kernel::HostRequest::shell(string(items, FIRST_ARGUMENT_INDEX)?, max_chars)?)
        }
        TOOL_TAG => {
            expect_arity(items, TOOL_ARITY)?;
            Ok(animus_adapter_rlm_kernel::HostRequest::tool(
                string(items, FIRST_ARGUMENT_INDEX)?,
                string(items, SECOND_ARGUMENT_INDEX)?,
                max_chars,
            )?)
        }
        SUBAGENT_TAG => child_request(items, policy).map(animus_adapter_rlm_kernel::HostRequest::Subagent),
        CONTEXT_READ_TAG => {
            expect_arity(items, CONTEXT_READ_ARITY)?;
            Ok(animus_adapter_rlm_kernel::HostRequest::context_read(string(items, FIRST_ARGUMENT_INDEX)?)?)
        }
        KERNEL_CLEAR_TAG => {
            expect_arity(items, KERNEL_CLEAR_ARITY)?;
            Ok(animus_adapter_rlm_kernel::HostRequest::KernelClear)
        }
        _other => Err(invalid_host_request("unsupported operation class")),
    }
}

fn child_request(
    items: &[steel::SteelVal],
    policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
) -> Result<animus_adapter_rlm_kernel::ChildSpawnRequest, SteelRlmError> {
    expect_arity(items, SUBAGENT_ARITY)?;
    Ok(animus_adapter_rlm_kernel::ChildSpawnRequest::new(
        string(items, FIRST_ARGUMENT_INDEX)?,
        string(items, SECOND_ARGUMENT_INDEX)?,
        unsigned_u32(items, CHILD_DEPTH_INDEX)?,
        unsigned_u32(items, CHILD_FAN_OUT_INDEX)?,
        unsigned_u64(items, CHILD_TOKEN_BUDGET_INDEX)?,
        unsigned_u64(items, CHILD_WALL_TIME_INDEX)?,
        unsigned_u32(items, CHILD_RESULT_BUDGET_INDEX)?,
        false,
        false,
    )
    .and_then(|request| {
        animus_adapter_rlm_kernel::admit_child_spawn(policy, &request)?;
        Ok(request)
    })?)
}

fn symbol(items: &[steel::SteelVal], index: usize) -> Result<&str, SteelRlmError> {
    match items.get(index) {
        Some(steel::SteelVal::SymbolV(value)) => Ok(value.as_str()),
        _other => Err(invalid_host_request("expected symbol tag")),
    }
}

fn string(items: &[steel::SteelVal], index: usize) -> Result<&str, SteelRlmError> {
    match items.get(index) {
        Some(steel::SteelVal::StringV(value)) => Ok(value.as_str()),
        _other => Err(invalid_host_request("expected string field")),
    }
}

fn unsigned_u32(items: &[steel::SteelVal], index: usize) -> Result<u32, SteelRlmError> {
    let value = integer(items, index)?;
    u32::try_from(value).map_err(|_error| invalid_host_request("integer must fit u32"))
}

fn unsigned_u64(items: &[steel::SteelVal], index: usize) -> Result<u64, SteelRlmError> {
    let value = integer(items, index)?;
    u64::try_from(value).map_err(|_error| invalid_host_request("integer must be non-negative"))
}

fn integer(items: &[steel::SteelVal], index: usize) -> Result<isize, SteelRlmError> {
    match items.get(index) {
        Some(steel::SteelVal::IntV(value)) => Ok(*value),
        _other => Err(invalid_host_request("expected integer field")),
    }
}

fn expect_arity(items: &[steel::SteelVal], expected: usize) -> Result<(), SteelRlmError> {
    if items.len() == expected {
        Ok(())
    } else {
        Err(invalid_host_request("unexpected host-request arity"))
    }
}

fn invalid_host_request(message: &str) -> SteelRlmError {
    SteelRlmError::InvalidHostRequest {
        message: String::from(message),
    }
}

fn bounded_value_text(value: &steel::SteelVal) -> Result<String, SteelRlmError> {
    let rendered = match value {
        steel::SteelVal::StringV(text) => String::from(text.as_str()),
        steel::SteelVal::IntV(number) => number.to_string(),
        steel::SteelVal::BoolV(true) => String::from("#t"),
        steel::SteelVal::BoolV(false) => String::from("#f"),
        steel::SteelVal::Void => String::from("#<void>"),
        other => format!("{other:?}"),
    };
    let actual_chars = u32::try_from(rendered.chars().count()).unwrap_or(u32::MAX);
    if actual_chars > MAX_VISIBLE_VALUE_CHARS {
        return Err(SteelRlmError::ToolOutput {
            message: String::from("Steel value exceeds model-visible bound"),
        });
    }
    Ok(rendered)
}

fn evaluation_receipt(
    policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
    plan: &animus_adapter_rlm_kernel::EvaluationPlan,
    operation: Option<animus_adapter_rlm_kernel::HostOperationClass>,
    preserved: bool,
) -> Result<animus_adapter_rlm_kernel::Receipt, SteelRlmError> {
    let counts = [
        animus_adapter_rlm_kernel::ReceiptCount::new("source_chars", u64::from(plan.source_chars()))?,
        animus_adapter_rlm_kernel::ReceiptCount::new("fuel", plan.metrics().fuel)?,
        animus_adapter_rlm_kernel::ReceiptCount::new("recursion_depth", u64::from(plan.metrics().recursion_depth))?,
        animus_adapter_rlm_kernel::ReceiptCount::new("allocation_bytes", plan.metrics().allocation_bytes)?,
    ];
    let operations = operation.map_or_else(Vec::new, |value| vec![value]);
    let non_claims = string_refs(policy.non_claims());
    Ok(animus_adapter_rlm_kernel::Receipt::from_draft(&animus_adapter_rlm_kernel::ReceiptDraft {
        status: animus_adapter_rlm_kernel::RlmKernelStatus::Completed,
        policy_identity: policy.identity(),
        operation_classes: operations.as_slice(),
        counts: &counts,
        child_lineage_hashes: &[],
        artifact_hashes: &[plan.source_hash()],
        kernel_state_preserved: preserved,
        non_claims: non_claims.as_slice(),
        variable_value: None,
        code_body: None,
        child_transcript: None,
        credential: None,
        raw_path: None,
        claims: &["bounded_processing"],
    })?)
}

fn string_refs(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

fn validate_tool_request(request: &animus_adapters::ToolRequest) -> Result<(), SteelRlmError> {
    if request.name() != animus_adapter_rlm_kernel::STEEL_REPL_TOOL_NAME {
        return Err(SteelRlmError::ToolRequest {
            message: String::from("tool name does not match steel_repl"),
        });
    }
    if request.schema_identity() != animus_adapter_rlm_kernel::STEEL_REPL_TOOL_SCHEMA {
        return Err(SteelRlmError::ToolRequest {
            message: String::from("tool schema does not match Steel REPL schema"),
        });
    }
    if request.approval_posture() == animus_adapters::tool::ApprovalPosture::Denied {
        return Err(SteelRlmError::ToolRequest {
            message: String::from("tool approval is denied"),
        });
    }
    if request.arguments().len() != 1 {
        return Err(SteelRlmError::ToolRequest {
            message: String::from("Steel REPL requires exactly one code argument"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIME_AGENT_REVISION: &str = "c5991bc853d27754aed345c13ff4d2e05c40f1f4";
    const SOURCE_CHAR_LIMIT: u32 = 4_096;
    const ENVIRONMENT_CHAR_LIMIT: u32 = 16_384;
    const BINDING_LIMIT: u32 = 16;
    const DEFINITION_LIMIT: u32 = 32;
    const FUEL_LIMIT: u64 = 100_000;
    const RECURSION_LIMIT: u32 = 64;
    const ALLOCATION_LIMIT: u64 = 1_048_576;
    const HOST_REQUEST_CHAR_LIMIT: u32 = 8_192;
    const CHILD_DEPTH_LIMIT: u32 = 2;
    const CHILD_FAN_OUT_LIMIT: u32 = 4;
    const CHILD_TOKEN_LIMIT: u64 = 16_000;
    const CHILD_WALL_TIME_LIMIT_MS: u64 = 60_000;
    const CHILD_RESULT_CHAR_LIMIT: u32 = 8_192;
    const TOOL_VISIBLE_BUDGET: u32 = 512;
    const CHILD_PROFILE: &str = "focused-child";
    const CORRELATION_ID: &str = "steel-cell-1";

    #[test]
    fn persistent_kernel_and_tool_adapter_keep_state_across_calls() {
        // r[verify agent_harness_runtime.rlm_kernel.environment.persists]
        // r[verify agent_harness_runtime.rlm_kernel.verification.positive]
        let mut kernel = SteelRlmKernel::new(policy()).expect("kernel must initialize");
        let binding = animus_adapter_rlm_kernel::ContextBinding::new(
            "task_context",
            "adapter boundaries",
            ENVIRONMENT_CHAR_LIMIT,
        )
        .expect("binding must parse");
        kernel.bind_prompt_context(&[binding]).expect("context must bind");
        kernel.evaluate("(define (double value) (* value 2))").expect("definition must commit");
        let result = kernel.evaluate("(double 21)").expect("helper must persist");
        assert!(matches!(
            result,
            SteelRlmEvaluation::Value { value, .. } if value == "42"
        ));
        assert!(kernel.plan_compaction().expect("compaction must plan").kernel_state_preserved());

        let mut adapter = SteelRlmToolAdapter::new(policy()).expect("adapter must initialize");
        adapter.call(&tool_request("(define shared 7)")).expect("definition cell must run");
        let response = adapter.call(&tool_request("(+ shared 1)")).expect("later cell must see state");
        assert_eq!(response.output(), "value:8");
    }

    #[test]
    fn typed_host_requests_are_admitted_without_running_effects() {
        // r[verify agent_harness_runtime.rlm_kernel.host_requests.admitted]
        let mut kernel = SteelRlmKernel::new(policy()).expect("kernel must initialize");
        let result = kernel.evaluate("(animus-host-read \"resource:README\")").expect("typed request must parse");
        assert!(matches!(
            result,
            SteelRlmEvaluation::HostRequest {
                request: animus_adapter_rlm_kernel::HostRequest::FileRead { .. },
                admission,
                ..
            } if admission.operation() == animus_adapter_rlm_kernel::HostOperationClass::FileRead
        ));
    }

    #[test]
    fn direct_effects_unadmitted_requests_and_failed_cells_do_not_commit() {
        // r[verify agent_harness_runtime.rlm_kernel.boundary.no_ambient]
        // r[verify agent_harness_runtime.rlm_kernel.host_requests.rejected]
        // r[verify agent_harness_runtime.rlm_kernel.environment.bounded]
        let mut kernel = SteelRlmKernel::new(restricted_policy()).expect("kernel must initialize");
        let initial = kernel.environment().clone();
        assert!(kernel.evaluate("(create-directory! \"forbidden\")").is_err());
        assert_eq!(kernel.environment(), &initial);
        assert!(kernel.evaluate("(define (forever) (forever)) (forever)").is_err());
        assert_eq!(kernel.environment(), &initial);
        assert!(matches!(
            kernel.evaluate("(animus-host-shell \"cargo test\")"),
            Err(SteelRlmError::Contract(animus_adapter_rlm_kernel::Error::OperationNotAdmitted { .. }))
        ));
        assert_eq!(kernel.environment(), &initial);
    }

    fn policy() -> animus_adapter_rlm_kernel::RlmKernelPolicy {
        policy_with_operations(&[
            animus_adapter_rlm_kernel::HostOperationClass::FileRead,
            animus_adapter_rlm_kernel::HostOperationClass::FileEdit,
            animus_adapter_rlm_kernel::HostOperationClass::Shell,
            animus_adapter_rlm_kernel::HostOperationClass::Tool,
            animus_adapter_rlm_kernel::HostOperationClass::Subagent,
            animus_adapter_rlm_kernel::HostOperationClass::ContextRead,
            animus_adapter_rlm_kernel::HostOperationClass::KernelClear,
        ])
    }

    fn restricted_policy() -> animus_adapter_rlm_kernel::RlmKernelPolicy {
        policy_with_operations(&[animus_adapter_rlm_kernel::HostOperationClass::ContextRead])
    }

    fn policy_with_operations(
        operations: &[animus_adapter_rlm_kernel::HostOperationClass],
    ) -> animus_adapter_rlm_kernel::RlmKernelPolicy {
        animus_adapter_rlm_kernel::RlmKernelPolicy::from_draft(&animus_adapter_rlm_kernel::RlmKernelPolicyDraft {
            schema: animus_adapter_rlm_kernel::RLM_KERNEL_POLICY_SCHEMA,
            cohort: "prime-agent-rlm-2026-08-06",
            cohort_revision: PRIME_AGENT_REVISION,
            profile: "model-facing-steel",
            bounds: animus_adapter_rlm_kernel::ResourceBounds {
                max_source_chars: SOURCE_CHAR_LIMIT,
                max_environment_chars: ENVIRONMENT_CHAR_LIMIT,
                max_bindings: BINDING_LIMIT,
                max_definitions: DEFINITION_LIMIT,
                max_evaluation_fuel: FUEL_LIMIT,
                max_recursion_depth: RECURSION_LIMIT,
                max_allocation_bytes: ALLOCATION_LIMIT,
                max_host_request_chars: HOST_REQUEST_CHAR_LIMIT,
                max_child_depth: CHILD_DEPTH_LIMIT,
                max_child_fan_out: CHILD_FAN_OUT_LIMIT,
                max_child_tokens: CHILD_TOKEN_LIMIT,
                max_child_wall_time_ms: CHILD_WALL_TIME_LIMIT_MS,
                max_child_result_chars: CHILD_RESULT_CHAR_LIMIT,
            },
            admitted_operations: operations,
            admitted_child_profiles: &[CHILD_PROFILE],
            non_claims: animus_adapter_rlm_kernel::REQUIRED_NON_CLAIMS,
        })
        .expect("test policy must parse")
    }

    fn tool_request(source: &str) -> animus_adapters::ToolRequest {
        let arguments =
            animus_adapters::tool::PreparedArguments::prepare(&animus_adapters::tool::ArgumentPreparationRequest {
                current_schema_identity: animus_adapter_rlm_kernel::STEEL_REPL_TOOL_SCHEMA,
                allowed_field_names: &[CODE_ARGUMENT],
                compatibility_rules: &[],
                fields: &[animus_adapters::tool::ArgumentFieldDraft {
                    name: Some(CODE_ARGUMENT),
                    value: Some(source),
                }],
            })
            .expect("arguments must prepare");
        animus_adapters::ToolRequest::from_draft(&animus_adapters::tool::RequestDraft {
            name: Some(animus_adapter_rlm_kernel::STEEL_REPL_TOOL_NAME),
            arguments: Some(&arguments),
            correlation_id: Some(CORRELATION_ID),
            output_policy: animus_adapters::tool::OutputPolicy {
                visible_char_budget: TOOL_VISIBLE_BUDGET,
            },
            mutation_targets: &[],
            required_capability_labels: &[animus_adapter_rlm_kernel::STEEL_REPL_CAPABILITY],
            approval_posture: "pre_approved",
        })
        .expect("tool request must parse")
    }
}

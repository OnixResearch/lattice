use crate::openai::Transport;

const ONE_DECISION: u32 = 1;
const ONE_MODEL_CALL: u64 = 1;

struct BoundedLoopFabric<Fabric> {
    inner: Fabric,
    decision_count: u32,
    turn_limit: u32,
}

impl<Fabric> BoundedLoopFabric<Fabric> {
    fn new(inner: Fabric, turn_limit: u32) -> Result<Self, crate::agent_node::AgentError> {
        super::validate_turn_limit(turn_limit)?;
        Ok(Self {
            inner,
            decision_count: 0,
            turn_limit,
        })
    }
}

#[derive(Debug)]
enum BoundedLoopError<Inner> {
    Admission(crate::agent_node::AgentError),
    Inner(Inner),
}

impl<Inner> core::fmt::Display for BoundedLoopError<Inner>
where Inner: core::fmt::Display
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Admission(error) => error.fmt(formatter),
            Self::Inner(error) => error.fmt(formatter),
        }
    }
}

impl<Fabric> animus_runtime::LoopFabric for BoundedLoopFabric<Fabric>
where Fabric: animus_runtime::LoopFabric
{
    type Error = BoundedLoopError<Fabric::Error>;

    fn decide(
        &mut self,
        turn: &animus_runtime::RuntimeLoopTurn,
    ) -> Result<animus_runtime::RuntimeLoopDecision, Self::Error> {
        super::validate_decision_count(super::DecisionBudget {
            decision_count: self.decision_count,
            turn_limit: self.turn_limit,
        })
        .map_err(BoundedLoopError::Admission)?;
        self.decision_count = self.decision_count.checked_add(ONE_DECISION).ok_or_else(|| {
            BoundedLoopError::Admission(crate::agent_node::AgentError::Adapter(
                "Animus decision count overflow".to_string(),
            ))
        })?;
        self.inner.decide(turn).map_err(BoundedLoopError::Inner)
    }
}

struct LatticeModelAdapter<'a> {
    runtime: &'a crate::openai::RuntimeConfig,
    request: &'a crate::agent_node::AgentRequest,
    context: &'a crate::agent_node::ExecutionContext,
    global_output_max_bytes: u64,
    transport: &'a dyn Transport,
    model_call_count: u64,
}

impl<'a> LatticeModelAdapter<'a> {
    const fn new(
        runtime: &'a crate::openai::RuntimeConfig,
        request: &'a crate::agent_node::AgentRequest,
        context: &'a crate::agent_node::ExecutionContext,
        global_output_max_bytes: u64,
        transport: &'a dyn Transport,
    ) -> Self {
        Self {
            runtime,
            request,
            context,
            global_output_max_bytes,
            transport,
            model_call_count: 0,
        }
    }

    fn model_label(&self) -> &str {
        self.request
            .model_hint
            .as_deref()
            .and_then(non_empty_trimmed)
            .or_else(|| non_empty_trimmed(&self.runtime.model))
            .unwrap_or("unresolved")
    }

    fn provider_request(&self, prompt: &str) -> crate::agent_node::AgentRequest {
        crate::agent_node::AgentRequest {
            target: crate::agent_node::AgentTarget::Animus,
            prompt: prompt.to_string(),
            output_mode: self.request.output_mode,
            model_hint: self.request.model_hint.clone(),
            tool_policy: String::new(),
            context_refs: Vec::new(),
            cwd: None,
            session: None,
            text_inputs: Vec::new(),
            image_inputs: Vec::new(),
            json_inputs: Vec::new(),
        }
    }
}

impl animus_adapters::ComponentAdapter<animus_adapters::ModelRequest, animus_adapters::ModelResponse>
    for LatticeModelAdapter<'_>
{
    type Error = crate::agent_node::AgentError;

    fn call(&mut self, request: &animus_adapters::ModelRequest) -> Result<animus_adapters::ModelResponse, Self::Error> {
        self.model_call_count = self
            .model_call_count
            .checked_add(ONE_MODEL_CALL)
            .ok_or_else(|| crate::agent_node::AgentError::Adapter("Animus model call count overflow".to_string()))?;
        let provider_request = self.provider_request(request.prompt());
        let provider_response = crate::openai::execute(
            self.runtime,
            &provider_request,
            self.context,
            self.global_output_max_bytes,
            self.transport,
        )?;
        let message = provider_response_message(provider_response.output)?;
        animus_adapters::ModelResponse::new(&message).map_err(|error| {
            crate::agent_node::AgentError::Adapter(format!("Animus rejected the model response: {error}"))
        })
    }
}

#[derive(Debug)]
struct DisabledToolError;

impl core::fmt::Display for DisabledToolError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("Animus tools are disabled by Lattice")
    }
}

struct DisabledToolAdapter;

impl animus_adapters::ComponentAdapter<animus_adapters::ToolRequest, animus_adapters::ToolResponse>
    for DisabledToolAdapter
{
    type Error = DisabledToolError;

    fn call(&mut self, _request: &animus_adapters::ToolRequest) -> Result<animus_adapters::ToolResponse, Self::Error> {
        Err(DisabledToolError)
    }
}

// r[impl lattice.agent_runtime.animus_leaf]
// r[impl lattice.agent_runtime.animus_fabric]
// r[impl lattice.agent_runtime.animus_effects]
// r[impl lattice.agent_runtime.animus_output]
pub fn execute(
    runtime: &crate::openai::RuntimeConfig,
    request: &crate::agent_node::AgentRequest,
    context: &crate::agent_node::ExecutionContext,
    global_output_max_bytes: u64,
    transport: &dyn Transport,
) -> Result<crate::agent_node::AgentResponse, crate::agent_node::AgentError> {
    execute_with_projection(
        crate::openai::ProjectedChat {
            runtime,
            request,
            projection: crate::agent_prompt::StructuredInputProjection::JsonPretty,
            global_output_max_bytes,
        },
        context,
        transport,
    )
}

pub fn execute_with_projection(
    input: crate::openai::ProjectedChat<'_>,
    context: &crate::agent_node::ExecutionContext,
    transport: &dyn Transport,
) -> Result<crate::agent_node::AgentResponse, crate::agent_node::AgentError> {
    let crate::openai::ProjectedChat {
        runtime,
        request,
        projection,
        global_output_max_bytes,
    } = input;
    let plan = super::plan_execution_with_projection(request, projection)?;
    debug_assert!(!plan.summary.trim().is_empty());
    debug_assert_eq!(plan.program_identity, super::BUILTIN_PROGRAM_IDENTITY);
    debug_assert!(plan.turn_limit > 0);
    let runtime_input = animus_runtime::RuntimeInput::new(&plan.summary).map_err(|error| {
        crate::agent_node::AgentError::PreflightRejected(format!("invalid Animus runtime input: {error}"))
    })?;
    let steel_fabric = animus_steel::SteelLoopFabric::new(plan.program).map_err(|error| {
        crate::agent_node::AgentError::PreflightRejected(format!("invalid Animus Steel program: {error}"))
    })?;
    let mut bounded_fabric = BoundedLoopFabric::new(steel_fabric, plan.turn_limit)?;
    let mut model_adapter = LatticeModelAdapter::new(runtime, request, context, global_output_max_bytes, transport);
    let adapters =
        animus_runtime::RuntimeAdapters::<_, DisabledToolAdapter>::new_optional(Some(&mut model_adapter), None);
    let runtime_result = animus_runtime::run_agent_loop(
        &runtime_input,
        animus_runtime::RuntimeConfig::new(animus_runtime::DEFAULT_MAX_TURNS),
        &mut bounded_fabric,
        adapters,
    );
    map_report(&runtime_result, &model_adapter, plan.output_mode, context)
}

fn map_report(
    runtime_result: &animus_runtime::RunReport,
    model_adapter: &LatticeModelAdapter<'_>,
    output_mode: crate::agent_node::AgentOutputMode,
    context: &crate::agent_node::ExecutionContext,
) -> Result<crate::agent_node::AgentResponse, crate::agent_node::AgentError> {
    match runtime_result.status() {
        animus_runtime::RuntimeStatus::FinalResponse { response } => Ok(crate::agent_node::AgentResponse {
            output: super::decode_final_response(output_mode, response.message())?,
            metadata: super::safe_metadata(super::SafeMetadataInput {
                event_count: u64::try_from(runtime_result.events().len()).map_err(|_| {
                    crate::agent_node::AgentError::Adapter("Animus event count is unsupported".to_string())
                })?,
                model_call_count: model_adapter.model_call_count,
                model_label: model_adapter.model_label(),
            }),
        }),
        animus_runtime::RuntimeStatus::CompletionCandidate { .. } => Err(crate::agent_node::AgentError::Adapter(
            "Animus runtime requested unsupported host-owned completion evaluation".to_string(),
        )),
        animus_runtime::RuntimeStatus::Stopped => Err(crate::agent_node::AgentError::Adapter(
            "Animus runtime stopped without a final response".to_string(),
        )),
        animus_runtime::RuntimeStatus::MaxTurnsExceeded { max_turns } => Err(crate::agent_node::AgentError::Adapter(
            format!("Animus runtime exceeded its {max_turns}-turn limit"),
        )),
        animus_runtime::RuntimeStatus::Failed { failure } => Err(crate::agent_node::AgentError::Adapter(
            context.redact(&format!("Animus runtime failed: {}", failure.message())),
        )),
    }
}

fn provider_response_message(output: crate::agent_node::AgentOutput) -> Result<String, crate::agent_node::AgentError> {
    match output {
        crate::agent_node::AgentOutput::Text(message) => Ok(message),
        crate::agent_node::AgentOutput::Json(value) => serde_json::to_string(&value).map_err(|error| {
            crate::agent_node::AgentError::Adapter(format!("encode Animus model JSON response: {error}"))
        }),
        crate::agent_node::AgentOutput::Image(_) => Err(crate::agent_node::AgentError::OutputTypeMismatch {
            expected: crate::agent_node::AgentOutputMode::Text,
            actual: crate::agent_node::AgentOutputMode::Image,
        }),
        crate::agent_node::AgentOutput::Artifact(_) => Err(crate::agent_node::AgentError::OutputTypeMismatch {
            expected: crate::agent_node::AgentOutputMode::Text,
            actual: crate::agent_node::AgentOutputMode::Artifact,
        }),
    }
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

#[cfg(test)]
mod tests;

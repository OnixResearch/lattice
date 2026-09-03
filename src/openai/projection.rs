//! Projection-aware chat planning and bounded execution.

const HTTPS_SCHEME: &str = "https";

/// One admitted projection-aware chat request.
pub struct ProjectedChat<'a> {
    pub runtime: &'a super::RuntimeConfig,
    pub request: &'a crate::agent_node::AgentRequest,
    pub projection: crate::agent_prompt::StructuredInputProjection,
    pub global_output_max_bytes: u64,
}

struct ChatInput<'a> {
    model: &'a str,
    prompt: &'a str,
    output_mode: crate::agent_node::AgentOutputMode,
}

// r[impl n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.plan]
pub fn plan_chat(
    runtime: &super::RuntimeConfig,
    request: &crate::agent_node::AgentRequest,
    global_output_max_bytes: u64,
) -> Result<super::ChatPlan, crate::agent_node::AgentError> {
    plan_chat_with_projection(ProjectedChat {
        runtime,
        request,
        projection: crate::agent_prompt::StructuredInputProjection::JsonPretty,
        global_output_max_bytes,
    })
}

// r[impl lattice.agent_runtime.projection_boundaries.scenario.wire]
pub fn plan_chat_with_projection(input: ProjectedChat<'_>) -> Result<super::ChatPlan, crate::agent_node::AgentError> {
    let ProjectedChat {
        runtime,
        request,
        projection,
        global_output_max_bytes,
    } = input;
    super::core::validate_supported_request(request)?;
    let endpoint = super::core::validate_endpoint(runtime)?;
    let model = request
        .model_hint
        .as_deref()
        .and_then(super::core::non_empty_trimmed)
        .or_else(|| super::core::non_empty_trimmed(&runtime.model))
        .ok_or_else(|| super::core::preflight("OpenAI-compatible model is empty"))?;
    super::core::validate_numeric_bounds(runtime, global_output_max_bytes)?;
    let api_key_env =
        super::core::optional_non_empty(&runtime.api_key_env, "OpenAI-compatible API-key environment binding")?;
    if api_key_env.is_some() && endpoint.scheme() != HTTPS_SCHEME {
        return Err(super::core::preflight("OpenAI-compatible bearer credentials require an HTTPS endpoint"));
    }
    let prompt = crate::agent_node::render_prompt_with_projection(request, projection)?;
    let body = encode_chat_request(runtime, ChatInput {
        model,
        prompt: &prompt,
        output_mode: request.output_mode,
    })?;
    let request_bytes = u64::try_from(body.len())
        .map_err(|_| super::core::preflight("OpenAI-compatible request body length is unsupported"))?;
    if request_bytes > runtime.max_request_bytes {
        return Err(super::core::preflight(format!(
            "OpenAI-compatible request body exceeds {} bytes",
            runtime.max_request_bytes
        )));
    }
    debug_assert!(endpoint.host_str().is_some());
    debug_assert!(!model.is_empty());
    debug_assert!(request_bytes <= runtime.max_request_bytes);
    Ok(super::ChatPlan {
        adapter_label: crate::agent_node::agent_target_label(request.target),
        endpoint: endpoint.to_string(),
        model: model.to_string(),
        api_key_env,
        timeout_secs: runtime.timeout_secs,
        max_response_bytes: runtime.max_response_bytes.min(global_output_max_bytes),
        output_mode: request.output_mode,
        body,
    })
}

// r[impl n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.credentials]
pub fn execute(
    runtime: &super::RuntimeConfig,
    request: &crate::agent_node::AgentRequest,
    context: &crate::agent_node::ExecutionContext,
    global_output_max_bytes: u64,
    transport: &dyn Transport,
) -> Result<crate::agent_node::AgentResponse, crate::agent_node::AgentError> {
    execute_with_projection(
        ProjectedChat {
            runtime,
            request,
            projection: crate::agent_prompt::StructuredInputProjection::JsonPretty,
            global_output_max_bytes,
        },
        context,
        transport,
    )
}

// r[impl lattice.agent_runtime.projection_boundaries.scenario.output]
pub fn execute_with_projection(
    input: ProjectedChat<'_>,
    context: &crate::agent_node::ExecutionContext,
    transport: &dyn Transport,
) -> Result<crate::agent_node::AgentResponse, crate::agent_node::AgentError> {
    let plan = plan_chat_with_projection(input)?;
    let bearer_token = match plan.api_key_env.as_deref() {
        Some(binding) => Some(context.secret(binding).ok_or_else(|| {
            crate::agent_node::AgentError::Credential(format!(
                "missing OpenAI-compatible credential binding: {binding}"
            ))
        })?),
        None => None,
    };
    let response = transport.execute(&plan, bearer_token)?;
    let response_bytes = u64::try_from(response.body.len()).map_err(|_| {
        crate::agent_node::AgentError::Adapter("OpenAI-compatible response length is unsupported".to_string())
    })?;
    if response_bytes > plan.max_response_bytes {
        return Err(crate::agent_node::AgentError::Adapter(format!(
            "OpenAI-compatible response exceeded {} bytes",
            plan.max_response_bytes
        )));
    }
    debug_assert!(plan.max_response_bytes > 0);
    debug_assert!(response_bytes <= plan.max_response_bytes);
    super::core::decode_response(&plan, &response, context)
}

use super::Transport;

const MILLIUNITS_PER_UNIT: f64 = 1_000.0;
const JSON_OUTPUT_SYSTEM_PROMPT: &str =
    "Return exactly one valid JSON object. Do not wrap it in Markdown fences or include commentary outside JSON.";

fn encode_chat_request(
    runtime: &super::RuntimeConfig,
    input: ChatInput<'_>,
) -> Result<Vec<u8>, crate::agent_node::AgentError> {
    let mut messages = Vec::new();
    if let Some(system_prompt) = combined_system_prompt(runtime, input.output_mode) {
        messages.push(serde_json::json!({"role": "system", "content": system_prompt}));
    }
    let user_prompt = user_prompt(input.prompt, input.output_mode);
    messages.push(serde_json::json!({"role": "user", "content": user_prompt}));
    let mut body = serde_json::json!({
        "model": input.model,
        "stream": false,
        "temperature": f64::from(runtime.temperature_milliunits) / MILLIUNITS_PER_UNIT,
        "max_tokens": runtime.max_tokens,
        "messages": messages,
    });
    if input.output_mode == crate::agent_node::AgentOutputMode::Json {
        body["response_format"] = serde_json::json!({"type": "json_object"});
    }
    if let Some(enable_thinking) = runtime.enable_thinking {
        body["chat_template_kwargs"] = serde_json::json!({"enable_thinking": enable_thinking});
    }
    debug_assert!(body["messages"].as_array().is_some_and(|messages| !messages.is_empty()));
    debug_assert_eq!(body["stream"], serde_json::Value::Bool(false));
    serde_json::to_vec(&body)
        .map_err(|error| crate::agent_node::AgentError::Adapter(format!("encode OpenAI-compatible request: {error}")))
}

fn user_prompt(prompt: &str, output_mode: crate::agent_node::AgentOutputMode) -> String {
    if output_mode == crate::agent_node::AgentOutputMode::Json {
        format!("{prompt}\n\n{JSON_OUTPUT_SYSTEM_PROMPT}")
    } else {
        prompt.to_string()
    }
}

fn combined_system_prompt(
    runtime: &super::RuntimeConfig,
    output_mode: crate::agent_node::AgentOutputMode,
) -> Option<String> {
    let configured = super::core::non_empty_trimmed(&runtime.system_prompt);
    match (configured, output_mode == crate::agent_node::AgentOutputMode::Json) {
        (Some(configured), true) => Some(format!("{configured}\n\n{JSON_OUTPUT_SYSTEM_PROMPT}")),
        (Some(configured), false) => Some(configured.to_string()),
        (None, true) => Some(JSON_OUTPUT_SYSTEM_PROMPT.to_string()),
        (None, false) => None,
    }
}

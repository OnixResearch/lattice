use super::Transport;

const HTTP_SCHEME: &str = "http";
const HTTPS_SCHEME: &str = "https";
const HTTP_SUCCESS_STATUS_MIN: u16 = 200;
const HTTP_SUCCESS_STATUS_MAX_EXCLUSIVE: u16 = 300;
const MILLIUNITS_PER_UNIT: f64 = 1_000.0;
const MAX_TEMPERATURE_MILLIUNITS: u16 = 2_000;
const MAX_TIMEOUT_SECS: u64 = 600;
const MAX_TOKEN_LIMIT: u32 = 1_000_000;
const MAX_BODY_BYTES: u64 = 67_108_864;
const ERROR_DIAGNOSTIC_MAX_CHARS: usize = 2_048;
const JSON_OUTPUT_SYSTEM_PROMPT: &str =
    "Return exactly one valid JSON object. Do not wrap it in Markdown fences or include commentary outside JSON.";

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
    validate_supported_request(request)?;
    let endpoint = validate_endpoint(runtime)?;
    let model = request
        .model_hint
        .as_deref()
        .and_then(non_empty_trimmed)
        .or_else(|| non_empty_trimmed(&runtime.model))
        .ok_or_else(|| preflight("OpenAI-compatible model is empty"))?;
    validate_numeric_bounds(runtime, global_output_max_bytes)?;
    let api_key_env = optional_non_empty(&runtime.api_key_env, "OpenAI-compatible API-key environment binding")?;
    if api_key_env.is_some() && endpoint.scheme() != HTTPS_SCHEME {
        return Err(preflight("OpenAI-compatible bearer credentials require an HTTPS endpoint"));
    }
    let prompt = crate::agent_node::render_agent_command_prompt(request);
    let body = encode_chat_request(runtime, ChatInput {
        model,
        prompt: &prompt,
        output_mode: request.output_mode,
    })?;
    let request_bytes =
        u64::try_from(body.len()).map_err(|_| preflight("OpenAI-compatible request body length is unsupported"))?;
    if request_bytes > runtime.max_request_bytes {
        return Err(preflight(format!("OpenAI-compatible request body exceeds {} bytes", runtime.max_request_bytes)));
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

// r[impl n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.success]
// r[impl n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.failure]
pub fn decode_response(
    plan: &super::ChatPlan,
    response: &super::HttpResponse,
    context: &crate::agent_node::ExecutionContext,
) -> Result<crate::agent_node::AgentResponse, crate::agent_node::AgentError> {
    if !(HTTP_SUCCESS_STATUS_MIN..HTTP_SUCCESS_STATUS_MAX_EXCLUSIVE).contains(&response.status) {
        let body = bounded_diagnostic(&String::from_utf8_lossy(&response.body));
        return Err(crate::agent_node::AgentError::Adapter(
            context.redact(&format!("OpenAI-compatible endpoint returned HTTP {}: {body}", response.status)),
        ));
    }
    let body = std::str::from_utf8(&response.body).map_err(|_| {
        crate::agent_node::AgentError::Adapter("OpenAI-compatible response is not valid UTF-8".to_string())
    })?;
    let envelope: super::ResponseEnvelope = serde_json::from_str(body).map_err(|error| {
        crate::agent_node::AgentError::Adapter(format!("malformed OpenAI-compatible response JSON: {error}"))
    })?;
    let choice = envelope.choices.first().ok_or_else(|| {
        crate::agent_node::AgentError::Adapter("OpenAI-compatible response has no choices".to_string())
    })?;
    let content = choice.message.content.as_deref().and_then(non_empty_trimmed).ok_or_else(|| {
        crate::agent_node::AgentError::Adapter("OpenAI-compatible response content is empty or missing".to_string())
    })?;
    debug_assert!((HTTP_SUCCESS_STATUS_MIN..HTTP_SUCCESS_STATUS_MAX_EXCLUSIVE).contains(&response.status));
    debug_assert!(!content.is_empty());
    let output = decode_content(plan.output_mode, content)?;
    Ok(crate::agent_node::AgentResponse {
        output,
        metadata: serde_json::json!({
            "adapter": plan.adapter_label,
            "provider": "openai_compatible",
            "model": envelope.model.as_deref().unwrap_or(&plan.model),
            "status": response.status,
            "finish_reason": choice.finish_reason,
            "usage": envelope.usage,
        }),
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
    let plan = plan_chat(runtime, request, global_output_max_bytes)?;
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
    decode_response(&plan, &response, context)
}

fn decode_content(
    output_mode: crate::agent_node::AgentOutputMode,
    content: &str,
) -> Result<crate::agent_node::AgentOutput, crate::agent_node::AgentError> {
    match output_mode {
        crate::agent_node::AgentOutputMode::Text => Ok(crate::agent_node::AgentOutput::Text(content.to_string())),
        crate::agent_node::AgentOutputMode::Json => {
            serde_json::from_str(content).map(crate::agent_node::AgentOutput::Json).map_err(|error| {
                crate::agent_node::AgentError::Adapter(format!(
                    "OpenAI-compatible content is not valid requested JSON: {error}"
                ))
            })
        }
        crate::agent_node::AgentOutputMode::Image | crate::agent_node::AgentOutputMode::Artifact => {
            Err(preflight("OpenAI-compatible runtime supports only text and JSON outputs"))
        }
    }
}

fn validate_supported_request(request: &crate::agent_node::AgentRequest) -> Result<(), crate::agent_node::AgentError> {
    if !request.image_inputs.is_empty() {
        return Err(preflight("OpenAI-compatible runtime does not support image inputs"));
    }
    if request.session.as_deref().and_then(non_empty_trimmed).is_some() {
        return Err(preflight("OpenAI-compatible runtime does not support sessions"));
    }
    match request.output_mode {
        crate::agent_node::AgentOutputMode::Text | crate::agent_node::AgentOutputMode::Json => Ok(()),
        crate::agent_node::AgentOutputMode::Image | crate::agent_node::AgentOutputMode::Artifact => {
            Err(preflight("OpenAI-compatible runtime supports only text and JSON outputs"))
        }
    }
}

fn validate_endpoint(runtime: &super::RuntimeConfig) -> Result<url::Url, crate::agent_node::AgentError> {
    let endpoint =
        non_empty_trimmed(&runtime.endpoint).ok_or_else(|| preflight("OpenAI-compatible endpoint is empty"))?;
    let parsed =
        url::Url::parse(endpoint).map_err(|error| preflight(format!("invalid OpenAI-compatible endpoint: {error}")))?;
    if !matches!(parsed.scheme(), HTTP_SCHEME | HTTPS_SCHEME) {
        return Err(preflight("OpenAI-compatible endpoint must use HTTP or HTTPS"));
    }
    if parsed.host_str().is_none() {
        return Err(preflight("OpenAI-compatible endpoint must include a host"));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(preflight("OpenAI-compatible endpoint must not embed credentials"));
    }
    if parsed.fragment().is_some() {
        return Err(preflight("OpenAI-compatible endpoint must not include a fragment"));
    }
    Ok(parsed)
}

fn validate_numeric_bounds(
    runtime: &super::RuntimeConfig,
    global_output_max_bytes: u64,
) -> Result<(), crate::agent_node::AgentError> {
    if runtime.timeout_secs == 0 || runtime.timeout_secs > MAX_TIMEOUT_SECS {
        return Err(preflight(format!("OpenAI-compatible timeout must be between 1 and {MAX_TIMEOUT_SECS} seconds")));
    }
    if runtime.max_tokens == 0 || runtime.max_tokens > MAX_TOKEN_LIMIT {
        return Err(preflight(format!("OpenAI-compatible max_tokens must be between 1 and {MAX_TOKEN_LIMIT}")));
    }
    if runtime.temperature_milliunits > MAX_TEMPERATURE_MILLIUNITS {
        return Err(preflight(format!(
            "OpenAI-compatible temperature_milliunits must not exceed {MAX_TEMPERATURE_MILLIUNITS}"
        )));
    }
    if runtime.max_request_bytes == 0 || runtime.max_request_bytes > MAX_BODY_BYTES {
        return Err(preflight(format!("OpenAI-compatible request bound must be between 1 and {MAX_BODY_BYTES} bytes")));
    }
    if runtime.max_response_bytes == 0 || runtime.max_response_bytes > MAX_BODY_BYTES {
        return Err(preflight(format!(
            "OpenAI-compatible response bound must be between 1 and {MAX_BODY_BYTES} bytes"
        )));
    }
    if global_output_max_bytes == 0 {
        return Err(preflight("global agent output bound must be positive"));
    }
    debug_assert!(runtime.timeout_secs <= MAX_TIMEOUT_SECS);
    debug_assert!(runtime.max_tokens <= MAX_TOKEN_LIMIT);
    debug_assert!(runtime.max_request_bytes <= MAX_BODY_BYTES);
    Ok(())
}

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
    let configured = non_empty_trimmed(&runtime.system_prompt);
    match (configured, output_mode == crate::agent_node::AgentOutputMode::Json) {
        (Some(configured), true) => Some(format!("{configured}\n\n{JSON_OUTPUT_SYSTEM_PROMPT}")),
        (Some(configured), false) => Some(configured.to_string()),
        (None, true) => Some(JSON_OUTPUT_SYSTEM_PROMPT.to_string()),
        (None, false) => None,
    }
}

fn optional_non_empty(value: &Option<String>, label: &str) -> Result<Option<String>, crate::agent_node::AgentError> {
    match value {
        Some(value) => non_empty_trimmed(value)
            .map(str::to_string)
            .map(Some)
            .ok_or_else(|| preflight(format!("{label} is empty"))),
        None => Ok(None),
    }
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

fn preflight(message: impl Into<String>) -> crate::agent_node::AgentError {
    crate::agent_node::AgentError::PreflightRejected(message.into())
}

fn bounded_diagnostic(value: &str) -> String {
    value.chars().take(ERROR_DIAGNOSTIC_MAX_CHARS).collect()
}

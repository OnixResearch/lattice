const HTTP_SCHEME: &str = "http";
const HTTPS_SCHEME: &str = "https";
const HTTP_SUCCESS_STATUS_MIN: u16 = 200;
const HTTP_SUCCESS_STATUS_MAX_EXCLUSIVE: u16 = 300;
const MAX_TIMEOUT_SECS: u64 = 600;
const MAX_TOKEN_LIMIT: u32 = 1_000_000;
const MAX_TEMPERATURE_MILLIUNITS: u16 = 2_000;
const MAX_BODY_BYTES: u64 = 67_108_864;
const ERROR_DIAGNOSTIC_MAX_CHARS: usize = 2_048;

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
    super::reasoning::admit_finish_reason(choice.finish_reason.as_deref())?;
    let content = choice.message.content.as_deref().and_then(non_empty_trimmed).ok_or_else(|| {
        crate::agent_node::AgentError::Adapter("OpenAI-compatible response content is empty or missing".to_string())
    })?;
    let final_content = super::reasoning::admit_final_content(content)?;
    debug_assert!((HTTP_SUCCESS_STATUS_MIN..HTTP_SUCCESS_STATUS_MAX_EXCLUSIVE).contains(&response.status));
    debug_assert!(!final_content.text.is_empty());
    let output = decode_content(plan.output_mode, final_content.text)?;
    Ok(crate::agent_node::AgentResponse {
        output,
        metadata: serde_json::json!({
            "adapter": plan.adapter_label,
            "provider": "openai_compatible",
            "model": envelope.model.as_deref().unwrap_or(&plan.model),
            "status": response.status,
            "finish_reason": choice.finish_reason,
            "usage": envelope.usage,
            "reasoning_suppressed": final_content.reasoning_suppressed,
        }),
    })
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

pub(super) fn validate_supported_request(
    request: &crate::agent_node::AgentRequest,
) -> Result<(), crate::agent_node::AgentError> {
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

pub(super) fn validate_endpoint(runtime: &super::RuntimeConfig) -> Result<url::Url, crate::agent_node::AgentError> {
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

pub(super) fn validate_numeric_bounds(
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

pub(super) fn optional_non_empty(
    value: &Option<String>,
    label: &str,
) -> Result<Option<String>, crate::agent_node::AgentError> {
    match value {
        Some(value) => non_empty_trimmed(value)
            .map(str::to_string)
            .map(Some)
            .ok_or_else(|| preflight(format!("{label} is empty"))),
        None => Ok(None),
    }
}

pub(super) fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

pub(super) fn preflight(message: impl Into<String>) -> crate::agent_node::AgentError {
    crate::agent_node::AgentError::PreflightRejected(message.into())
}

fn bounded_diagnostic(value: &str) -> String {
    value.chars().take(ERROR_DIAGNOSTIC_MAX_CHARS).collect()
}

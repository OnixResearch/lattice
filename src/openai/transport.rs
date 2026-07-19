use std::io::Read;

const RESPONSE_BUFFER_INITIAL_BYTES: u64 = 8_192;
const CONTENT_TYPE_HEADER: &str = "content-type";
const JSON_CONTENT_TYPE: &str = "application/json";
const TRANSPORT_WORKER_NAME: &str = "n8nclone-openai-http";

#[derive(Debug, Clone, Copy, Default)]
pub struct ReqwestClient;

impl super::Transport for ReqwestClient {
    fn execute(
        &self,
        plan: &super::ChatPlan,
        bearer_token: Option<&str>,
    ) -> Result<super::HttpResponse, crate::agent_node::AgentError> {
        let owned_plan = plan.clone();
        let owned_bearer_token = bearer_token.map(str::to_owned);
        let worker = std::thread::Builder::new()
            .name(TRANSPORT_WORKER_NAME.to_string())
            .spawn(move || execute_on_worker(&owned_plan, owned_bearer_token.as_deref()))
            .map_err(|error| {
                crate::agent_node::AgentError::Adapter(format!("start OpenAI-compatible transport worker: {error}"))
            })?;
        worker.join().map_err(|_| {
            crate::agent_node::AgentError::Adapter("OpenAI-compatible transport worker panicked".to_string())
        })?
    }
}

fn execute_on_worker(
    plan: &super::ChatPlan,
    bearer_token: Option<&str>,
) -> Result<super::HttpResponse, crate::agent_node::AgentError> {
    let client = build_client(plan.timeout_secs)?;
    let response = send_request(&client, plan, bearer_token)?;
    read_response(response, plan.max_response_bytes)
}

fn build_client(timeout_secs: u64) -> Result<reqwest::blocking::Client, crate::agent_node::AgentError> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .map_err(|error| {
            crate::agent_node::AgentError::Adapter(format!("build OpenAI-compatible HTTP client: {error}"))
        })
}

fn send_request(
    client: &reqwest::blocking::Client,
    plan: &super::ChatPlan,
    bearer_token: Option<&str>,
) -> Result<reqwest::blocking::Response, crate::agent_node::AgentError> {
    let mut request =
        client.post(&plan.endpoint).header(CONTENT_TYPE_HEADER, JSON_CONTENT_TYPE).body(plan.body.clone());
    if let Some(token) = bearer_token {
        request = request.bearer_auth(token);
    }
    request.send().map_err(|error| {
        if error.is_timeout() {
            crate::agent_node::AgentError::Adapter(format!(
                "OpenAI-compatible request timed out after {} seconds",
                plan.timeout_secs
            ))
        } else {
            crate::agent_node::AgentError::Adapter(format!("OpenAI-compatible transport failed: {error}"))
        }
    })
}

fn read_response(
    response: reqwest::blocking::Response,
    max_response_bytes: u64,
) -> Result<super::HttpResponse, crate::agent_node::AgentError> {
    if max_response_bytes == 0 {
        return Err(crate::agent_node::AgentError::Adapter(
            "OpenAI-compatible response bound must be positive".to_string(),
        ));
    }
    let status = response.status().as_u16();
    let read_limit_bytes = max_response_bytes.checked_add(1).ok_or_else(|| {
        crate::agent_node::AgentError::Adapter("OpenAI-compatible response bound overflow".to_string())
    })?;
    let initial_capacity_bytes =
        usize::try_from(max_response_bytes.min(RESPONSE_BUFFER_INITIAL_BYTES)).map_err(|_| {
            crate::agent_node::AgentError::Adapter("OpenAI-compatible response bound is unsupported".to_string())
        })?;
    let mut body = Vec::with_capacity(initial_capacity_bytes);
    response
        .take(read_limit_bytes)
        .read_to_end(&mut body)
        .map_err(|error| crate::agent_node::AgentError::Adapter(format!("read OpenAI-compatible response: {error}")))?;
    let actual_response_bytes = u64::try_from(body.len()).map_err(|_| {
        crate::agent_node::AgentError::Adapter("OpenAI-compatible response length is unsupported".to_string())
    })?;
    if actual_response_bytes > max_response_bytes {
        return Err(crate::agent_node::AgentError::Adapter(format!(
            "OpenAI-compatible response exceeded {max_response_bytes} bytes"
        )));
    }
    debug_assert!(initial_capacity_bytes <= body.capacity());
    debug_assert!(actual_response_bytes <= max_response_bytes);
    Ok(super::HttpResponse { status, body })
}

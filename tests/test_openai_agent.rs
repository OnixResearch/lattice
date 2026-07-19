//! Tests for the native OpenAI-compatible agent runtime.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use n8nclone::agent_node::AgentAdapter;
use n8nclone::agent_node::AgentError;
use n8nclone::agent_node::AgentOutput;
use n8nclone::agent_node::AgentOutputMode;
use n8nclone::agent_node::AgentRequest;
use n8nclone::agent_node::AgentRuntimeConfig;
use n8nclone::agent_node::AgentTarget;
use n8nclone::agent_node::ExecutionContext as AgentExecutionContext;
use n8nclone::agent_node::ProductionAgentAdapter;
use n8nclone::config::load_app_config;
use n8nclone::engine::value::ImageMetadata;
use n8nclone::engine::value::ImageValue;
use n8nclone::openai::ChatPlan as OpenAiChatPlan;
use n8nclone::openai::HttpResponse as OpenAiHttpResponse;
use n8nclone::openai::ReqwestClient as ReqwestOpenAiTransport;
use n8nclone::openai::RuntimeConfig as OpenAiRuntimeConfig;
use n8nclone::openai::Transport as OpenAiTransport;
use n8nclone::openai::decode_response as decode_openai_chat_response;
use n8nclone::openai::plan_chat as plan_openai_chat;

const CONFIG_EXAMPLE_PATH: &str = "examples/config/openai-compatible-agent.ncl";
const ENDPOINT: &str = "https://example.invalid/v1/chat/completions";
const MODEL: &str = "test-model";
const MODEL_OVERRIDE: &str = "override-model";
const PROMPT: &str = "Return a concise answer";
const TEXT_CONTENT: &str = "bounded answer";
const JSON_CONTENT: &str = r#"{"ok":true}"#;
const API_KEY_BINDING: &str = "OPENAI_API_KEY";
const API_KEY_VALUE: &str = "secret-test-token";
const API_KEY_REFERENCE: &str = "openai_test_key";
const PI_RUNTIME_KEY: &str = "pi";
const CONFIGURED_RUNTIME_COUNT: usize = 2;
const HTTP_STATUS_OK: u16 = 200;
const HTTP_STATUS_REDIRECT: u16 = 302;
const HTTP_STATUS_UNAVAILABLE: u16 = 503;
const TIMEOUT_SECS: u64 = 30;
const TIMEOUT_TEST_SECS: u64 = 1;
const TIMEOUT_SERVER_DELAY_MILLIS: u64 = 1_500;
const MAX_TOKENS: u32 = 1_024;
const TEMPERATURE_MILLIUNITS: u16 = 250;
const MAX_REQUEST_BYTES: u64 = 65_536;
const MAX_RESPONSE_BYTES: u64 = 65_536;
const GLOBAL_OUTPUT_MAX_BYTES: usize = 65_536;
const INVALID_TEMPERATURE_MILLIUNITS: u16 = 2_001;
const SMALL_RESPONSE_MAX_BYTES: u64 = 128;
const OVERSIZED_BODY_BYTES: usize = 129;
const LOCAL_READ_BUFFER_BYTES: usize = 8 * 1_024;
const TEST_IMAGE_MEDIA_TYPE: &str = "image/png";
const INVALID_UTF8_RESPONSE: &[u8] = &[0xff];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObservedCall {
    plan: OpenAiChatPlan,
    bearer_token: Option<String>,
}

#[derive(Clone)]
struct StaticTransport {
    result: Result<OpenAiHttpResponse, AgentError>,
    calls: Arc<Mutex<Vec<ObservedCall>>>,
}

impl StaticTransport {
    fn new(result: Result<OpenAiHttpResponse, AgentError>) -> Self {
        Self {
            result,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn calls(&self) -> Vec<ObservedCall> {
        self.calls.lock().expect("transport calls lock succeeds").clone()
    }
}

impl OpenAiTransport for StaticTransport {
    fn execute(&self, plan: &OpenAiChatPlan, bearer_token: Option<&str>) -> Result<OpenAiHttpResponse, AgentError> {
        self.calls.lock().expect("transport calls lock succeeds").push(ObservedCall {
            plan: plan.clone(),
            bearer_token: bearer_token.map(str::to_string),
        });
        self.result.clone()
    }
}

fn runtime() -> OpenAiRuntimeConfig {
    OpenAiRuntimeConfig {
        endpoint: ENDPOINT.to_string(),
        model: MODEL.to_string(),
        api_key_env: None,
        timeout_secs: TIMEOUT_SECS,
        max_tokens: MAX_TOKENS,
        temperature_milliunits: TEMPERATURE_MILLIUNITS,
        max_request_bytes: MAX_REQUEST_BYTES,
        max_response_bytes: MAX_RESPONSE_BYTES,
        enable_thinking: Some(false),
        system_prompt: "Follow the requested output contract.".to_string(),
    }
}

fn request(output_mode: AgentOutputMode) -> AgentRequest {
    AgentRequest {
        target: AgentTarget::Pi,
        prompt: PROMPT.to_string(),
        output_mode,
        model_hint: None,
        tool_policy: String::new(),
        context_refs: Vec::new(),
        cwd: None,
        session: None,
        text_inputs: Vec::new(),
        image_inputs: Vec::new(),
        json_inputs: Vec::new(),
    }
}

fn runtime_registry(runtime: OpenAiRuntimeConfig) -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        runtimes: BTreeMap::new(),
        openai_compatible: BTreeMap::from([(PI_RUNTIME_KEY.to_string(), runtime)]),
        max_output_bytes: GLOBAL_OUTPUT_MAX_BYTES,
    }
}

fn response(content: Option<&str>) -> OpenAiHttpResponse {
    OpenAiHttpResponse {
        status: HTTP_STATUS_OK,
        body: serde_json::to_vec(&serde_json::json!({
            "id": "chatcmpl-test",
            "model": MODEL,
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": content,
                    "reasoning_content": "must never become output"
                },
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 3, "completion_tokens": 2}
        }))
        .expect("response JSON encodes"),
    }
}

fn adapter_with_transport(runtime: OpenAiRuntimeConfig, transport: StaticTransport) -> ProductionAgentAdapter {
    ProductionAgentAdapter::new_with_openai_transport(runtime_registry(runtime), Arc::new(transport))
}

fn delayed_response_server() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("timeout listener binds");
    let address = listener.local_addr().expect("timeout listener address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("timeout request accepted");
        let mut request_bytes = [0_u8; LOCAL_READ_BUFFER_BYTES];
        let _bytes_read = stream.read(&mut request_bytes).expect("timeout request read");
        thread::sleep(std::time::Duration::from_millis(TIMEOUT_SERVER_DELAY_MILLIS));
    });
    (format!("http://{address}/v1/chat/completions"), handle)
}

fn one_response_server(status: u16, body: Vec<u8>, location: Option<&'static str>) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("local listener binds");
    let address = listener.local_addr().expect("local listener address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("local request accepted");
        let mut request_bytes = [0_u8; LOCAL_READ_BUFFER_BYTES];
        let _bytes_read = stream.read(&mut request_bytes).expect("local request read");
        let location_header = location.map(|value| format!("Location: {value}\r\n")).unwrap_or_default();
        let response = format!(
            "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nContent-Type: application/json\r\n{location_header}Connection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(response.as_bytes()).expect("local response headers written");
        stream.write_all(&body).expect("local response body written");
    });
    (format!("http://{address}/v1/chat/completions"), handle)
}

#[path = "openai/execution.rs"]
mod execution;
#[path = "openai/planning.rs"]
mod planning;

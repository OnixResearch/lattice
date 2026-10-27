//! Tests for the bounded Animus leaf-agent runtime.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use lattice::agent_node::AgentAdapter;
use lattice::agent_node::AgentError;
use lattice::agent_node::AgentOutput;
use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentRequest;
use lattice::agent_node::AgentRuntimeConfig;
use lattice::agent_node::AgentTarget;
use lattice::agent_node::ExecutionContext;
use lattice::agent_node::ProductionAgentAdapter;
use lattice::animus_agent::BUILTIN_PROGRAM;
use lattice::animus_agent::BUILTIN_PROGRAM_IDENTITY;
use lattice::animus_agent::DecisionBudget;
use lattice::animus_agent::ProgramIdentityCheck;
use lattice::animus_agent::TURN_LIMIT;
use lattice::animus_agent::decode_final_response;
use lattice::animus_agent::plan_execution;
use lattice::animus_agent::program_identity;
use lattice::animus_agent::validate_decision_count;
use lattice::animus_agent::validate_execution_admission;
use lattice::animus_agent::validate_program_identity;
use lattice::config::load_app_config;
use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::workflow::NodeConfig;
use lattice::openai::ChatPlan;
use lattice::openai::HttpResponse;
use lattice::openai::RuntimeConfig as OpenAiRuntimeConfig;
use lattice::openai::Transport;
use lattice::workflow_format::load_workflow_nickel_file;
use lattice::workflow_validation::validate_workflow;

const CONFIG_EXAMPLE_PATH: &str = "examples/config/animus-agent.ncl";
const WORKFLOW_EXAMPLE_PATH: &str = "examples/workflows/animus-bounded-agent.ncl";
const ANIMUS_RUNTIME_KEY: &str = "animus";
const PI_RUNTIME_KEY: &str = "pi";
const ENDPOINT: &str = "https://example.invalid/v1/chat/completions";
const MODEL: &str = "bounded-model";
const PROMPT: &str = "Return one bounded answer";
const TEXT_INPUT: &str = "additional context";
const TEXT_RESPONSE: &str = "bounded answer";
const AUTHORITY_RESPONSE: &str = "mutate graph trigger credential budget approval persistence orchestration";
const JSON_RESPONSE: &str = r#"{"ok":true}"#;
const API_KEY_BINDING: &str = "ANIMUS_API_KEY";
const API_KEY_VALUE: &str = "animus-secret-test-token";
const HTTP_STATUS_OK: u16 = 200;
const TIMEOUT_SECS: u64 = 30;
const MAX_TOKENS: u32 = 1_024;
const TEMPERATURE_MILLIUNITS: u16 = 0;
const MAX_REQUEST_BYTES: u64 = 65_536;
const MAX_RESPONSE_BYTES: u64 = 65_536;
const GLOBAL_OUTPUT_MAX_BYTES: usize = 65_536;
const EXPECTED_TRANSPORT_CALL_COUNT: usize = 1;
const EXPECTED_MODEL_CALL_COUNT: u64 = 1;
const NO_MODEL_CALLS: usize = 0;
const NO_EVENTS: u64 = 0;
const NO_DIAGNOSTICS: usize = 0;
const PROMPT_TOKEN_COUNT: u64 = 3;
const COMPLETION_TOKEN_COUNT: u64 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObservedCall {
    plan: ChatPlan,
    bearer_token: Option<String>,
}

#[derive(Clone)]
struct StaticTransport {
    result: Result<HttpResponse, AgentError>,
    calls: Arc<Mutex<Vec<ObservedCall>>>,
}

impl StaticTransport {
    fn new(result: Result<HttpResponse, AgentError>) -> Self {
        Self {
            result,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn calls(&self) -> Vec<ObservedCall> {
        self.calls.lock().expect("transport calls lock succeeds").clone()
    }
}

impl Transport for StaticTransport {
    fn execute(&self, plan: &ChatPlan, bearer_token: Option<&str>) -> Result<HttpResponse, AgentError> {
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
        system_prompt: String::new(),
    }
}

fn request(output_mode: AgentOutputMode) -> AgentRequest {
    AgentRequest {
        target: AgentTarget::Animus,
        prompt: PROMPT.to_string(),
        output_mode,
        model_hint: None,
        tool_policy: "none".to_string(),
        context_refs: Vec::new(),
        cwd: None,
        session: None,
        text_inputs: vec![TEXT_INPUT.to_string()],
        image_inputs: Vec::new(),
        json_inputs: Vec::new(),
    }
}

fn runtime_registry(key: &str, runtime: OpenAiRuntimeConfig) -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        runtimes: BTreeMap::new(),
        openai_compatible: BTreeMap::from([(key.to_string(), runtime)]),
        structured_input_projection: Default::default(),
        max_output_bytes: GLOBAL_OUTPUT_MAX_BYTES,
        local_process: Default::default(),
    }
}

fn response(content: &str) -> HttpResponse {
    HttpResponse {
        status: HTTP_STATUS_OK,
        body: serde_json::to_vec(&serde_json::json!({
            "model": MODEL,
            "choices": [{
                "message": {"role": "assistant", "content": content},
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": PROMPT_TOKEN_COUNT,
                "completion_tokens": COMPLETION_TOKEN_COUNT
            }
        }))
        .expect("response JSON encodes"),
    }
}

fn adapter(runtime: OpenAiRuntimeConfig, transport: StaticTransport) -> ProductionAgentAdapter {
    ProductionAgentAdapter::new_with_openai_transport(
        runtime_registry(ANIMUS_RUNTIME_KEY, runtime),
        Arc::new(transport),
    )
}

// r[verify lattice.agent_runtime.animus_fabric]
// r[verify lattice.agent_runtime.animus_output]
#[test]
fn pure_plan_binds_the_program_identity_and_typed_output() {
    let plan = plan_execution(&request(AgentOutputMode::Text)).expect("valid Animus plan");

    assert_eq!(program_identity(BUILTIN_PROGRAM), BUILTIN_PROGRAM_IDENTITY);
    assert_eq!(plan.program_identity, BUILTIN_PROGRAM_IDENTITY);
    assert_eq!(plan.turn_limit, TURN_LIMIT);
    assert!(plan.summary.contains(PROMPT));
    assert!(plan.summary.contains(TEXT_INPUT));
    assert_eq!(
        decode_final_response(AgentOutputMode::Json, JSON_RESPONSE).expect("JSON response admitted"),
        AgentOutput::Json(serde_json::json!({"ok": true}))
    );
}

// r[verify lattice.agent_runtime.animus_fabric]
// r[verify lattice.agent_runtime.animus_output]
// r[verify lattice.agent_runtime.animus_fabric.scenario.identity]
#[test]
fn pure_admission_rejects_identity_turn_input_and_output_failures() {
    let changed_program = "(define (changed) #t)";
    let identity_error = validate_program_identity(ProgramIdentityCheck {
        program: changed_program,
        expected_identity: BUILTIN_PROGRAM_IDENTITY,
    })
    .expect_err("program drift rejected");
    let admission_error = validate_execution_admission(
        &request(AgentOutputMode::Text),
        ProgramIdentityCheck {
            program: changed_program,
            expected_identity: BUILTIN_PROGRAM_IDENTITY,
        },
        TURN_LIMIT,
    )
    .expect_err("identity drift rejected by execution admission");
    let turn_error = validate_decision_count(DecisionBudget {
        decision_count: TURN_LIMIT,
        turn_limit: TURN_LIMIT,
    })
    .expect_err("exhausted turn rejected");
    let mut image_request = request(AgentOutputMode::Text);
    image_request.image_inputs.push(lattice::engine::value::ImageValue::new(
        vec![0_u8],
        "image/png".to_string(),
        lattice::engine::value::ImageMetadata::default(),
    ));
    let image_error = plan_execution(&image_request).expect_err("image input rejected");
    let artifact_error = plan_execution(&request(AgentOutputMode::Artifact)).expect_err("artifact output rejected");
    let mut session_request = request(AgentOutputMode::Text);
    session_request.session = Some("mutable-session".to_string());
    let session_error = plan_execution(&session_request).expect_err("session rejected");
    let json_error = decode_final_response(AgentOutputMode::Json, "not-json").expect_err("bad JSON rejected");

    assert!(identity_error.to_string().contains("identity mismatch"));
    assert!(admission_error.to_string().contains("identity mismatch"));
    assert!(turn_error.to_string().contains("exhausted"));
    assert!(image_error.to_string().contains("image inputs"));
    assert!(artifact_error.to_string().contains("only text and JSON"));
    assert!(session_error.to_string().contains("sessions"));
    assert!(json_error.to_string().contains("not valid requested JSON"));
}

// r[verify lattice.agent_runtime.animus_leaf]
// r[verify lattice.agent_runtime.animus_fabric]
// r[verify lattice.agent_runtime.animus_output]
// r[verify lattice.agent_runtime.animus_fabric.scenario.complete]
// r[verify lattice.agent_runtime.animus_output.scenario.typed]
#[test]
fn production_animus_route_runs_one_model_call_for_text_and_json() {
    let text_transport = StaticTransport::new(Ok(response(TEXT_RESPONSE)));
    let text_adapter = adapter(runtime(), text_transport.clone());
    let text_result = text_adapter
        .execute(&request(AgentOutputMode::Text), &ExecutionContext::default())
        .expect("text loop succeeds");
    let json_transport = StaticTransport::new(Ok(response(JSON_RESPONSE)));
    let json_adapter = adapter(runtime(), json_transport.clone());
    let json_result = json_adapter
        .execute(&request(AgentOutputMode::Json), &ExecutionContext::default())
        .expect("JSON loop succeeds");

    assert_eq!(text_result.output, AgentOutput::Text(TEXT_RESPONSE.to_string()));
    assert_eq!(json_result.output, AgentOutput::Json(serde_json::json!({"ok": true})));
    assert_eq!(text_transport.calls().len(), EXPECTED_TRANSPORT_CALL_COUNT);
    assert_eq!(json_transport.calls().len(), EXPECTED_TRANSPORT_CALL_COUNT);
    assert_eq!(text_result.metadata["adapter"], ANIMUS_RUNTIME_KEY);
    assert_eq!(text_result.metadata["program_identity"], BUILTIN_PROGRAM_IDENTITY);
    assert_eq!(text_result.metadata["model_call_count"], EXPECTED_MODEL_CALL_COUNT);
    assert!(text_result.metadata["event_count"].as_u64().is_some_and(|count| count > NO_EVENTS));
    assert_eq!(text_result.metadata["tools"], "disabled");
    assert!(!text_result.metadata.to_string().contains(PROMPT));
    assert!(!text_result.metadata.to_string().contains(TEXT_RESPONSE));
}

// r[verify lattice.agent_runtime.animus_effects]
// r[verify lattice.agent_runtime.animus_effects.scenario.credential]
#[test]
fn credential_is_used_only_at_lattice_transport_and_is_absent_from_evidence() {
    let mut credential_runtime = runtime();
    credential_runtime.api_key_env = Some(API_KEY_BINDING.to_string());
    let transport = StaticTransport::new(Ok(response(TEXT_RESPONSE)));
    let adapter = adapter(credential_runtime, transport.clone());
    let context = ExecutionContext::new(HashMap::from([(API_KEY_BINDING.to_string(), API_KEY_VALUE.to_string())]));

    let result = adapter.execute(&request(AgentOutputMode::Text), &context).expect("credential-bound loop succeeds");
    let calls = transport.calls();

    assert_eq!(calls.len(), EXPECTED_TRANSPORT_CALL_COUNT);
    assert_eq!(calls[0].bearer_token.as_deref(), Some(API_KEY_VALUE));
    assert!(!String::from_utf8_lossy(&calls[0].plan.body).contains(API_KEY_VALUE));
    assert!(!format!("{:?}", result.output).contains(API_KEY_VALUE));
    assert!(!result.metadata.to_string().contains(API_KEY_VALUE));
}

// r[verify lattice.agent_runtime.animus_effects]
// r[verify lattice.agent_runtime.animus_compatibility]
// r[verify lattice.agent_runtime.animus_compatibility.scenario.missing]
// r[verify lattice.agent_runtime.animus_effects.scenario.provider-failure]
#[test]
fn missing_or_failed_animus_provider_does_not_fallback() {
    let missing_transport = StaticTransport::new(Ok(response(TEXT_RESPONSE)));
    let missing_adapter = ProductionAgentAdapter::new_with_openai_transport(
        AgentRuntimeConfig {
            runtimes: BTreeMap::new(),
            openai_compatible: BTreeMap::new(),
            structured_input_projection: Default::default(),
            max_output_bytes: GLOBAL_OUTPUT_MAX_BYTES,
            local_process: Default::default(),
        },
        Arc::new(missing_transport.clone()),
    );
    let missing_request = request(AgentOutputMode::Text);
    let original_request = missing_request.clone();
    let missing_error = missing_adapter
        .execute(&missing_request, &ExecutionContext::default())
        .expect_err("missing route rejected");
    let failed_transport =
        StaticTransport::new(Err(AgentError::Adapter(format!("fixture provider failed with {API_KEY_VALUE}"))));
    let failed_adapter = adapter(runtime(), failed_transport.clone());
    let failed_context =
        ExecutionContext::new(HashMap::from([(API_KEY_BINDING.to_string(), API_KEY_VALUE.to_string())]));
    let failed_error = failed_adapter
        .execute(&request(AgentOutputMode::Text), &failed_context)
        .expect_err("provider failure returned");
    let failed_diagnostic = failed_error.to_string();

    assert!(missing_error.to_string().contains("missing animus agent runtime"));
    assert_eq!(missing_request, original_request);
    assert_eq!(missing_transport.calls().len(), NO_MODEL_CALLS);
    assert!(failed_diagnostic.contains("<redacted>"));
    assert!(!failed_diagnostic.contains(API_KEY_VALUE));
    assert_eq!(failed_transport.calls().len(), EXPECTED_TRANSPORT_CALL_COUNT);
}

// r[verify lattice.agent_runtime.animus_leaf.scenario.authority]
#[test]
fn authority_bearing_response_remains_ordinary_agent_output() {
    let transport = StaticTransport::new(Ok(response(AUTHORITY_RESPONSE)));
    let adapter = adapter(runtime(), transport.clone());

    let result = adapter
        .execute(&request(AgentOutputMode::Text), &ExecutionContext::default())
        .expect("authority-shaped content remains ordinary output");

    assert_eq!(result.output, AgentOutput::Text(AUTHORITY_RESPONSE.to_string()));
    assert_eq!(transport.calls().len(), EXPECTED_TRANSPORT_CALL_COUNT);
    for forbidden_key in [
        "graph",
        "trigger",
        "credential",
        "budget",
        "approval",
        "persistence",
        "orchestration",
    ] {
        assert!(result.metadata.get(forbidden_key).is_none());
    }
}

// r[verify lattice.agent_runtime.animus_output.scenario.unsupported]
#[test]
fn unsupported_animus_inputs_fail_before_provider_dispatch() {
    let transport = StaticTransport::new(Ok(response(TEXT_RESPONSE)));
    let adapter = adapter(runtime(), transport.clone());
    let artifact_error = adapter
        .execute(&request(AgentOutputMode::Artifact), &ExecutionContext::default())
        .expect_err("artifact output rejected before dispatch");

    assert!(artifact_error.to_string().contains("only text and JSON"));
    assert_eq!(transport.calls().len(), NO_MODEL_CALLS);
}

// r[verify lattice.agent_runtime.animus_leaf]
// r[verify lattice.agent_runtime.animus_compatibility]
// r[verify lattice.agent_runtime.animus_leaf.scenario.execute]
#[tokio::test]
async fn typed_examples_load_validate_and_execute_through_the_runner() {
    let config = load_app_config(Path::new(CONFIG_EXAMPLE_PATH)).expect("Animus app config imports");
    let workflow = load_workflow_nickel_file(Path::new(WORKFLOW_EXAMPLE_PATH)).expect("Animus workflow imports");
    let diagnostics = validate_workflow(&workflow);
    let node = workflow.graph.nodes().next().expect("Animus workflow has one node");
    let NodeConfig::Agent(settings) = workflow.node_config(node.id).expect("Animus node has typed config") else {
        panic!("expected Animus agent config");
    };
    let transport = StaticTransport::new(Ok(response(TEXT_RESPONSE)));
    let adapter = ProductionAgentAdapter::new_with_openai_transport(config.agent_runtime, Arc::new(transport.clone()));
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node.id, &HashMap::new()).await;

    assert_eq!(diagnostics.len(), NO_DIAGNOSTICS);
    assert_eq!(settings.target, AgentTarget::Animus);
    assert!(matches!(result.status, NodeStatus::Completed { .. }), "unexpected status: {:?}", result.status);
    assert_eq!(transport.calls().len(), EXPECTED_TRANSPORT_CALL_COUNT);
}

// r[verify lattice.agent_runtime.animus_compatibility]
// r[verify lattice.agent_runtime.animus_compatibility.scenario.routes]
#[test]
fn existing_direct_openai_route_does_not_invoke_animus() {
    let transport = StaticTransport::new(Ok(response(TEXT_RESPONSE)));
    let adapter = ProductionAgentAdapter::new_with_openai_transport(
        runtime_registry(PI_RUNTIME_KEY, runtime()),
        Arc::new(transport.clone()),
    );
    let mut pi_request = request(AgentOutputMode::Text);
    pi_request.target = AgentTarget::Pi;

    let result = adapter.execute(&pi_request, &ExecutionContext::default()).expect("existing direct route succeeds");

    assert_eq!(result.output, AgentOutput::Text(TEXT_RESPONSE.to_string()));
    assert_eq!(result.metadata["adapter"], PI_RUNTIME_KEY);
    assert!(result.metadata.get("program_identity").is_none());
    assert_eq!(transport.calls().len(), EXPECTED_TRANSPORT_CALL_COUNT);
}

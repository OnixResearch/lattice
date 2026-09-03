use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use lattice::agent_node::AgentAdapter;
use lattice::agent_node::AgentOutput;
use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentRequest;
use lattice::agent_node::AgentRuntimeConfig;
use lattice::agent_node::AgentTarget;
use lattice::agent_node::ExecutionContext;
use lattice::agent_node::ProductionAgentAdapter;
use lattice::agent_node::StructuredInputProjectionPolicy;
use lattice::agent_node::plan_agent_command;
use lattice::agent_prompt::ProjectionPilotInput;
use lattice::agent_prompt::StructuredInputProjection;
use lattice::agent_prompt::evaluate_pilot;
use lattice::agent_prompt::render_structured_inputs;
use lattice::animus_agent::plan_execution_with_projection;
use lattice::openai::ChatPlan;
use lattice::openai::HttpResponse;
use lattice::openai::ProjectedChat;
use lattice::openai::RuntimeConfig as OpenAiRuntimeConfig;
use lattice::openai::Transport;
use lattice::openai::decode_response;
use lattice::openai::plan_chat_with_projection;

const HTTP_OK: u16 = 200;
const TIMEOUT_SECS: u64 = 30;
const TOKEN_LIMIT: u32 = 256;
const REQUEST_LIMIT_BYTES: u64 = 65_536;
const RESPONSE_LIMIT_BYTES: u64 = 65_536;
const GLOBAL_OUTPUT_LIMIT_BYTES: u64 = 65_536;
const TEMPERATURE_MILLIUNITS: u16 = 0;

#[derive(Default)]
struct RecordingTransport {
    calls: AtomicUsize,
}

impl RecordingTransport {
    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Transport for RecordingTransport {
    fn execute(
        &self,
        _plan: &ChatPlan,
        _bearer_token: Option<&str>,
    ) -> Result<HttpResponse, lattice::agent_node::AgentError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(HttpResponse {
            status: HTTP_OK,
            body: br#"{"model":"controlled-model","choices":[{"message":{"content":"done"},"finish_reason":"stop"}],"usage":{"prompt_tokens":5}}"#.to_vec(),
        })
    }
}

fn agent_request(target: AgentTarget, input: serde_json::Value) -> AgentRequest {
    AgentRequest {
        target,
        prompt: "Return the selected owner.".to_string(),
        output_mode: AgentOutputMode::Text,
        model_hint: None,
        tool_policy: "No tools.".to_string(),
        context_refs: vec!["context:fixture".to_string()],
        cwd: None,
        session: None,
        text_inputs: vec!["Owner lookup".to_string()],
        image_inputs: Vec::new(),
        json_inputs: vec![input],
    }
}

fn runtime() -> OpenAiRuntimeConfig {
    OpenAiRuntimeConfig {
        endpoint: "http://127.0.0.1:13305/v1/chat/completions".to_string(),
        model: "controlled-model".to_string(),
        api_key_env: None,
        timeout_secs: TIMEOUT_SECS,
        max_tokens: TOKEN_LIMIT,
        temperature_milliunits: TEMPERATURE_MILLIUNITS,
        max_request_bytes: REQUEST_LIMIT_BYTES,
        max_response_bytes: RESPONSE_LIMIT_BYTES,
        enable_thinking: Some(false),
        system_prompt: String::new(),
    }
}

fn user_prompt(plan: &ChatPlan) -> String {
    let body: serde_json::Value = serde_json::from_slice(&plan.body).expect("chat plan body is JSON");
    body["messages"]
        .as_array()
        .and_then(|messages| messages.last())
        .and_then(|message| message["content"].as_str())
        .expect("chat plan has a user prompt")
        .to_string()
}

// r[verify lattice.agent_runtime.structured_input_projection]
#[test]
fn runtime_policy_accepts_closed_profiles_and_rejects_unknown_entries() {
    let selected: AgentRuntimeConfig = serde_json::from_value(serde_json::json!({
        "runtimes": {},
        "openai_compatible": {},
        "structured_input_projection": {"pi": "toon_v3"}
    }))
    .expect("closed projection policy decodes");
    let unknown_profile = serde_json::from_value::<AgentRuntimeConfig>(serde_json::json!({
        "structured_input_projection": {"pi": "toon_v4"}
    }));
    let unknown_target = serde_json::from_value::<AgentRuntimeConfig>(serde_json::json!({
        "structured_input_projection": {"other": "toon_v3"}
    }));
    let wrong_type = serde_json::from_value::<AgentRuntimeConfig>(serde_json::json!({
        "structured_input_projection": {"pi": true}
    }));

    assert_eq!(selected.structured_input_projection.for_target(AgentTarget::Pi), StructuredInputProjection::ToonV3);
    assert_eq!(
        selected.structured_input_projection.for_target(AgentTarget::Clankers),
        StructuredInputProjection::JsonPretty
    );
    assert!(unknown_profile.is_err());
    assert!(unknown_target.is_err());
    assert!(wrong_type.is_err());
}

// r[verify lattice.agent_runtime.structured_input_projection.scenario.selected]
// r[verify lattice.agent_runtime.toon_v3_projection.scenario.supported]
#[test]
fn selected_profile_has_identical_command_openai_and_animus_prompt() {
    let input = serde_json::json!({"users": [{"id": 1, "name": "Ada"}]});
    let request = agent_request(AgentTarget::Pi, input.clone());
    let mut command_config = AgentRuntimeConfig::default();
    command_config.structured_input_projection.pi = Some(StructuredInputProjection::ToonV3);
    let command_plan = plan_agent_command(&command_config, &request).expect("command prompt projects");
    let command_prompt = command_plan.args.last().expect("command plan has prompt");

    let runtime = runtime();
    let openai_plan = plan_chat_with_projection(ProjectedChat {
        runtime: &runtime,
        request: &request,
        projection: StructuredInputProjection::ToonV3,
        global_output_max_bytes: GLOBAL_OUTPUT_LIMIT_BYTES,
    })
    .expect("OpenAI prompt projects");
    let animus_request = agent_request(AgentTarget::Animus, input);
    let animus_plan = plan_execution_with_projection(&animus_request, StructuredInputProjection::ToonV3)
        .expect("Animus prompt projects");

    assert_eq!(command_prompt, &user_prompt(&openai_plan));
    assert_eq!(command_prompt, &animus_plan.summary);
    assert!(command_prompt.contains("TOON v3 input 1"));
    assert!(!command_prompt.contains("Compact JSON input"));
}

// r[verify lattice.agent_runtime.structured_input_projection.scenario.default]
#[test]
fn omitted_policy_preserves_exact_pretty_json_prompt() {
    let request = agent_request(AgentTarget::Pi, serde_json::json!({"enabled": true}));
    let plan = plan_agent_command(&AgentRuntimeConfig::default(), &request).expect("default command prompt renders");
    let prompt = plan.args.last().expect("command plan has prompt");

    assert!(prompt.contains("JSON input 1:\n{\n  \"enabled\": true\n}"));
    assert!(!prompt.contains("Compact JSON input"));
    assert!(!prompt.contains("TOON v3 input"));
}

// r[verify lattice.agent_runtime.projection_boundaries.scenario.wire]
// r[verify lattice.agent_runtime.projection_boundaries.scenario.output]
#[test]
fn toon_input_keeps_provider_envelope_and_requested_output_as_json() {
    let mut request = agent_request(AgentTarget::Pi, serde_json::json!({"users": [{"id": 1, "name": "Ada"}]}));
    request.output_mode = AgentOutputMode::Json;
    let runtime = runtime();
    let plan = plan_chat_with_projection(ProjectedChat {
        runtime: &runtime,
        request: &request,
        projection: StructuredInputProjection::ToonV3,
        global_output_max_bytes: GLOBAL_OUTPUT_LIMIT_BYTES,
    })
    .expect("TOON input plan builds");
    let envelope: serde_json::Value = serde_json::from_slice(&plan.body).expect("provider envelope remains JSON");
    let response = HttpResponse {
        status: HTTP_OK,
        body: br#"{"model":"controlled-model","choices":[{"message":{"content":"{\"owner\":\"Ada\"}"},"finish_reason":"stop"}],"usage":null}"#.to_vec(),
    };
    let decoded = decode_response(&plan, &response, &ExecutionContext::default())
        .expect("requested JSON output uses the existing decoder");
    let durable_request = serde_json::to_value(&request).expect("typed request remains JSON");

    assert!(user_prompt(&plan).contains("TOON v3 input 1"));
    assert!(envelope["messages"].is_array());
    assert_eq!(decoded.output, AgentOutput::Json(serde_json::json!({"owner": "Ada"})));
    assert!(durable_request.get("structured_input_projection").is_none());
}

// r[verify lattice.agent_runtime.toon_v3_projection.scenario.rejected]
#[test]
fn invalid_toon_projection_stops_every_route_before_effects() {
    let invalid_input = serde_json::json!({});
    let request = agent_request(AgentTarget::Pi, invalid_input.clone());
    let mut command_config = AgentRuntimeConfig::default();
    command_config.structured_input_projection.pi = Some(StructuredInputProjection::ToonV3);
    assert!(plan_agent_command(&command_config, &request).is_err());
    assert!(
        plan_chat_with_projection(ProjectedChat {
            runtime: &runtime(),
            request: &request,
            projection: StructuredInputProjection::ToonV3,
            global_output_max_bytes: GLOBAL_OUTPUT_LIMIT_BYTES,
        })
        .is_err()
    );
    let animus_request = agent_request(AgentTarget::Animus, invalid_input);
    assert!(plan_execution_with_projection(&animus_request, StructuredInputProjection::ToonV3).is_err());

    let transport = Arc::new(RecordingTransport::default());
    let adapter = ProductionAgentAdapter::new_with_openai_transport(
        AgentRuntimeConfig {
            runtimes: BTreeMap::new(),
            openai_compatible: BTreeMap::from([("pi".to_string(), runtime())]),
            structured_input_projection: StructuredInputProjectionPolicy {
                pi: Some(StructuredInputProjection::ToonV3),
                clankers: None,
                animus: None,
            },
            max_output_bytes: GLOBAL_OUTPUT_LIMIT_BYTES as usize,
            local_process: Default::default(),
        },
        transport.clone(),
    );
    assert!(adapter.execute(&request, &ExecutionContext::default()).is_err());
    assert_eq!(transport.call_count(), 0);
}

// r[verify lattice.agent_runtime.projection_boundaries.scenario.evidence]
#[test]
fn successful_route_records_only_safe_projection_facts() {
    let transport = Arc::new(RecordingTransport::default());
    let adapter = ProductionAgentAdapter::new_with_openai_transport(
        AgentRuntimeConfig {
            runtimes: BTreeMap::new(),
            openai_compatible: BTreeMap::from([("pi".to_string(), runtime())]),
            structured_input_projection: StructuredInputProjectionPolicy {
                pi: Some(StructuredInputProjection::JsonCompact),
                clankers: None,
                animus: None,
            },
            max_output_bytes: GLOBAL_OUTPUT_LIMIT_BYTES as usize,
            local_process: Default::default(),
        },
        transport.clone(),
    );
    let response = adapter
        .execute(
            &agent_request(AgentTarget::Pi, serde_json::json!({"private": "not retained"})),
            &ExecutionContext::default(),
        )
        .expect("controlled route returns a response");
    let projection = &response.metadata["structured_input_projection"];

    assert_eq!(transport.call_count(), 1);
    assert_eq!(projection["profile"], "json_compact");
    assert_eq!(projection["implementation"], "serde_json");
    assert_eq!(projection["structured_input_count"], 1);
    let serialized = serde_json::to_string(&response.metadata).expect("metadata serializes");
    assert!(!serialized.contains("not retained"));
    assert!(!serialized.contains("Return the selected owner"));
}

// r[verify lattice.agent_runtime.projection_evaluation]
// r[verify lattice.agent_runtime.projection_evaluation.scenario.blocked]
#[test]
fn checked_corpus_projects_and_inconclusive_report_replays_exactly() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/structured-input-projection/pilot-corpus.json"))
            .expect("pilot corpus is valid JSON");
    let tasks = corpus["tasks"].as_array().expect("pilot corpus has tasks");
    for task in tasks {
        let input = task["input"].clone();
        let compact = render_structured_inputs(std::slice::from_ref(&input), StructuredInputProjection::JsonCompact)
            .expect("compact corpus projection succeeds");
        let toon = render_structured_inputs(std::slice::from_ref(&input), StructuredInputProjection::ToonV3)
            .expect("TOON corpus projection strictly round trips");
        assert!(!compact[0].body.is_empty());
        assert!(!toon[0].body.is_empty());
    }

    let observations: ProjectionPilotInput =
        serde_json::from_str(include_str!("fixtures/structured-input-projection/pilot-observations.json"))
            .expect("pilot observations are valid");
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/structured-input-projection/pilot-report.json"))
            .expect("pilot report is valid JSON");
    let actual = serde_json::to_value(evaluate_pilot(&observations)).expect("pilot report serializes");
    let attempt: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/structured-input-projection/pilot-attempt.json"))
            .expect("pilot attempt is valid JSON");

    assert_eq!(actual, expected);
    assert_eq!(attempt["status"], "provider_unavailable");
    assert_eq!(attempt["task_dispatch_count"], 0);
    assert_eq!(attempt["prompt_token_observations"], 0);
    assert_eq!(attempt["recommendation"], "inconclusive_keep_json_pretty_default");
}

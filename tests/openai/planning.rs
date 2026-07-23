use super::*;

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.plan]
#[test]
fn typed_nickel_example_loads_native_runtime_routes() {
    let config = load_app_config(Path::new(CONFIG_EXAMPLE_PATH));

    assert!(config.agent_runtime.runtimes.is_empty());
    assert_eq!(config.agent_runtime.openai_compatible.len(), CONFIGURED_RUNTIME_COUNT);
    let runtime = config.agent_runtime.openai_compatible.get(PI_RUNTIME_KEY).expect("Pi runtime is configured");
    assert_eq!(runtime.model, EXAMPLE_MODEL);
    assert_eq!(runtime.timeout_secs, EXAMPLE_TIMEOUT_SECS);
    assert_eq!(runtime.max_tokens, EXAMPLE_MAX_TOKENS);
}

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.plan]
#[test]
fn planning_emits_bounded_non_streaming_chat_request() {
    let mut text_request = request(AgentOutputMode::Text);
    text_request.model_hint = Some(MODEL_OVERRIDE.to_string());

    let plan = plan_openai_chat(&runtime(), &text_request, MAX_RESPONSE_BYTES).expect("valid plan");
    let body: serde_json::Value = serde_json::from_slice(&plan.body).expect("request body JSON");
    let json_plan =
        plan_openai_chat(&runtime(), &request(AgentOutputMode::Json), MAX_RESPONSE_BYTES).expect("valid JSON plan");
    let json_body: serde_json::Value = serde_json::from_slice(&json_plan.body).expect("JSON request body");

    assert_eq!(plan.endpoint, ENDPOINT);
    assert_eq!(plan.model, MODEL_OVERRIDE);
    assert_eq!(plan.timeout_secs, TIMEOUT_SECS);
    assert_eq!(body["model"], MODEL_OVERRIDE);
    assert_eq!(body["stream"], false);
    assert_eq!(body["max_tokens"], MAX_TOKENS);
    assert_eq!(body["chat_template_kwargs"]["enable_thinking"], false);
    assert!(body.get("response_format").is_none());
    assert_eq!(json_body["response_format"]["type"], "json_object");
    assert!(
        json_body["messages"][0]["content"]
            .as_str()
            .expect("JSON output system prompt")
            .contains("valid JSON object")
    );
    assert!(
        json_body["messages"][1]["content"]
            .as_str()
            .expect("JSON user prompt")
            .ends_with("commentary outside JSON.")
    );
    let user_content = body["messages"][1]["content"].as_str().expect("user content string");
    assert!(user_content.starts_with(PROMPT));
    assert!(user_content.contains(MODEL_OVERRIDE));
    assert!(!String::from_utf8_lossy(&plan.body).contains(API_KEY_VALUE));
}

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.plan]
#[test]
fn planning_rejects_unsafe_or_unsupported_inputs() {
    let mut cases = Vec::new();
    let mut invalid_scheme = runtime();
    invalid_scheme.endpoint = "file:///tmp/model".to_string();
    cases.push((invalid_scheme, request(AgentOutputMode::Text), "HTTP or HTTPS"));
    let mut embedded_credentials = runtime();
    embedded_credentials.endpoint = "https://user:pass@example.invalid/v1/chat/completions".to_string();
    cases.push((embedded_credentials, request(AgentOutputMode::Text), "must not embed credentials"));
    let mut fragment = runtime();
    fragment.endpoint = format!("{ENDPOINT}#fragment");
    cases.push((fragment, request(AgentOutputMode::Text), "must not include a fragment"));
    let mut insecure_bearer = runtime();
    insecure_bearer.endpoint = "http://localhost/v1/chat/completions".to_string();
    insecure_bearer.api_key_env = Some(API_KEY_BINDING.to_string());
    cases.push((insecure_bearer, request(AgentOutputMode::Text), "require an HTTPS"));
    let mut invalid_temperature = runtime();
    invalid_temperature.temperature_milliunits = INVALID_TEMPERATURE_MILLIUNITS;
    cases.push((invalid_temperature, request(AgentOutputMode::Text), "temperature_milliunits"));
    let mut session_request = request(AgentOutputMode::Text);
    session_request.session = Some("session".to_string());
    cases.push((runtime(), session_request, "does not support sessions"));
    cases.push((runtime(), request(AgentOutputMode::Artifact), "only text and JSON"));
    let mut image_request = request(AgentOutputMode::Text);
    image_request.image_inputs.push(ImageValue::new(
        vec![0],
        TEST_IMAGE_MEDIA_TYPE.to_string(),
        ImageMetadata::default(),
    ));
    cases.push((runtime(), image_request, "does not support image inputs"));

    for (runtime, request, expected) in cases {
        let error = plan_openai_chat(&runtime, &request, MAX_RESPONSE_BYTES).expect_err("invalid plan rejected");
        assert!(error.to_string().contains(expected), "expected {expected:?}, got {error}");
    }
}

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.route]
#[test]
fn production_adapter_rejects_missing_or_ambiguous_routes_before_transport() {
    let transport = StaticTransport::new(Ok(response(Some(TEXT_CONTENT))));
    let missing = ProductionAgentAdapter::new_with_openai_transport(
        AgentRuntimeConfig {
            runtimes: BTreeMap::new(),
            openai_compatible: BTreeMap::new(),
            max_output_bytes: GLOBAL_OUTPUT_MAX_BYTES,
            local_process: Default::default(),
        },
        Arc::new(transport.clone()),
    );
    let missing_error = missing
        .execute(&request(AgentOutputMode::Text), &AgentExecutionContext::default())
        .expect_err("missing route rejected");
    let mut ambiguous_config = runtime_registry(runtime());
    ambiguous_config
        .runtimes
        .insert(PI_RUNTIME_KEY.to_string(), lattice::agent_node::AgentCommandRuntimeConfig {
            command: "pi".to_string(),
            args: Vec::new(),
        });
    let ambiguous = ProductionAgentAdapter::new_with_openai_transport(ambiguous_config, Arc::new(transport.clone()));
    let ambiguous_error = ambiguous
        .execute(&request(AgentOutputMode::Text), &AgentExecutionContext::default())
        .expect_err("ambiguous route rejected");

    assert!(missing_error.to_string().contains("missing pi agent runtime"));
    assert!(ambiguous_error.to_string().contains("ambiguous pi agent runtime"));
    assert!(transport.calls().is_empty());
}

use super::*;

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible]
// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.success]
// r[verify lattice.agent_runtime.inline_reasoning_boundary.scenario.plain]
#[test]
fn production_adapter_decodes_text_and_json_outputs() {
    let text_transport = StaticTransport::new(Ok(response(Some(TEXT_CONTENT))));
    let text_adapter = adapter_with_transport(runtime(), text_transport.clone());
    let text = text_adapter
        .execute(&request(AgentOutputMode::Text), &AgentExecutionContext::default())
        .expect("text response succeeds");
    let json_transport = StaticTransport::new(Ok(response(Some(JSON_CONTENT))));
    let json_adapter = adapter_with_transport(runtime(), json_transport.clone());
    let json = json_adapter
        .execute(&request(AgentOutputMode::Json), &AgentExecutionContext::default())
        .expect("JSON response succeeds");

    assert_eq!(text.output, AgentOutput::Text(TEXT_CONTENT.to_string()));
    assert_eq!(text.metadata["provider"], "openai_compatible");
    assert_eq!(text.metadata["reasoning_suppressed"], false);
    assert_eq!(json.output, AgentOutput::Json(serde_json::json!({"ok": true})));
    assert_eq!(text_transport.calls().len(), 1);
    assert_eq!(json_transport.calls().len(), 1);
}

// r[verify lattice.agent_runtime.inline_reasoning_boundary]
// r[verify lattice.agent_runtime.inline_reasoning_boundary.scenario.closed]
// r[verify lattice.agent_runtime.inline_reasoning_boundary.scenario.json]
#[test]
fn decoder_suppresses_one_leading_reasoning_block_before_output_typing() {
    let text_plan =
        plan_openai_chat(&runtime(), &request(AgentOutputMode::Text), MAX_RESPONSE_BYTES).expect("valid text plan");
    let text = decode_openai_chat_response(
        &text_plan,
        &response(Some(INLINE_REASONING_TEXT_CONTENT)),
        &AgentExecutionContext::default(),
    )
    .expect("closed reasoning block must yield final text");
    let json_plan =
        plan_openai_chat(&runtime(), &request(AgentOutputMode::Json), MAX_RESPONSE_BYTES).expect("valid JSON plan");
    let json = decode_openai_chat_response(
        &json_plan,
        &response(Some(INLINE_REASONING_JSON_CONTENT)),
        &AgentExecutionContext::default(),
    )
    .expect("closed reasoning block must yield final JSON");

    assert_eq!(text.output, AgentOutput::Text(TEXT_CONTENT.to_string()));
    assert_eq!(text.metadata["reasoning_suppressed"], true);
    assert_eq!(json.output, AgentOutput::Json(serde_json::json!({"ok": true})));
    assert_eq!(json.metadata["reasoning_suppressed"], true);
    assert!(!format!("{:?}", text.output).contains(INLINE_REASONING_PRIVATE));
}

// r[verify lattice.agent_runtime.inline_reasoning_boundary]
// r[verify lattice.agent_runtime.inline_reasoning_boundary.scenario.malformed]
#[test]
fn decoder_rejects_ambiguous_reasoning_without_echoing_private_content() {
    let plan = plan_openai_chat(&runtime(), &request(AgentOutputMode::Text), MAX_RESPONSE_BYTES).expect("valid plan");

    for malformed_content in MALFORMED_INLINE_REASONING_CONTENT {
        let error =
            decode_openai_chat_response(&plan, &response(Some(malformed_content)), &AgentExecutionContext::default())
                .expect_err("ambiguous reasoning markup must fail");
        let diagnostic = error.to_string();
        assert!(!diagnostic.contains(INLINE_REASONING_PRIVATE));
        assert!(!diagnostic.contains(PROMPT));
    }
}

// r[verify lattice.agent_runtime.inline_reasoning_boundary]
// r[verify lattice.agent_runtime.inline_reasoning_boundary.scenario.malformed]
#[test]
fn decoder_reports_token_exhaustion_without_admitting_partial_content() {
    let cases = [
        (AgentOutputMode::Text, TOKEN_LIMIT_CONTENT),
        (AgentOutputMode::Text, TEXT_CONTENT),
        (AgentOutputMode::Json, JSON_CONTENT),
    ];

    for (output_mode, content) in cases {
        let plan = plan_openai_chat(&runtime(), &request(output_mode), MAX_RESPONSE_BYTES).expect("valid plan");
        let error = decode_openai_chat_response(
            &plan,
            &response_with_finish_reason(Some(content), TOKEN_LIMIT_FINISH_REASON),
            &AgentExecutionContext::default(),
        )
        .expect_err("token-exhausted response must fail before output typing");
        let diagnostic = error.to_string();

        assert!(diagnostic.contains("exhausted max_tokens"));
        assert!(diagnostic.contains("increase the configured max_tokens bound"));
        assert!(!diagnostic.contains(INLINE_REASONING_PRIVATE));
        assert!(!diagnostic.contains(PROMPT));
        assert!(!diagnostic.contains(content));
    }
}

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.credentials]
#[tokio::test]
async fn executor_passes_resolved_node_credential_to_native_adapter() {
    let mut credential_runtime = runtime();
    credential_runtime.api_key_env = Some(API_KEY_BINDING.to_string());
    let transport = StaticTransport::new(Ok(response(Some(TEXT_CONTENT))));
    let adapter = adapter_with_transport(credential_runtime, transport.clone());
    let provider = lattice::credentials::EnvironmentCredentialProvider::from_map(HashMap::from([(
        API_KEY_REFERENCE.to_string(),
        API_KEY_VALUE.to_string(),
    )]));
    let mut workflow = lattice::engine::workflow::Workflow::new("native-openai-credential");
    let node_id = workflow.add_agent_node("native-openai", AgentTarget::Pi, PROMPT, AgentOutputMode::Text);
    workflow.set_node_credentials(node_id, lattice::credentials::NodeCredentialRefs {
        env: HashMap::from([(
            API_KEY_BINDING.to_string(),
            lattice::credentials::SecretRef::new("env", API_KEY_REFERENCE, None),
        )]),
    });
    let executor = lattice::engine::executor::Executor::new_with_runtime_clients(
        Box::new(provider),
        Box::new(lattice::http::NoopHttpClient),
        Box::new(lattice::file_node::NoopFileSystem),
        Box::new(lattice::image_node::NoopImageIo),
        Box::new(adapter),
    );

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;
    let calls = transport.calls();

    assert!(matches!(result.status, lattice::engine::executor::NodeStatus::Completed { .. }));
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].bearer_token.as_deref(), Some(API_KEY_VALUE));
}

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.credentials]
#[test]
fn bearer_credential_is_required_and_redacted() {
    let mut credential_runtime = runtime();
    credential_runtime.api_key_env = Some(API_KEY_BINDING.to_string());
    let transport = StaticTransport::new(Ok(OpenAiHttpResponse {
        status: HTTP_STATUS_UNAVAILABLE,
        body: format!("upstream echoed {API_KEY_VALUE}").into_bytes(),
    }));
    let adapter = adapter_with_transport(credential_runtime, transport.clone());
    let missing = adapter
        .execute(&request(AgentOutputMode::Text), &AgentExecutionContext::default())
        .expect_err("missing credential rejected");
    let context = AgentExecutionContext::new(HashMap::from([(API_KEY_BINDING.to_string(), API_KEY_VALUE.to_string())]));
    let redacted = adapter
        .execute(&request(AgentOutputMode::Text), &context)
        .expect_err("non-success response rejected");
    let calls = transport.calls();

    assert!(missing.to_string().contains(API_KEY_BINDING));
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].bearer_token.as_deref(), Some(API_KEY_VALUE));
    assert!(!redacted.to_string().contains(API_KEY_VALUE));
    assert!(redacted.to_string().contains("<redacted>"));
}

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.failure]
#[test]
fn decoder_rejects_malformed_missing_empty_and_wrong_typed_content() {
    let plan = plan_openai_chat(&runtime(), &request(AgentOutputMode::Text), MAX_RESPONSE_BYTES).expect("valid plan");
    let malformed = OpenAiHttpResponse {
        status: HTTP_STATUS_OK,
        body: b"not-json".to_vec(),
    };
    let no_choices = OpenAiHttpResponse {
        status: HTTP_STATUS_OK,
        body: br#"{"choices":[]}"#.to_vec(),
    };
    let invalid_utf8 = OpenAiHttpResponse {
        status: HTTP_STATUS_OK,
        body: INVALID_UTF8_RESPONSE.to_vec(),
    };
    let reasoning_only = response(None);

    assert!(decode_openai_chat_response(&plan, &malformed, &AgentExecutionContext::default()).is_err());
    assert!(decode_openai_chat_response(&plan, &no_choices, &AgentExecutionContext::default()).is_err());
    assert!(decode_openai_chat_response(&plan, &invalid_utf8, &AgentExecutionContext::default()).is_err());
    let reasoning_error = decode_openai_chat_response(&plan, &reasoning_only, &AgentExecutionContext::default())
        .expect_err("reasoning content is not substituted");
    assert!(reasoning_error.to_string().contains("content is empty or missing"));

    let json_plan =
        plan_openai_chat(&runtime(), &request(AgentOutputMode::Json), MAX_RESPONSE_BYTES).expect("valid JSON plan");
    let invalid_json_content = response(Some("not requested JSON"));
    let json_error = decode_openai_chat_response(&json_plan, &invalid_json_content, &AgentExecutionContext::default())
        .expect_err("invalid requested JSON rejected");
    assert!(json_error.to_string().contains("not valid requested JSON"));
}

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.failure]
#[test]
fn adapter_rejects_transport_failure_and_oversized_response() {
    let timeout_transport = StaticTransport::new(Err(AgentError::Adapter("fixture timeout".to_string())));
    let timeout_adapter = adapter_with_transport(runtime(), timeout_transport.clone());
    let timeout = timeout_adapter
        .execute(&request(AgentOutputMode::Text), &AgentExecutionContext::default())
        .expect_err("transport failure returned");
    let mut small_runtime = runtime();
    small_runtime.max_response_bytes = SMALL_RESPONSE_MAX_BYTES;
    let oversized_transport = StaticTransport::new(Ok(OpenAiHttpResponse {
        status: HTTP_STATUS_OK,
        body: vec![b'x'; OVERSIZED_BODY_BYTES],
    }));
    let oversized_adapter = adapter_with_transport(small_runtime, oversized_transport);
    let oversized = oversized_adapter
        .execute(&request(AgentOutputMode::Text), &AgentExecutionContext::default())
        .expect_err("oversized response rejected");

    assert!(timeout.to_string().contains("fixture timeout"));
    assert!(oversized.to_string().contains("exceeded"));
}

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.real-graph]
// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.success]
#[tokio::test]
async fn production_transport_calls_local_compatible_endpoint() {
    let response_body = response(Some(TEXT_CONTENT)).body;
    let (endpoint, server) = one_response_server(HTTP_STATUS_OK, response_body, None);
    let mut local_runtime = runtime();
    local_runtime.endpoint = endpoint;
    local_runtime.enable_thinking = None;
    let adapter = ProductionAgentAdapter::new_with_openai_transport(
        runtime_registry(local_runtime),
        Arc::new(ReqwestOpenAiTransport),
    );

    let result = adapter
        .execute(&request(AgentOutputMode::Text), &AgentExecutionContext::default())
        .expect("local compatible endpoint succeeds");
    server.join().expect("local compatible server joins");

    assert_eq!(result.output, AgentOutput::Text(TEXT_CONTENT.to_string()));
}

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.failure]
#[test]
fn production_transport_enforces_request_timeout() {
    let (endpoint, server) = delayed_response_server();
    let mut local_runtime = runtime();
    local_runtime.endpoint = endpoint;
    local_runtime.timeout_secs = TIMEOUT_TEST_SECS;
    local_runtime.enable_thinking = None;
    let plan = plan_openai_chat(&local_runtime, &request(AgentOutputMode::Text), MAX_RESPONSE_BYTES)
        .expect("local timeout plan");

    let error = ReqwestOpenAiTransport.execute(&plan, None).expect_err("delayed endpoint times out");
    server.join().expect("timeout server joins");

    assert!(error.to_string().contains("timed out"), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.failure]
#[test]
fn production_transport_does_not_follow_redirects() {
    let (endpoint, server) = one_response_server(HTTP_STATUS_REDIRECT, b"redirect".to_vec(), Some("/redirected"));
    let mut local_runtime = runtime();
    local_runtime.endpoint = endpoint;
    local_runtime.enable_thinking = None;
    let plan = plan_openai_chat(&local_runtime, &request(AgentOutputMode::Text), MAX_RESPONSE_BYTES)
        .expect("local HTTP plan without credentials");

    let response = ReqwestOpenAiTransport.execute(&plan, None).expect("redirect response captured");
    server.join().expect("redirect server joins");

    assert_eq!(response.status, HTTP_STATUS_REDIRECT);
}

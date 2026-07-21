use super::*;

const MCP_ID: &str = "mcp-review-tools";
const ACP_ID: &str = "acp-coder";
const COMMAND_LABEL: &str = "review-server";
const ENDPOINT_LABEL: &str = "review-http";
const TOOL_READ: &str = "tool.read";
const TOOL_WRITE: &str = "tool.write";
const RESOURCE: &str = "repo-summary";
const PROMPT: &str = "review-prompt";
const SCHEMA: &str = "schema.review";
const CONTRACT: &str = "json-result";
const TIMEOUT: &str = "standard";
const ENV_KEY: &str = "MCP_TOKEN";
const MODEL_PROFILE: &str = "standard-reasoning";
const LIMITATION: &str = "no-background-writes";
const HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const HASH_CHARS: usize = 64;
const FIRST_INDEX: usize = 0;
const ENV_COUNT: u32 = 1;
const ARG_COUNT: u32 = 2;
const STATUS_DENIED: &str = "denied";

fn mcp_stdio() -> McpDescriptorDraft<'static> {
    McpDescriptorDraft {
        identity: MCP_ID,
        transport: TRANSPORT_STDIO,
        command_label: Some(COMMAND_LABEL),
        endpoint_label: None,
        tools: &[TOOL_READ],
        resources: &[RESOURCE],
        prompts: &[PROMPT],
        schema_ids: &[SCHEMA],
        result_contracts: &[CONTRACT],
        timeout_class: TIMEOUT,
        environment_keys: &[ENV_KEY],
        available_tool_ceilings: &[TOOL_READ],
        provenance_hash: HASH,
    }
}

fn acp_descriptor() -> AcpDescriptorDraft<'static> {
    AcpDescriptorDraft {
        identity: ACP_ID,
        model_profile: MODEL_PROFILE,
        animus_permission_mode: PERMISSION_ASK,
        external_permission_mode: PERMISSION_ASK,
        session_correlation: SESSION_JOIN_BY_LABEL,
        known_limitations: &[LIMITATION],
        tool_pass_through_policy: POLICY_INTERSECTION,
        host_admits_unrestricted: false,
    }
}

#[test]
fn mcp_descriptors_accept_stdio_and_streamable_http_labels() {
    let stdio = McpDescriptor::validate(&mcp_stdio()).expect("stdio descriptor should parse");
    let http = McpDescriptor::validate(&McpDescriptorDraft {
        transport: TRANSPORT_STREAMABLE_HTTP,
        command_label: None,
        endpoint_label: Some(ENDPOINT_LABEL),
        ..mcp_stdio()
    })
    .expect("http descriptor should parse");

    assert_eq!(stdio.transport(), McpTransportKind::Stdio);
    assert_eq!(stdio.command_label(), Some(COMMAND_LABEL));
    assert_eq!(http.transport(), McpTransportKind::StreamableHttp);
    assert_eq!(http.endpoint_label(), Some(ENDPOINT_LABEL));
    assert_eq!(stdio.environment_keys()[FIRST_INDEX], ENV_KEY);
}

#[test]
fn mcp_descriptors_reject_unsafe_runtime_claims() {
    assert!(matches!(
        McpDescriptor::validate(&McpDescriptorDraft {
            transport: "websocket",
            ..mcp_stdio()
        }),
        Err(InteropError::UnsupportedValue { .. })
    ));
    assert_eq!(
        McpDescriptor::validate(&McpDescriptorDraft {
            tools: &[TOOL_READ, TOOL_READ],
            ..mcp_stdio()
        }),
        Err(InteropError::DuplicateValue {
            field: TOOL_FIELD.as_str(),
            value: Text::from(TOOL_READ),
        })
    );
    assert_eq!(
        McpDescriptor::validate(&McpDescriptorDraft {
            timeout_class: "",
            ..mcp_stdio()
        }),
        Err(InteropError::MissingTimeout)
    );
    assert_eq!(
        McpDescriptor::validate(&McpDescriptorDraft {
            environment_keys: &["MCP_TOKEN=secret"],
            ..mcp_stdio()
        }),
        Err(InteropError::EnvironmentValueLeak)
    );
    assert_eq!(
        McpDescriptor::validate(&McpDescriptorDraft {
            transport: TRANSPORT_STREAMABLE_HTTP,
            command_label: None,
            endpoint_label: Some("https://example.test"),
            ..mcp_stdio()
        }),
        Err(InteropError::UnsafeRuntimeLabel)
    );
}

#[test]
fn acp_descriptors_reject_unsafe_permission_downgrades() {
    let descriptor = AcpDescriptor::validate(&acp_descriptor()).expect("ACP descriptor parses");

    assert_eq!(descriptor.identity(), ACP_ID);
    assert_eq!(
        descriptor.session_correlation(),
        AcpSessionCorrelation::JoinByLabel
    );
    assert_eq!(
        AcpDescriptor::validate(&AcpDescriptorDraft {
            external_permission_mode: PERMISSION_UNRESTRICTED,
            ..acp_descriptor()
        }),
        Err(InteropError::UnsafePermissionMapping)
    );
    assert_eq!(
        AcpDescriptor::validate(&AcpDescriptorDraft {
            session_correlation: "",
            ..acp_descriptor()
        }),
        Err(InteropError::MissingSessionCorrelation)
    );
}

#[test]
fn descriptor_validation_is_data_only() {
    let rendered = alloc::format!("{PROHIBITED_INTEROP_EFFECTS:?}");
    let descriptor = McpDescriptor::validate(&mcp_stdio()).expect("descriptor parses");

    assert_eq!(descriptor.identity(), MCP_ID);
    assert!(rendered.contains("stdio process launch"));
    assert!(rendered.contains("credential lookup"));
}

#[test]
fn tool_pass_through_is_intersection_only() {
    let resolution = resolve_tool_pass_through(ToolPassThroughInput {
        requested_tools: &[TOOL_READ, TOOL_WRITE],
        animus_capability_ceilings: &[TOOL_READ, TOOL_WRITE],
        permission_mode_tools: &[TOOL_READ],
        host_admission: &[TOOL_READ],
        external_agent_mode: &[TOOL_READ, TOOL_WRITE],
        available_tools: &[TOOL_READ],
    })
    .expect("tools should resolve");

    assert_eq!(resolution.allowed_tools(), &[Text::from(TOOL_READ)]);
    assert_eq!(resolution.denied_tools(), &[Text::from(TOOL_WRITE)]);
}

#[test]
fn interop_receipts_are_hash_and_count_only() {
    let receipt = InteropReceipt::new(&InteropReceiptDraft {
        operation: OP_PASS_THROUGH,
        status: STATUS_DENIED,
        bridge_hash: HASH,
        command_hash: Some(HASH),
        endpoint_hash: Some(HASH),
        external_session_hash: Some(HASH),
        env_key_count: ENV_COUNT,
        argument_count: ARG_COUNT,
        redactions: &["command_hashed", "endpoint_hashed", "session_hashed"],
    })
    .expect("receipt should parse");

    assert_eq!(receipt.operation(), InteropOperationKind::PassThrough);
    assert_eq!(receipt.env_key_count(), ENV_COUNT);
    assert_eq!(receipt.argument_count(), ARG_COUNT);
    assert_eq!(receipt.identity_hash().len(), HASH_CHARS);
    assert_eq!(
        validate_interop_receipt_text("bridge hash redactions bounded"),
        Ok(())
    );
    assert_eq!(
        validate_interop_receipt_text("endpoint=https://example.test args=raw api_key=x"),
        Err(InteropError::ReceiptLeak)
    );
}

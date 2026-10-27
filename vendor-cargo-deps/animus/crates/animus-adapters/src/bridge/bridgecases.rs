use super::*;

const MCP_ID: &str = "mcp-review-tools";
const TOOL_READ: &str = "tool.read";
const SCHEMA: &str = "schema.review";
const HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const BLAKE3_HASH_HEX_CHARS: usize = 64;
const SOURCE_METADATA: &str = "host-registered-mcp";
const RESULT_BOUNDS: &str = "bounded-json-result";
const RESOURCE_POLICY: &str = "resource.repo-summary";
const ABILITY_POLICY: &str = "ability.read";
const STATUS_OK: &str = "ok";
const REDACTED_RESOURCE: &str = "resource-hash-repo-summary";
const TRUNCATION_NONE: &str = "none";
const TRUNCATION_BOUNDED: &str = "bounded";
const TRANSPORT_SAFE_RECEIPT: &str = "server=mcp-review-tools resource=resource-hash";
const TRANSPORT_LEAK_RECEIPT: &str =
    "raw_payload={...} provider_payload=/home/user/secret args=token";
const UNSAFE_RESOURCE_URL: &str = "https://example.test/raw";
const BAD_HASH: &str = "bad-hash";

fn mcp_descriptor() -> McpDescriptor {
    McpDescriptor::validate(&McpDescriptorDraft {
        identity: MCP_ID,
        transport: TRANSPORT_STDIO,
        command_label: Some("review-server"),
        endpoint_label: None,
        tools: &[TOOL_READ],
        resources: &["repo-summary"],
        prompts: &["review-prompt"],
        schema_ids: &[SCHEMA],
        result_contracts: &["json-result"],
        timeout_class: "standard",
        environment_keys: &["MCP_TOKEN"],
        available_tool_ceilings: &[TOOL_READ],
        provenance_hash: HASH,
    })
    .expect("descriptor parses")
}

fn valid_mapping() -> McpToolMappingDraft<'static> {
    McpToolMappingDraft {
        server_label: Some(MCP_ID),
        tool_name: Some(TOOL_READ),
        schema_identity: Some(SCHEMA),
        capability_labels: &[TOOL_READ],
        source_metadata: Some(SOURCE_METADATA),
        result_bounds: Some(RESULT_BOUNDS),
        resource_policy_refs: &[RESOURCE_POLICY],
        ability_policy_refs: &[ABILITY_POLICY],
    }
}

fn ready_dispatch_request(definition: &crate::tool::Definition) -> McpDispatchRequest<'_> {
    McpDispatchRequest {
        definition: Some(definition),
        argument_status: McpArgumentStatus::Valid,
        executor_status: crate::tool::ExecutorStatus::Available,
        capability_status: crate::tool::CapabilityStatus::Allowed,
        mutation_status: McpMutationStatus::Independent,
        output_boundary_status: crate::tool::OutputBoundaryStatus::Bounded,
    }
}

fn safe_receipt_draft() -> McpReceiptDraft<'static> {
    McpReceiptDraft {
        server_label: MCP_ID,
        tool_name: TOOL_READ,
        schema_identity: SCHEMA,
        status: STATUS_OK,
        capability_labels: &[TOOL_READ],
        redacted_resource_ids: &[REDACTED_RESOURCE],
        truncation_state: TRUNCATION_NONE,
        response_hash: HASH,
    }
}

#[test]
fn untrusted_mcp_servers_are_not_advertised_and_do_not_probe() {
    // r[verify agent_harness_runtime.claude_code_mcp.scope]
    // r[verify agent_harness_runtime.claude_code_mcp.scope.untrusted]
    let descriptor = mcp_descriptor();
    let untrusted = plan_mcp_advertisement(McpAdvertisementInput {
        descriptor: Some(&descriptor),
        trust: McpServerTrust::Untrusted,
    });
    let registered = plan_mcp_advertisement(McpAdvertisementInput {
        descriptor: Some(&descriptor),
        trust: McpServerTrust::Registered,
    });
    let missing_descriptor = plan_mcp_advertisement(McpAdvertisementInput {
        descriptor: None,
        trust: McpServerTrust::Registered,
    });

    assert!(!untrusted.advertised());
    assert!(!untrusted.transport_probe_started());
    assert!(registered.advertised());
    assert!(!registered.transport_probe_started());
    assert!(!missing_descriptor.advertised());
    assert!(!missing_descriptor.transport_probe_started());
}

#[test]
fn mcp_tool_mapping_converts_to_animus_definition_and_rejects_incomplete() {
    // r[verify agent_harness_runtime.claude_code_mcp.mapping]
    // r[verify agent_harness_runtime.claude_code_mcp.mapping.incomplete]
    let mapping = McpToolMapping::validate(&valid_mapping()).expect("mapping parses");
    let definition = mapping
        .to_tool_definition()
        .expect("mapping should produce tool definition");

    assert_eq!(mapping.server_label(), MCP_ID);
    assert_eq!(mapping.tool_name(), TOOL_READ);
    assert_eq!(mapping.schema_identity(), SCHEMA);
    assert_eq!(
        mapping.resource_policy_refs(),
        &[Text::from(RESOURCE_POLICY)]
    );
    assert_eq!(mapping.ability_policy_refs(), &[Text::from(ABILITY_POLICY)]);
    assert_eq!(definition.name(), TOOL_READ);
    assert_eq!(definition.parameter_schema_identity(), SCHEMA);
    assert_eq!(definition.capability_labels(), &[Text::from(TOOL_READ)]);
    assert_eq!(definition.result_metadata_shape(), RESULT_BOUNDS);

    assert!(matches!(
        McpToolMapping::validate(&McpToolMappingDraft {
            schema_identity: None,
            ..valid_mapping()
        }),
        Err(InteropError::Contract(
            crate::AdapterContractError::EmptyField { .. }
        ))
    ));
    assert_eq!(
        McpToolMapping::validate(&McpToolMappingDraft {
            capability_labels: &[],
            ..valid_mapping()
        }),
        Err(InteropError::EmptyCollection {
            field: "mcp_capability_label"
        })
    );
    assert!(matches!(
        McpToolMapping::validate(&McpToolMappingDraft {
            source_metadata: None,
            ..valid_mapping()
        }),
        Err(InteropError::Contract(
            crate::AdapterContractError::EmptyField { .. }
        ))
    ));
    assert!(matches!(
        McpToolMapping::validate(&McpToolMappingDraft {
            result_bounds: None,
            ..valid_mapping()
        }),
        Err(InteropError::Contract(
            crate::AdapterContractError::EmptyField { .. }
        ))
    ));
    assert_eq!(
        McpToolMapping::validate(&McpToolMappingDraft {
            resource_policy_refs: &[],
            ..valid_mapping()
        }),
        Err(InteropError::EmptyCollection {
            field: "mcp_resource_policy_ref"
        })
    );
}

#[test]
fn mcp_dispatch_uses_animus_gates_before_server_adapter_call() {
    // r[verify agent_harness_runtime.claude_code_mcp.dispatch]
    // r[verify agent_harness_runtime.claude_code_mcp.dispatch.denied]
    let mapping = McpToolMapping::validate(&valid_mapping()).expect("mapping parses");
    let definition = mapping
        .to_tool_definition()
        .expect("mapping should produce tool definition");
    let ready = evaluate_mcp_dispatch(ready_dispatch_request(&definition));
    let denied = evaluate_mcp_dispatch(McpDispatchRequest {
        capability_status: crate::tool::CapabilityStatus::Denied,
        ..ready_dispatch_request(&definition)
    });
    let invalid_arguments = evaluate_mcp_dispatch(McpDispatchRequest {
        argument_status: McpArgumentStatus::Invalid,
        ..ready_dispatch_request(&definition)
    });
    let serialized = evaluate_mcp_dispatch(McpDispatchRequest {
        mutation_status: McpMutationStatus::Serialized,
        ..ready_dispatch_request(&definition)
    });

    assert_eq!(
        ready.dispatch().status(),
        crate::tool::DispatchStatus::Ready
    );
    assert!(ready.server_adapter_called());
    assert_eq!(
        denied.dispatch().failure_class(),
        Some(crate::tool::FailureClass::DeniedCapability)
    );
    assert!(!denied.server_adapter_called());
    assert_eq!(
        invalid_arguments.dispatch().failure_class(),
        Some(crate::tool::FailureClass::InvalidInput)
    );
    assert!(!invalid_arguments.server_adapter_called());
    assert_eq!(
        serialized.dispatch().status(),
        crate::tool::DispatchStatus::Ready
    );
    assert!(!serialized.server_adapter_called());
}

#[test]
fn mcp_transport_failures_never_fallback() {
    // r[verify agent_harness_runtime.claude_code_mcp.transport]
    // r[verify agent_harness_runtime.claude_code_mcp.transport.unsupported]
    let ready_case = evaluate_mcp_transport(McpTransportState::Available);
    let blocked_case = evaluate_mcp_transport(McpTransportState::Unsupported);
    let elapsed_case = evaluate_mcp_transport(McpTransportState::Timeout);
    let bad_response_case = evaluate_mcp_transport(McpTransportState::MalformedResponse);

    assert!(ready_case.available());
    assert_eq!(ready_case.failure_class(), None);
    assert!(!ready_case.fallback_attempted());
    assert!(!blocked_case.available());
    assert_eq!(
        blocked_case.failure_class(),
        Some(McpTransportFailureClass::Unsupported)
    );
    assert!(!blocked_case.fallback_attempted());
    assert_eq!(
        elapsed_case.failure_class(),
        Some(McpTransportFailureClass::Timeout)
    );
    assert!(!elapsed_case.fallback_attempted());
    assert_eq!(
        bad_response_case.failure_class(),
        Some(McpTransportFailureClass::MalformedResponse)
    );
    assert!(!bad_response_case.fallback_attempted());
}

#[test]
fn mcp_receipts_redact_payloads_and_validate_hashes() {
    // r[verify agent_harness_runtime.claude_code_mcp.receipts]
    // r[verify agent_harness_runtime.claude_code_mcp.receipts.malformed]
    let receipt = McpReceipt::new(&McpReceiptDraft {
        truncation_state: TRUNCATION_BOUNDED,
        ..safe_receipt_draft()
    })
    .expect("receipt should parse");

    assert_eq!(receipt.server_label(), MCP_ID);
    assert_eq!(receipt.tool_name(), TOOL_READ);
    assert_eq!(
        receipt.truncation_state(),
        McpReceiptTruncationState::Bounded
    );
    assert_eq!(receipt.response_hash(), HASH);
    assert_eq!(receipt.identity_hash().len(), BLAKE3_HASH_HEX_CHARS);
    assert_eq!(validate_mcp_receipt_text(TRANSPORT_SAFE_RECEIPT), Ok(()));
    assert_eq!(
        validate_mcp_receipt_text(TRANSPORT_LEAK_RECEIPT),
        Err(InteropError::ReceiptLeak)
    );
    assert_eq!(
        McpReceipt::new(&McpReceiptDraft {
            redacted_resource_ids: &[UNSAFE_RESOURCE_URL],
            ..safe_receipt_draft()
        }),
        Err(InteropError::ReceiptLeak)
    );
    assert!(matches!(
        McpReceipt::new(&McpReceiptDraft {
            response_hash: BAD_HASH,
            ..safe_receipt_draft()
        }),
        Err(InteropError::InvalidHash { .. })
    ));
}

use super::super::*;
use super::support::*;

#[test]
fn external_subagents_require_injected_descriptors() {
    let descriptor = ExternalSubagentDescriptor::validate(&ExternalSubagentDescriptorDraft {
        protocol: PROTOCOL_MCP,
        identity: "mcp-reviewer",
        service_label: "host-mcp-service",
        declared_tools: &[TOOL_READ],
        credential_label: None,
        ambient_cli: false,
        inherited_credentials: false,
    })
    .expect("descriptor should validate");

    assert_eq!(descriptor.protocol(), ExternalSubagentProtocol::Mcp);
    assert_eq!(descriptor.declared_tools()[0], TOOL_READ);
    assert_eq!(
        ExternalSubagentDescriptor::validate(&ExternalSubagentDescriptorDraft {
            ambient_cli: true,
            protocol: PROTOCOL_ACP,
            identity: "ambient",
            service_label: "host-acp-service",
            declared_tools: &[TOOL_READ],
            credential_label: None,
            inherited_credentials: false,
        }),
        Err(SubagentError::AmbientExternalDiscoveryDenied)
    );
    assert_eq!(
        ExternalSubagentDescriptor::validate(&ExternalSubagentDescriptorDraft {
            inherited_credentials: true,
            protocol: PROTOCOL_ACP,
            identity: "ambient",
            service_label: "host-acp-service",
            declared_tools: &[TOOL_READ],
            credential_label: None,
            ambient_cli: false,
        }),
        Err(SubagentError::InheritedCredentialDenied)
    );
}

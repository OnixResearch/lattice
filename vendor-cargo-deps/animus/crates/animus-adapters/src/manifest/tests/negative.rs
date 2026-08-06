use super::super::*;
use super::support::*;

#[test]
fn rejects_missing_required_identity() {
    let mut draft = safe_stdio_draft();
    draft.identity = None;
    let error =
        PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS).expect_err("missing identity should fail");

    assert_eq!(error, PluginDescriptorError::MissingField {
        field: IDENTITY_FIELD.as_str(),
    });
}

#[test]
fn rejects_unknown_toolset_kind_and_trust_status() {
    let mut bad_kind = safe_stdio_draft();
    bad_kind.toolset_kind = Some(UNKNOWN_TOOLSET_KIND);
    let kind_error = PluginDescriptor::validate(&bad_kind, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("unknown toolset kind should fail");
    let mut bad_trust = safe_stdio_draft();
    bad_trust.trust_status = Some(UNKNOWN_TRUST_STATUS);
    let trust_error = PluginDescriptor::validate(&bad_trust, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("unknown trust status should fail");

    assert_eq!(kind_error, PluginDescriptorError::UnknownDescriptorValue {
        field: TOOLSET_KIND_FIELD.as_str(),
        value: Text::from(UNKNOWN_TOOLSET_KIND),
    });
    assert_eq!(trust_error, PluginDescriptorError::UnknownDescriptorValue {
        field: TRUST_STATUS_FIELD.as_str(),
        value: Text::from(UNKNOWN_TRUST_STATUS),
    });
}

#[test]
fn rejects_unknown_runtime_kind() {
    let mut draft = safe_stdio_draft();
    draft.runtime_kind = Some(UNKNOWN_RUNTIME_KIND);
    let error = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("unknown runtime kind should fail");

    assert_eq!(error, PluginDescriptorError::UnknownDescriptorValue {
        field: RUNTIME_KIND_FIELD.as_str(),
        value: Text::from(UNKNOWN_RUNTIME_KIND),
    });
}

#[test]
fn rejects_missing_launch_command_for_stdio() {
    let mut draft = safe_stdio_draft();
    draft.launch_policy.command_label = None;
    let error = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("stdio descriptors require a launch label");

    assert_eq!(error, PluginDescriptorError::MissingLaunchCommand {
        runtime_kind: RuntimeKind::Stdio,
    });
}

#[test]
fn rejects_missing_tool_metadata() {
    let tools = [ToolDescriptorDraft {
        name: Some(TOOL_NAME),
        description: None,
        parameter_schema_identity: Some(SCHEMA_IDENTITY),
        result_contract_identity: Some(RESULT_IDENTITY),
        capability_labels: TOOL_CAPABILITIES,
    }];
    let mut missing_description = safe_stdio_draft();
    missing_description.declared_tools = &tools;
    let description_error = PluginDescriptor::validate(&missing_description, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("missing tool description should fail");

    assert_eq!(missing_description.declared_tools.len(), tools.len());
    assert_eq!(missing_description.declared_tools[0].description, None);
    assert_eq!(description_error, PluginDescriptorError::MissingField {
        field: TOOL_DESCRIPTION_FIELD.as_str(),
    });
}

#[test]
fn rejects_missing_schema_result_and_capabilities() {
    let missing_schema_tools = [ToolDescriptorDraft {
        name: Some(TOOL_NAME),
        description: Some(TOOL_DESCRIPTION),
        parameter_schema_identity: None,
        result_contract_identity: Some(RESULT_IDENTITY),
        capability_labels: TOOL_CAPABILITIES,
    }];
    let missing_result_tools = [ToolDescriptorDraft {
        name: Some(TOOL_NAME),
        description: Some(TOOL_DESCRIPTION),
        parameter_schema_identity: Some(SCHEMA_IDENTITY),
        result_contract_identity: None,
        capability_labels: TOOL_CAPABILITIES,
    }];
    let missing_capability_tools = [ToolDescriptorDraft {
        name: Some(TOOL_NAME),
        description: Some(TOOL_DESCRIPTION),
        parameter_schema_identity: Some(SCHEMA_IDENTITY),
        result_contract_identity: Some(RESULT_IDENTITY),
        capability_labels: &[],
    }];
    let mut missing_schema = safe_stdio_draft();
    missing_schema.declared_tools = &missing_schema_tools;
    let schema_error = PluginDescriptor::validate(&missing_schema, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("missing schema identity should fail");
    let mut missing_result = safe_stdio_draft();
    missing_result.declared_tools = &missing_result_tools;
    let result_error = PluginDescriptor::validate(&missing_result, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("missing result identity should fail");
    let mut missing_capability = safe_stdio_draft();
    missing_capability.declared_tools = &missing_capability_tools;
    let capability_error = PluginDescriptor::validate(&missing_capability, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("missing capability labels should fail");

    assert_eq!(schema_error, PluginDescriptorError::MissingField {
        field: TOOL_SCHEMA_FIELD.as_str(),
    });
    assert_eq!(result_error, PluginDescriptorError::MissingField {
        field: TOOL_RESULT_FIELD.as_str(),
    });
    assert_eq!(capability_error, PluginDescriptorError::MissingField {
        field: TOOL_CAPABILITY_FIELD.as_str(),
    });
}

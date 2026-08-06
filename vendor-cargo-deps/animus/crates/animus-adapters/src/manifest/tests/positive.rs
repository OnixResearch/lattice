use super::super::*;
use super::support::*;

#[test]
fn accepts_safe_stdio_descriptor() {
    let manifest = PluginDescriptor::validate(&safe_stdio_draft(), BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect("safe descriptor should validate");

    assert_eq!(manifest.identity(), IDENTITY);
    assert_eq!(manifest.version(), VERSION);
    assert_eq!(manifest.toolset_kind(), ToolsetKind::ReviewGate);
    assert_eq!(manifest.runtime_kind(), RuntimeKind::Stdio);
    assert_eq!(manifest.provenance(), PROVENANCE);
    assert_eq!(manifest.trust_status(), TrustStatus::Trusted);
    assert_eq!(manifest.declared_tools().len(), DECLARED_TOOLS.len());
    assert_eq!(manifest.declared_tools()[0].name(), TOOL_NAME);
    assert_eq!(manifest.declared_tools()[0].description(), TOOL_DESCRIPTION);
    assert_eq!(manifest.declared_tools()[0].parameter_schema_identity(), SCHEMA_IDENTITY);
    assert_eq!(manifest.declared_tools()[0].result_contract_identity(), RESULT_IDENTITY);
    assert_eq!(manifest.declared_tools()[0].capability_labels(), &[Text::from(CAPABILITY)]);
    assert_eq!(manifest.capabilities(), &[Text::from(CAPABILITY)]);
    assert_eq!(manifest.launch_policy().command_label(), Some(COMMAND_LABEL));
    assert_eq!(manifest.launch_policy().arguments(), &[Text::from(ARGUMENT_LABEL)]);
    assert_eq!(manifest.launch_policy().working_directory(), WorkingDirectoryMode::ProjectRoot);
    assert_eq!(manifest.launch_policy().environment_allowlist(), &[Text::from(ENVIRONMENT_NAME)]);
    assert_eq!(manifest.launch_policy().writable_roots(), &[Text::from(SAFE_WRITABLE_ROOT)]);
    assert_eq!(manifest.sandbox().mode(), SandboxMode::Isolated);
    assert_eq!(manifest.sandbox().network(), NetworkPolicy::Denied);
    assert!(manifest.redaction().redact_transcripts());
    assert!(manifest.redaction().redact_secrets());

    let definitions = manifest.tool_definitions(EXECUTORS).expect("manifest tools should project into definitions");
    assert_eq!(definitions.len(), DECLARED_TOOLS.len());
    assert_eq!(definitions[0].name(), TOOL_NAME);
    assert_eq!(definitions[0].status(), crate::tool::DefinitionStatus::Active);
    assert_eq!(definitions[0].source_identity(), IDENTITY);
    assert_eq!(definitions[0].provenance(), PROVENANCE);
    assert_eq!(definitions[0].capability_labels(), &[Text::from(CAPABILITY)]);
}

#[test]
fn accepts_host_descriptor_without_launch_command() {
    let mut draft = safe_stdio_draft();
    draft.runtime_kind = Some(RUNTIME_KIND_HOST);
    draft.declared_tools = TWO_DECLARED_TOOLS;
    draft.launch_policy.command_label = None;
    draft.launch_policy.working_directory = Some(WORKING_DIRECTORY_MANIFEST_DIRECTORY);
    let manifest = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect("host descriptor should not require a launch label");

    assert_eq!(manifest.runtime_kind(), RuntimeKind::Host);
    assert_eq!(manifest.declared_tools().len(), TWO_DECLARED_TOOLS.len());
    assert_eq!(manifest.launch_policy().working_directory(), WorkingDirectoryMode::ManifestDirectory);
}

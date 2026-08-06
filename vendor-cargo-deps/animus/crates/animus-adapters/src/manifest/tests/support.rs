use super::super::*;

pub(super) const IDENTITY: &str = "example.safe";
pub(super) const VERSION: &str = "1.0.0";
pub(super) const PROVENANCE: &str = "bundle:hermes-review-gate";
pub(super) const UNSAFE_PROVENANCE: &str = "/home/example/plugin.yaml";
pub(super) const TOOL_NAME: &str = "safe.lookup";
pub(super) const SECOND_TOOL_NAME: &str = "safe.summarize";
pub(super) const BUILT_IN_TOOL_NAME: &str = "builtin.search";
pub(super) const CATALOG_TOOL_NAME: &str = "catalog.lookup";
pub(super) const TOOL_DESCRIPTION: &str = "safe lookup descriptor";
pub(super) const SECOND_TOOL_DESCRIPTION: &str = "safe summary descriptor";
pub(super) const SCHEMA_IDENTITY: &str = "schema.safe.lookup.v1";
pub(super) const RESULT_IDENTITY: &str = "result.safe.lookup.v1";
pub(super) const CAPABILITY: &str = "workspace-read";
pub(super) const METRICS_CAPABILITY: &str = "metrics-write";
pub(super) const COMMAND_LABEL: &str = "safe-binary";
pub(super) const ARGUMENT_LABEL: &str = "--json";
pub(super) const ENVIRONMENT_NAME: &str = "ANIMUS_SAFE_TOKEN";
pub(super) const SAFE_WRITABLE_ROOT: &str = "cache/safe";
pub(super) const UNKNOWN_RUNTIME_KIND: &str = "daemon";
pub(super) const UNKNOWN_TOOLSET_KIND: &str = "sidecar";
pub(super) const UNKNOWN_TRUST_STATUS: &str = "maybe";
pub(super) const PATH_ESCAPE_ROOT: &str = "../outside";
pub(super) const ABSOLUTE_ROOT: &str = "/var/tmp";
pub(super) const ENVIRONMENT_VALUE: &str = "SECRET=value";
pub(super) const ROLE_ID: &str = "action.safe.lookup";
pub(super) const SECOND_ROLE_ID: &str = "action.safe.summarize";
pub(super) const EVENT_ROLE_ID: &str = "event.safe.request";
pub(super) const PANE_ROLE_ID: &str = "pane.safe.review";
pub(super) const LINK_ROLE_ID: &str = "link.safe.issue";
pub(super) const ROLE_TITLE: &str = "Safe Lookup";
pub(super) const SECOND_ROLE_TITLE: &str = "Safe Summary";
pub(super) const ROLE_CONTEXT: &str = "workspace";
pub(super) const EVENT_NAME: &str = "request-started";
pub(super) const UNKNOWN_EVENT_NAME: &str = "future-event";
pub(super) const LINK_SCHEME: &str = "animus";
pub(super) const UNSAFE_ROLE_LABEL: &str = "../unsafe";
pub(super) const UNKNOWN_ROLE_KIND: &str = "daemon-action";
pub(super) const HOST_VERSION: &str = "1.0.0";
pub(super) const NEWER_HOST_VERSION: &str = "9.0.0";
pub(super) const INVALID_VERSION: &str = "latest";
pub(super) const PLATFORM: &str = "linux";
pub(super) const UNSUPPORTED_PLATFORM: &str = "haiku";
pub(super) const HOST_CAPABILITY: &str = "plugin-actions";
pub(super) const MISSING_HOST_CAPABILITY: &str = "missing-capability";
pub(super) const WORKSPACE_LABEL: &str = "workspace:main";
pub(super) const TAB_LABEL: &str = "tab:primary";
pub(super) const PANE_LABEL: &str = "pane:main";
pub(super) const AGENT_STATUS_LABEL: &str = "agent:idle";
pub(super) const WORKTREE_PROVENANCE_LABEL: &str = "worktree:clean";
pub(super) const SESSION_PROVENANCE_LABEL: &str = "session:bounded";
pub(super) const REQUEST_IDENTITY: &str = "request:abc123";
pub(super) const PLUGIN_ROOT_LABEL: &str = "plugin-root:bundle";
pub(super) const PLUGIN_CONFIG_LABEL: &str = "plugin-config:default";
pub(super) const PLUGIN_STATE_LABEL: &str = "plugin-state:empty";
pub(super) const SOURCE_METADATA_LABEL: &str = "source:fixture";
pub(super) const RAW_PROMPT_CONTEXT: &str = "RAW_PROMPT: hello";
pub(super) const TRANSCRIPT_CONTEXT: &str = "TRANSCRIPT: shell output";
pub(super) const CREDENTIAL_CONTEXT: &str = "SECRET=value";
pub(super) const PRIVATE_PATH_CONTEXT: &str = "/home/example/session";
pub(super) const UNBOUNDED_LOG_CONTEXT: &str = "UNBOUNDED_LOG: lines";
pub(super) const PROVIDER_PAYLOAD_CONTEXT: &str = "PROVIDER_PAYLOAD: body";
pub(super) const BUILT_IN_TOOLS: &[&str] = &[BUILT_IN_TOOL_NAME];
pub(super) const CATALOG_TOOLS: &[&str] = &[CATALOG_TOOL_NAME];
pub(super) const CAPABILITIES: &[&str] = &[CAPABILITY];
pub(super) const TOOL_CAPABILITIES: &[&str] = &[CAPABILITY];
pub(super) const METRICS_TOOL_CAPABILITIES: &[&str] = &[METRICS_CAPABILITY];
pub(super) const EXECUTORS: &[&str] = &[TOOL_NAME];
pub(super) const NO_EXECUTORS: &[&str] = &[];
pub(super) const ARGUMENTS: &[&str] = &[ARGUMENT_LABEL];
pub(super) const ENVIRONMENT_ALLOWLIST: &[&str] = &[ENVIRONMENT_NAME];
pub(super) const WRITABLE_ROOTS: &[&str] = &[SAFE_WRITABLE_ROOT];
pub(super) const ROLE_CONTEXTS: &[&str] = &[ROLE_CONTEXT];
pub(super) const HOST_CAPABILITIES: &[&str] = &[HOST_CAPABILITY];
pub(super) const PLATFORMS: &[&str] = &[PLATFORM];
pub(super) const MANIFEST_COMPATIBILITY: PluginCompatibilityDraft<'static> = PluginCompatibilityDraft {
    minimum_host_version: Some(HOST_VERSION),
    required_host_capabilities: HOST_CAPABILITIES,
    supported_platforms: PLATFORMS,
};
pub(super) const INHERITED_COMPATIBILITY: PluginCompatibilityDraft<'static> = PluginCompatibilityDraft {
    minimum_host_version: None,
    required_host_capabilities: &[],
    supported_platforms: &[],
};
pub(super) const NEWER_COMPATIBILITY: PluginCompatibilityDraft<'static> = PluginCompatibilityDraft {
    minimum_host_version: Some(NEWER_HOST_VERSION),
    required_host_capabilities: HOST_CAPABILITIES,
    supported_platforms: PLATFORMS,
};
pub(super) const UNSUPPORTED_PLATFORM_COMPATIBILITY: PluginCompatibilityDraft<'static> = PluginCompatibilityDraft {
    minimum_host_version: Some(HOST_VERSION),
    required_host_capabilities: HOST_CAPABILITIES,
    supported_platforms: &[UNSUPPORTED_PLATFORM],
};
pub(super) const MISSING_CAPABILITY_COMPATIBILITY: PluginCompatibilityDraft<'static> = PluginCompatibilityDraft {
    minimum_host_version: Some(HOST_VERSION),
    required_host_capabilities: &[MISSING_HOST_CAPABILITY],
    supported_platforms: PLATFORMS,
};
pub(super) const DECLARED_TOOLS: &[ToolDescriptorDraft<'static>] = &[ToolDescriptorDraft {
    name: Some(TOOL_NAME),
    description: Some(TOOL_DESCRIPTION),
    parameter_schema_identity: Some(SCHEMA_IDENTITY),
    result_contract_identity: Some(RESULT_IDENTITY),
    capability_labels: TOOL_CAPABILITIES,
}];
pub(super) const DUPLICATE_DECLARED_TOOLS: &[ToolDescriptorDraft<'static>] = &[
    ToolDescriptorDraft {
        name: Some(TOOL_NAME),
        description: Some(TOOL_DESCRIPTION),
        parameter_schema_identity: Some(SCHEMA_IDENTITY),
        result_contract_identity: Some(RESULT_IDENTITY),
        capability_labels: TOOL_CAPABILITIES,
    },
    ToolDescriptorDraft {
        name: Some(TOOL_NAME),
        description: Some(TOOL_DESCRIPTION),
        parameter_schema_identity: Some(SCHEMA_IDENTITY),
        result_contract_identity: Some(RESULT_IDENTITY),
        capability_labels: TOOL_CAPABILITIES,
    },
];
pub(super) const BUILT_IN_COLLISION_TOOLS: &[ToolDescriptorDraft<'static>] = &[ToolDescriptorDraft {
    name: Some(BUILT_IN_TOOL_NAME),
    description: Some(TOOL_DESCRIPTION),
    parameter_schema_identity: Some(SCHEMA_IDENTITY),
    result_contract_identity: Some(RESULT_IDENTITY),
    capability_labels: TOOL_CAPABILITIES,
}];
pub(super) const CATALOG_COLLISION_TOOLS: &[ToolDescriptorDraft<'static>] = &[ToolDescriptorDraft {
    name: Some(CATALOG_TOOL_NAME),
    description: Some(TOOL_DESCRIPTION),
    parameter_schema_identity: Some(SCHEMA_IDENTITY),
    result_contract_identity: Some(RESULT_IDENTITY),
    capability_labels: TOOL_CAPABILITIES,
}];
pub(super) const TWO_DECLARED_TOOLS: &[ToolDescriptorDraft<'static>] = &[
    ToolDescriptorDraft {
        name: Some(TOOL_NAME),
        description: Some(TOOL_DESCRIPTION),
        parameter_schema_identity: Some(SCHEMA_IDENTITY),
        result_contract_identity: Some(RESULT_IDENTITY),
        capability_labels: TOOL_CAPABILITIES,
    },
    ToolDescriptorDraft {
        name: Some(SECOND_TOOL_NAME),
        description: Some(SECOND_TOOL_DESCRIPTION),
        parameter_schema_identity: Some(SCHEMA_IDENTITY),
        result_contract_identity: Some(RESULT_IDENTITY),
        capability_labels: METRICS_TOOL_CAPABILITIES,
    },
];

pub(super) const DECLARED_ROLES: &[PluginRoleDraft<'static>] = &[
    PluginRoleDraft {
        kind: Some("action"),
        id: Some(ROLE_ID),
        title: Some(ROLE_TITLE),
        context_labels: ROLE_CONTEXTS,
        command_label: Some(COMMAND_LABEL),
        artifact_label: None,
        event_name: None,
        link_scheme: None,
        compatibility: INHERITED_COMPATIBILITY,
    },
    PluginRoleDraft {
        kind: Some("event-hook"),
        id: Some(EVENT_ROLE_ID),
        title: Some(ROLE_TITLE),
        context_labels: ROLE_CONTEXTS,
        command_label: Some(COMMAND_LABEL),
        artifact_label: None,
        event_name: Some(EVENT_NAME),
        link_scheme: None,
        compatibility: INHERITED_COMPATIBILITY,
    },
    PluginRoleDraft {
        kind: Some("host-pane-entrypoint"),
        id: Some(PANE_ROLE_ID),
        title: Some(ROLE_TITLE),
        context_labels: ROLE_CONTEXTS,
        command_label: None,
        artifact_label: Some(COMMAND_LABEL),
        event_name: None,
        link_scheme: None,
        compatibility: INHERITED_COMPATIBILITY,
    },
    PluginRoleDraft {
        kind: Some("link-handler"),
        id: Some(LINK_ROLE_ID),
        title: Some(ROLE_TITLE),
        context_labels: ROLE_CONTEXTS,
        command_label: Some(COMMAND_LABEL),
        artifact_label: None,
        event_name: None,
        link_scheme: Some(LINK_SCHEME),
        compatibility: INHERITED_COMPATIBILITY,
    },
];
pub(super) const DUPLICATE_ROLES: &[PluginRoleDraft<'static>] = &[
    PluginRoleDraft {
        kind: Some("action"),
        id: Some(ROLE_ID),
        title: Some(ROLE_TITLE),
        context_labels: ROLE_CONTEXTS,
        command_label: Some(COMMAND_LABEL),
        artifact_label: None,
        event_name: None,
        link_scheme: None,
        compatibility: INHERITED_COMPATIBILITY,
    },
    PluginRoleDraft {
        kind: Some("action"),
        id: Some(ROLE_ID),
        title: Some(SECOND_ROLE_TITLE),
        context_labels: ROLE_CONTEXTS,
        command_label: Some(COMMAND_LABEL),
        artifact_label: None,
        event_name: None,
        link_scheme: None,
        compatibility: INHERITED_COMPATIBILITY,
    },
];

pub(super) fn safe_stdio_draft() -> PluginDescriptorDraft<'static> {
    PluginDescriptorDraft {
        identity: Some(IDENTITY),
        version: Some(VERSION),
        toolset_kind: Some(TOOLSET_KIND_REVIEW_GATE),
        runtime_kind: Some(RUNTIME_KIND_STDIO),
        provenance: Some(PROVENANCE),
        trust_status: Some(TRUST_STATUS_TRUSTED),
        declared_tools: DECLARED_TOOLS,
        declared_roles: &[],
        capabilities: CAPABILITIES,
        compatibility: MANIFEST_COMPATIBILITY,
        launch_policy: LaunchPolicyDraft {
            command_label: Some(COMMAND_LABEL),
            arguments: ARGUMENTS,
            working_directory: Some(WORKING_DIRECTORY_PROJECT_ROOT),
            environment_allowlist: ENVIRONMENT_ALLOWLIST,
            writable_roots: WRITABLE_ROOTS,
        },
        sandbox: SandboxRequirementsDraft {
            mode: Some(SANDBOX_MODE_ISOLATED),
            network: Some(NETWORK_POLICY_DENIED),
        },
        redaction: RedactionPolicyDraft {
            redact_transcripts: true,
            redact_secrets: true,
        },
    }
}

#![allow(non_trait_imports)]

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

const IDENTITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("manifest_identity");
const VERSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("manifest_version");
const TOOLSET_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("toolset_kind");
const RUNTIME_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("runtime_kind");
const TRUST_STATUS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("manifest_trust_status");
const PROVENANCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("manifest_provenance");
const TOOL_NAME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("declared_tool_name");
const TOOL_DESCRIPTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("declared_tool_description");
const TOOL_SCHEMA_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("declared_tool_parameter_schema_identity");
const TOOL_RESULT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("declared_tool_result_contract_identity");
const TOOL_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("declared_tool_capability");
const CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("capability");
const LAUNCH_COMMAND_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("launch_command_label");
const LAUNCH_ARGUMENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("launch_argument");
const WORKING_DIRECTORY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("working_directory_mode");
const ENVIRONMENT_ALLOWLIST_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("environment_allowlist_name");
const WRITABLE_ROOT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("writable_root");
const SANDBOX_MODE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("sandbox_mode");
const NETWORK_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("network_policy");

const RUNTIME_KIND_HOST: &str = "host";
const RUNTIME_KIND_STDIO: &str = "stdio";
const RUNTIME_KIND_WASM: &str = "wasm";
const TOOLSET_KIND_GENERIC: &str = "generic";
const TOOLSET_KIND_REVIEW_GATE: &str = "review-gate";
const TRUST_STATUS_TRUSTED: &str = "trusted";
const TRUST_STATUS_UNTRUSTED: &str = "untrusted";
const WORKING_DIRECTORY_PROJECT_ROOT: &str = "project-root";
const WORKING_DIRECTORY_MANIFEST_DIRECTORY: &str = "manifest-directory";
const SANDBOX_MODE_UNCONFINED: &str = "unconfined";
const SANDBOX_MODE_ISOLATED: &str = "isolated";
const NETWORK_POLICY_DENIED: &str = "denied";
const NETWORK_POLICY_LOOPBACK: &str = "loopback";
const NETWORK_POLICY_OUTBOUND: &str = "outbound";
const ENVIRONMENT_VALUE_SEPARATOR: char = '=';
const UNIX_PATH_SEPARATOR: char = '/';
const WINDOWS_PATH_SEPARATOR: char = '\\';
const HOME_PATH_PREFIX: char = '~';
const PATH_ESCAPE_SEGMENT: &str = "..";
const WINDOWS_DRIVE_SEPARATOR: u8 = b':';
const WINDOWS_DRIVE_SEPARATOR_INDEX: usize = 1;
const PROMPT_GUIDELINE_COUNT: usize = 1;
const INJECTED_EXECUTOR_GUIDELINE: &str =
    "Use only host-injected executors for this manifest tool.";

/// Descriptor-side effects that manifest validation explicitly does not perform.
pub const PROHIBITED_PLUGIN_MANIFEST_EFFECTS: &[&str] = &[
    "stdio process launch",
    "wasm module loading",
    "environment inheritance",
    "secret reads",
    "network connections",
    "tool execution",
    "pane opening",
    "registry persistence",
];

impl PluginDescriptor {
    /// Validates untrusted manifest-shaped data as a descriptor only.
    ///
    /// # Errors
    ///
    /// Returns [`PluginDescriptorError`] when required fields are missing, field
    /// text is invalid, runtime/sandbox/network labels are unknown, tool names
    /// collide, or path/network declarations are unsafe.
    pub fn validate(
        draft: &PluginDescriptorDraft<'_>,
        built_in_tool_names: &[&str],
        existing_catalog_tool_names: &[&str],
    ) -> Result<Self, PluginDescriptorError> {
        let identity = validation::required_text(IDENTITY_FIELD, draft.identity)?;
        let version = validation::required_text(VERSION_FIELD, draft.version)?;
        let toolset_kind = validation::validate_toolset_kind(draft.toolset_kind)?;
        let runtime_text = validation::required_text(RUNTIME_KIND_FIELD, draft.runtime_kind)?;
        let runtime_kind = validation::parse_runtime_kind(&runtime_text)?;
        let provenance = validation::validate_provenance(draft.provenance)?;
        let trust_status = validation::validate_trust_status(draft.trust_status)?;
        let declared_tools = validation::validate_tools(
            draft.declared_tools,
            built_in_tool_names,
            existing_catalog_tool_names,
        )?;
        let compatibility = action::check::validate_manifest_compatibility(&draft.compatibility)?;
        let declared_roles = action::check::validate_roles(draft.declared_roles, &compatibility)?;
        let capabilities =
            validation::validate_required_texts(CAPABILITY_FIELD, draft.capabilities)?;
        let launch_policy = validation::validate_launch_policy(runtime_kind, &draft.launch_policy)?;
        let sandbox = validation::validate_sandbox_requirements(&draft.sandbox)?;
        validation::assert_network_sandbox_combination(sandbox.network, sandbox.mode)?;
        let redaction = RedactionPolicy {
            redact_transcripts: draft.redaction.redact_transcripts,
            redact_secrets: draft.redaction.redact_secrets,
        };

        Ok(Self {
            identity,
            version,
            toolset_kind,
            runtime_kind,
            provenance,
            trust_status,
            declared_tools,
            declared_roles,
            capabilities,
            compatibility,
            launch_policy,
            sandbox,
            redaction,
        })
    }

    /// Builds typed tool definitions from manifest metadata and injected executor names.
    ///
    /// Definitions are active only when the manifest is trusted and a matching
    /// executor name is supplied by the host. No process, module, script, or
    /// provider fallback is attempted here.
    ///
    /// # Errors
    ///
    /// Returns [`crate::tool::ContractError`] when projected metadata violates
    /// the tool-definition boundary contract.
    pub fn tool_definitions(
        &self,
        injected_executor_names: &[&str],
    ) -> Result<Collection<crate::tool::Definition>, crate::tool::ContractError> {
        let mut definitions = Collection::with_capacity(self.declared_tools.len());
        for tool in &self.declared_tools {
            let capability_labels = tool.capability_label_strs();
            let prompt_guidelines = [INJECTED_EXECUTOR_GUIDELINE; PROMPT_GUIDELINE_COUNT];
            let draft = crate::tool::DefinitionDraft {
                name: Some(tool.name()),
                label: Some(tool.name()),
                description: Some(tool.description()),
                parameter_schema_identity: Some(tool.parameter_schema_identity()),
                prompt_snippet: Some(tool.description()),
                prompt_guidelines: prompt_guidelines.as_slice(),
                source_identity: Some(self.identity()),
                provenance: Some(self.provenance()),
                capability_labels: capability_labels.as_slice(),
                result_metadata_shape: Some(tool.result_contract_identity()),
                status: self.definition_status(tool.name(), injected_executor_names),
            };
            definitions.push(crate::tool::Definition::validate(&draft)?);
        }
        Ok(definitions)
    }

    fn definition_status(
        &self,
        tool_name: &str,
        injected_executor_names: &[&str],
    ) -> crate::tool::DefinitionStatus {
        if self.trust_status != TrustStatus::Trusted {
            return crate::tool::DefinitionStatus::Inactive;
        }
        if injected_executor_names.contains(&tool_name) {
            crate::tool::DefinitionStatus::Active
        } else {
            crate::tool::DefinitionStatus::Inactive
        }
    }
}

mod accessors;
mod action;
mod error;
mod types;
mod validation;

pub use action::{
    PluginCompatibility, PluginCompatibilityDraft, PluginDiagnostic, PluginDiagnosticKind,
    PluginDiagnosticSeverity, PluginHostCompatibilityDraft, PluginInvocationContext,
    PluginInvocationContextDraft, PluginInvocationContextLeak, PluginRegistryOperationDraft,
    PluginRegistryOperationKind, PluginRegistryPlan, PluginRegistryPlanStatus, PluginRole,
    PluginRoleDraft, PluginRoleKind,
};
pub use types::{
    LaunchPolicy, LaunchPolicyDraft, NetworkPolicy, PluginDescriptor, PluginDescriptorDraft,
    RedactionPolicy, RedactionPolicyDraft, RuntimeKind, SandboxMode, SandboxRequirements,
    SandboxRequirementsDraft, ToolDescriptor, ToolDescriptorDraft, ToolsetKind, TrustStatus,
    WorkingDirectoryMode,
};

/// Descriptor manifest validation error.
pub type PluginDescriptorError = self::error::ValidationFailure;

#[cfg(test)]
mod tests;

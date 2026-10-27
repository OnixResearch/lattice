type Collection<T> = super::Collection<T>;
type LaunchPolicy = super::LaunchPolicy;
type LaunchPolicyDraft<'a> = super::LaunchPolicyDraft<'a>;
type NetworkPolicy = super::NetworkPolicy;
type PluginDescriptorError = super::PluginDescriptorError;
type RuntimeKind = super::RuntimeKind;
type SandboxMode = super::SandboxMode;
type SandboxRequirements = super::SandboxRequirements;
type SandboxRequirementsDraft<'a> = super::SandboxRequirementsDraft<'a>;
type Text = super::Text;
type ToolDescriptor = super::ToolDescriptor;
type ToolDescriptorDraft<'a> = super::ToolDescriptorDraft<'a>;
type WorkingDirectoryMode = super::WorkingDirectoryMode;

mod launch;
mod path;
mod tools;

pub(in crate::manifest) fn validate_launch_policy(
    runtime_kind: RuntimeKind,
    draft: &LaunchPolicyDraft<'_>,
) -> Result<LaunchPolicy, PluginDescriptorError> {
    launch::prepare_policy(runtime_kind, draft)
}

pub(in crate::manifest) fn validate_tools(
    drafts: &[ToolDescriptorDraft<'_>],
    built_in_tool_names: &[&str],
    existing_catalog_tool_names: &[&str],
) -> Result<Collection<ToolDescriptor>, PluginDescriptorError> {
    tools::collect_descriptors(drafts, built_in_tool_names, existing_catalog_tool_names)
}

const ENVIRONMENT_ALLOWLIST_FIELD: crate::validation::FieldName =
    super::ENVIRONMENT_ALLOWLIST_FIELD;
const ENVIRONMENT_VALUE_SEPARATOR: char = super::ENVIRONMENT_VALUE_SEPARATOR;
const HOME_PATH_PREFIX: char = super::HOME_PATH_PREFIX;
const LAUNCH_ARGUMENT_FIELD: crate::validation::FieldName = super::LAUNCH_ARGUMENT_FIELD;
const LAUNCH_COMMAND_LABEL_FIELD: crate::validation::FieldName = super::LAUNCH_COMMAND_LABEL_FIELD;
const NETWORK_POLICY_DENIED: &str = super::NETWORK_POLICY_DENIED;
const NETWORK_POLICY_FIELD: crate::validation::FieldName = super::NETWORK_POLICY_FIELD;
const NETWORK_POLICY_LOOPBACK: &str = super::NETWORK_POLICY_LOOPBACK;
const NETWORK_POLICY_OUTBOUND: &str = super::NETWORK_POLICY_OUTBOUND;
const PATH_ESCAPE_SEGMENT: &str = super::PATH_ESCAPE_SEGMENT;
const PROVENANCE_FIELD: crate::validation::FieldName = super::PROVENANCE_FIELD;
const RUNTIME_KIND_FIELD: crate::validation::FieldName = super::RUNTIME_KIND_FIELD;
const RUNTIME_KIND_HOST: &str = super::RUNTIME_KIND_HOST;
const RUNTIME_KIND_STDIO: &str = super::RUNTIME_KIND_STDIO;
const RUNTIME_KIND_WASM: &str = super::RUNTIME_KIND_WASM;
const SANDBOX_MODE_FIELD: crate::validation::FieldName = super::SANDBOX_MODE_FIELD;
const SANDBOX_MODE_ISOLATED: &str = super::SANDBOX_MODE_ISOLATED;
const SANDBOX_MODE_UNCONFINED: &str = super::SANDBOX_MODE_UNCONFINED;
const TOOL_CAPABILITY_FIELD: crate::validation::FieldName = super::TOOL_CAPABILITY_FIELD;
const TOOL_DESCRIPTION_FIELD: crate::validation::FieldName = super::TOOL_DESCRIPTION_FIELD;
const TOOL_NAME_FIELD: crate::validation::FieldName = super::TOOL_NAME_FIELD;
const TOOL_RESULT_FIELD: crate::validation::FieldName = super::TOOL_RESULT_FIELD;
const TOOL_SCHEMA_FIELD: crate::validation::FieldName = super::TOOL_SCHEMA_FIELD;
const TOOLSET_KIND_FIELD: crate::validation::FieldName = super::TOOLSET_KIND_FIELD;
const TOOLSET_KIND_GENERIC: &str = super::TOOLSET_KIND_GENERIC;
const TOOLSET_KIND_REVIEW_GATE: &str = super::TOOLSET_KIND_REVIEW_GATE;
const TRUST_STATUS_FIELD: crate::validation::FieldName = super::TRUST_STATUS_FIELD;
const TRUST_STATUS_TRUSTED: &str = super::TRUST_STATUS_TRUSTED;
const TRUST_STATUS_UNTRUSTED: &str = super::TRUST_STATUS_UNTRUSTED;
const UNIX_PATH_SEPARATOR: char = super::UNIX_PATH_SEPARATOR;
const WINDOWS_DRIVE_SEPARATOR: u8 = super::WINDOWS_DRIVE_SEPARATOR;
const WINDOWS_DRIVE_SEPARATOR_INDEX: usize = super::WINDOWS_DRIVE_SEPARATOR_INDEX;
const WINDOWS_PATH_SEPARATOR: char = super::WINDOWS_PATH_SEPARATOR;
const WORKING_DIRECTORY_FIELD: crate::validation::FieldName = super::WORKING_DIRECTORY_FIELD;
const WORKING_DIRECTORY_MANIFEST_DIRECTORY: &str = super::WORKING_DIRECTORY_MANIFEST_DIRECTORY;
const WORKING_DIRECTORY_PROJECT_ROOT: &str = super::WORKING_DIRECTORY_PROJECT_ROOT;
const WRITABLE_ROOT_FIELD: crate::validation::FieldName = super::WRITABLE_ROOT_FIELD;

const UNSAFE_PROVENANCE_MARKERS: &[&str] = &["../", "/home/", "/tmp/", "C:\\\\", "\\\\Users\\\\"];

pub(super) fn validate_sandbox_requirements(
    draft: &SandboxRequirementsDraft<'_>,
) -> Result<SandboxRequirements, PluginDescriptorError> {
    let mode = parse_sandbox_mode(&required_text(SANDBOX_MODE_FIELD, draft.mode)?)?;
    let network = parse_network_policy(&required_text(NETWORK_POLICY_FIELD, draft.network)?)?;
    Ok(SandboxRequirements { mode, network })
}

pub(super) fn assert_network_sandbox_combination(
    network: NetworkPolicy,
    sandbox: SandboxMode,
) -> Result<(), PluginDescriptorError> {
    if network == NetworkPolicy::Outbound && sandbox == SandboxMode::Unconfined {
        return Err(PluginDescriptorError::UnsafeNetworkSandboxCombination { network, sandbox });
    }
    Ok(())
}

pub(super) fn validate_toolset_kind(
    value: Option<&str>,
) -> Result<super::ToolsetKind, PluginDescriptorError> {
    let normalized = required_text(TOOLSET_KIND_FIELD, value)?;
    match normalized.as_str() {
        TOOLSET_KIND_GENERIC => Ok(super::ToolsetKind::Generic),
        TOOLSET_KIND_REVIEW_GATE => Ok(super::ToolsetKind::ReviewGate),
        unknown => Err(PluginDescriptorError::UnknownDescriptorValue {
            field: TOOLSET_KIND_FIELD.as_str(),
            value: Text::from(unknown),
        }),
    }
}

pub(super) fn validate_trust_status(
    value: Option<&str>,
) -> Result<super::TrustStatus, PluginDescriptorError> {
    let normalized = required_text(TRUST_STATUS_FIELD, value)?;
    match normalized.as_str() {
        TRUST_STATUS_TRUSTED => Ok(super::TrustStatus::Trusted),
        TRUST_STATUS_UNTRUSTED => Ok(super::TrustStatus::Untrusted),
        unknown => Err(PluginDescriptorError::UnknownDescriptorValue {
            field: TRUST_STATUS_FIELD.as_str(),
            value: Text::from(unknown),
        }),
    }
}

pub(super) fn validate_provenance(value: Option<&str>) -> Result<Text, PluginDescriptorError> {
    let provenance = required_text(PROVENANCE_FIELD, value)?;
    if provenance.starts_with(super::UNIX_PATH_SEPARATOR)
        || UNSAFE_PROVENANCE_MARKERS
            .iter()
            .any(|marker| provenance.contains(marker))
    {
        return Err(PluginDescriptorError::UnsafeProvenance { value: provenance });
    }
    Ok(provenance)
}

pub(super) fn parse_runtime_kind(value: &str) -> Result<RuntimeKind, PluginDescriptorError> {
    match value {
        RUNTIME_KIND_HOST => Ok(RuntimeKind::Host),
        RUNTIME_KIND_STDIO => Ok(RuntimeKind::Stdio),
        RUNTIME_KIND_WASM => Ok(RuntimeKind::Wasm),
        unknown => Err(PluginDescriptorError::UnknownDescriptorValue {
            field: RUNTIME_KIND_FIELD.as_str(),
            value: Text::from(unknown),
        }),
    }
}

fn parse_working_directory_mode(
    value: &str,
) -> Result<WorkingDirectoryMode, PluginDescriptorError> {
    match value {
        WORKING_DIRECTORY_PROJECT_ROOT => Ok(WorkingDirectoryMode::ProjectRoot),
        WORKING_DIRECTORY_MANIFEST_DIRECTORY => Ok(WorkingDirectoryMode::ManifestDirectory),
        unknown => Err(PluginDescriptorError::UnknownDescriptorValue {
            field: WORKING_DIRECTORY_FIELD.as_str(),
            value: Text::from(unknown),
        }),
    }
}

fn parse_sandbox_mode(value: &str) -> Result<SandboxMode, PluginDescriptorError> {
    match value {
        SANDBOX_MODE_UNCONFINED => Ok(SandboxMode::Unconfined),
        SANDBOX_MODE_ISOLATED => Ok(SandboxMode::Isolated),
        unknown => Err(PluginDescriptorError::UnknownDescriptorValue {
            field: SANDBOX_MODE_FIELD.as_str(),
            value: Text::from(unknown),
        }),
    }
}

fn parse_network_policy(value: &str) -> Result<NetworkPolicy, PluginDescriptorError> {
    match value {
        NETWORK_POLICY_DENIED => Ok(NetworkPolicy::Denied),
        NETWORK_POLICY_LOOPBACK => Ok(NetworkPolicy::Loopback),
        NETWORK_POLICY_OUTBOUND => Ok(NetworkPolicy::Outbound),
        unknown => Err(PluginDescriptorError::UnknownDescriptorValue {
            field: NETWORK_POLICY_FIELD.as_str(),
            value: Text::from(unknown),
        }),
    }
}

pub(super) fn validate_required_texts(
    field_name: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, PluginDescriptorError> {
    assert!(
        values.len() <= crate::MAX_ADAPTER_FIELD_CHARS,
        "required text count must fit the adapter field bound"
    );
    if values.is_empty() {
        return Err(PluginDescriptorError::MissingField {
            field: field_name.as_str(),
        });
    }
    let mut normalized = Collection::with_capacity(values.len());
    for value in values {
        normalized.push(required_text_value(field_name, value)?);
    }
    Ok(normalized)
}

fn validate_optional_texts(
    field_name: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, PluginDescriptorError> {
    assert!(
        values.len() <= crate::MAX_ADAPTER_FIELD_CHARS,
        "optional text count must fit the adapter field bound"
    );
    let mut normalized = Collection::with_capacity(values.len());
    for value in values {
        normalized.push(optional_text(field_name, value)?);
    }
    Ok(normalized)
}

pub(super) fn required_text(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Text, PluginDescriptorError> {
    match value {
        Some(raw) => required_text_value(field_name, raw),
        None => Err(PluginDescriptorError::MissingField {
            field: field_name.as_str(),
        }),
    }
}

fn required_text_value(
    field_name: crate::validation::FieldName,
    value: &str,
) -> Result<Text, PluginDescriptorError> {
    crate::validation::validate_required_adapter_field(field_name, value).map_err(|reason| {
        PluginDescriptorError::FieldRejected {
            field: field_name.as_str(),
            reason,
        }
    })?;
    Ok(Text::from(value.trim()))
}

fn optional_text(
    field_name: crate::validation::FieldName,
    value: &str,
) -> Result<Text, PluginDescriptorError> {
    crate::validation::validate_optional_adapter_field(field_name, value).map_err(|reason| {
        PluginDescriptorError::FieldRejected {
            field: field_name.as_str(),
            reason,
        }
    })?;
    Ok(Text::from(value))
}

fn contains_name(names: &[&str], candidate: &str) -> bool {
    names.contains(&candidate)
}

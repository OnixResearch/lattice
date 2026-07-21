use super::{
    LAUNCH_ARGUMENT_FIELD, LAUNCH_COMMAND_LABEL_FIELD, LaunchPolicy, LaunchPolicyDraft,
    PluginDescriptorError, RuntimeKind, WORKING_DIRECTORY_FIELD, WorkingDirectoryMode,
    parse_working_directory_mode, path, required_text_value, validate_optional_texts,
};

pub(in crate::manifest) fn prepare_policy(
    runtime_kind: RuntimeKind,
    draft: &LaunchPolicyDraft<'_>,
) -> Result<LaunchPolicy, PluginDescriptorError> {
    assert!(
        draft.arguments.len() <= crate::MAX_ADAPTER_FIELD_CHARS,
        "argument count must fit the adapter field bound"
    );
    assert!(
        draft.environment_allowlist.len() <= crate::MAX_ADAPTER_FIELD_CHARS,
        "environment allowlist count must fit the adapter field bound"
    );
    assert!(
        draft.writable_roots.len() <= crate::MAX_ADAPTER_FIELD_CHARS,
        "writable root count must fit the adapter field bound"
    );
    let command_label = match draft.command_label {
        Some(value) => Some(required_text_value(LAUNCH_COMMAND_LABEL_FIELD, value)?),
        None if runtime_kind.requires_launch_command() => {
            return Err(PluginDescriptorError::MissingLaunchCommand { runtime_kind });
        }
        None => None,
    };
    let arguments = validate_optional_texts(LAUNCH_ARGUMENT_FIELD, draft.arguments)?;
    let working_directory = match draft.working_directory {
        Some(value) => {
            parse_working_directory_mode(&required_text_value(WORKING_DIRECTORY_FIELD, value)?)?
        }
        None => WorkingDirectoryMode::ProjectRoot,
    };
    let environment_allowlist = path::validate_environment_allowlist(draft.environment_allowlist)?;
    let writable_roots = path::validate_writable_roots(draft.writable_roots)?;

    Ok(LaunchPolicy {
        command_label,
        arguments,
        working_directory,
        environment_allowlist,
        writable_roots,
    })
}

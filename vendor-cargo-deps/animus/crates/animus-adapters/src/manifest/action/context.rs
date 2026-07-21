use super::consts::{
    CREDENTIAL_MARKER, HOST_PATH_HOME_MARKER, HOST_PATH_TEMP_MARKER, KNOWN_EVENT_NAMES,
    PROVIDER_PAYLOAD_MARKER, RAW_PROMPT_MARKER, ROLE_CARRIAGE_RETURN_TEXT, ROLE_KIND_ACTION,
    ROLE_KIND_EVENT_HOOK, ROLE_KIND_HOST_PANE_ENTRYPOINT, ROLE_KIND_LINK_HANDLER,
    ROLE_NEWLINE_TEXT, ROLE_PATH_SEPARATOR, ROLE_PATH_SEPARATOR_TEXT, ROLE_PIPE_TEXT,
    ROLE_SHELL_COMMAND_TEXT, ROLE_SHELL_VARIABLE_TEXT, ROLE_WINDOWS_SEPARATOR_TEXT,
    TRANSCRIPT_MARKER, UNBOUNDED_LOG_MARKER, UNSAFE_PATH_PARENT, UNSAFE_SHELL_AND,
    WINDOWS_USERS_MARKER,
};
use super::types::{
    PluginDiagnostic, PluginDiagnosticKind, PluginDiagnosticSeverity, PluginInvocationContextLeak,
    PluginRegistryOperationKind, PluginRoleKind,
};
use super::{Collection, Text};

pub(super) fn validate_invocation_label(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Text, super::super::PluginDescriptorError> {
    assert!(
        field.as_str().len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "invocation field label must fit adapter bounds"
    );
    let Some(raw) = value else {
        return Err(super::super::PluginDescriptorError::MissingField {
            field: field.as_str(),
        });
    };
    crate::validation::validate_required_adapter_field(field, raw).map_err(|reason| {
        super::super::PluginDescriptorError::FieldRejected {
            field: field.as_str(),
            reason,
        }
    })?;
    let label = Text::from(raw.trim());
    assert!(
        label.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "invocation label must fit adapter bounds after validation"
    );
    if let Some(leak) = invocation_leak(label.as_str()) {
        return Err(
            super::super::PluginDescriptorError::UnsafeInvocationContext {
                field: field.as_str(),
                leak,
            },
        );
    }
    if !safe_role_label(label.as_str()) {
        return Err(super::super::PluginDescriptorError::UnsafeRoleLabel {
            field: field.as_str(),
            value: label,
        });
    }
    Ok(label)
}

fn invocation_leak(value: &str) -> Option<PluginInvocationContextLeak> {
    assert!(
        !value.is_empty(),
        "invocation leak scans require validated labels"
    );
    assert!(
        value.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "invocation leak scan labels must fit adapter bounds"
    );
    let marker_leaks = [
        (RAW_PROMPT_MARKER, PluginInvocationContextLeak::RawPrompt),
        (TRANSCRIPT_MARKER, PluginInvocationContextLeak::Transcript),
        (CREDENTIAL_MARKER, PluginInvocationContextLeak::Credential),
        (
            UNBOUNDED_LOG_MARKER,
            PluginInvocationContextLeak::UnboundedLog,
        ),
        (
            PROVIDER_PAYLOAD_MARKER,
            PluginInvocationContextLeak::ProviderPayload,
        ),
    ];
    for (marker, leak) in marker_leaks {
        if value.contains(marker) {
            return Some(leak);
        }
    }
    private_path_leak(value)
}

fn private_path_leak(value: &str) -> Option<PluginInvocationContextLeak> {
    if is_private_path_label(value) {
        Some(PluginInvocationContextLeak::FullPrivatePath)
    } else {
        None
    }
}

fn is_private_path_label(value: &str) -> bool {
    value.starts_with(ROLE_PATH_SEPARATOR) || private_path_markers_match(value)
}

fn private_path_markers_match(value: &str) -> bool {
    [
        HOST_PATH_HOME_MARKER,
        HOST_PATH_TEMP_MARKER,
        WINDOWS_USERS_MARKER,
    ]
    .iter()
    .any(|marker| value.contains(marker))
}

pub(super) fn validate_required_safe_labels(
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, super::super::PluginDescriptorError> {
    assert!(
        values.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "safe label count must fit the adapter field bound"
    );
    if values.is_empty() {
        return Err(super::super::PluginDescriptorError::MissingField {
            field: field.as_str(),
        });
    }
    let mut labels = Collection::with_capacity(values.len());
    for value in values {
        labels.push(validate_required_safe_value(field, value)?);
    }
    Ok(labels)
}

pub(super) fn validate_required_safe_label(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Text, super::super::PluginDescriptorError> {
    match value {
        Some(raw) => validate_required_safe_value(field, raw),
        None => Err(super::super::PluginDescriptorError::MissingField {
            field: field.as_str(),
        }),
    }
}

pub(super) fn validate_optional_safe_label(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<Text>, super::super::PluginDescriptorError> {
    value
        .map(|raw| validate_required_safe_value(field, raw))
        .transpose()
}

fn validate_required_safe_value(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<Text, super::super::PluginDescriptorError> {
    crate::validation::validate_required_adapter_field(field, value).map_err(|reason| {
        super::super::PluginDescriptorError::FieldRejected {
            field: field.as_str(),
            reason,
        }
    })?;
    let label = Text::from(value.trim());
    if !safe_role_label(label.as_str()) {
        return Err(super::super::PluginDescriptorError::UnsafeRoleLabel {
            field: field.as_str(),
            value: label,
        });
    }
    Ok(label)
}

pub(super) fn host_label(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Text, ()> {
    let Some(raw) = value else {
        return Err(());
    };
    crate::validation::validate_required_adapter_field(field, raw).map_err(|_reason| ())?;
    let label = Text::from(raw.trim());
    if safe_role_label(label.as_str()) {
        Ok(label)
    } else {
        Err(())
    }
}

pub(super) fn host_labels(
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, ()> {
    assert!(
        values.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "host capability count must fit the adapter field bound"
    );
    let mut labels = Collection::with_capacity(values.len());
    for value in values {
        labels.push(host_label(field, Some(value))?);
    }
    Ok(labels)
}

fn safe_role_label(value: &str) -> bool {
    !unsafe_role_label(value)
}

fn unsafe_role_label(value: &str) -> bool {
    [
        UNSAFE_PATH_PARENT,
        UNSAFE_SHELL_AND,
        ROLE_PATH_SEPARATOR_TEXT,
        ROLE_WINDOWS_SEPARATOR_TEXT,
        ROLE_SHELL_VARIABLE_TEXT,
        ROLE_SHELL_COMMAND_TEXT,
        ROLE_PIPE_TEXT,
        ROLE_NEWLINE_TEXT,
        ROLE_CARRIAGE_RETURN_TEXT,
    ]
    .iter()
    .any(|marker| value.contains(marker))
}

pub(super) fn parse_role_kind(
    value: &str,
) -> Result<PluginRoleKind, super::super::PluginDescriptorError> {
    match value {
        ROLE_KIND_ACTION => Ok(PluginRoleKind::Action),
        ROLE_KIND_EVENT_HOOK => Ok(PluginRoleKind::EventHook),
        ROLE_KIND_HOST_PANE_ENTRYPOINT => Ok(PluginRoleKind::HostPaneEntrypoint),
        ROLE_KIND_LINK_HANDLER => Ok(PluginRoleKind::LinkHandler),
        unknown => Err(super::super::PluginDescriptorError::UnsupportedRole {
            value: Text::from(unknown),
        }),
    }
}

pub(super) fn target_failure_kind(operation: PluginRegistryOperationKind) -> PluginDiagnosticKind {
    match operation {
        PluginRegistryOperationKind::PlanEvent => PluginDiagnosticKind::UnsupportedEvent,
        PluginRegistryOperationKind::PlanInvocation
        | PluginRegistryOperationKind::PlanPaneEntrypoint => {
            PluginDiagnosticKind::UnknownEntrypoint
        }
        PluginRegistryOperationKind::Link
        | PluginRegistryOperationKind::Unlink
        | PluginRegistryOperationKind::Enable
        | PluginRegistryOperationKind::Disable
        | PluginRegistryOperationKind::List
        | PluginRegistryOperationKind::ActionList => PluginDiagnosticKind::MalformedManifest,
    }
}

pub(super) fn diagnostic(
    kind: PluginDiagnosticKind,
    severity: PluginDiagnosticSeverity,
) -> PluginDiagnostic {
    PluginDiagnostic { kind, severity }
}

pub(super) fn is_known_event(event_name: &str) -> bool {
    KNOWN_EVENT_NAMES.contains(&event_name)
}

pub(super) fn contains_text(values: &[Text], candidate: &str) -> bool {
    values.iter().any(|value| value.as_str() == candidate)
}

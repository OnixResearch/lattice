use super::consts::{
    COMPATIBILITY_CAPABILITY_FIELD, COMPATIBILITY_PLATFORM_FIELD, COMPATIBILITY_VERSION_FIELD,
    HOST_CAPABILITY_FIELD, HOST_PLATFORM_FIELD, HOST_VERSION_FIELD, ROLE_ARTIFACT_LABEL_FIELD,
    ROLE_COMMAND_LABEL_FIELD, ROLE_COMMAND_OR_ARTIFACT_FIELD, ROLE_CONTEXT_FIELD, ROLE_EVENT_FIELD,
    ROLE_ID_FIELD, ROLE_KIND_FIELD, ROLE_LINK_SCHEME_FIELD, ROLE_TITLE_FIELD, VERSION_SEPARATOR,
};
use super::context::{
    contains_text, diagnostic, host_label, host_labels, parse_role_kind,
    validate_optional_safe_label, validate_required_safe_label, validate_required_safe_labels,
};
use super::types::{
    PluginCompatibility, PluginCompatibilityDraft, PluginDiagnostic, PluginDiagnosticKind,
    PluginDiagnosticSeverity, PluginHostCompatibility, PluginHostCompatibilityDraft, PluginRole,
    PluginRoleDraft, PluginRoleKind,
};
use super::{Collection, Text};

const STATIC_COMPATIBILITY_DIAGNOSTIC_CAPACITY: usize = 2;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SemverParts {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Clone, Copy)]
struct HostVersionPair<'a> {
    required: &'a str,
    actual: &'a str,
}

pub(in crate::manifest) fn validate_manifest_compatibility(
    draft: &PluginCompatibilityDraft<'_>,
) -> Result<PluginCompatibility, super::super::PluginDescriptorError> {
    let version = validate_version_label(COMPATIBILITY_VERSION_FIELD, draft.minimum_host_version)?;
    let capabilities = validate_required_safe_labels(
        COMPATIBILITY_CAPABILITY_FIELD,
        draft.required_host_capabilities,
    )?;
    let platforms =
        validate_required_safe_labels(COMPATIBILITY_PLATFORM_FIELD, draft.supported_platforms)?;
    Ok(PluginCompatibility {
        minimum_host_version: version,
        required_host_capabilities: capabilities,
        supported_platforms: platforms,
    })
}

fn validate_version_label(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Text, super::super::PluginDescriptorError> {
    let label = validate_required_safe_label(field, value)?;
    if parse_semver(label.as_str()).is_none() {
        return Err(
            super::super::PluginDescriptorError::InvalidCompatibilityVersion {
                field: field.as_str(),
                value: label,
            },
        );
    }
    Ok(label)
}

fn validate_required_version_value(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<Text, super::super::PluginDescriptorError> {
    let label = validate_required_safe_label(field, Some(value))?;
    if parse_semver(label.as_str()).is_none() {
        return Err(
            super::super::PluginDescriptorError::InvalidCompatibilityVersion {
                field: field.as_str(),
                value: label,
            },
        );
    }
    Ok(label)
}

fn required_version_is_newer(pair: HostVersionPair<'_>) -> bool {
    let Some(required_parts) = parse_semver(pair.required) else {
        return true;
    };
    let Some(actual_parts) = parse_semver(pair.actual) else {
        return true;
    };
    actual_parts < required_parts
}

fn parse_semver(value: &str) -> Option<SemverParts> {
    let mut parts = value.split(VERSION_SEPARATOR);
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    let patch = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(SemverParts {
        major,
        minor,
        patch,
    })
}

pub(in crate::manifest) fn validate_roles(
    drafts: &[PluginRoleDraft<'_>],
    manifest_compatibility: &PluginCompatibility,
) -> Result<Collection<PluginRole>, super::super::PluginDescriptorError> {
    assert!(
        drafts.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "plugin role count must fit the adapter field bound"
    );
    let mut roles: Collection<PluginRole> = Collection::with_capacity(drafts.len());
    for draft in drafts {
        let role = validate_role(draft, manifest_compatibility)?;
        if roles.iter().any(|existing| existing.id() == role.id()) {
            return Err(super::super::PluginDescriptorError::DuplicateRole {
                id: Text::from(role.id()),
            });
        }
        roles.push(role);
    }
    Ok(roles)
}

fn validate_role(
    draft: &PluginRoleDraft<'_>,
    manifest_compatibility: &PluginCompatibility,
) -> Result<PluginRole, super::super::PluginDescriptorError> {
    assert!(
        draft.context_labels.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "role context labels must fit adapter bounds"
    );
    assert!(
        !manifest_compatibility
            .required_host_capabilities()
            .is_empty(),
        "manifest compatibility must be validated before roles"
    );
    let role_kind_label = validate_required_safe_label(ROLE_KIND_FIELD, draft.kind)?;
    let kind = parse_role_kind(&role_kind_label)?;
    let id = validate_required_safe_label(ROLE_ID_FIELD, draft.id)?;
    let title = validate_required_safe_label(ROLE_TITLE_FIELD, draft.title)?;
    let context_labels = validate_required_safe_labels(ROLE_CONTEXT_FIELD, draft.context_labels)?;
    let command_label =
        validate_optional_safe_label(ROLE_COMMAND_LABEL_FIELD, draft.command_label)?;
    let artifact_label =
        validate_optional_safe_label(ROLE_ARTIFACT_LABEL_FIELD, draft.artifact_label)?;
    require_dispatch_label(command_label.as_ref(), artifact_label.as_ref())?;
    let event_name = validate_event_name(kind, draft.event_name)?;
    let link_scheme = validate_optional_safe_label(ROLE_LINK_SCHEME_FIELD, draft.link_scheme)?;
    let compatibility =
        validate_effective_compatibility(&draft.compatibility, manifest_compatibility)?;
    Ok(PluginRole {
        kind,
        id,
        title,
        context_labels,
        command_label,
        artifact_label,
        event_name,
        link_scheme,
        compatibility,
    })
}

fn validate_event_name(
    kind: PluginRoleKind,
    value: Option<&str>,
) -> Result<Option<Text>, super::super::PluginDescriptorError> {
    if matches!(kind, PluginRoleKind::EventHook) && value.is_none() {
        return Err(super::super::PluginDescriptorError::MissingField {
            field: ROLE_EVENT_FIELD.as_str(),
        });
    }
    validate_optional_safe_label(ROLE_EVENT_FIELD, value)
}

fn require_dispatch_label(
    command_label: Option<&Text>,
    artifact_label: Option<&Text>,
) -> Result<(), super::super::PluginDescriptorError> {
    if command_label.is_none() && artifact_label.is_none() {
        return Err(super::super::PluginDescriptorError::MissingField {
            field: ROLE_COMMAND_OR_ARTIFACT_FIELD.as_str(),
        });
    }
    Ok(())
}

fn validate_effective_compatibility(
    draft: &PluginCompatibilityDraft<'_>,
    manifest_compatibility: &PluginCompatibility,
) -> Result<PluginCompatibility, super::super::PluginDescriptorError> {
    assert!(
        !manifest_compatibility.minimum_host_version().is_empty(),
        "manifest compatibility version must be validated before role compatibility"
    );
    assert!(
        !manifest_compatibility.supported_platforms().is_empty(),
        "manifest compatibility platforms must be validated before role compatibility"
    );
    let minimum_host_version = if let Some(version) = draft.minimum_host_version {
        validate_required_version_value(COMPATIBILITY_VERSION_FIELD, version)?
    } else {
        Text::from(manifest_compatibility.minimum_host_version())
    };
    let required_host_capabilities = if draft.required_host_capabilities.is_empty() {
        manifest_compatibility.required_host_capabilities().to_vec()
    } else {
        validate_required_safe_labels(
            COMPATIBILITY_CAPABILITY_FIELD,
            draft.required_host_capabilities,
        )?
    };
    let platform_labels = if draft.supported_platforms.is_empty() {
        manifest_compatibility.supported_platforms().to_vec()
    } else {
        validate_required_safe_labels(COMPATIBILITY_PLATFORM_FIELD, draft.supported_platforms)?
    };
    Ok(PluginCompatibility {
        minimum_host_version,
        required_host_capabilities,
        supported_platforms: platform_labels,
    })
}

pub(super) fn validate_host_compatibility(
    draft: &PluginHostCompatibilityDraft<'_>,
) -> Result<PluginHostCompatibility, ()> {
    let host_version = host_label(HOST_VERSION_FIELD, draft.host_version)?;
    if parse_semver(host_version.as_str()).is_none() {
        return Err(());
    }
    let capabilities = host_labels(HOST_CAPABILITY_FIELD, draft.capabilities)?;
    let platform = host_label(HOST_PLATFORM_FIELD, draft.platform)?;
    Ok(PluginHostCompatibility {
        host_version,
        capabilities,
        platform,
    })
}

pub(super) fn compatibility_diagnostics(
    required: &PluginCompatibility,
    host: &PluginHostCompatibility,
) -> Collection<PluginDiagnostic> {
    assert!(
        !required.minimum_host_version().is_empty(),
        "required compatibility must include a version"
    );
    assert!(
        !host.host_version.is_empty(),
        "host compatibility must include a version"
    );
    let required_capability_count = required.required_host_capabilities().len();
    let reserved_diagnostic_count =
        required_capability_count.saturating_add(STATIC_COMPATIBILITY_DIAGNOSTIC_CAPACITY);
    let mut diagnostics = Collection::with_capacity(reserved_diagnostic_count);
    if required_version_is_newer(HostVersionPair {
        required: required.minimum_host_version(),
        actual: host.host_version.as_str(),
    }) {
        diagnostics.push(diagnostic(
            PluginDiagnosticKind::NewerHostRequired,
            PluginDiagnosticSeverity::Failure,
        ));
    }
    for capability in required.required_host_capabilities() {
        if !contains_text(host.capabilities.as_slice(), capability.as_str()) {
            diagnostics.push(diagnostic(
                PluginDiagnosticKind::MissingHostCapability,
                PluginDiagnosticSeverity::Failure,
            ));
        }
    }
    if !contains_text(required.supported_platforms(), host.platform.as_str()) {
        diagnostics.push(diagnostic(
            PluginDiagnosticKind::UnsupportedPlatform,
            PluginDiagnosticSeverity::Failure,
        ));
    }
    diagnostics
}

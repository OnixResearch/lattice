use super::super::super::*;
use super::super::support::*;

pub(super) fn descriptor_with_drafts(role_drafts: &[PluginRoleDraft<'_>]) -> PluginDescriptor {
    let mut draft = safe_stdio_draft();
    draft.declared_roles = role_drafts;
    PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS).expect("descriptor with roles should validate")
}

pub(super) fn role_with_title(title: Option<&'static str>) -> PluginRoleDraft<'static> {
    PluginRoleDraft {
        kind: Some("action"),
        id: Some(ROLE_ID),
        title,
        context_labels: ROLE_CONTEXTS,
        command_label: Some(COMMAND_LABEL),
        artifact_label: None,
        event_name: None,
        link_scheme: None,
        compatibility: INHERITED_COMPATIBILITY,
    }
}

pub(super) fn role_with_kind(kind: Option<&'static str>) -> PluginRoleDraft<'static> {
    PluginRoleDraft {
        kind,
        ..role_with_title(Some(ROLE_TITLE))
    }
}

pub(super) fn role_with_command(command_label: Option<&'static str>) -> PluginRoleDraft<'static> {
    PluginRoleDraft {
        command_label,
        ..role_with_title(Some(ROLE_TITLE))
    }
}

pub(super) fn role_with_dispatch(
    command_label: Option<&'static str>,
    artifact_label: Option<&'static str>,
) -> PluginRoleDraft<'static> {
    PluginRoleDraft {
        command_label,
        artifact_label,
        ..role_with_title(Some(ROLE_TITLE))
    }
}

pub(super) fn role_with_compatibility(compatibility: PluginCompatibilityDraft<'static>) -> PluginRoleDraft<'static> {
    PluginRoleDraft {
        compatibility,
        ..role_with_title(Some(ROLE_TITLE))
    }
}

pub(super) fn event_role_with_name(event_name: Option<&'static str>) -> PluginRoleDraft<'static> {
    PluginRoleDraft {
        kind: Some("event-hook"),
        id: Some(EVENT_ROLE_ID),
        title: Some(ROLE_TITLE),
        context_labels: ROLE_CONTEXTS,
        command_label: Some(COMMAND_LABEL),
        artifact_label: None,
        event_name,
        link_scheme: None,
        compatibility: INHERITED_COMPATIBILITY,
    }
}

pub(super) fn link_role_with_scheme(link_scheme: Option<&'static str>) -> PluginRoleDraft<'static> {
    PluginRoleDraft {
        kind: Some("link-handler"),
        id: Some(LINK_ROLE_ID),
        title: Some(ROLE_TITLE),
        context_labels: ROLE_CONTEXTS,
        command_label: Some(COMMAND_LABEL),
        artifact_label: None,
        event_name: None,
        link_scheme,
        compatibility: INHERITED_COMPATIBILITY,
    }
}

pub(super) fn safe_context() -> PluginInvocationContext {
    PluginInvocationContext::validate(&safe_context_draft()).expect("safe context should validate")
}

pub(super) fn safe_context_draft() -> PluginInvocationContextDraft<'static> {
    PluginInvocationContextDraft {
        workspace_label: Some(WORKSPACE_LABEL),
        tab_label: Some(TAB_LABEL),
        pane_label: Some(PANE_LABEL),
        agent_status_label: Some(AGENT_STATUS_LABEL),
        worktree_provenance_label: Some(WORKTREE_PROVENANCE_LABEL),
        session_provenance_label: Some(SESSION_PROVENANCE_LABEL),
        request_identity: Some(REQUEST_IDENTITY),
        plugin_root_label: Some(PLUGIN_ROOT_LABEL),
        plugin_config_label: Some(PLUGIN_CONFIG_LABEL),
        plugin_state_label: Some(PLUGIN_STATE_LABEL),
        source_metadata_label: Some(SOURCE_METADATA_LABEL),
    }
}

pub(super) fn host() -> PluginHostCompatibilityDraft<'static> {
    PluginHostCompatibilityDraft {
        host_version: Some(HOST_VERSION),
        capabilities: HOST_CAPABILITIES,
        platform: Some(PLATFORM),
    }
}

#[derive(Clone, Copy)]
pub(super) struct RegistryPolicyState {
    plugin_enabled: bool,
    policy_allows: bool,
}

pub(super) const ENABLED_AND_ALLOWED: RegistryPolicyState = RegistryPolicyState {
    plugin_enabled: true,
    policy_allows: true,
};

pub(super) const DISABLED_BUT_ALLOWED: RegistryPolicyState = RegistryPolicyState {
    plugin_enabled: false,
    policy_allows: true,
};

pub(super) const ENABLED_BUT_DENIED: RegistryPolicyState = RegistryPolicyState {
    plugin_enabled: true,
    policy_allows: false,
};

#[expect(clippy::too_many_arguments, reason = "fixture mirrors registry request fields")]
pub(super) fn registry_request<'a>(
    operation: PluginRegistryOperationKind,
    entrypoint_id: Option<&'a str>,
    event_name: Option<&'a str>,
    state: RegistryPolicyState,
    context: Option<&'a PluginInvocationContext>,
    host: PluginHostCompatibilityDraft<'a>,
) -> PluginRegistryOperationDraft<'a> {
    PluginRegistryOperationDraft {
        operation,
        entrypoint_id,
        event_name,
        plugin_enabled: state.plugin_enabled,
        policy_allows: state.policy_allows,
        context,
        host,
    }
}

pub(super) fn assert_blocked_with(plan: &PluginRegistryPlan, kind: PluginDiagnosticKind) {
    assert_eq!(plan.status(), PluginRegistryPlanStatus::Blocked);
    assert!(
        plan.diagnostics().iter().any(|diagnostic| {
            diagnostic.kind() == kind && diagnostic.severity() == PluginDiagnosticSeverity::Failure
        })
    );
    assert!(plan.command_label().is_none());
}

pub(super) fn assert_warning_with(plan: &PluginRegistryPlan, kind: PluginDiagnosticKind) {
    assert_eq!(plan.status(), PluginRegistryPlanStatus::Warning);
    assert!(
        plan.diagnostics().iter().any(|diagnostic| {
            diagnostic.kind() == kind && diagnostic.severity() == PluginDiagnosticSeverity::Warning
        })
    );
}

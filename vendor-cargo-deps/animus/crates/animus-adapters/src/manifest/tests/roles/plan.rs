use super::super::super::*;
use super::super::support::*;
use super::fixtures::*;

#[test]
fn plans_invocation_and_blocks_incompatible_hosts_without_dispatch() {
    let descriptor = descriptor_with_drafts(DECLARED_ROLES);
    let context = safe_context();
    let plan = descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::PlanInvocation,
        Some(ROLE_ID),
        None,
        ENABLED_AND_ALLOWED,
        Some(&context),
        host(),
    ));

    let newer_roles = [role_with_compatibility(NEWER_COMPATIBILITY)];
    let newer_descriptor = descriptor_with_drafts(&newer_roles);
    let newer_plan = newer_descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::PlanInvocation,
        Some(ROLE_ID),
        None,
        ENABLED_AND_ALLOWED,
        Some(&context),
        host(),
    ));

    let platform_roles = [role_with_compatibility(UNSUPPORTED_PLATFORM_COMPATIBILITY)];
    let platform_descriptor = descriptor_with_drafts(&platform_roles);
    let platform_plan = platform_descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::PlanInvocation,
        Some(ROLE_ID),
        None,
        ENABLED_AND_ALLOWED,
        Some(&context),
        host(),
    ));

    let capability_roles = [role_with_compatibility(MISSING_CAPABILITY_COMPATIBILITY)];
    let capability_descriptor = descriptor_with_drafts(&capability_roles);
    let capability_plan = capability_descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::PlanInvocation,
        Some(ROLE_ID),
        None,
        ENABLED_AND_ALLOWED,
        Some(&context),
        host(),
    ));

    assert_eq!(plan.status(), PluginRegistryPlanStatus::Planned);
    assert_eq!(plan.entrypoint_id(), Some(ROLE_ID));
    assert_eq!(plan.command_label(), Some(COMMAND_LABEL));
    assert_eq!(plan.artifact_label(), None);
    assert!(plan.diagnostics().is_empty());
    assert_blocked_with(&newer_plan, PluginDiagnosticKind::NewerHostRequired);
    assert_blocked_with(&platform_plan, PluginDiagnosticKind::UnsupportedPlatform);
    assert_blocked_with(&capability_plan, PluginDiagnosticKind::MissingHostCapability);
}

#[test]
fn distinguishes_warnings_from_invocation_blockers() {
    let unknown_event_roles = [event_role_with_name(Some(UNKNOWN_EVENT_NAME))];
    let unknown_event_descriptor = descriptor_with_drafts(&unknown_event_roles);
    let link_plan = unknown_event_descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::Link,
        None,
        None,
        ENABLED_AND_ALLOWED,
        None,
        host(),
    ));

    let missing_scheme_roles = [link_role_with_scheme(None)];
    let missing_scheme_descriptor = descriptor_with_drafts(&missing_scheme_roles);
    let missing_scheme_plan = missing_scheme_descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::Link,
        None,
        None,
        ENABLED_AND_ALLOWED,
        None,
        host(),
    ));

    let context = safe_context();
    let future_event_operation_plan =
        descriptor_with_drafts(DECLARED_ROLES).plan_registry_operation(&registry_request(
            PluginRegistryOperationKind::PlanEvent,
            None,
            Some(UNKNOWN_EVENT_NAME),
            ENABLED_AND_ALLOWED,
            Some(&context),
            host(),
        ));

    assert_eq!(link_plan.status(), PluginRegistryPlanStatus::Warning);
    assert_eq!(missing_scheme_plan.status(), PluginRegistryPlanStatus::Warning);
    assert_eq!(future_event_operation_plan.status(), PluginRegistryPlanStatus::Blocked);
    assert_warning_with(&link_plan, PluginDiagnosticKind::UnknownEvent);
    assert_warning_with(&missing_scheme_plan, PluginDiagnosticKind::MissingOptionalMetadata);
    assert_blocked_with(&future_event_operation_plan, PluginDiagnosticKind::UnsupportedEvent);
}

#[test]
fn registry_planning_fails_closed_without_host_side_effects() {
    let descriptor = descriptor_with_drafts(DECLARED_ROLES);
    let context = safe_context();
    let disabled = descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::PlanInvocation,
        Some(ROLE_ID),
        None,
        DISABLED_BUT_ALLOWED,
        Some(&context),
        host(),
    ));
    let unknown = descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::PlanInvocation,
        Some(SECOND_ROLE_ID),
        None,
        ENABLED_AND_ALLOWED,
        Some(&context),
        host(),
    ));
    let denied = descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::PlanInvocation,
        Some(ROLE_ID),
        None,
        ENABLED_BUT_DENIED,
        Some(&context),
        host(),
    ));
    let malformed_context = descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::PlanInvocation,
        Some(ROLE_ID),
        None,
        ENABLED_AND_ALLOWED,
        None,
        host(),
    ));
    let pane = descriptor.plan_registry_operation(&registry_request(
        PluginRegistryOperationKind::PlanPaneEntrypoint,
        Some(PANE_ROLE_ID),
        None,
        ENABLED_AND_ALLOWED,
        Some(&context),
        host(),
    ));

    assert_blocked_with(&disabled, PluginDiagnosticKind::DisabledPlugin);
    assert_blocked_with(&unknown, PluginDiagnosticKind::UnknownEntrypoint);
    assert_blocked_with(&denied, PluginDiagnosticKind::PolicyDenied);
    assert_blocked_with(&malformed_context, PluginDiagnosticKind::MalformedContext);
    assert_eq!(pane.status(), PluginRegistryPlanStatus::Planned);
    assert_eq!(pane.artifact_label(), Some(COMMAND_LABEL));
    assert!(disabled.command_label().is_none());
    assert!(disabled.artifact_label().is_none());
    assert!(PROHIBITED_PLUGIN_MANIFEST_EFFECTS.contains(&"tool execution"));
    assert!(PROHIBITED_PLUGIN_MANIFEST_EFFECTS.contains(&"pane opening"));
    assert!(PROHIBITED_PLUGIN_MANIFEST_EFFECTS.contains(&"registry persistence"));
}

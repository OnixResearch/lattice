use super::super::super::*;
use super::super::support::*;
use super::fixtures::*;

#[test]
fn accepts_role_descriptors_and_effective_compatibility() {
    let descriptor = descriptor_with_drafts(DECLARED_ROLES);

    assert_eq!(descriptor.declared_roles().len(), DECLARED_ROLES.len());
    assert_eq!(descriptor.declared_roles()[0].kind(), PluginRoleKind::Action);
    assert_eq!(descriptor.declared_roles()[0].id(), ROLE_ID);
    assert_eq!(descriptor.declared_roles()[0].title(), ROLE_TITLE);
    assert_eq!(descriptor.declared_roles()[0].context_labels(), &[Text::from(ROLE_CONTEXT)]);
    assert_eq!(descriptor.declared_roles()[1].event_name(), Some(EVENT_NAME));
    assert_eq!(descriptor.declared_roles()[2].artifact_label(), Some(COMMAND_LABEL));
    assert_eq!(descriptor.declared_roles()[3].link_scheme(), Some(LINK_SCHEME));
    assert_eq!(descriptor.compatibility().minimum_host_version(), HOST_VERSION);
    assert_eq!(descriptor.declared_roles()[0].compatibility().supported_platforms(), &[Text::from(PLATFORM)]);
}

#[test]
fn rejects_duplicate_missing_unsafe_and_bad_kind_declarations() {
    let mut duplicate = safe_stdio_draft();
    duplicate.declared_roles = DUPLICATE_ROLES;
    let duplicate_error = PluginDescriptor::validate(&duplicate, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("duplicate role IDs should fail");

    let missing_title_roles = [role_with_title(None)];
    let mut missing_title = safe_stdio_draft();
    missing_title.declared_roles = &missing_title_roles;
    let missing_title_error = PluginDescriptor::validate(&missing_title, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("missing role title should fail");

    let missing_dispatch_roles = [role_with_dispatch(None, None)];
    let mut missing_dispatch = safe_stdio_draft();
    missing_dispatch.declared_roles = &missing_dispatch_roles;
    let missing_dispatch_error = PluginDescriptor::validate(&missing_dispatch, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("missing command or artifact label should fail");

    let unsafe_roles = [role_with_command(Some(UNSAFE_ROLE_LABEL))];
    let mut unsafe_descriptor = safe_stdio_draft();
    unsafe_descriptor.declared_roles = &unsafe_roles;
    let unsafe_error = PluginDescriptor::validate(&unsafe_descriptor, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("unsafe role labels should fail");

    let bad_kind_role_drafts = [role_with_kind(Some(UNKNOWN_ROLE_KIND))];
    let mut bad_kind_descriptor_draft = safe_stdio_draft();
    bad_kind_descriptor_draft.declared_roles = &bad_kind_role_drafts;
    let bad_kind_validation_error =
        PluginDescriptor::validate(&bad_kind_descriptor_draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
            .expect_err("unsupported role kinds should fail");

    assert_eq!(duplicate_error, PluginDescriptorError::DuplicateRole {
        id: Text::from(ROLE_ID),
    });
    assert_eq!(missing_title_error, PluginDescriptorError::MissingField {
        field: "plugin_role_title",
    });
    assert_eq!(missing_dispatch_error, PluginDescriptorError::MissingField {
        field: "plugin_role_command_or_artifact_label",
    });
    assert_eq!(unsafe_error, PluginDescriptorError::UnsafeRoleLabel {
        field: "plugin_role_command_label",
        value: Text::from(UNSAFE_ROLE_LABEL),
    });
    assert_eq!(bad_kind_validation_error, PluginDescriptorError::UnsupportedRole {
        value: Text::from(UNKNOWN_ROLE_KIND),
    });
}

#[test]
fn rejects_invalid_or_missing_manifest_compatibility() {
    let mut invalid_version = safe_stdio_draft();
    invalid_version.compatibility = PluginCompatibilityDraft {
        minimum_host_version: Some(INVALID_VERSION),
        required_host_capabilities: HOST_CAPABILITIES,
        supported_platforms: PLATFORMS,
    };
    let invalid_version_error = PluginDescriptor::validate(&invalid_version, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("invalid host version should fail");

    let mut missing_capabilities = safe_stdio_draft();
    missing_capabilities.compatibility = PluginCompatibilityDraft {
        minimum_host_version: Some(HOST_VERSION),
        required_host_capabilities: &[],
        supported_platforms: PLATFORMS,
    };
    let missing_capability_error = PluginDescriptor::validate(&missing_capabilities, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("missing host capabilities should fail");

    assert_eq!(invalid_version_error, PluginDescriptorError::InvalidCompatibilityVersion {
        field: "plugin_minimum_host_version",
        value: Text::from(INVALID_VERSION),
    });
    assert_eq!(missing_capability_error, PluginDescriptorError::MissingField {
        field: "plugin_required_host_capability",
    });
}

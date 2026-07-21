use super::super::*;
use super::support::*;

#[test]
fn missing_host_binding_keeps_tool_inactive_without_fallback() {
    let manifest = PluginDescriptor::validate(&safe_stdio_draft(), BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect("safe descriptor should validate");
    let definitions = manifest
        .tool_definitions(NO_EXECUTORS)
        .expect("missing executor should not block metadata projection");
    let decision = crate::tool::evaluate_dispatch(crate::tool::DispatchRequest {
        definition: Some(&definitions[0]),
        executor_status: crate::tool::ExecutorStatus::Missing,
        capability_status: crate::tool::CapabilityStatus::Allowed,
        output_boundary_status: crate::tool::OutputBoundaryStatus::Bounded,
    });

    assert_eq!(
        definitions[0].status(),
        crate::tool::DefinitionStatus::Inactive
    );
    assert_eq!(decision.status(), crate::tool::DispatchStatus::Failed);
    assert_eq!(
        decision.failure_class(),
        Some(crate::tool::FailureClass::MissingExecutor)
    );
    assert!(!decision.fallback_attempted());
}

#[test]
fn untrusted_descriptor_projects_inactive_definitions() {
    let mut draft = safe_stdio_draft();
    draft.trust_status = Some(TRUST_STATUS_UNTRUSTED);
    let descriptor = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect("untrusted descriptor should parse as data");
    let definitions = descriptor
        .tool_definitions(EXECUTORS)
        .expect("untrusted descriptor should still project metadata");

    assert_eq!(descriptor.trust_status(), TrustStatus::Untrusted);
    assert_eq!(
        definitions[0].status(),
        crate::tool::DefinitionStatus::Inactive
    );
}

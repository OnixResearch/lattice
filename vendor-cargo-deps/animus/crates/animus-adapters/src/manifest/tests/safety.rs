use super::super::*;
use super::support::*;

#[test]
fn rejects_unsafe_provenance() {
    let mut draft = safe_stdio_draft();
    draft.provenance = Some(UNSAFE_PROVENANCE);
    let error = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("path-like provenance should fail");

    assert_eq!(
        error,
        PluginDescriptorError::UnsafeProvenance {
            value: Text::from(UNSAFE_PROVENANCE),
        }
    );
}

#[test]
fn rejects_path_escape_writable_root() {
    let mut draft = safe_stdio_draft();
    draft.launch_policy.writable_roots = &[PATH_ESCAPE_ROOT];
    let error = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("path escapes should fail");

    assert_eq!(
        error,
        PluginDescriptorError::PathEscape {
            value: Text::from(PATH_ESCAPE_ROOT),
        }
    );
}

#[test]
fn rejects_absolute_writable_root() {
    let mut draft = safe_stdio_draft();
    draft.launch_policy.writable_roots = &[ABSOLUTE_ROOT];
    let error = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("absolute writable roots should fail");

    assert_eq!(
        error,
        PluginDescriptorError::AbsoluteWritableRoot {
            value: Text::from(ABSOLUTE_ROOT),
        }
    );
}

#[test]
fn rejects_environment_value_leakage() {
    let mut draft = safe_stdio_draft();
    draft.launch_policy.environment_allowlist = &[ENVIRONMENT_VALUE];
    let error = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("environment allowlist values should fail");

    assert_eq!(
        error,
        PluginDescriptorError::EnvironmentValueLeak {
            entry: Text::from(ENVIRONMENT_VALUE),
        }
    );
}

#[test]
fn rejects_unsafe_network_without_sandbox() {
    let mut draft = safe_stdio_draft();
    draft.sandbox = SandboxRequirementsDraft {
        mode: Some(SANDBOX_MODE_UNCONFINED),
        network: Some(NETWORK_POLICY_OUTBOUND),
    };
    let error = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("outbound network without isolation should fail");

    assert_eq!(
        error,
        PluginDescriptorError::UnsafeNetworkSandboxCombination {
            network: NetworkPolicy::Outbound,
            sandbox: SandboxMode::Unconfined,
        }
    );
}

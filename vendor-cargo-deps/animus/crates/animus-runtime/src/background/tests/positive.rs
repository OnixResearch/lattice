#[test]
fn valid_profile_resolves_without_side_effects() {
    // r[verify agent_harness_runtime.process_job_profiles.resolution.valid]
    let resolved = super::support::manifest(super::support::valid_profile())
        .resolve(
            super::support::PROFILE_NAME,
            super::support::OWNER_LABEL,
            &super::support::policy(),
        )
        .expect("valid profile should resolve");

    assert_eq!(
        resolved.request.backend,
        super::super::ProcessJobBackend::Pueue
    );
    assert_eq!(
        resolved.request.program.as_deref(),
        Some(super::support::CARGO_PROGRAM)
    );
    assert_eq!(resolved.request.command, None);
    assert_eq!(
        resolved.request.args,
        vec![super::support::CHECK_ARG, super::support::TESTS_ARG]
    );
    assert!(resolved.request.notification.notify_on_complete);
    assert_eq!(resolved.request.env[0].key, super::support::APP_MODE_KEY);
    assert_eq!(
        resolved.request.env[0].value,
        super::support::APP_MODE_VALUE
    );
    assert_eq!(
        resolved.identity.profile_name(),
        super::support::PROFILE_NAME
    );
    assert_eq!(
        resolved.identity.source_label(),
        super::super::INLINE_SOURCE_LABEL
    );
    assert_eq!(
        resolved.identity.selected_backend(),
        super::super::ProcessJobBackend::Pueue
    );
    assert_eq!(
        resolved
            .request
            .metadata
            .get(super::support::PURPOSE_KEY)
            .map(String::as_str),
        Some(super::support::PURPOSE_VALUE),
    );
    assert!(
        !resolved
            .request
            .metadata
            .values()
            .any(|value| value == super::support::APP_MODE_VALUE)
    );
}

#[test]
fn source_precedence_prefers_explicit_then_workspace_then_global() {
    // r[verify agent_harness_runtime.process_job_profiles.precedence.duplicate]
    let sources = [
        super::support::source(
            super::super::ProcessJobProfileSourcePrecedence::Global,
            super::support::GLOBAL_SOURCE,
            super::support::command_profile(),
        ),
        super::support::source(
            super::super::ProcessJobProfileSourcePrecedence::Workspace,
            super::support::WORKSPACE_SOURCE,
            super::support::valid_profile(),
        ),
        super::support::source(
            super::super::ProcessJobProfileSourcePrecedence::Explicit,
            super::support::EXPLICIT_SOURCE,
            super::super::ProcessJobProfile {
                backend: Some(super::super::ProcessJobBackend::Native),
                ..super::support::command_profile()
            },
        ),
    ];
    let resolved =
        super::super::resolve_process_job_profile(super::super::ProcessJobProfileResolveRequest {
            sources: sources.as_slice(),
            profile_name: super::support::PROFILE_NAME,
            owner_label: super::support::OWNER_LABEL,
            policy: &super::support::policy(),
        })
        .expect("explicit source should win");

    assert_eq!(
        resolved.identity.source_label(),
        super::support::EXPLICIT_SOURCE
    );
    assert_eq!(
        resolved.request.backend,
        super::super::ProcessJobBackend::Native
    );
    assert_eq!(
        resolved.request.command.as_deref(),
        Some(super::support::CARGO_CHECK_COMMAND),
    );
    assert_eq!(resolved.request.program, None);
}

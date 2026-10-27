#[test]
fn same_precedence_duplicate_profiles_fail_closed() {
    // r[verify agent_harness_runtime.process_job_profiles.precedence.duplicate]
    let sources = [
        super::support::source(
            super::super::ProcessJobProfileSourcePrecedence::Workspace,
            "workspace-a",
            super::support::command_profile(),
        ),
        super::support::source(
            super::super::ProcessJobProfileSourcePrecedence::Workspace,
            "workspace-b",
            super::support::valid_profile(),
        ),
    ];
    let error =
        super::super::resolve_process_job_profile(super::super::ProcessJobProfileResolveRequest {
            sources: sources.as_slice(),
            profile_name: super::support::PROFILE_NAME,
            owner_label: super::support::OWNER_LABEL,
            policy: &super::support::policy(),
        })
        .expect_err("duplicates at same precedence must fail");

    assert_eq!(
        error.kind(),
        super::super::ProcessJobProfileErrorKind::DuplicateProfile
    );
    assert_eq!(error.field(), super::super::SOURCE_FIELD);
    assert!(error.reason().contains("workspace-a"));
    assert!(error.reason().contains("workspace-b"));
}

#[test]
fn policy_rejects_secret_env_paths_and_unsupported_backend() {
    // r[verify agent_harness_runtime.process_job_profiles.policy.secret_env]
    let cases = [
        (
            super::super::ProcessJobProfile {
                env: vec![super::super::ProcessJobEnvironmentVariable {
                    key: String::from(super::support::APP_TOKEN_KEY),
                    value: String::from("redacted"),
                }],
                ..super::support::command_profile()
            },
            super::super::ProcessJobProfileErrorKind::DisallowedEnvironmentKey,
        ),
        (
            super::super::ProcessJobProfile {
                cwd: Some(String::from(super::support::ESCAPING_PATH)),
                ..super::support::command_profile()
            },
            super::super::ProcessJobProfileErrorKind::DisallowedCwd,
        ),
        (
            super::super::ProcessJobProfile {
                backend: Some(super::super::ProcessJobBackend::Systemd),
                ..super::support::command_profile()
            },
            super::super::ProcessJobProfileErrorKind::DisallowedBackend,
        ),
    ];

    assert_eq!(cases.len(), super::support::SECRET_ENV_POLICY_CASES);
    for (profile, expected_kind) in cases {
        let error = super::support::manifest(profile)
            .resolve(
                super::support::PROFILE_NAME,
                super::support::OWNER_LABEL,
                &super::support::policy(),
            )
            .expect_err("invalid profile should fail before dispatch");
        assert_eq!(error.kind(), expected_kind);
    }
}

#[test]
fn policy_rejects_command_shape_args_writable_path_and_limits() {
    let cases = [
        (
            super::super::ProcessJobProfile {
                writable_paths: vec![String::from(super::support::ESCAPING_PATH)],
                ..super::support::command_profile()
            },
            super::super::ProcessJobProfileErrorKind::DisallowedWritablePath,
        ),
        (
            super::super::ProcessJobProfile {
                command: Some(String::from(super::support::CARGO_CHECK_COMMAND)),
                program: Some(String::from(super::support::CARGO_PROGRAM)),
                ..super::support::command_profile()
            },
            super::super::ProcessJobProfileErrorKind::InvalidCommandShape,
        ),
        (
            super::super::ProcessJobProfile {
                args: vec![String::from(super::support::CHECK_ARG)],
                ..super::support::command_profile()
            },
            super::super::ProcessJobProfileErrorKind::InvalidCommandShape,
        ),
        (
            super::super::ProcessJobProfile {
                resources: super::super::ProcessJobResourceLimits {
                    log_bytes: Some(super::support::TOO_MANY_LOG_BYTES),
                    ..super::super::ProcessJobResourceLimits::default()
                },
                ..super::support::command_profile()
            },
            super::super::ProcessJobProfileErrorKind::ResourceLimitExceeded,
        ),
    ];

    assert!(cases.len() > super::support::SECRET_ENV_POLICY_CASES);
    for (profile, expected_kind) in cases {
        let error = super::support::manifest(profile)
            .resolve(
                super::support::PROFILE_NAME,
                super::support::OWNER_LABEL,
                &super::support::policy(),
            )
            .expect_err("invalid profile should fail before dispatch");
        assert_eq!(error.kind(), expected_kind);
    }
}

#[test]
fn unsupported_schema_and_invalid_policy_fail_before_request() {
    // r[verify agent_harness_runtime.process_job_profiles.nickel.drift]
    let future_manifest = super::super::ProcessJobProfileManifest {
        schema_version: super::support::FUTURE_SCHEMA_VERSION,
        profiles: super::support::manifest(super::support::command_profile()).profiles,
    };
    let schema_error = future_manifest
        .resolve(
            super::support::PROFILE_NAME,
            super::support::OWNER_LABEL,
            &super::support::policy(),
        )
        .expect_err("future schema should fail");
    let bad_policy = super::super::ProcessJobProfilePolicy {
        policy_hash: String::from(super::support::INVALID_HASH),
        ..super::support::policy()
    };
    let policy_error = super::support::manifest(super::support::command_profile())
        .resolve(
            super::support::PROFILE_NAME,
            super::support::OWNER_LABEL,
            &bad_policy,
        )
        .expect_err("invalid policy identity should fail");

    assert_eq!(
        schema_error.kind(),
        super::super::ProcessJobProfileErrorKind::UnsupportedSchemaVersion,
    );
    assert_eq!(
        policy_error.kind(),
        super::super::ProcessJobProfileErrorKind::UnsafeIdentity,
    );
}

#[test]
fn identity_envelope_rejects_leaks_and_excludes_raw_env_values() {
    // r[verify agent_harness_runtime.process_job_profiles.identity.rejects_env]
    let resolved = super::support::manifest(super::support::valid_profile())
        .resolve(
            super::support::PROFILE_NAME,
            super::support::OWNER_LABEL,
            &super::support::policy(),
        )
        .expect("valid profile should resolve");
    let leak_error = super::super::ProcessJobProfileSource::new(
        super::super::ProcessJobProfileSourcePrecedence::Workspace,
        super::support::FULL_PATH_SOURCE,
        super::support::manifest(super::support::command_profile()),
    )
    .expect_err("source labels must not contain full paths");
    let invalid_identity = super::super::envelope::identity_with_source(
        &resolved.identity,
        "systemd://Authorization=token",
    );
    let identity_error = invalid_identity
        .validate()
        .expect_err("credential-looking locators fail");

    assert_eq!(
        resolved.identity.schema_version(),
        super::super::PROCESS_JOB_PROFILE_SCHEMA_VERSION,
    );
    assert!(
        !resolved
            .request
            .metadata
            .values()
            .any(|value| value == super::support::APP_MODE_VALUE)
    );
    assert_eq!(
        leak_error.kind(),
        super::super::ProcessJobProfileErrorKind::UnsafeIdentity,
    );
    assert_eq!(
        identity_error.kind(),
        super::super::ProcessJobProfileErrorKind::UnsafeIdentity,
    );
}

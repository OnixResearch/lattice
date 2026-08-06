pub(crate) mod helpers;

#[derive(Clone, Copy)]
pub(crate) struct IdentityTextInput<'a> {
    pub(crate) field: &'static str,
    pub(crate) value: &'a str,
    pub(crate) profile_name: &'a str,
}

#[derive(Clone, Copy)]
pub(crate) struct HashInput<'a> {
    pub(crate) field: &'static str,
    pub(crate) value: &'a str,
    pub(crate) profile_name: &'a str,
}

#[derive(Clone, Copy)]
struct PathInput<'a> {
    field: &'static str,
    path: &'a str,
    profile_name: &'a str,
    prefixes: &'a [String],
}

pub(crate) fn validate_source_label(label: &str) -> Result<(), super::ProcessJobProfileError> {
    helpers::validate_identity_field(IdentityTextInput {
        field: super::SOURCE_LABEL_FIELD,
        value: label,
        profile_name: super::PROFILE_FIELD,
    })
}

pub(crate) fn validate_profile_label(label: &str) -> Result<(), super::ProcessJobProfileError> {
    helpers::validate_safe_label(helpers::LabelInput {
        field: super::PROFILE_FIELD,
        value: label,
        profile_name: label,
    })
}

pub(crate) fn validate_schema_version(
    profile_name: &str,
    schema_version: u16,
) -> Result<(), super::ProcessJobProfileError> {
    if schema_version == super::PROCESS_JOB_PROFILE_SCHEMA_VERSION {
        return Ok(());
    }
    Err(super::ProcessJobProfileError::new(
        super::ProcessJobProfileErrorKind::UnsupportedSchemaVersion,
        profile_name,
        super::IDENTITY_SCHEMA_VERSION_FIELD,
        format!("unsupported schema version {schema_version}"),
    ))
}

pub(crate) fn validate_policy(
    profile_name: &str,
    policy: &super::ProcessJobProfilePolicy,
) -> Result<(), super::ProcessJobProfileError> {
    debug_assert!(!profile_name.is_empty());
    debug_assert!(!policy.policy_source.is_empty());
    helpers::validate_identity_field(IdentityTextInput {
        field: super::POLICY_FIELD,
        value: policy.policy_source.as_str(),
        profile_name,
    })?;
    helpers::validate_hash_field(HashInput {
        field: super::IDENTITY_POLICY_HASH_FIELD,
        value: policy.policy_hash.as_str(),
        profile_name,
    })?;
    if !policy.allowed_backends.contains(&policy.default_backend) {
        return Err(super::ProcessJobProfileError::new(
            super::ProcessJobProfileErrorKind::DisallowedBackend,
            profile_name,
            super::BACKEND_FIELD,
            "default backend is not allowed by policy",
        ));
    }
    Ok(())
}

pub(crate) fn validate_command_shape(
    profile_name: &str,
    profile: &super::ProcessJobProfile,
) -> Result<(), super::ProcessJobProfileError> {
    debug_assert!(!profile_name.is_empty());
    debug_assert!(profile.command.is_some() || profile.program.is_some() || profile.args.is_empty());
    let has_command = profile.command.as_deref().is_some_and(|value| !value.trim().is_empty());
    let has_program = profile.program.as_deref().is_some_and(|value| !value.trim().is_empty());
    if has_command == has_program {
        return Err(helpers::command_shape_error(helpers::CommandShapeErrorInput {
            profile_name,
            reason: "set exactly one of command or program",
        }));
    }
    if has_command && !profile.args.is_empty() {
        return Err(helpers::command_shape_error(helpers::CommandShapeErrorInput {
            profile_name,
            reason: "args are valid only with program",
        }));
    }
    for arg in &profile.args {
        if arg.trim().is_empty() {
            return Err(super::ProcessJobProfileError::new(
                super::ProcessJobProfileErrorKind::InvalidArgument,
                profile_name,
                super::ARGS_FIELD,
                "arguments must be non-empty",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_backend(
    profile_name: &str,
    backend: super::ProcessJobBackend,
    policy: &super::ProcessJobProfilePolicy,
) -> Result<(), super::ProcessJobProfileError> {
    if policy.allowed_backends.contains(&backend) {
        return Ok(());
    }
    Err(super::ProcessJobProfileError::new(
        super::ProcessJobProfileErrorKind::DisallowedBackend,
        profile_name,
        super::BACKEND_FIELD,
        format!("backend `{}` is not allowed", backend.as_str()),
    ))
}

pub(crate) fn validate_env(
    profile_name: &str,
    env: &[super::ProcessJobEnvironmentVariable],
    policy: &super::ProcessJobProfilePolicy,
) -> Result<(), super::ProcessJobProfileError> {
    debug_assert!(!profile_name.is_empty());
    debug_assert!(env.iter().all(|variable| !variable.key.is_empty()));
    let mut seen = std::collections::BTreeSet::new();
    for variable in env {
        if !helpers::is_allowed_env_key(variable.key.as_str(), policy.allowed_env_prefixes.as_slice()) {
            return Err(helpers::env_error(helpers::EnvErrorInput {
                profile_name,
                key: variable.key.as_str(),
                class: "disallowed",
            }));
        }
        if !seen.insert(variable.key.as_str()) {
            return Err(helpers::env_error(helpers::EnvErrorInput {
                profile_name,
                key: variable.key.as_str(),
                class: "duplicate",
            }));
        }
    }
    Ok(())
}

pub(crate) fn validate_paths(
    profile_name: &str,
    profile: &super::ProcessJobProfile,
    policy: &super::ProcessJobProfilePolicy,
) -> Result<(), super::ProcessJobProfileError> {
    if let Some(cwd) = profile.cwd.as_deref() {
        validate_path(PathInput {
            field: super::CWD_FIELD,
            path: cwd,
            profile_name,
            prefixes: policy.allowed_cwd_prefixes.as_slice(),
        })?;
    }
    for writable_path in &profile.writable_paths {
        validate_path(PathInput {
            field: super::WRITABLE_PATH_FIELD,
            path: writable_path.as_str(),
            profile_name,
            prefixes: policy.allowed_writable_path_prefixes.as_slice(),
        })?;
    }
    Ok(())
}

fn validate_path(input: PathInput<'_>) -> Result<(), super::ProcessJobProfileError> {
    debug_assert!(!input.profile_name.is_empty());
    debug_assert!(!input.field.is_empty());
    let trimmed = input.path.trim();
    if trimmed.is_empty() || trimmed.contains(super::PARENT_PATH_SEGMENT) {
        return Err(helpers::path_error(helpers::PathErrorInput {
            field: input.field,
            profile_name: input.profile_name,
        }));
    }
    if !helpers::path_within_any_prefix(trimmed, input.prefixes) {
        return Err(helpers::path_error(helpers::PathErrorInput {
            field: input.field,
            profile_name: input.profile_name,
        }));
    }
    Ok(())
}

pub(crate) fn validate_resources(
    profile_name: &str,
    resources: &super::ProcessJobResourceLimits,
    policy: &super::ProcessJobProfilePolicy,
) -> Result<(), super::ProcessJobProfileError> {
    debug_assert!(!profile_name.is_empty());
    debug_assert_ne!(policy.max_timeout_seconds, Some(0));
    for (field, actual, maximum) in [
        (
            "timeout_seconds",
            resources.timeout_seconds.map(u64::from),
            policy.max_timeout_seconds.map(u64::from),
        ),
        ("memory_bytes", resources.memory_bytes, policy.max_memory_bytes),
        ("cpu_millis", resources.cpu_millis.map(u64::from), policy.max_cpu_millis.map(u64::from)),
        ("log_bytes", resources.log_bytes, policy.max_log_bytes),
    ] {
        validate_resource_limit(field, actual, maximum, profile_name)?;
    }
    Ok(())
}

fn validate_resource_limit(
    field: &'static str,
    actual: Option<u64>,
    maximum: Option<u64>,
    profile_name: &str,
) -> Result<(), super::ProcessJobProfileError> {
    let Some(actual) = actual else {
        return Ok(());
    };
    if actual == 0 || maximum.is_some_and(|limit| actual > limit) {
        return Err(super::ProcessJobProfileError::new(
            super::ProcessJobProfileErrorKind::ResourceLimitExceeded,
            profile_name,
            super::RESOURCE_FIELD,
            format!("{field} exceeds policy or is zero"),
        ));
    }
    Ok(())
}

pub(crate) fn validate_metadata(
    profile_name: &str,
    metadata: &[super::ProcessJobMetadataField],
) -> Result<(), super::ProcessJobProfileError> {
    debug_assert!(!profile_name.is_empty());
    debug_assert!(metadata.iter().all(|field| !field.key.is_empty()));
    let mut seen = std::collections::BTreeSet::new();
    for field in metadata {
        if !helpers::metadata_key_is_safe(field.key.as_str()) {
            return Err(super::ProcessJobProfileError::new(
                super::ProcessJobProfileErrorKind::UnsafeMetadata,
                profile_name,
                super::METADATA_FIELD,
                format!("metadata key `{}` is not an identity metadata key", field.key),
            ));
        }
        helpers::validate_identity_field(IdentityTextInput {
            field: super::METADATA_FIELD,
            value: field.value.as_str(),
            profile_name,
        })?;
        if !seen.insert(field.key.as_str()) {
            return Err(super::ProcessJobProfileError::new(
                super::ProcessJobProfileErrorKind::UnsafeMetadata,
                profile_name,
                super::METADATA_FIELD,
                format!("duplicate metadata key `{}`", field.key),
            ));
        }
    }
    Ok(())
}

const UNIQUE_SOURCE_COUNT: usize = 1;

#[derive(Clone, Copy)]
struct ResolutionContext<'a> {
    profile_name: &'a str,
    owner_label: &'a str,
    policy: &'a super::ProcessJobProfilePolicy,
    source_label: &'a str,
}

// r[impl agent_harness_runtime.process_job_profiles.resolution]
/// Resolves a named process-job profile from already-loaded sources and policy.
///
/// No filesystem, process, queue, storage, pueue, systemd, shell, clock, or
/// network authority is used by this function.
///
/// # Errors
///
/// Returns [`super::ProcessJobProfileError`] when the profile cannot be selected
/// or violates the supplied policy.
pub fn resolve_process_job_profile(
    request: super::ProcessJobProfileResolveRequest<'_>,
) -> Result<super::ResolvedProcessJobProfile, super::ProcessJobProfileError> {
    let selected = select_profile_source(request.sources, request.profile_name)?;
    resolve_manifest(
        &selected.manifest,
        ResolutionContext {
            profile_name: request.profile_name,
            owner_label: request.owner_label,
            policy: request.policy,
            source_label: selected.safe_label.as_str(),
        },
    )
}

impl super::ProcessJobProfileManifest {
    /// Resolves a named profile using inline source evidence.
    ///
    /// # Errors
    ///
    /// Returns [`super::ProcessJobProfileError`] when the profile is missing or invalid.
    pub fn resolve(
        &self,
        profile_name: &str,
        owner_label: &str,
        policy: &super::ProcessJobProfilePolicy,
    ) -> Result<super::ResolvedProcessJobProfile, super::ProcessJobProfileError> {
        resolve_manifest(
            self,
            ResolutionContext {
                profile_name,
                owner_label,
                policy,
                source_label: super::INLINE_SOURCE_LABEL,
            },
        )
    }
}

fn resolve_manifest(
    manifest: &super::ProcessJobProfileManifest,
    context: ResolutionContext<'_>,
) -> Result<super::ResolvedProcessJobProfile, super::ProcessJobProfileError> {
    super::checks::validate_schema_version(context.profile_name, manifest.schema_version)?;
    let profile = find_unique_profile(manifest, context.profile_name)?;
    resolve_profile(
        &profile.profile,
        ResolutionContext {
            profile_name: profile.name.as_str(),
            ..context
        },
    )
}

fn find_unique_profile<'a>(
    manifest: &'a super::ProcessJobProfileManifest,
    profile_name: &str,
) -> Result<&'a super::NamedProcessJobProfile, super::ProcessJobProfileError> {
    debug_assert!(!profile_name.is_empty());
    debug_assert!(
        manifest
            .profiles
            .iter()
            .all(|profile| !profile.name.is_empty())
    );
    let mut matches = manifest
        .profiles
        .iter()
        .filter(|profile| profile.name == profile_name);
    let Some(first) = matches.next() else {
        return Err(super::ProcessJobProfileError::new(
            super::ProcessJobProfileErrorKind::UnknownProfile,
            profile_name,
            super::PROFILE_FIELD,
            "profile was not present in any selected manifest",
        ));
    };
    if matches.next().is_some() {
        return Err(super::ProcessJobProfileError::new(
            super::ProcessJobProfileErrorKind::DuplicateProfile,
            profile_name,
            super::PROFILE_FIELD,
            "manifest contains duplicate profile names at the same precedence",
        ));
    }
    Ok(first)
}

fn resolve_profile(
    profile: &super::ProcessJobProfile,
    context: ResolutionContext<'_>,
) -> Result<super::ResolvedProcessJobProfile, super::ProcessJobProfileError> {
    debug_assert!(!context.profile_name.is_empty());
    debug_assert!(!context.source_label.is_empty());
    super::checks::validate_profile_label(context.profile_name)?;
    super::checks::validate_profile_label(context.owner_label)?;
    super::checks::helpers::validate_identity_field(super::checks::IdentityTextInput {
        field: super::SOURCE_LABEL_FIELD,
        value: context.source_label,
        profile_name: context.profile_name,
    })?;
    super::checks::validate_policy(context.profile_name, context.policy)?;
    super::checks::validate_command_shape(context.profile_name, profile)?;
    super::checks::validate_env(context.profile_name, profile.env.as_slice(), context.policy)?;
    super::checks::validate_paths(context.profile_name, profile, context.policy)?;
    super::checks::validate_resources(context.profile_name, &profile.resources, context.policy)?;
    super::checks::validate_metadata(context.profile_name, profile.metadata.as_slice())?;
    let backend = profile.backend.unwrap_or(context.policy.default_backend);
    super::checks::validate_backend(context.profile_name, backend, context.policy)?;
    build_resolution(profile, context, backend)
}

fn build_resolution(
    profile: &super::ProcessJobProfile,
    context: ResolutionContext<'_>,
    backend: super::ProcessJobBackend,
) -> Result<super::ResolvedProcessJobProfile, super::ProcessJobProfileError> {
    debug_assert!(!context.profile_name.is_empty());
    debug_assert!(context.policy.allowed_backends.contains(&backend));
    let profile_hash = super::digest::profile_hash(
        super::digest::ProfileHashInput {
            profile_name: context.profile_name,
            source_label: context.source_label,
        },
        profile,
    );
    let mut request = start_request(profile, context, backend, profile_hash.as_str());
    let request_hash = super::digest::request_hash(&request);
    request.metadata.insert(
        String::from(super::IDENTITY_REQUEST_HASH_FIELD),
        request_hash.clone(),
    );
    let identity =
        super::envelope::ProcessJobProfileIdentity::new(super::envelope::ProfileIdentityInput {
            profile_name: context.profile_name,
            source_label: context.source_label,
            policy_hash: context.policy.policy_hash.as_str(),
            selected_backend: backend,
            profile_hash,
            request_hash,
        })?;
    Ok(super::ResolvedProcessJobProfile { request, identity })
}

fn start_request(
    profile: &super::ProcessJobProfile,
    context: ResolutionContext<'_>,
    backend: super::ProcessJobBackend,
    profile_hash: &str,
) -> super::StartProcessJobRequest {
    debug_assert!(!profile_hash.is_empty());
    debug_assert!(!context.owner_label.is_empty());
    let mut request = super::StartProcessJobRequest {
        backend,
        command: trimmed_optional(profile.command.as_deref()),
        program: trimmed_optional(profile.program.as_deref()),
        args: profile
            .args
            .iter()
            .map(|arg| arg.trim().to_owned())
            .collect(),
        cwd: trimmed_optional(profile.cwd.as_deref()),
        writable_paths: profile
            .writable_paths
            .iter()
            .map(|path| path.trim().to_owned())
            .collect(),
        env: profile.env.clone(),
        resources: profile.resources.clone(),
        notification: profile.notification,
        owner_label: String::from(context.owner_label),
        metadata: identity_metadata(context, backend, profile_hash),
    };
    for field in &profile.metadata {
        request
            .metadata
            .insert(field.key.clone(), field.value.clone());
    }
    request
}

// r[impl agent_harness_runtime.process_job_profiles.precedence]
fn select_profile_source<'a>(
    sources: &'a [super::ProcessJobProfileSource],
    profile_name: &str,
) -> Result<&'a super::ProcessJobProfileSource, super::ProcessJobProfileError> {
    debug_assert!(!profile_name.is_empty());
    debug_assert!(sources.iter().all(|source| !source.safe_label.is_empty()));
    super::checks::validate_profile_label(profile_name)?;
    let mut matches: std::collections::BTreeMap<u8, Vec<&super::ProcessJobProfileSource>> =
        std::collections::BTreeMap::new();
    for source in sources {
        if source
            .manifest
            .profiles
            .iter()
            .any(|entry| entry.name == profile_name)
        {
            matches
                .entry(source.precedence.rank())
                .or_default()
                .push(source);
        }
    }
    select_highest_precedence(&matches, profile_name)
}

fn select_highest_precedence<'a>(
    matches: &std::collections::BTreeMap<u8, Vec<&'a super::ProcessJobProfileSource>>,
    profile_name: &str,
) -> Result<&'a super::ProcessJobProfileSource, super::ProcessJobProfileError> {
    debug_assert!(!profile_name.is_empty());
    debug_assert!(matches.values().all(|sources| !sources.is_empty()));
    let Some((_rank, selected_sources)) = matches.iter().next_back() else {
        return Err(super::ProcessJobProfileError::new(
            super::ProcessJobProfileErrorKind::UnknownProfile,
            profile_name,
            super::SOURCE_FIELD,
            "profile was absent from all sources",
        ));
    };
    if selected_sources.len() != UNIQUE_SOURCE_COUNT {
        let labels = selected_sources
            .iter()
            .map(|source| source.safe_label.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(super::ProcessJobProfileError::new(
            super::ProcessJobProfileErrorKind::DuplicateProfile,
            profile_name,
            super::SOURCE_FIELD,
            format!("duplicate profile names at equal precedence: {labels}"),
        ));
    }
    selected_sources.first().copied().ok_or_else(|| {
        super::ProcessJobProfileError::new(
            super::ProcessJobProfileErrorKind::UnknownProfile,
            profile_name,
            super::SOURCE_FIELD,
            "profile source selection was empty",
        )
    })
}

// r[impl agent_harness_runtime.process_job_profiles.identity]
fn identity_metadata(
    context: ResolutionContext<'_>,
    backend: super::ProcessJobBackend,
    profile_hash: &str,
) -> std::collections::BTreeMap<String, String> {
    debug_assert!(!profile_hash.is_empty());
    debug_assert!(!context.policy.policy_hash.is_empty());
    let mut metadata = std::collections::BTreeMap::from([
        (
            String::from(super::IDENTITY_PROFILE_NAME_FIELD),
            String::from(context.profile_name),
        ),
        (
            String::from(super::IDENTITY_SCHEMA_VERSION_FIELD),
            super::PROCESS_JOB_PROFILE_SCHEMA_VERSION.to_string(),
        ),
        (
            String::from(super::IDENTITY_SOURCE_FIELD),
            String::from(context.source_label),
        ),
        (
            String::from(super::IDENTITY_POLICY_HASH_FIELD),
            context.policy.policy_hash.clone(),
        ),
        (
            String::from(super::IDENTITY_PROFILE_HASH_FIELD),
            String::from(profile_hash),
        ),
    ]);
    metadata.insert(
        String::from("identity.profile.backend"),
        String::from(backend.as_str()),
    );
    metadata
}

fn trimmed_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
}

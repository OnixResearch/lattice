pub(crate) fn profile_hash(context: ProfileHashInput<'_>, profile: &super::ProcessJobProfile) -> String {
    debug_assert!(!context.profile_name.is_empty());
    debug_assert!(!context.source_label.is_empty());
    let mut parts = vec![
        String::from(super::PROCESS_JOB_PROFILE_SCHEMA),
        String::from(super::PROFILE_HASH_PREFIX),
        String::from(context.profile_name),
        String::from(super::SOURCE_HASH_PREFIX),
        String::from(context.source_label),
        String::from(profile.backend.map(super::ProcessJobBackend::as_str).unwrap_or_default()),
        profile.command.clone().unwrap_or_default(),
        profile.program.clone().unwrap_or_default(),
        profile.cwd.clone().unwrap_or_default(),
    ];
    parts.extend(profile.args.iter().cloned());
    parts.extend(profile.writable_paths.iter().cloned());
    parts.extend(sorted(env_hashes(profile.env.as_slice())));
    parts.extend(sorted(metadata_hashes(profile.metadata.as_slice())));
    strings(parts.as_slice())
}

pub(crate) fn request_hash(request: &super::StartProcessJobRequest) -> String {
    debug_assert!(!request.owner_label.is_empty());
    debug_assert!(!request.metadata.is_empty());
    let mut parts = vec![
        String::from(super::PROCESS_JOB_PROFILE_SCHEMA),
        String::from(super::REQUEST_HASH_PREFIX),
        String::from(request.backend.as_str()),
        request.command.clone().unwrap_or_default(),
        request.program.clone().unwrap_or_default(),
        request.cwd.clone().unwrap_or_default(),
        request.owner_label.clone(),
    ];
    parts.extend(request.args.iter().cloned());
    parts.extend(request.writable_paths.iter().cloned());
    parts.extend(sorted(env_hashes(request.env.as_slice())));
    for (key, value) in &request.metadata {
        if key != super::IDENTITY_REQUEST_HASH_FIELD {
            parts.push(key.clone());
            parts.push(value.clone());
        }
    }
    strings(parts.as_slice())
}

pub(crate) fn policy_hash(policy: &super::ProcessJobProfilePolicy) -> String {
    debug_assert!(!policy.policy_source.is_empty());
    debug_assert!(policy.allowed_backends.contains(&policy.default_backend));
    let mut parts = vec![
        String::from(super::PROCESS_JOB_PROFILE_SCHEMA),
        String::from(super::POLICY_HASH_PREFIX),
        String::from(policy.default_backend.as_str()),
        policy.policy_source.clone(),
    ];
    for backend in &policy.allowed_backends {
        parts.push(String::from(backend.as_str()));
    }
    parts.extend(policy.allowed_env_prefixes.iter().cloned());
    parts.extend(policy.allowed_cwd_prefixes.iter().cloned());
    parts.extend(policy.allowed_writable_path_prefixes.iter().cloned());
    parts.push(optional_u64_part(policy.max_timeout_seconds.map(u64::from)));
    parts.push(optional_u64_part(policy.max_memory_bytes));
    parts.push(optional_u64_part(policy.max_cpu_millis.map(u64::from)));
    parts.push(optional_u64_part(policy.max_log_bytes));
    strings(parts.as_slice())
}

#[derive(Clone, Copy)]
pub(crate) struct ProfileHashInput<'a> {
    pub(crate) profile_name: &'a str,
    pub(crate) source_label: &'a str,
}

pub(crate) fn hash_parts(parts: &[&str]) -> String {
    let mut builder = Builder::new();
    for part in parts {
        builder.push(part);
    }
    builder.finish()
}

fn strings(parts: &[String]) -> String {
    let mut builder = Builder::new();
    for part in parts {
        builder.push(part.as_str());
    }
    builder.finish()
}

struct Builder {
    normalized: String,
}

impl Builder {
    fn new() -> Self {
        Self {
            normalized: String::new(),
        }
    }

    fn push(&mut self, part: &str) {
        self.normalized.push(super::HASH_PART_SEPARATOR);
        self.normalized.push_str(part.len().to_string().as_str());
        self.normalized.push(super::HASH_LENGTH_SEPARATOR);
        self.normalized.push_str(part);
    }

    fn finish(self) -> String {
        blake3::hash(self.normalized.as_bytes()).to_hex().to_string()
    }
}

fn sorted(mut hashes: Vec<String>) -> Vec<String> {
    hashes.sort();
    hashes
}

fn optional_u64_part(value: Option<u64>) -> String {
    value.map_or_else(|| String::from(super::EMPTY_HASH_MARKER), |value| value.to_string())
}

fn env_hashes(env: &[super::ProcessJobEnvironmentVariable]) -> Vec<String> {
    env.iter()
        .map(|variable| hash_parts(&[super::ENV_HASH_PREFIX, variable.key.as_str(), variable.value.as_str()]))
        .collect()
}

fn metadata_hashes(metadata: &[super::ProcessJobMetadataField]) -> Vec<String> {
    metadata
        .iter()
        .map(|field| hash_parts(&[super::METADATA_HASH_PREFIX, field.key.as_str(), field.value.as_str()]))
        .collect()
}

//! Pure parsing and admission for generated `SecretSpec` policy.

use animus_adapters::secret_resolution::{
    CredentialRequirement, SecretDestinationClass, SecretProviderScheme, SecretResolutionBounds,
    SecretResolutionIssue, SecretResolutionPlan, SecretResolutionRequest,
    SecretResolverAvailability, SecretResolverCapabilities, admit_secret_resolution,
};
use serde::Deserialize;

/// Reviewed `SecretSpec` release.
pub const SECRETSPEC_VERSION: &str = "0.17.0";
/// Reviewed `SecretSpec` tag commit.
pub const SECRETSPEC_COMMIT: &str = "a8794e46ec9664a0e1a3869cc3105d0853937e48";
/// Reviewed `SecretSpec` crates.io checksum.
pub const SECRETSPEC_CRATE_CHECKSUM: &str =
    "68498f9695bb3662c157b8fd4b4665a594f1157de022ff5b0f891af4c7ec75d2";
/// Reviewed bounded-exec commit.
pub const BOUNDED_EXEC_COMMIT: &str = "29dac88ecded94457572db3fdfaaaab95fa91525";
/// Stable resolver implementation identity.
pub const RESOLVER_IDENTITY: &str = "secretspec-v0.17.0-bounded-worker-v1";
const POLICY_SCHEMA_VERSION: u32 = 1;
const REQUIRED_LICENSE: &str = "Apache-2.0";
const REQUIRED_BOUNDED_EXEC_LICENSE: &str = "AGPL-3.0-or-later";
const REQUIRED_FEATURE: &str = "sops";
const FRAME_OVERHEAD_BYTES: usize = 32;
const MAX_INPUT_BYTES: usize = 65_536;
const MAX_VALUE_BYTES: usize = 1_048_576;
const MAX_STDERR_BYTES: usize = 65_536;
const MAX_TIMEOUT_MS: u64 = 60_000;
const MAX_POLL_INTERVAL_MS: u64 = 1_000;
const MAX_TEARDOWN_TIMEOUT_MS: u64 = 10_000;
const REQUIRED_NON_CLAIMS: [&str; 7] = [
    "caller-authority",
    "credential-correctness",
    "credential-freshness",
    "provider-integrity",
    "audit-completeness",
    "production-readiness",
    "release-eligibility",
];

/// Checked generated policy and its exact BLAKE3 identity.
pub struct SecretSpecPolicy {
    identity: String,
    profiles: Vec<ProfilePolicy>,
}

impl SecretSpecPolicy {
    /// Return the BLAKE3 identity of the exact checked JSON bytes.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Find an exact admitted profile selection.
    #[must_use]
    pub fn profile(&self, profile: &str) -> Option<&ProfilePolicy> {
        self.profiles.iter().find(|entry| entry.profile == profile)
    }

    /// Admit one exact request against checked policy.
    ///
    /// # Errors
    ///
    /// Returns a stable issue when policy and request metadata differ.
    pub fn admit(
        &self,
        request: SecretResolutionRequest,
    ) -> Result<SecretResolutionPlan, SecretResolutionIssue> {
        let profile = self
            .profile(&request.profile)
            .ok_or(SecretResolutionIssue::MissingProfile)?;
        if request.resolver_identity != RESOLVER_IDENTITY
            || request.manifest_identity != profile.manifest_identity
            || request.scope != profile.scope
            || request.declaration != profile.declaration
            || request.provider != profile.provider
            || request.purpose != profile.purpose
            || request.destination != profile.destination
            || request.policy_identity != self.identity
            || request.bounds != profile.bounds
        {
            return Err(SecretResolutionIssue::InvalidMetadata);
        }
        let requirement = profile.requirement();
        let capabilities = SecretResolverCapabilities {
            read: true,
            bounded_worker: true,
            providers: vec![
                SecretProviderScheme::Environment,
                SecretProviderScheme::SystemdCredential,
                SecretProviderScheme::Sops,
            ],
        };
        admit_secret_resolution(
            request,
            &requirement,
            &capabilities,
            SecretResolverAvailability::Available,
        )
    }
}

/// One exact profile, scope, declaration, provider, purpose, and bounds tuple.
pub struct ProfilePolicy {
    profile: String,
    scope: String,
    declaration: String,
    manifest_identity: String,
    provider: SecretProviderScheme,
    provider_selector: String,
    purpose: String,
    destination: SecretDestinationClass,
    bounds: SecretResolutionBounds,
}

impl ProfilePolicy {
    /// Exact public profile name.
    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// Exact public scope name.
    #[must_use]
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// Exact public declaration name.
    #[must_use]
    pub fn declaration(&self) -> &str {
        &self.declaration
    }

    /// Expected exact manifest identity.
    #[must_use]
    pub fn manifest_identity(&self) -> &str {
        &self.manifest_identity
    }

    /// Allowed provider family.
    #[must_use]
    pub const fn provider(&self) -> SecretProviderScheme {
        self.provider
    }

    /// Public purpose label.
    #[must_use]
    pub fn purpose(&self) -> &str {
        &self.purpose
    }

    /// Allowed destination class.
    #[must_use]
    pub const fn destination(&self) -> SecretDestinationClass {
        self.destination
    }

    /// Exact worker limits.
    #[must_use]
    pub const fn bounds(&self) -> SecretResolutionBounds {
        self.bounds
    }

    pub(crate) fn provider_selector(&self) -> &str {
        &self.provider_selector
    }

    fn requirement(&self) -> CredentialRequirement {
        CredentialRequirement {
            declaration: self.declaration.clone(),
            scope: self.scope.clone(),
            allowed_providers: vec![self.provider],
            purpose: self.purpose.clone(),
            destination: self.destination,
        }
    }
}

/// Generated policy parsing failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyError {
    /// JSON syntax or shape is invalid.
    InvalidJson,
    /// The policy schema is unsupported.
    UnsupportedSchema,
    /// The dependency cohort differs from the reviewed release.
    CohortMismatch,
    /// The dependency license differs from the reviewed license.
    LicenseMismatch,
    /// The enabled feature set differs from the minimal reviewed set.
    FeatureMismatch,
    /// A profile contains invalid or duplicate metadata.
    InvalidProfile,
    /// A provider selector is not an exact admitted read-only selector.
    InvalidProviderSelector,
    /// Bounds are zero, inconsistent, or cannot contain one framed value.
    InvalidBounds,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPolicy {
    schema_version: u32,
    resolver_kind: String,
    ambient: RawAmbient,
    audit: RawAudit,
    write_operations_allowed: bool,
    non_claims: Vec<String>,
    secretspec: RawCohort,
    bounded_exec: RawBoundedExec,
    profiles: Vec<RawProfile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAmbient {
    inherit_environment: bool,
    manifest_discovery: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAudit {
    enabled: bool,
    authoritative: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCohort {
    version: String,
    commit: String,
    crate_checksum: String,
    license: String,
    features: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBoundedExec {
    commit: String,
    license: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    profile: String,
    scope: String,
    declaration: String,
    manifest_identity: String,
    provider: String,
    provider_selector: String,
    purpose: String,
    destination: String,
    bounds: RawBounds,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBounds {
    value_count_max: usize,
    input_max_bytes: usize,
    value_max_bytes: usize,
    stdout_max_bytes: usize,
    stderr_max_bytes: usize,
    timeout_ms: u64,
    poll_interval_ms: u64,
    teardown_timeout_ms: u64,
}

/// Parse and validate exact generated policy bytes.
///
/// # Errors
///
/// Returns a stable error when JSON, cohort, feature, selector, profile, or
/// execution bounds differ from the reviewed contract.
pub fn parse_policy_json(bytes: &[u8]) -> Result<SecretSpecPolicy, PolicyError> {
    let raw: RawPolicy = serde_json::from_slice(bytes).map_err(|_| PolicyError::InvalidJson)?;
    if raw.schema_version != POLICY_SCHEMA_VERSION {
        return Err(PolicyError::UnsupportedSchema);
    }
    if raw.resolver_kind != RESOLVER_IDENTITY
        || raw.ambient.inherit_environment
        || raw.ambient.manifest_discovery
        || raw.audit.enabled
        || raw.audit.authoritative
        || raw.write_operations_allowed
        || raw.non_claims != REQUIRED_NON_CLAIMS
    {
        return Err(PolicyError::InvalidProfile);
    }
    if raw.secretspec.version != SECRETSPEC_VERSION
        || raw.secretspec.commit != SECRETSPEC_COMMIT
        || raw.secretspec.crate_checksum != SECRETSPEC_CRATE_CHECKSUM
        || raw.bounded_exec.commit != BOUNDED_EXEC_COMMIT
    {
        return Err(PolicyError::CohortMismatch);
    }
    if raw.secretspec.license != REQUIRED_LICENSE
        || raw.bounded_exec.license != REQUIRED_BOUNDED_EXEC_LICENSE
    {
        return Err(PolicyError::LicenseMismatch);
    }
    if raw.secretspec.features != [REQUIRED_FEATURE] {
        return Err(PolicyError::FeatureMismatch);
    }
    if raw.profiles.is_empty() {
        return Err(PolicyError::InvalidProfile);
    }

    let mut profiles = Vec::with_capacity(raw.profiles.len());
    for profile in raw.profiles {
        if profiles
            .iter()
            .any(|entry: &ProfilePolicy| entry.profile == profile.profile)
        {
            return Err(PolicyError::InvalidProfile);
        }
        profiles.push(convert_profile(profile)?);
    }
    Ok(SecretSpecPolicy {
        identity: blake3::hash(bytes).to_hex().to_string(),
        profiles,
    })
}

fn convert_profile(raw: RawProfile) -> Result<ProfilePolicy, PolicyError> {
    if !valid_label(&raw.profile)
        || !valid_label(&raw.scope)
        || !valid_label(&raw.declaration)
        || !valid_label(&raw.purpose)
        || !valid_blake3(&raw.manifest_identity)
    {
        return Err(PolicyError::InvalidProfile);
    }
    let provider = match raw.provider.as_str() {
        "environment" => SecretProviderScheme::Environment,
        "systemd_credential" => SecretProviderScheme::SystemdCredential,
        "sops" => SecretProviderScheme::Sops,
        _ => return Err(PolicyError::InvalidProfile),
    };
    validate_provider_selector(provider, &raw.provider_selector)?;
    let destination = match raw.destination.as_str() {
        "provider_dispatch" => SecretDestinationClass::ProviderDispatch,
        "service_start" => SecretDestinationClass::ServiceStart,
        "continuous_integration" => SecretDestinationClass::ContinuousIntegration,
        _ => return Err(PolicyError::InvalidProfile),
    };
    let bounds = SecretResolutionBounds {
        value_count_max: raw.bounds.value_count_max,
        input_max_bytes: raw.bounds.input_max_bytes,
        value_max_bytes: raw.bounds.value_max_bytes,
        stdout_max_bytes: raw.bounds.stdout_max_bytes,
        stderr_max_bytes: raw.bounds.stderr_max_bytes,
        timeout_ms: raw.bounds.timeout_ms,
        poll_interval_ms: raw.bounds.poll_interval_ms,
        teardown_timeout_ms: raw.bounds.teardown_timeout_ms,
    };
    let required_stdout = bounds
        .value_max_bytes
        .checked_add(FRAME_OVERHEAD_BYTES)
        .ok_or(PolicyError::InvalidBounds)?;
    if bounds.value_count_max != 1
        || bounds.input_max_bytes == 0
        || bounds.input_max_bytes > MAX_INPUT_BYTES
        || bounds.value_max_bytes == 0
        || bounds.value_max_bytes > MAX_VALUE_BYTES
        || bounds.stdout_max_bytes != required_stdout
        || bounds.stderr_max_bytes == 0
        || bounds.stderr_max_bytes > MAX_STDERR_BYTES
        || bounds.timeout_ms == 0
        || bounds.timeout_ms > MAX_TIMEOUT_MS
        || bounds.poll_interval_ms == 0
        || bounds.poll_interval_ms > bounds.timeout_ms
        || bounds.poll_interval_ms > MAX_POLL_INTERVAL_MS
        || bounds.teardown_timeout_ms == 0
        || bounds.teardown_timeout_ms > MAX_TEARDOWN_TIMEOUT_MS
    {
        return Err(PolicyError::InvalidBounds);
    }
    Ok(ProfilePolicy {
        profile: raw.profile,
        scope: raw.scope,
        declaration: raw.declaration,
        manifest_identity: raw.manifest_identity,
        provider,
        provider_selector: raw.provider_selector,
        purpose: raw.purpose,
        destination,
        bounds,
    })
}

fn validate_provider_selector(
    provider: SecretProviderScheme,
    selector: &str,
) -> Result<(), PolicyError> {
    if selector.contains('@') || selector.contains("..") || selector.contains('#') {
        return Err(PolicyError::InvalidProviderSelector);
    }
    let valid = match provider {
        SecretProviderScheme::Environment => selector == "env://",
        SecretProviderScheme::SystemdCredential => selector == "systemd-credential://",
        SecretProviderScheme::Sops => {
            selector.starts_with("sops://")
                && selector.len() > "sops://".len()
                && !selector.starts_with("sops:///")
                && selector
                    .split_once('?')
                    .is_none_or(|(_, query)| matches!(query, "format=json" | "format=yaml"))
        }
    };
    valid
        .then_some(())
        .ok_or(PolicyError::InvalidProviderSelector)
}

fn valid_label(value: &str) -> bool {
    const MAX_LABEL_CHARS: usize = 128;
    !value.is_empty()
        && value.chars().count() <= MAX_LABEL_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_blake3(value: &str) -> bool {
    const BLAKE3_HEX_CHARS: usize = 64;
    value.len() == BLAKE3_HEX_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

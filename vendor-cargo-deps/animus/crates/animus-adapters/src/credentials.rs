//! Authority-free credential-resolution admission contracts.
//!
//! These types contain public metadata only. They do not contain manifest paths,
//! provider credentials, secret values, or value-derived identities.

extern crate alloc;

/// Maximum public identifier length at this boundary.
pub const MAX_SECRET_METADATA_CHARS: usize = 128;
/// Required lowercase BLAKE3 hexadecimal length.
pub const BLAKE3_HEX_CHARS: usize = 64;

/// The selected read-only provider family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretProviderScheme {
    /// Required process-environment lookup.
    Environment,
    /// A systemd credential supplied to the service.
    SystemdCredential,
    /// A SOPS-encrypted document.
    Sops,
}

impl SecretProviderScheme {
    /// Return the stable public provider label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Environment => "environment",
            Self::SystemdCredential => "systemd-credential",
            Self::Sops => "sops",
        }
    }
}

/// Credential operations admitted by the generic boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretResolutionOperation {
    /// Read one declared value.
    Read,
    /// Create or generate a value.
    Create,
    /// Update an existing value.
    Update,
    /// Delete a value.
    Delete,
}

/// Public destination classes for one resolved credential.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretDestinationClass {
    /// Inject the value into one typed provider dispatch.
    ProviderDispatch,
    /// Inject the value into one bounded service start.
    ServiceStart,
    /// Inject the value into one bounded continuous-integration job.
    ContinuousIntegration,
}

/// Named positive limits for one host resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecretResolutionBounds {
    /// Maximum values returned by one resolution.
    pub value_count_max: usize,
    /// Maximum worker input bytes.
    pub input_max_bytes: usize,
    /// Maximum returned secret bytes.
    pub value_max_bytes: usize,
    /// Maximum worker stdout bytes.
    pub stdout_max_bytes: usize,
    /// Maximum worker stderr bytes.
    pub stderr_max_bytes: usize,
    /// Resolution deadline in milliseconds.
    pub timeout_ms: u64,
    /// Child status poll interval in milliseconds.
    pub poll_interval_ms: u64,
    /// Owned-process teardown deadline in milliseconds.
    pub teardown_timeout_ms: u64,
}

impl SecretResolutionBounds {
    const fn are_positive(self) -> bool {
        self.value_count_max > 0
            && self.input_max_bytes > 0
            && self.value_max_bytes > 0
            && self.stdout_max_bytes > 0
            && self.stderr_max_bytes > 0
            && self.timeout_ms > 0
            && self.poll_interval_ms > 0
            && self.teardown_timeout_ms > 0
            && self.poll_interval_ms <= self.timeout_ms
    }
}

/// Public caller requirement for one declared credential.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialRequirement {
    /// Exact declaration name.
    pub declaration: alloc::string::String,
    /// Exact scope that contains the declaration.
    pub scope: alloc::string::String,
    /// Allowed read-only providers.
    pub allowed_providers: alloc::vec::Vec<SecretProviderScheme>,
    /// Bounded public purpose label.
    pub purpose: alloc::string::String,
    /// Allowed destination class.
    pub destination: SecretDestinationClass,
}

/// Public request to resolve one credential through an injected service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretResolutionRequest {
    /// Stable resolver implementation identity.
    pub resolver_identity: alloc::string::String,
    /// BLAKE3 identity of the exact manifest bytes.
    pub manifest_identity: alloc::string::String,
    /// Explicit `SecretSpec` profile.
    pub profile: alloc::string::String,
    /// Explicit `SecretSpec` scope.
    pub scope: alloc::string::String,
    /// Exact declaration name.
    pub declaration: alloc::string::String,
    /// Explicit provider family.
    pub provider: SecretProviderScheme,
    /// Public purpose label.
    pub purpose: alloc::string::String,
    /// Destination class.
    pub destination: SecretDestinationClass,
    /// Requested operation.
    pub operation: SecretResolutionOperation,
    /// Whether provider fallback was requested.
    pub provider_fallback: bool,
    /// BLAKE3 identity of the checked policy bytes.
    pub policy_identity: alloc::string::String,
    /// Explicit worker limits.
    pub bounds: SecretResolutionBounds,
}

/// Capabilities of the injected resolver service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretResolverCapabilities {
    /// Whether read resolution is supported.
    pub read: bool,
    /// Whether an owned bounded worker is available.
    pub bounded_worker: bool,
    /// Provider families supported by this service.
    pub providers: alloc::vec::Vec<SecretProviderScheme>,
}

/// Availability of the injected resolver service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretResolverAvailability {
    /// The service is available for dispatch.
    Available,
    /// No service was injected.
    Missing,
    /// Policy disabled the service.
    Disabled,
}

/// Validated metadata-only plan for an imperative host shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretResolutionPlan {
    request: SecretResolutionRequest,
}

impl SecretResolutionPlan {
    /// Return the admitted request metadata.
    #[must_use]
    pub const fn request(&self) -> &SecretResolutionRequest {
        &self.request
    }
}

/// Stable fail-closed admission categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretResolutionIssue {
    /// A required public field is empty or malformed.
    InvalidMetadata,
    /// A BLAKE3 public identity is malformed.
    InvalidIdentity,
    /// An implicit profile was requested.
    MissingProfile,
    /// The request names an undeclared scope.
    UndeclaredScope,
    /// The request names an unknown declaration.
    UnknownDeclaration,
    /// The selected provider is not allowed.
    ProviderDenied,
    /// Provider fallback is not admitted.
    ProviderFallbackDenied,
    /// Only read operations are admitted.
    WriteDenied,
    /// The purpose does not match the requirement.
    PurposeMismatch,
    /// The destination does not match the requirement.
    DestinationMismatch,
    /// One or more execution limits are not positive.
    InvalidBounds,
    /// The injected service is missing or disabled.
    ResolverUnavailable,
    /// The service lacks a required capability.
    CapabilityMissing,
}

/// Admit one metadata-only read plan without provider access.
///
/// # Errors
///
/// Returns a stable issue before host dispatch when any explicit selection or
/// capability does not match the caller requirement.
pub fn admit_secret_resolution(
    request: SecretResolutionRequest,
    requirement: &CredentialRequirement,
    capabilities: &SecretResolverCapabilities,
    availability: SecretResolverAvailability,
) -> Result<SecretResolutionPlan, SecretResolutionIssue> {
    if availability != SecretResolverAvailability::Available {
        return Err(SecretResolutionIssue::ResolverUnavailable);
    }
    if request.profile.is_empty() {
        return Err(SecretResolutionIssue::MissingProfile);
    }
    let metadata = [
        request.resolver_identity.as_str(),
        request.profile.as_str(),
        request.scope.as_str(),
        request.declaration.as_str(),
        request.purpose.as_str(),
    ];
    if metadata.iter().any(|value| !valid_label(value)) {
        return Err(SecretResolutionIssue::InvalidMetadata);
    }
    if !valid_blake3(&request.manifest_identity) || !valid_blake3(&request.policy_identity) {
        return Err(SecretResolutionIssue::InvalidIdentity);
    }
    if request.scope != requirement.scope {
        return Err(SecretResolutionIssue::UndeclaredScope);
    }
    if request.declaration != requirement.declaration {
        return Err(SecretResolutionIssue::UnknownDeclaration);
    }
    if !requirement.allowed_providers.contains(&request.provider) {
        return Err(SecretResolutionIssue::ProviderDenied);
    }
    if request.provider_fallback {
        return Err(SecretResolutionIssue::ProviderFallbackDenied);
    }
    if request.operation != SecretResolutionOperation::Read {
        return Err(SecretResolutionIssue::WriteDenied);
    }
    if request.purpose != requirement.purpose {
        return Err(SecretResolutionIssue::PurposeMismatch);
    }
    if request.destination != requirement.destination {
        return Err(SecretResolutionIssue::DestinationMismatch);
    }
    if !request.bounds.are_positive() {
        return Err(SecretResolutionIssue::InvalidBounds);
    }
    if !capabilities.read || !capabilities.bounded_worker {
        return Err(SecretResolutionIssue::CapabilityMissing);
    }
    if !capabilities.providers.contains(&request.provider) {
        return Err(SecretResolutionIssue::CapabilityMissing);
    }
    debug_assert!(request.bounds.are_positive());
    debug_assert_eq!(request.operation, SecretResolutionOperation::Read);
    Ok(SecretResolutionPlan { request })
}

fn valid_label(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_SECRET_METADATA_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Injected host service for one admitted secret-resolution plan.
pub trait SecretResolutionService {
    /// Opaque material type owned by the host implementation.
    type Material;

    /// Resolve one admitted plan.
    ///
    /// # Errors
    ///
    /// Returns a redacted, implementation-owned failure.
    fn resolve(
        &mut self,
        plan: &SecretResolutionPlan,
    ) -> Result<Self::Material, SecretResolutionIssue>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    const IDENTITY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const VALUE_MAX_BYTES: usize = 1_024;
    const FRAME_OVERHEAD_BYTES: usize = 32;
    const INPUT_MAX_BYTES: usize = 4_096;
    const STDERR_MAX_BYTES: usize = 1_024;
    const TIMEOUT_MS: u64 = 5_000;
    const POLL_INTERVAL_MS: u64 = 10;
    const TEARDOWN_TIMEOUT_MS: u64 = 1_000;
    const STDOUT_MAX_BYTES: usize = VALUE_MAX_BYTES.saturating_add(FRAME_OVERHEAD_BYTES);
    const NEGATIVE_CASE_COUNT: usize = 7;

    fn requirement() -> CredentialRequirement {
        CredentialRequirement {
            declaration: "API_KEY".to_string(),
            scope: "provider_api".to_string(),
            allowed_providers: alloc::vec![SecretProviderScheme::Environment],
            purpose: "provider_authentication".to_string(),
            destination: SecretDestinationClass::ProviderDispatch,
        }
    }

    fn request() -> SecretResolutionRequest {
        SecretResolutionRequest {
            resolver_identity: "secretspec-v0.17.0".to_string(),
            manifest_identity: IDENTITY.to_string(),
            profile: "development".to_string(),
            scope: "provider_api".to_string(),
            declaration: "API_KEY".to_string(),
            provider: SecretProviderScheme::Environment,
            purpose: "provider_authentication".to_string(),
            destination: SecretDestinationClass::ProviderDispatch,
            operation: SecretResolutionOperation::Read,
            provider_fallback: false,
            policy_identity: IDENTITY.to_string(),
            bounds: SecretResolutionBounds {
                value_count_max: 1,
                input_max_bytes: INPUT_MAX_BYTES,
                value_max_bytes: VALUE_MAX_BYTES,
                stdout_max_bytes: STDOUT_MAX_BYTES,
                stderr_max_bytes: STDERR_MAX_BYTES,
                timeout_ms: TIMEOUT_MS,
                poll_interval_ms: POLL_INTERVAL_MS,
                teardown_timeout_ms: TEARDOWN_TIMEOUT_MS,
            },
        }
    }

    fn capabilities() -> SecretResolverCapabilities {
        SecretResolverCapabilities {
            read: true,
            bounded_worker: true,
            providers: alloc::vec![SecretProviderScheme::Environment],
        }
    }

    #[test]
    fn explicit_read_is_admitted() {
        let admitted = admit_secret_resolution(
            request(),
            &requirement(),
            &capabilities(),
            SecretResolverAvailability::Available,
        )
        .expect("explicit read should pass");
        assert_eq!(admitted.request().declaration, "API_KEY");
    }

    #[test]
    fn implicit_unknown_fallback_write_and_bad_bounds_fail_closed() {
        let cases = [
            ("profile", SecretResolutionIssue::MissingProfile),
            ("scope", SecretResolutionIssue::UndeclaredScope),
            ("declaration", SecretResolutionIssue::UnknownDeclaration),
            ("provider", SecretResolutionIssue::ProviderDenied),
            ("fallback", SecretResolutionIssue::ProviderFallbackDenied),
            ("write", SecretResolutionIssue::WriteDenied),
            ("bounds", SecretResolutionIssue::InvalidBounds),
        ];
        assert_eq!(cases.len(), NEGATIVE_CASE_COUNT);
        for (case, expected) in cases {
            let mut candidate = request();
            match case {
                "profile" => candidate.profile.clear(),
                "scope" => candidate.scope = "other".to_string(),
                "declaration" => candidate.declaration = "OTHER".to_string(),
                "provider" => candidate.provider = SecretProviderScheme::Sops,
                "fallback" => candidate.provider_fallback = true,
                "write" => candidate.operation = SecretResolutionOperation::Update,
                "bounds" => candidate.bounds.timeout_ms = 0,
                _ => unreachable!("closed fixture set"),
            }
            assert_eq!(
                admit_secret_resolution(
                    candidate,
                    &requirement(),
                    &capabilities(),
                    SecretResolverAvailability::Available,
                ),
                Err(expected),
                "case {case}"
            );
        }
    }

    #[test]
    fn public_metadata_rejects_secret_shaped_or_credential_shaped_fields() {
        let mut secret_value = request();
        secret_value.purpose = "token with spaces".to_string();
        assert_eq!(
            admit_secret_resolution(
                secret_value,
                &requirement(),
                &capabilities(),
                SecretResolverAvailability::Available,
            ),
            Err(SecretResolutionIssue::InvalidMetadata)
        );

        let mut credential_uri = request();
        credential_uri.resolver_identity = "sops://user:credential@host".to_string();
        assert_eq!(
            admit_secret_resolution(
                credential_uri,
                &requirement(),
                &capabilities(),
                SecretResolverAvailability::Available,
            ),
            Err(SecretResolutionIssue::InvalidMetadata)
        );
    }
}

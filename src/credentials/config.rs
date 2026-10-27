//! Always-compiled, authority-free SecretSpec provider and host configuration.
//! No SDK, provider input, filesystem or process access is required for validation.

use std::collections::HashSet;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
pub const REVIEWED_COHORT: &str = "0.17.0";
const MAX_METADATA_BYTES: usize = 128;
const BLAKE3_HEX_BYTES: usize = 64;
const MAX_INPUT_BYTES: usize = 65_536;
const MAX_VALUE_BYTES: usize = 1_048_576;
const MAX_STDERR_BYTES: usize = 65_536;
const FRAME_OVERHEAD_BYTES: usize = 32;
const MAX_TIMEOUT_MS: u64 = 60_000;
const MAX_POLL_INTERVAL_MS: u64 = 1_000;
const MAX_TEARDOWN_TIMEOUT_MS: u64 = 10_000;
const MAX_DECLARATIONS: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderScheme {
    Environment,
    SystemdCredential,
    Sops,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclarationMapping {
    pub key: String,
    pub field: Option<String>,
    pub declaration: String,
    pub scheme: ProviderScheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestBounds {
    pub input_max_bytes: usize,
    pub value_max_bytes: usize,
    pub stdout_max_bytes: usize,
    pub stderr_max_bytes: usize,
    pub timeout_ms: u64,
    pub poll_interval_ms: u64,
    pub teardown_timeout_ms: u64,
}
// r[impl lattice.credentials.animus_secret_providers.config]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretSpecProviderConfig {
    pub id: String,
    pub cohort: String,
    pub manifest_identity: String,
    pub policy_identity: String,
    pub profile: String,
    pub scope: String,
    pub declarations: Vec<DeclarationMapping>,
    pub allowed_schemes: Vec<ProviderScheme>,
    pub reason: String,
    pub bounds: RequestBounds,
}

/// Fixed diagnostic vocabulary. No provider-controlled text crosses this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretSpecDiagnostic {
    InvalidConfiguration,
    UnsupportedCohort,
    MissingDeclaration,
    ProviderDenied,
    ContextDenied,
    ReasonRequired,
    CapabilityMissing,
    Unavailable,
    ManifestRejected,
    ProviderFailed,
    TimedOut,
    Cancelled,
    ValueRejected,
    CleanupUncertain,
}

impl SecretSpecDiagnostic {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidConfiguration => "credential_secretspec_invalid_configuration",
            Self::UnsupportedCohort => "credential_secretspec_unsupported_cohort",
            Self::MissingDeclaration => "credential_secretspec_missing_declaration",
            Self::ProviderDenied => "credential_secretspec_provider_denied",
            Self::ContextDenied => "credential_secretspec_context_denied",
            Self::ReasonRequired => "credential_secretspec_reason_required",
            Self::CapabilityMissing => "credential_secretspec_capability_missing",
            Self::Unavailable => "credential_secretspec_unavailable",
            Self::ManifestRejected => "credential_secretspec_manifest_rejected",
            Self::ProviderFailed => "credential_secretspec_provider_failed",
            Self::TimedOut => "credential_secretspec_timed_out",
            Self::Cancelled => "credential_secretspec_cancelled",
            Self::ValueRejected => "credential_secretspec_value_rejected",
            Self::CleanupUncertain => "credential_secretspec_cleanup_uncertain",
        }
    }
}

impl std::fmt::Display for SecretSpecDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}
impl std::error::Error for SecretSpecDiagnostic {}

impl SecretSpecProviderConfig {
    pub fn validate(&self) -> Result<(), SecretSpecDiagnostic> {
        if self.cohort != REVIEWED_COHORT {
            return Err(SecretSpecDiagnostic::UnsupportedCohort);
        }
        if ![&self.id, &self.profile, &self.scope, &self.reason].into_iter().all(|s| safe_label(s))
            || !safe_identity(&self.manifest_identity)
            || !safe_identity(&self.policy_identity)
            || !self.bounds.validate()
            || self.declarations.is_empty()
            || self.declarations.len() > MAX_DECLARATIONS
            || self.allowed_schemes.is_empty()
        {
            return Err(SecretSpecDiagnostic::InvalidConfiguration);
        }
        let mut schemes = HashSet::new();
        if self.allowed_schemes.iter().any(|scheme| !schemes.insert(*scheme)) {
            return Err(SecretSpecDiagnostic::InvalidConfiguration);
        }
        let mut keys = HashSet::new();
        for entry in &self.declarations {
            if !safe_label(&entry.key)
                || !safe_label(&entry.declaration)
                || entry.field.as_deref().is_some_and(|field| !safe_label(field))
                || !keys.insert((&entry.key, &entry.field))
            {
                return Err(SecretSpecDiagnostic::InvalidConfiguration);
            }
            if !self.allowed_schemes.contains(&entry.scheme) {
                return Err(SecretSpecDiagnostic::ProviderDenied);
            }
        }
        Ok(())
    }
}

pub(super) fn safe_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_METADATA_BYTES
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
fn safe_identity(value: &str) -> bool {
    value.len() == BLAKE3_HEX_BYTES && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
/// Trusted operator deployment settings, separate from public provider metadata.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretSpecHostConfig {
    pub provider_id: String,
    pub policy_path: PathBuf,
    pub manifest_path: PathBuf,
    pub worker_executable: PathBuf,
    pub working_directory: PathBuf,
    pub input: HostInputConfig,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum HostInputConfig {
    Environment {
        variable: String,
    },
    SystemdCredential {
        directory: PathBuf,
    },
    Sops {
        guard_directory: PathBuf,
        executable: PathBuf,
        credential_variables: Vec<SopsCredentialVariable>,
    },
}

impl std::fmt::Debug for SecretSpecHostConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretSpecHostConfig(<private deployment>)")
    }
}

impl std::fmt::Debug for HostInputConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HostInputConfig(<private deployment>)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SopsCredentialVariable {
    pub credential: SopsCredentialKind,
    pub variable: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SopsCredentialKind {
    AgeKey,
    AwsSecretAccessKey,
    AzureClientSecret,
    GoogleOauthAccessToken,
    VaultToken,
}

pub(super) fn absolute_coordinate(path: &std::path::Path) -> bool {
    path.is_absolute() && !path.components().any(|component| matches!(component, std::path::Component::ParentDir))
}

impl SecretSpecHostConfig {
    /// Pure shape validation; does not open a manifest, policy or provider input.
    pub fn validate(&self) -> Result<(), SecretSpecDiagnostic> {
        let valid_coordinates = [
            &self.policy_path,
            &self.manifest_path,
            &self.worker_executable,
            &self.working_directory,
        ]
        .into_iter()
        .all(|path| absolute_coordinate(path));
        let valid_source = match &self.input {
            HostInputConfig::Environment { variable } => safe_label(variable),
            HostInputConfig::SystemdCredential { directory } => absolute_coordinate(directory),
            HostInputConfig::Sops {
                guard_directory,
                executable,
                credential_variables,
            } => {
                let mut seen = HashSet::new();
                absolute_coordinate(guard_directory)
                    && absolute_coordinate(executable)
                    && credential_variables
                        .iter()
                        .all(|entry| safe_label(&entry.variable) && seen.insert(entry.credential))
            }
        };
        if !safe_label(&self.provider_id) || !valid_coordinates || !valid_source {
            return Err(SecretSpecDiagnostic::InvalidConfiguration);
        }
        Ok(())
    }
}

impl RequestBounds {
    fn validate(self) -> bool {
        (1..=MAX_INPUT_BYTES).contains(&self.input_max_bytes)
            && (1..=MAX_VALUE_BYTES).contains(&self.value_max_bytes)
            && self.stdout_max_bytes == self.value_max_bytes.saturating_add(FRAME_OVERHEAD_BYTES)
            && self.stdout_max_bytes <= MAX_VALUE_BYTES + FRAME_OVERHEAD_BYTES
            && (1..=MAX_STDERR_BYTES).contains(&self.stderr_max_bytes)
            && (1..=MAX_TIMEOUT_MS).contains(&self.timeout_ms)
            && (1..=MAX_POLL_INTERVAL_MS).contains(&self.poll_interval_ms)
            && self.poll_interval_ms <= self.timeout_ms
            && (1..=MAX_TEARDOWN_TIMEOUT_MS).contains(&self.teardown_timeout_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL: &str = "hostile-secret-provider-value-not-public";

    fn config(scheme: ProviderScheme) -> SecretSpecProviderConfig {
        SecretSpecProviderConfig {
            id: "reviewed-secret".into(),
            cohort: REVIEWED_COHORT.into(),
            manifest_identity: "0123456789abcdef".repeat(4),
            policy_identity: "0123456789abcdef".repeat(4),
            profile: "service".into(),
            scope: "workflow".into(),
            reason: "workflow-authentication".into(),
            declarations: vec![DeclarationMapping {
                key: "token".into(),
                field: None,
                declaration: "API_TOKEN".into(),
                scheme,
            }],
            allowed_schemes: vec![scheme],
            bounds: RequestBounds {
                input_max_bytes: 4096,
                value_max_bytes: 1024,
                stdout_max_bytes: 1056,
                stderr_max_bytes: 1024,
                timeout_ms: 5000,
                poll_interval_ms: 10,
                teardown_timeout_ms: 1000,
            },
        }
    }
    // r[verify lattice.credentials.animus_secret_providers.config.scenario.valid]
    // r[verify lattice.credentials.animus_secret_providers.config.scenario.invalid]
    #[test]
    fn closed_configuration_rejects_unsafe_metadata_ambiguity_cohorts_bounds_and_plaintext() {
        for scheme in [
            ProviderScheme::Environment,
            ProviderScheme::SystemdCredential,
            ProviderScheme::Sops,
        ] {
            let cfg = config(scheme);
            assert!(cfg.validate().is_ok());
            let serialized = serde_json::to_value(&cfg).unwrap();
            for field in [
                "plaintext",
                "password",
                "manifest_path",
                "worker_executable",
                "fallback",
            ] {
                let mut unsafe_shape = serialized.clone();
                unsafe_shape[field] = SENTINEL.into();
                assert!(serde_json::from_value::<SecretSpecProviderConfig>(unsafe_shape).is_err());
            }
            let mut invalid = Vec::new();
            let mut candidate = cfg.clone();
            candidate.profile.clear();
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.scope.clear();
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.reason.clear();
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.id = "unsafe\nmetadata".into();
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.manifest_identity = "not-a-blake3".into();
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.declarations.push(candidate.declarations[0].clone());
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.allowed_schemes.clear();
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.allowed_schemes.push(scheme);
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.cohort = "0.18.0".into();
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.bounds.timeout_ms = 0;
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.bounds.timeout_ms = 60001;
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.bounds.value_max_bytes = 0;
            invalid.push(candidate);
            let mut candidate = cfg.clone();
            candidate.bounds.stdout_max_bytes = 1024;
            invalid.push(candidate);
            let mut candidate = cfg;
            candidate.bounds.value_max_bytes = 1048577;
            invalid.push(candidate);
            for candidate in invalid {
                assert!(candidate.validate().is_err());
            }
        }
    }
}

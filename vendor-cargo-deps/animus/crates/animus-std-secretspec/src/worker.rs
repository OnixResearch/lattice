//! SecretSpec SDK worker core.

use crate::protocol::{ProtocolError, WorkerRequest, decode_request, encode_secret};
use animus_adapters::secret_resolution::BLAKE3_HEX_CHARS;
use secretspec::{RESOLVE_SCHEMA_VERSION, ResolvedSource, Secrets};
use zeroize::Zeroizing;

/// Worker success exit code.
pub const WORKER_EXIT_SUCCESS: i32 = 0;
/// Invalid private request exit code.
pub const WORKER_EXIT_INVALID_REQUEST: i32 = 64;
/// Manifest load or identity rejection exit code.
pub const WORKER_EXIT_MANIFEST_REJECTED: i32 = 65;
/// Provider process nonzero failure exit code.
pub const WORKER_EXIT_PROVIDER_FAILED: i32 = 66;
/// Required declaration was not resolved exit code.
pub const WORKER_EXIT_DECLARATION_MISSING: i32 = 67;
/// Secret or process output exceeded its bound exit code.
pub const WORKER_EXIT_OUTPUT_LIMIT: i32 = 68;
/// Required provider executable is absent exit code.
pub const WORKER_EXIT_PROVIDER_EXECUTABLE_MISSING: i32 = 69;
/// Provider returned malformed data exit code.
pub const WORKER_EXIT_PROVIDER_OUTPUT_MALFORMED: i32 = 70;
/// Unexpected worker failure exit code.
pub const WORKER_EXIT_INTERNAL: i32 = 71;

/// Private worker failure. Its exit code is the only parent-visible detail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerFailure {
    /// Request shape or explicit selection is invalid.
    InvalidRequest,
    /// Manifest access, parsing, or identity failed.
    ManifestRejected,
    /// Provider execution failed.
    ProviderFailed,
    /// Required declaration was absent.
    DeclarationMissing,
    /// A returned value exceeded its bound.
    OutputLimit,
    /// The required provider executable was absent.
    ProviderExecutableMissing,
    /// Provider output could not be parsed.
    ProviderOutputMalformed,
    /// The private frame could not be encoded.
    Internal,
}

impl WorkerFailure {
    /// Convert this internal failure to one stable process exit code.
    #[must_use]
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::InvalidRequest => WORKER_EXIT_INVALID_REQUEST,
            Self::ManifestRejected => WORKER_EXIT_MANIFEST_REJECTED,
            Self::ProviderFailed => WORKER_EXIT_PROVIDER_FAILED,
            Self::DeclarationMissing => WORKER_EXIT_DECLARATION_MISSING,
            Self::OutputLimit => WORKER_EXIT_OUTPUT_LIMIT,
            Self::ProviderExecutableMissing => WORKER_EXIT_PROVIDER_EXECUTABLE_MISSING,
            Self::ProviderOutputMalformed => WORKER_EXIT_PROVIDER_OUTPUT_MALFORMED,
            Self::Internal => WORKER_EXIT_INTERNAL,
        }
    }
}

/// Execute the private worker protocol without printing diagnostics.
///
/// # Errors
///
/// Returns a stable private failure for conversion to a process exit code.
#[doc(hidden)]
pub fn run_worker_bytes(input: &[u8]) -> Result<Zeroizing<Vec<u8>>, WorkerFailure> {
    let request = decode_request(input).map_err(|_| WorkerFailure::InvalidRequest)?;
    resolve_request(&request)
}

/// Resolve one exact declaration through the pinned `SecretSpec` SDK.
fn resolve_request(request: &WorkerRequest) -> Result<Zeroizing<Vec<u8>>, WorkerFailure> {
    validate_request(request)?;
    let manifest_bytes =
        std::fs::read(&request.manifest_path).map_err(|_| WorkerFailure::ManifestRejected)?;
    if blake3::hash(&manifest_bytes).to_hex().as_str() != request.manifest_identity {
        return Err(WorkerFailure::ManifestRejected);
    }

    let mut spec = Secrets::load_from(&request.manifest_path)
        .map_err(|_| WorkerFailure::ManifestRejected)?
        .with_reason(request.purpose.clone());
    spec.set_provider(request.provider_selector.clone());
    spec.set_profile(request.profile.clone());
    spec.set_scope(request.scope.clone());
    spec.set_ignore_ambient_scope(true);

    let mut response = spec
        .resolve()
        .map_err(|error| classify_provider_error(&error))?;
    if response.schema_version != RESOLVE_SCHEMA_VERSION
        || response.profile != request.profile
        || response.scope.as_deref() != Some(request.scope.as_str())
        || !response.missing_required.is_empty()
        || !response.missing_optional.is_empty()
        || response.secrets.len() != 1
        || !provider_matches(&response.provider, &request.provider_scheme)
    {
        return Err(WorkerFailure::DeclarationMissing);
    }
    let mut secret = response
        .secrets
        .remove(&request.declaration)
        .ok_or(WorkerFailure::DeclarationMissing)?;
    if secret.as_path
        || secret.path.is_some()
        || secret.source != ResolvedSource::Provider
        || !secret
            .source_provider
            .as_deref()
            .is_some_and(|source| provider_matches(source, &request.provider_scheme))
    {
        return Err(WorkerFailure::ProviderOutputMalformed);
    }
    let value = secret
        .value
        .take()
        .ok_or(WorkerFailure::DeclarationMissing)?;
    let value = Zeroizing::new(value.into_bytes());
    if value.is_empty() {
        return Err(WorkerFailure::DeclarationMissing);
    }
    if value.len() > request.value_max_bytes {
        return Err(WorkerFailure::OutputLimit);
    }
    encode_secret(value.as_slice()).map_err(|error| match error {
        ProtocolError::ValueTooLarge => WorkerFailure::OutputLimit,
        ProtocolError::Serialize | ProtocolError::InvalidFrame => WorkerFailure::Internal,
    })
}

fn validate_request(request: &WorkerRequest) -> Result<(), WorkerFailure> {
    if !request.manifest_path.is_absolute()
        || request.profile.is_empty()
        || request.scope.is_empty()
        || request.declaration.is_empty()
        || request.provider_selector.is_empty()
        || request.purpose.is_empty()
        || request.value_max_bytes == 0
        || request.manifest_identity.len() != BLAKE3_HEX_CHARS
    {
        return Err(WorkerFailure::InvalidRequest);
    }
    let exact_selector = match request.provider_scheme.as_str() {
        "environment" => request.provider_selector == "env://",
        "systemd-credential" => request.provider_selector == "systemd-credential://",
        "sops" => request.provider_selector.starts_with("sops://"),
        _ => false,
    };
    if !exact_selector || request.provider_selector.contains('@') {
        return Err(WorkerFailure::InvalidRequest);
    }
    Ok(())
}

fn provider_matches(observed: &str, expected: &str) -> bool {
    match expected {
        "environment" => observed == "env" || observed == "env://",
        "systemd-credential" => {
            observed == "systemd-credential" || observed == "systemd-credential://"
        }
        "sops" => observed == "sops" || observed.starts_with("sops://"),
        _ => false,
    }
}

fn classify_provider_error(error: &secretspec::SecretSpecError) -> WorkerFailure {
    let redacted = error.to_string();
    if redacted.contains("ANIMUS_SOPS_EXECUTABLE_MISSING") {
        WorkerFailure::ProviderExecutableMissing
    } else if redacted.contains("ANIMUS_SOPS_OUTPUT_LIMIT") {
        WorkerFailure::OutputLimit
    } else if redacted.contains("ANIMUS_SOPS_MALFORMED_OUTPUT") {
        WorkerFailure::ProviderOutputMalformed
    } else {
        WorkerFailure::ProviderFailed
    }
}

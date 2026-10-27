//! Imperative bounded-worker shell.

use crate::material::{CredentialOrigin, ResolvedCredential};
use crate::policy::{ProfilePolicy, SecretSpecPolicy};
use crate::protocol::{ProtocolError, WorkerRequest, decode_secret, encode_request};
use crate::receipt::{ResolutionOutcome, ResolutionReceipt};
use crate::sops_guard::{
    SOPS_EXECUTABLE_ENV, SOPS_OUTPUT_MAX_ENV, SOPS_POLL_INTERVAL_ENV, SOPS_STDERR_MAX_ENV,
    SOPS_TEARDOWN_TIMEOUT_ENV, SOPS_TIMEOUT_ENV,
};
use crate::worker::{
    WORKER_EXIT_DECLARATION_MISSING, WORKER_EXIT_INVALID_REQUEST, WORKER_EXIT_MANIFEST_REJECTED,
    WORKER_EXIT_OUTPUT_LIMIT, WORKER_EXIT_PROVIDER_EXECUTABLE_MISSING, WORKER_EXIT_PROVIDER_FAILED,
    WORKER_EXIT_PROVIDER_OUTPUT_MALFORMED, WORKER_EXIT_SUCCESS,
};
use animus_adapters::secret_resolution::{
    SecretProviderScheme, SecretResolutionIssue, SecretResolutionPlan, SecretResolutionService,
};
use bounded_exec::{
    CommandSpec, Disposition, EnvironmentMode, ExecutionLimits, Input, OutcomePolicy, RunRequest,
    TerminationScope,
};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use zeroize::Zeroizing;

const AUDIT_DISABLED_CONFIG: &str = "[audit]\nenabled = false\n";
const HOME_DIRECTORY_NAME: &str = "home";
const CONFIG_DIRECTORY_NAME: &str = "config";
const SECRETSPEC_CONFIG_DIRECTORY_NAME: &str = "secretspec";
const GLOBAL_CONFIG_FILE_NAME: &str = "config.toml";
const SUCCESS_EXIT_CODES: [i32; 1] = [WORKER_EXIT_SUCCESS];

/// Private deployment coordinates for the imperative shell.
pub struct ResolverDeployment {
    /// Absolute internal resolver-worker executable.
    pub worker_executable: PathBuf,
    /// Absolute working directory for the worker.
    pub working_directory: PathBuf,
    /// Absolute explicit `SecretSpec` manifest.
    pub manifest_path: PathBuf,
}

/// Named SOPS provider credential accepted by the pinned upstream cohort.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SopsCredentialName {
    /// SOPS age private key.
    AgeKey,
    /// AWS secret access key.
    AwsSecretAccessKey,
    /// Azure client secret.
    AzureClientSecret,
    /// Google OAuth access token.
    GoogleOauthAccessToken,
    /// Vault token used by SOPS.
    VaultToken,
    /// Test-only fake-SOPS mode selector.
    #[cfg(feature = "test-fixtures")]
    FixtureMode,
}

impl SopsCredentialName {
    const fn environment_name(self) -> &'static str {
        match self {
            Self::AgeKey => "SOPS_AGE_KEY",
            Self::AwsSecretAccessKey => "AWS_SECRET_ACCESS_KEY",
            Self::AzureClientSecret => "AZURE_CLIENT_SECRET",
            Self::GoogleOauthAccessToken => "GOOGLE_OAUTH_ACCESS_TOKEN",
            Self::VaultToken => "HC_VAULT_TOKEN",
            #[cfg(feature = "test-fixtures")]
            Self::FixtureMode => "ANIMUS_SOPS_FIXTURE_MODE",
        }
    }
}

struct SopsCredential {
    name: SopsCredentialName,
    value: Zeroizing<String>,
}

/// One-shot private provider inputs. This type is not cloneable or debuggable.
pub struct ProviderInputs {
    kind: ProviderInputKind,
}

enum ProviderInputKind {
    Environment(Zeroizing<String>),
    SystemdCredential(PathBuf),
    Sops {
        guard_directory: PathBuf,
        executable: PathBuf,
        credentials: Vec<SopsCredential>,
    },
}

impl ProviderInputs {
    /// Supply one required environment value without ambient inheritance.
    #[must_use]
    pub fn environment(value: String) -> Self {
        Self {
            kind: ProviderInputKind::Environment(Zeroizing::new(value)),
        }
    }

    /// Supply one explicit systemd credentials directory.
    #[must_use]
    pub fn systemd_credential(directory: PathBuf) -> Self {
        Self {
            kind: ProviderInputKind::SystemdCredential(directory),
        }
    }

    /// Supply the guarded SOPS executable path and optional provider credentials.
    #[must_use]
    pub fn sops(
        guard_directory: PathBuf,
        executable: PathBuf,
        credentials: Vec<(SopsCredentialName, String)>,
    ) -> Self {
        Self {
            kind: ProviderInputKind::Sops {
                guard_directory,
                executable,
                credentials: credentials
                    .into_iter()
                    .map(|(name, value)| SopsCredential {
                        name,
                        value: Zeroizing::new(value),
                    })
                    .collect(),
            },
        }
    }
}

/// Successful material and safe receipt.
pub struct ResolutionSuccess {
    /// Ephemeral, zeroizing material.
    pub credential: ResolvedCredential,
    /// Metadata-only receipt.
    pub receipt: ResolutionReceipt,
}

/// Redacted host failure with metadata-only receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionFailure {
    /// Stable bounded outcome.
    pub outcome: ResolutionOutcome,
    /// Metadata-only receipt.
    pub receipt: ResolutionReceipt,
}

/// One-shot `SecretSpec` resolver. It does not retain provider inputs after use.
pub struct BoundedSecretSpecResolver {
    policy: SecretSpecPolicy,
    deployment: ResolverDeployment,
    inputs: Option<ProviderInputs>,
}

impl BoundedSecretSpecResolver {
    /// Construct a one-shot resolver from checked policy and explicit host inputs.
    #[must_use]
    pub const fn new(
        policy: SecretSpecPolicy,
        deployment: ResolverDeployment,
        inputs: ProviderInputs,
    ) -> Self {
        Self {
            policy,
            deployment,
            inputs: Some(inputs),
        }
    }

    /// Resolve one admitted plan with explicit cancellation.
    ///
    /// # Errors
    ///
    /// Returns only a stable outcome and metadata-only receipt.
    pub fn resolve_with_cancellation(
        &mut self,
        plan: &SecretResolutionPlan,
        cancellation: &AtomicBool,
    ) -> Result<ResolutionSuccess, Box<ResolutionFailure>> {
        let request = plan.request();
        let request_identity = request_identity(request);
        let make_failure = |outcome| {
            Box::new(ResolutionFailure {
                outcome,
                receipt: ResolutionReceipt::new(request_identity.clone(), request, outcome),
            })
        };
        let profile = self
            .policy
            .profile(&request.profile)
            .ok_or_else(|| make_failure(ResolutionOutcome::PolicyRejected))?;
        let inputs = self
            .inputs
            .take()
            .ok_or_else(|| make_failure(ResolutionOutcome::PolicyRejected))?;
        validate_deployment(&self.deployment).map_err(&make_failure)?;
        validate_inputs(request.provider, &inputs).map_err(&make_failure)?;
        let manifest = std::fs::read(&self.deployment.manifest_path)
            .map_err(|_| make_failure(ResolutionOutcome::ManifestRejected))?;
        if blake3::hash(&manifest).to_hex().as_str() != profile.manifest_identity() {
            return Err(make_failure(ResolutionOutcome::ManifestRejected));
        }
        let private_home = prepare_private_home()
            .map_err(|_| make_failure(ResolutionOutcome::WorkerInfrastructureFailed))?;
        let worker_request = WorkerRequest {
            manifest_path: self.deployment.manifest_path.clone(),
            manifest_identity: profile.manifest_identity().to_string(),
            profile: profile.profile().to_string(),
            scope: profile.scope().to_string(),
            declaration: profile.declaration().to_string(),
            provider_selector: profile.provider_selector().to_string(),
            provider_scheme: profile.provider().as_str().to_string(),
            purpose: profile.purpose().to_string(),
            value_max_bytes: profile.bounds().value_max_bytes,
        };
        let input = encode_request(&worker_request)
            .map_err(|_| make_failure(ResolutionOutcome::WorkerProtocolRejected))?;
        if input.len() > profile.bounds().input_max_bytes {
            return Err(make_failure(ResolutionOutcome::WorkerProtocolRejected));
        }
        let environment = build_environment(profile, inputs, &private_home);
        let execution = run_worker(
            WorkerInvocation {
                deployment: &self.deployment,
                profile,
                input,
                environment,
            },
            cancellation,
        )
        .map_err(&make_failure)?;
        let material = decode_secret(Zeroizing::new(execution), profile.bounds().value_max_bytes)
            .map_err(|error| {
            make_failure(match error {
                ProtocolError::ValueTooLarge => ResolutionOutcome::OutputLimitExceeded,
                ProtocolError::InvalidFrame | ProtocolError::Serialize => {
                    ResolutionOutcome::WorkerProtocolRejected
                }
            })
        })?;
        let origin = CredentialOrigin {
            manifest_identity: request.manifest_identity.clone(),
            profile: request.profile.clone(),
            scope: request.scope.clone(),
            declaration: request.declaration.clone(),
            provider: request.provider,
        };
        Ok(ResolutionSuccess {
            credential: ResolvedCredential::new(material, origin),
            receipt: ResolutionReceipt::new(
                request_identity,
                request,
                ResolutionOutcome::Delivered,
            ),
        })
    }
}

impl SecretResolutionService for BoundedSecretSpecResolver {
    type Material = ResolutionSuccess;

    fn resolve(
        &mut self,
        plan: &SecretResolutionPlan,
    ) -> Result<Self::Material, SecretResolutionIssue> {
        let cancellation = AtomicBool::new(false);
        self.resolve_with_cancellation(plan, &cancellation)
            .map_err(|_| SecretResolutionIssue::CapabilityMissing)
    }
}

fn validate_deployment(deployment: &ResolverDeployment) -> Result<(), ResolutionOutcome> {
    if !deployment.worker_executable.is_absolute() || !deployment.working_directory.is_absolute() {
        return Err(ResolutionOutcome::WorkerUnavailable);
    }
    if !deployment.manifest_path.is_absolute() {
        return Err(ResolutionOutcome::ManifestRejected);
    }
    Ok(())
}

fn validate_inputs(
    provider: SecretProviderScheme,
    inputs: &ProviderInputs,
) -> Result<(), ResolutionOutcome> {
    let valid = match (provider, &inputs.kind) {
        (SecretProviderScheme::Environment, ProviderInputKind::Environment(value)) => {
            !value.is_empty()
        }
        (
            SecretProviderScheme::SystemdCredential,
            ProviderInputKind::SystemdCredential(directory),
        ) => directory.is_absolute(),
        (
            SecretProviderScheme::Sops,
            ProviderInputKind::Sops {
                guard_directory,
                executable,
                credentials,
            },
        ) => {
            guard_directory.is_absolute()
                && executable.is_absolute()
                && credentials.iter().all(|entry| !entry.value.is_empty())
                && no_duplicate_sops_credentials(credentials)
        }
        _ => false,
    };
    valid.then_some(()).ok_or(ResolutionOutcome::PolicyRejected)
}

fn no_duplicate_sops_credentials(credentials: &[SopsCredential]) -> bool {
    credentials.iter().enumerate().all(|(index, entry)| {
        credentials[index + 1..]
            .iter()
            .all(|other| entry.name != other.name)
    })
}

fn prepare_private_home() -> Result<tempfile::TempDir, std::io::Error> {
    let directory = tempfile::Builder::new()
        .prefix("animus-secretspec-")
        .tempdir()?;
    let config = directory
        .path()
        .join(CONFIG_DIRECTORY_NAME)
        .join(SECRETSPEC_CONFIG_DIRECTORY_NAME);
    std::fs::create_dir_all(&config)?;
    std::fs::create_dir_all(directory.path().join(HOME_DIRECTORY_NAME))?;
    std::fs::write(config.join(GLOBAL_CONFIG_FILE_NAME), AUDIT_DISABLED_CONFIG)?;
    Ok(directory)
}

fn build_environment(
    profile: &ProfilePolicy,
    inputs: ProviderInputs,
    private_home: &tempfile::TempDir,
) -> Vec<(OsString, OsString)> {
    let mut environment = vec![
        pair("HOME", private_home.path().join(HOME_DIRECTORY_NAME)),
        pair(
            "XDG_CONFIG_HOME",
            private_home.path().join(CONFIG_DIRECTORY_NAME),
        ),
        pair("RUST_BACKTRACE", "0"),
    ];
    match inputs.kind {
        ProviderInputKind::Environment(value) => {
            environment.push(pair(profile.declaration(), value.as_str()));
        }
        ProviderInputKind::SystemdCredential(directory) => {
            environment.push(pair("CREDENTIALS_DIRECTORY", directory));
        }
        ProviderInputKind::Sops {
            guard_directory,
            executable,
            credentials,
        } => {
            environment.push(pair("PATH", guard_directory));
            environment.push(pair(SOPS_EXECUTABLE_ENV, executable));
            let bounds = profile.bounds();
            environment.push(pair(SOPS_TIMEOUT_ENV, bounds.timeout_ms.to_string()));
            environment.push(pair(
                SOPS_OUTPUT_MAX_ENV,
                bounds.value_max_bytes.to_string(),
            ));
            environment.push(pair(
                SOPS_STDERR_MAX_ENV,
                bounds.stderr_max_bytes.to_string(),
            ));
            environment.push(pair(
                SOPS_POLL_INTERVAL_ENV,
                bounds.poll_interval_ms.to_string(),
            ));
            environment.push(pair(
                SOPS_TEARDOWN_TIMEOUT_ENV,
                bounds.teardown_timeout_ms.to_string(),
            ));
            for credential in credentials {
                environment.push(pair(
                    credential.name.environment_name(),
                    credential.value.as_str(),
                ));
            }
        }
    }
    environment
}

struct WorkerInvocation<'a> {
    deployment: &'a ResolverDeployment,
    profile: &'a ProfilePolicy,
    input: Vec<u8>,
    environment: Vec<(OsString, OsString)>,
}

fn run_worker(
    invocation: WorkerInvocation<'_>,
    cancellation: &AtomicBool,
) -> Result<Vec<u8>, ResolutionOutcome> {
    let bounds = invocation.profile.bounds();
    let limits = ExecutionLimits {
        timeout_ms: bounds.timeout_ms,
        stdin_max_bytes: bounds.input_max_bytes,
        stdout_max_bytes: bounds.stdout_max_bytes,
        stderr_max_bytes: bounds.stderr_max_bytes,
        poll_interval_ms: bounds.poll_interval_ms,
        teardown_timeout_ms: bounds.teardown_timeout_ms,
    };
    let outcome_policy = OutcomePolicy::new(SUCCESS_EXIT_CODES.to_vec(), true, true)
        .map_err(|_| ResolutionOutcome::WorkerInfrastructureFailed)?;
    let output = bounded_exec::run_with_cancellation(
        RunRequest {
            command: CommandSpec {
                program: invocation.deployment.worker_executable.clone(),
                args: Vec::new(),
                current_dir: invocation.deployment.working_directory.clone(),
                environment_mode: EnvironmentMode::Clear,
                environment: invocation.environment,
                input: Input::Bytes(invocation.input),
            },
            limits,
            termination_scope: TerminationScope::ProcessGroup,
            outcome_policy,
        },
        cancellation,
    )
    .map_err(|error| match error {
        bounded_exec::RunError::Io {
            operation: bounded_exec::Operation::Spawn,
            ..
        } => ResolutionOutcome::WorkerUnavailable,
        _ => ResolutionOutcome::WorkerInfrastructureFailed,
    })?;
    match output.disposition {
        Disposition::Succeeded => Ok(output.stdout.bytes),
        Disposition::TimedOut => Err(ResolutionOutcome::TimedOut),
        Disposition::Cancelled => Err(ResolutionOutcome::Cancelled),
        Disposition::OutputLimitExceeded(_) => Err(ResolutionOutcome::OutputLimitExceeded),
        Disposition::ExitFailed => Err(map_worker_exit(output.exit_code)),
    }
}

const fn map_worker_exit(exit_code: Option<i32>) -> ResolutionOutcome {
    match exit_code {
        Some(WORKER_EXIT_INVALID_REQUEST) => ResolutionOutcome::WorkerProtocolRejected,
        Some(WORKER_EXIT_MANIFEST_REJECTED) => ResolutionOutcome::ManifestRejected,
        Some(WORKER_EXIT_PROVIDER_FAILED) => ResolutionOutcome::ProviderProcessFailed,
        Some(WORKER_EXIT_DECLARATION_MISSING | WORKER_EXIT_PROVIDER_OUTPUT_MALFORMED) => {
            ResolutionOutcome::ProviderOutputMalformed
        }
        Some(WORKER_EXIT_OUTPUT_LIMIT) => ResolutionOutcome::OutputLimitExceeded,
        Some(WORKER_EXIT_PROVIDER_EXECUTABLE_MISSING) => {
            ResolutionOutcome::ProviderExecutableMissing
        }
        Some(_) | None => ResolutionOutcome::WorkerInfrastructureFailed,
    }
}

fn request_identity(
    request: &animus_adapters::secret_resolution::SecretResolutionRequest,
) -> String {
    let payload = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        request.resolver_identity,
        request.manifest_identity,
        request.profile,
        request.scope,
        request.declaration,
        request.provider.as_str(),
        request.purpose,
        request.policy_identity
    );
    blake3::hash(payload.as_bytes()).to_hex().to_string()
}

fn pair(name: impl AsRef<OsStr>, value: impl Into<OsString>) -> (OsString, OsString) {
    (name.as_ref().to_os_string(), value.into())
}

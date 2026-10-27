//! Explicit Lattice admission and a thin wrapper over the accepted Animus host.
//!
//! Provider configuration contains public metadata only. Deployment coordinates
//! and provider inputs are injected separately and never enter workflow state.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use animus_std_secretspec::BoundedSecretSpecResolver;
use animus_std_secretspec::ProviderInputs;
use animus_std_secretspec::RESOLVER_IDENTITY;
use animus_std_secretspec::ResolutionOutcome;
use animus_std_secretspec::ResolverDeployment;
use animus_std_secretspec::SopsCredentialName;
use animus_std_secretspec::parse_policy_json;
use animus_std_secretspec::secret_resolution::CredentialRequirement;
use animus_std_secretspec::secret_resolution::SecretDestinationClass;
use animus_std_secretspec::secret_resolution::SecretProviderScheme;
use animus_std_secretspec::secret_resolution::SecretResolutionBounds;
use animus_std_secretspec::secret_resolution::SecretResolutionOperation;
use animus_std_secretspec::secret_resolution::SecretResolutionPlan;
use animus_std_secretspec::secret_resolution::SecretResolutionRequest;
use animus_std_secretspec::secret_resolution::SecretResolverAvailability;
use animus_std_secretspec::secret_resolution::SecretResolverCapabilities;
use animus_std_secretspec::secret_resolution::admit_secret_resolution;

use super::CredentialDestination;
use super::CredentialError;
use super::CredentialProbeResult;
use super::CredentialProvider;
use super::CredentialResolutionContext;
use super::CredentialSubject;
use super::SecretRef;
use super::SecretValue;
use super::config::HostInputConfig;
use super::config::ProviderScheme;
use super::config::RequestBounds;
use super::config::SecretSpecDiagnostic;
use super::config::SecretSpecHostConfig;
use super::config::SecretSpecProviderConfig;
use super::config::SopsCredentialKind;
use super::config::absolute_coordinate;
use super::config::safe_label;

impl SecretSpecDiagnostic {
    pub(super) fn error(self) -> CredentialError {
        CredentialError::ProviderFailure {
            identity: "animus-secretspec".to_string(),
            message: self.code().to_string(),
        }
    }
}

impl ProviderScheme {
    fn animus(self) -> SecretProviderScheme {
        match self {
            Self::Environment => SecretProviderScheme::Environment,
            Self::SystemdCredential => SecretProviderScheme::SystemdCredential,
            Self::Sops => SecretProviderScheme::Sops,
        }
    }
}

impl RequestBounds {
    fn animus(self) -> SecretResolutionBounds {
        SecretResolutionBounds {
            value_count_max: 1,
            input_max_bytes: self.input_max_bytes,
            value_max_bytes: self.value_max_bytes,
            stdout_max_bytes: self.stdout_max_bytes,
            stderr_max_bytes: self.stderr_max_bytes,
            timeout_ms: self.timeout_ms,
            poll_interval_ms: self.poll_interval_ms,
            teardown_timeout_ms: self.teardown_timeout_ms,
        }
    }
}

/// Pure adaptation; the caller supplies the current admitted binding and facts.
// r[impl lattice.credentials.animus_secret_providers.request]
pub fn adapt_request(
    reference: &SecretRef,
    config: &SecretSpecProviderConfig,
    context: &CredentialResolutionContext,
    capabilities: &SecretResolverCapabilities,
    availability: SecretResolverAvailability,
) -> Result<SecretResolutionPlan, SecretSpecDiagnostic> {
    config.validate()?;
    if reference.provider != config.id || context.reference != *reference || !safe_label(&context.binding) {
        return Err(SecretSpecDiagnostic::ContextDenied);
    }
    let subject_admitted = match &context.subject {
        CredentialSubject::Node { workflow_id, node_id } => {
            safe_label(workflow_id)
                && safe_label(node_id)
                && !matches!(
                    context.destination,
                    CredentialDestination::WebhookAuthentication | CredentialDestination::TriggerInvocation
                )
        }
        CredentialSubject::TriggerAuthentication {
            workflow_id,
            trigger_id,
        } => {
            safe_label(workflow_id)
                && safe_label(trigger_id)
                && context.destination == CredentialDestination::WebhookAuthentication
        }
        CredentialSubject::TriggerInvocation {
            workflow_id,
            trigger_id,
        } => {
            safe_label(workflow_id)
                && safe_label(trigger_id)
                && context.destination == CredentialDestination::TriggerInvocation
        }
    };
    if !subject_admitted {
        return Err(SecretSpecDiagnostic::ContextDenied);
    }
    if context.reason != config.reason || !safe_label(&context.reason) {
        return Err(SecretSpecDiagnostic::ReasonRequired);
    }
    let mapping = config
        .declarations
        .iter()
        .find(|entry| entry.key == reference.key && entry.field == reference.field)
        .ok_or(SecretSpecDiagnostic::MissingDeclaration)?;
    let requirement = CredentialRequirement {
        declaration: mapping.declaration.clone(),
        scope: config.scope.clone(),
        allowed_providers: config.allowed_schemes.iter().map(|s| s.animus()).collect(),
        purpose: context.reason.clone(),
        destination: SecretDestinationClass::ProviderDispatch,
    };
    let request = SecretResolutionRequest {
        resolver_identity: RESOLVER_IDENTITY.to_string(),
        manifest_identity: config.manifest_identity.clone(),
        profile: config.profile.clone(),
        scope: config.scope.clone(),
        declaration: mapping.declaration.clone(),
        provider: mapping.scheme.animus(),
        purpose: context.reason.clone(),
        destination: SecretDestinationClass::ProviderDispatch,
        operation: SecretResolutionOperation::Read,
        provider_fallback: false,
        policy_identity: config.policy_identity.clone(),
        bounds: config.bounds.animus(),
    };
    admit_secret_resolution(request, &requirement, capabilities, availability)
        .map_err(|_| SecretSpecDiagnostic::CapabilityMissing)
}

/// Injected host contract contains no backend strings or durable value evidence.
pub trait SecretSpecHost: Send + Sync {
    fn capabilities(&self) -> SecretResolverCapabilities;
    fn resolve(
        &self,
        plan: &SecretResolutionPlan,
        cancellation: &AtomicBool,
    ) -> Result<SecretValue, SecretSpecDiagnostic>;
}

// r[impl lattice.credentials.animus_secret_providers.resolve]
pub struct AnimusSecretSpecProvider {
    config: SecretSpecProviderConfig,
    public_policy_identity: String,
    host: Arc<dyn SecretSpecHost>,
    cancellation: Arc<AtomicBool>,
}

impl AnimusSecretSpecProvider {
    pub fn new(
        config: SecretSpecProviderConfig,
        host: Arc<dyn SecretSpecHost>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<Self, SecretSpecDiagnostic> {
        config.validate()?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"lattice.credentials.secretspec-public-policy.v1");
        serde_json::to_writer(&mut hasher, &config).map_err(|_| SecretSpecDiagnostic::InvalidConfiguration)?;
        let public_policy_identity = hasher.finalize().to_hex().to_string();
        Ok(Self {
            config,
            public_policy_identity,
            host,
            cancellation,
        })
    }
}

impl CredentialProvider for AnimusSecretSpecProvider {
    fn resolution_reason(&self, reference: &SecretRef) -> Option<&str> {
        self.validate_reference(reference).ok().map(|()| self.config.reason.as_str())
    }

    fn resolution_policy_identity(&self, reference: &SecretRef) -> Result<Option<&str>, CredentialError> {
        self.validate_reference(reference)?;
        Ok(Some(&self.public_policy_identity))
    }

    fn validate_reference(&self, reference: &SecretRef) -> Result<(), CredentialError> {
        self.config.validate().map_err(SecretSpecDiagnostic::error)?;
        if reference.provider != self.config.id
            || !self
                .config
                .declarations
                .iter()
                .any(|entry| entry.key == reference.key && entry.field == reference.field)
        {
            return Err(SecretSpecDiagnostic::MissingDeclaration.error());
        }
        Ok(())
    }

    fn resolve(&self, _reference: &SecretRef) -> Result<SecretValue, CredentialError> {
        Err(SecretSpecDiagnostic::ContextDenied.error())
    }

    fn resolve_in_context(
        &self,
        reference: &SecretRef,
        context: &CredentialResolutionContext,
        cancellation: Option<&AtomicBool>,
    ) -> Result<SecretValue, CredentialError> {
        let plan = adapt_request(
            reference,
            &self.config,
            context,
            &self.host.capabilities(),
            SecretResolverAvailability::Available,
        )
        .map_err(SecretSpecDiagnostic::error)?;
        let current_cancellation = cancellation.unwrap_or(self.cancellation.as_ref());
        if self.cancellation.load(Ordering::Acquire) || current_cancellation.load(Ordering::Acquire) {
            return Err(SecretSpecDiagnostic::Cancelled.error());
        }
        let value = self.host.resolve(&plan, current_cancellation).map_err(SecretSpecDiagnostic::error)?;
        if self.cancellation.load(Ordering::Acquire) || current_cancellation.load(Ordering::Acquire) {
            return Err(SecretSpecDiagnostic::Cancelled.error());
        }
        if value.expose_for_execution().is_empty()
            || value.expose_for_execution().len() > self.config.bounds.value_max_bytes
            || value.expose_for_execution().contains('\0')
        {
            return Err(SecretSpecDiagnostic::ValueRejected.error());
        }
        Ok(value)
    }

    fn probe(&self, reference: &SecretRef) -> CredentialProbeResult {
        let configured = self.config.validate().is_ok()
            && reference.provider == self.config.id
            && self
                .config
                .declarations
                .iter()
                .any(|entry| entry.key == reference.key && entry.field == reference.field);
        CredentialProbeResult {
            identity: if configured {
                reference.identity()
            } else {
                "animus-secretspec".to_string()
            },
            reachable: false,
            message: if configured {
                "configured; reachability not probed"
            } else {
                "credential_secretspec_missing_declaration"
            }
            .to_string(),
        }
    }
}
impl SopsCredentialKind {
    fn animus(self) -> SopsCredentialName {
        match self {
            Self::AgeKey => SopsCredentialName::AgeKey,
            Self::AwsSecretAccessKey => SopsCredentialName::AwsSecretAccessKey,
            Self::AzureClientSecret => SopsCredentialName::AzureClientSecret,
            Self::GoogleOauthAccessToken => SopsCredentialName::GoogleOauthAccessToken,
            Self::VaultToken => SopsCredentialName::VaultToken,
        }
    }
}

const MAX_POLICY_BYTES: usize = 1_048_576;
const MAX_MANIFEST_BYTES: usize = 1_048_576;
const MAX_PROVIDER_CREDENTIAL_BYTES: usize = 65_536;

fn read_bounded_file(path: &std::path::Path, max_bytes: usize) -> Result<Vec<u8>, SecretSpecDiagnostic> {
    use std::io::Read;
    let coordinate = std::fs::metadata(path).map_err(|_| SecretSpecDiagnostic::Unavailable)?;
    if !coordinate.is_file() || coordinate.len() > max_bytes as u64 {
        return Err(SecretSpecDiagnostic::InvalidConfiguration);
    }
    let file = std::fs::File::open(path).map_err(|_| SecretSpecDiagnostic::Unavailable)?;
    let metadata = file.metadata().map_err(|_| SecretSpecDiagnostic::Unavailable)?;
    if !metadata.is_file() || metadata.len() > max_bytes as u64 {
        return Err(SecretSpecDiagnostic::InvalidConfiguration);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take((max_bytes + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| SecretSpecDiagnostic::Unavailable)?;
    if bytes.len() > max_bytes {
        return Err(SecretSpecDiagnostic::InvalidConfiguration);
    }
    Ok(bytes)
}

/// Compose a configured provider without ambient discovery or provider access.
pub fn build_provider(
    config: SecretSpecProviderConfig,
    host: &SecretSpecHostConfig,
    cancellation: Arc<AtomicBool>,
) -> Result<AnimusSecretSpecProvider, SecretSpecDiagnostic> {
    config.validate()?;
    host.validate()?;
    if host.provider_id != config.id {
        return Err(SecretSpecDiagnostic::InvalidConfiguration);
    }
    let policy_json = read_bounded_file(&host.policy_path, MAX_POLICY_BYTES)?;
    let policy = parse_policy_json(&policy_json).map_err(|_| SecretSpecDiagnostic::InvalidConfiguration)?;
    if policy.identity() != config.policy_identity {
        return Err(SecretSpecDiagnostic::InvalidConfiguration);
    }
    let manifest = read_bounded_file(&host.manifest_path, MAX_MANIFEST_BYTES)?;
    if blake3::hash(&manifest).to_hex().as_str() != config.manifest_identity {
        return Err(SecretSpecDiagnostic::ManifestRejected);
    }
    let source = match &host.input {
        HostInputConfig::Environment { variable } => HostInputSource::Environment {
            variable: variable.clone(),
        },
        HostInputConfig::SystemdCredential { directory } => HostInputSource::SystemdCredential {
            directory: directory.clone(),
        },
        HostInputConfig::Sops {
            guard_directory,
            executable,
            credential_variables,
        } => HostInputSource::Sops {
            guard_directory: guard_directory.clone(),
            executable: executable.clone(),
            credential_variables: credential_variables
                .iter()
                .map(|entry| (entry.credential.animus(), entry.variable.clone()))
                .collect(),
        },
    };
    let deployment = ResolverDeployment {
        manifest_path: host.manifest_path.clone(),
        worker_executable: host.worker_executable.clone(),
        working_directory: host.working_directory.clone(),
    };
    let bounded = BoundedSecretSpecHost {
        policy_json,
        deployment,
        source,
    };
    for mapping in &config.declarations {
        let reference = SecretRef::new(&config.id, &mapping.key, mapping.field.clone());
        let context = CredentialResolutionContext {
            subject: CredentialSubject::Node {
                workflow_id: "configuration".into(),
                node_id: "validation".into(),
            },
            binding: "configuration".into(),
            reference: reference.clone(),
            destination: CredentialDestination::AgentEnvironment,
            reason: config.reason.clone(),
        };
        let plan = adapt_request(
            &reference,
            &config,
            &context,
            &bounded.capabilities(),
            SecretResolverAvailability::Available,
        )?;
        policy.admit(plan.request().clone()).map_err(|_| SecretSpecDiagnostic::InvalidConfiguration)?;
    }
    AnimusSecretSpecProvider::new(config, Arc::new(bounded), cancellation)
}

/// Private host input coordinates, never a workflow/provider declaration.
/// Environment access is limited to these exact configured variable names.
pub enum HostInputSource {
    Environment {
        variable: String,
    },
    SystemdCredential {
        directory: PathBuf,
    },
    Sops {
        guard_directory: PathBuf,
        executable: PathBuf,
        credential_variables: Vec<(SopsCredentialName, String)>,
    },
}

/// Real host implementation. Each resolution constructs a fresh one-shot worker.
pub struct BoundedSecretSpecHost {
    policy_json: Vec<u8>,
    deployment: ResolverDeployment,
    source: HostInputSource,
}

impl BoundedSecretSpecHost {
    pub fn new(
        policy_json: Vec<u8>,
        deployment: ResolverDeployment,
        source: HostInputSource,
    ) -> Result<Self, SecretSpecDiagnostic> {
        if policy_json.len() > MAX_POLICY_BYTES {
            return Err(SecretSpecDiagnostic::InvalidConfiguration);
        }
        parse_policy_json(&policy_json).map_err(|_| SecretSpecDiagnostic::InvalidConfiguration)?;
        if !absolute_coordinate(&deployment.worker_executable)
            || !absolute_coordinate(&deployment.manifest_path)
            || !absolute_coordinate(&deployment.working_directory)
        {
            return Err(SecretSpecDiagnostic::InvalidConfiguration);
        }
        let valid_source = match &source {
            HostInputSource::Environment { variable } => safe_label(variable),
            HostInputSource::SystemdCredential { directory } => absolute_coordinate(directory),
            HostInputSource::Sops {
                guard_directory,
                executable,
                credential_variables,
            } => {
                absolute_coordinate(guard_directory)
                    && absolute_coordinate(executable)
                    && credential_variables.iter().all(|(_, variable)| safe_label(variable))
            }
        };
        if !valid_source {
            return Err(SecretSpecDiagnostic::InvalidConfiguration);
        }
        Ok(Self {
            policy_json,
            deployment,
            source,
        })
    }

    fn inputs(
        &self,
        scheme: SecretProviderScheme,
        value_max_bytes: usize,
    ) -> Result<ProviderInputs, SecretSpecDiagnostic> {
        match (&self.source, scheme) {
            (HostInputSource::Environment { variable }, SecretProviderScheme::Environment) => {
                let mut value = std::env::var(variable).map_err(|_| SecretSpecDiagnostic::Unavailable)?;
                if value.len() > value_max_bytes {
                    zeroize::Zeroize::zeroize(&mut value);
                    return Err(SecretSpecDiagnostic::ValueRejected);
                }
                Ok(ProviderInputs::environment(value))
            }
            (HostInputSource::SystemdCredential { directory }, SecretProviderScheme::SystemdCredential) => {
                Ok(ProviderInputs::systemd_credential(directory.clone()))
            }
            (
                HostInputSource::Sops {
                    guard_directory,
                    executable,
                    credential_variables,
                },
                SecretProviderScheme::Sops,
            ) => {
                let mut credentials: Vec<(SopsCredentialName, String)> = Vec::with_capacity(credential_variables.len());
                for (name, variable) in credential_variables {
                    match std::env::var(variable) {
                        Ok(mut value) if value.len() > MAX_PROVIDER_CREDENTIAL_BYTES => {
                            zeroize::Zeroize::zeroize(&mut value);
                            for (_, value) in &mut credentials {
                                zeroize::Zeroize::zeroize(value);
                            }
                            return Err(SecretSpecDiagnostic::ValueRejected);
                        }
                        Ok(value) => credentials.push((*name, value)),
                        Err(_) => {
                            for (_, value) in &mut credentials {
                                zeroize::Zeroize::zeroize(value);
                            }
                            return Err(SecretSpecDiagnostic::Unavailable);
                        }
                    }
                }
                Ok(ProviderInputs::sops(guard_directory.clone(), executable.clone(), credentials))
            }
            _ => Err(SecretSpecDiagnostic::ProviderDenied),
        }
    }
}

impl SecretSpecHost for BoundedSecretSpecHost {
    fn capabilities(&self) -> SecretResolverCapabilities {
        let provider = match &self.source {
            HostInputSource::Environment { .. } => SecretProviderScheme::Environment,
            HostInputSource::SystemdCredential { .. } => SecretProviderScheme::SystemdCredential,
            HostInputSource::Sops { .. } => SecretProviderScheme::Sops,
        };
        SecretResolverCapabilities {
            read: true,
            bounded_worker: true,
            providers: vec![provider],
        }
    }

    fn resolve(
        &self,
        plan: &SecretResolutionPlan,
        cancellation: &AtomicBool,
    ) -> Result<SecretValue, SecretSpecDiagnostic> {
        let policy = parse_policy_json(&self.policy_json).map_err(|_| SecretSpecDiagnostic::InvalidConfiguration)?;
        // Re-admit against the real host policy, not just caller capability facts.
        let checked = policy.admit(plan.request().clone()).map_err(|_| SecretSpecDiagnostic::ProviderDenied)?;
        if cancellation.load(Ordering::Acquire) {
            return Err(SecretSpecDiagnostic::Cancelled);
        }
        let manifest_metadata =
            std::fs::metadata(&self.deployment.manifest_path).map_err(|_| SecretSpecDiagnostic::ManifestRejected)?;
        if !manifest_metadata.is_file() || manifest_metadata.len() > MAX_MANIFEST_BYTES as u64 {
            return Err(SecretSpecDiagnostic::ManifestRejected);
        }
        let inputs = self.inputs(checked.request().provider, checked.request().bounds.value_max_bytes)?;
        let deployment = ResolverDeployment {
            worker_executable: self.deployment.worker_executable.clone(),
            working_directory: self.deployment.working_directory.clone(),
            manifest_path: self.deployment.manifest_path.clone(),
        };
        let mut resolver = BoundedSecretSpecResolver::new(policy, deployment, inputs);
        let success = resolver
            .resolve_with_cancellation(&checked, cancellation)
            .map_err(|failure| map_outcome(failure.outcome))?;
        success.credential.expose_to(|bytes| {
            std::str::from_utf8(bytes).map(SecretValue::new).map_err(|_| SecretSpecDiagnostic::ValueRejected)
        })
    }
}

// r[impl lattice.credentials.animus_secret_providers.redaction]
pub fn map_outcome(outcome: ResolutionOutcome) -> SecretSpecDiagnostic {
    match outcome {
        ResolutionOutcome::Delivered => SecretSpecDiagnostic::ValueRejected,
        ResolutionOutcome::PolicyRejected => SecretSpecDiagnostic::ProviderDenied,
        ResolutionOutcome::ManifestRejected => SecretSpecDiagnostic::ManifestRejected,
        ResolutionOutcome::WorkerUnavailable | ResolutionOutcome::ProviderExecutableMissing => {
            SecretSpecDiagnostic::Unavailable
        }
        ResolutionOutcome::ProviderProcessFailed => SecretSpecDiagnostic::ProviderFailed,
        ResolutionOutcome::ProviderOutputMalformed
        | ResolutionOutcome::WorkerProtocolRejected
        | ResolutionOutcome::OutputLimitExceeded => SecretSpecDiagnostic::ValueRejected,
        ResolutionOutcome::TimedOut => SecretSpecDiagnostic::TimedOut,
        ResolutionOutcome::Cancelled => SecretSpecDiagnostic::Cancelled,
        ResolutionOutcome::WorkerInfrastructureFailed => SecretSpecDiagnostic::CleanupUncertain,
    }
}

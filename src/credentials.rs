//! Credential references, provider interfaces, and redaction helpers.

use std::collections::BTreeMap;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::agent_node::ProcessOutput;
use crate::agent_node::ProcessRequest;
use crate::agent_node::ProcessRunner;

const REDACTION_MARKER: &str = "<redacted>";
const SECRET_IDENTITY_SEPARATOR: &str = ":";
const SECRET_FIELD_SEPARATOR: &str = ".";
const DEFAULT_PROVIDER_TIMEOUT_SECS: u64 = 5;
const DEFAULT_PROVIDER_OUTPUT_LIMIT_BYTES: usize = 4096;
const SUCCESS_STATUS_CODE: i32 = 0;
// r[impl lattice.project_identity.stable-identities]
const VAULT_STREAM_HASH_TAG: &[u8] = b"n8nclone-local-vault-stream-v1";
const VAULT_INTEGRITY_HASH_TAG: &[u8] = b"n8nclone-local-vault-integrity-v1";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecretRef {
    pub provider: String,
    pub key: String,
    pub field: Option<String>,
}

impl SecretRef {
    pub fn new(provider: impl Into<String>, key: impl Into<String>, field: Option<String>) -> Self {
        Self {
            provider: provider.into(),
            key: key.into(),
            field,
        }
    }

    pub fn identity(&self) -> String {
        match &self.field {
            Some(field) => {
                format!("{}{}{}{}{}", self.provider, SECRET_IDENTITY_SEPARATOR, self.key, SECRET_FIELD_SEPARATOR, field)
            }
            None => format!("{}{}{}", self.provider, SECRET_IDENTITY_SEPARATOR, self.key),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCredentialRefs {
    pub env: HashMap<String, SecretRef>,
}

impl NodeCredentialRefs {
    pub fn is_empty(&self) -> bool {
        self.env.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose_for_execution(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialError {
    MalformedRef { identity: String, reason: String },
    MissingSecret { identity: String },
    UnknownProvider { provider: String },
    UnsupportedFieldAccess { identity: String },
    ProviderFailure { identity: String, message: String },
    TamperedSecret { identity: String },
}

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRef { identity, reason } => write!(f, "malformed secret reference {identity}: {reason}"),
            Self::MissingSecret { identity } => write!(f, "missing secret: {identity}"),
            Self::UnknownProvider { provider } => write!(f, "unknown credential provider: {provider}"),
            Self::UnsupportedFieldAccess { identity } => {
                write!(f, "credential provider does not support field access: {identity}")
            }
            Self::ProviderFailure { identity, message } => {
                write!(f, "credential provider failed for {identity}: {message}")
            }
            Self::TamperedSecret { identity } => write!(f, "credential vault integrity check failed for {identity}"),
        }
    }
}

impl std::error::Error for CredentialError {}

pub trait CredentialProvider: Send + Sync {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, CredentialError>;
}

#[derive(Debug, Default)]
pub struct EmptyCredentialProvider;

impl CredentialProvider for EmptyCredentialProvider {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, CredentialError> {
        Err(CredentialError::MissingSecret {
            identity: secret_ref.identity(),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct MapCredentialProvider {
    secrets: HashMap<String, String>,
}

impl MapCredentialProvider {
    pub fn new(secrets: HashMap<String, String>) -> Self {
        Self { secrets }
    }
}

impl CredentialProvider for MapCredentialProvider {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, CredentialError> {
        self.secrets.get(&secret_ref.identity()).cloned().map(SecretValue::new).ok_or_else(|| {
            CredentialError::MissingSecret {
                identity: secret_ref.identity(),
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialProviderCapabilities {
    pub supports_fields: bool,
    pub supports_listing: bool,
    pub supports_test: bool,
    pub supports_write: bool,
}

impl CredentialProviderCapabilities {
    pub fn read_only_without_fields() -> Self {
        Self {
            supports_fields: false,
            supports_listing: false,
            supports_test: true,
            supports_write: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialProviderMetadata {
    pub id: String,
    pub display_name: String,
    pub capabilities: CredentialProviderCapabilities,
}

struct RegisteredCredentialProvider {
    metadata: CredentialProviderMetadata,
    provider: Box<dyn CredentialProvider>,
}

#[derive(Default)]
pub struct CredentialProviderRegistry {
    providers: HashMap<String, RegisteredCredentialProvider>,
}

impl CredentialProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    // r[impl n8nclone.credentials.provider_registry]
    pub fn insert_provider(
        &mut self,
        metadata: CredentialProviderMetadata,
        provider: Box<dyn CredentialProvider>,
    ) -> Result<(), CredentialError> {
        if metadata.id.trim().is_empty() {
            return Err(CredentialError::MalformedRef {
                identity: metadata.id,
                reason: "provider id is empty".to_string(),
            });
        }
        self.providers.insert(metadata.id.clone(), RegisteredCredentialProvider { metadata, provider });
        Ok(())
    }

    pub fn metadata(&self, provider_id: &str) -> Option<&CredentialProviderMetadata> {
        self.providers.get(provider_id).map(|registered| &registered.metadata)
    }

    // r[impl n8nclone.workflow_validation.credentials.providers]
    pub fn validate_secret_ref(&self, secret_ref: &SecretRef) -> Result<(), CredentialError> {
        validate_secret_ref(secret_ref)?;
        let Some(registered) = self.providers.get(&secret_ref.provider) else {
            return Err(CredentialError::UnknownProvider {
                provider: secret_ref.provider.clone(),
            });
        };
        if secret_ref.field.is_some() && !registered.metadata.capabilities.supports_fields {
            return Err(CredentialError::UnsupportedFieldAccess {
                identity: secret_ref.identity(),
            });
        }
        Ok(())
    }
}

impl CredentialProvider for CredentialProviderRegistry {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, CredentialError> {
        self.validate_secret_ref(secret_ref)?;
        let registered = self.providers.get(&secret_ref.provider).ok_or_else(|| CredentialError::UnknownProvider {
            provider: secret_ref.provider.clone(),
        })?;
        registered.provider.resolve(secret_ref)
    }
}

#[derive(Debug, Clone, Default)]
pub struct EnvironmentCredentialProvider {
    variables: HashMap<String, String>,
}

impl EnvironmentCredentialProvider {
    pub fn from_map(variables: HashMap<String, String>) -> Self {
        Self { variables }
    }

    pub fn from_current_env(keys: &[String]) -> Self {
        let mut variables = HashMap::new();
        for key in keys {
            if let Ok(value) = std::env::var(key) {
                variables.insert(key.clone(), value);
            }
        }
        Self { variables }
    }
}

// r[impl n8nclone.credentials.providers.env]
impl CredentialProvider for EnvironmentCredentialProvider {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, CredentialError> {
        validate_secret_ref(secret_ref)?;
        if secret_ref.field.is_some() {
            return Err(CredentialError::UnsupportedFieldAccess {
                identity: secret_ref.identity(),
            });
        }
        self.variables
            .get(&secret_ref.key)
            .filter(|value| !value.is_empty())
            .cloned()
            .map(SecretValue::new)
            .ok_or_else(|| CredentialError::MissingSecret {
                identity: secret_ref.identity(),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalVaultEntry {
    pub key: String,
    pub nonce_hex: String,
    pub ciphertext_hex: String,
    pub plaintext_hash: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCredentialVault {
    pub entries: HashMap<String, LocalVaultEntry>,
}

impl LocalCredentialVault {
    pub fn new(entries: HashMap<String, LocalVaultEntry>) -> Self {
        Self { entries }
    }

    pub fn insert(&mut self, entry: LocalVaultEntry) {
        self.entries.insert(entry.key.clone(), entry);
    }
}

#[derive(Debug, Clone)]
pub struct LocalVaultCredentialProvider {
    vault: LocalCredentialVault,
    key_material: String,
}

impl LocalVaultCredentialProvider {
    pub fn new(vault: LocalCredentialVault, key_material: impl Into<String>) -> Self {
        Self {
            vault,
            key_material: key_material.into(),
        }
    }
}

// r[impl n8nclone.credentials.providers.local_vault]
impl CredentialProvider for LocalVaultCredentialProvider {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, CredentialError> {
        validate_secret_ref(secret_ref)?;
        if secret_ref.field.is_some() {
            return Err(CredentialError::UnsupportedFieldAccess {
                identity: secret_ref.identity(),
            });
        }
        let entry = self.vault.entries.get(&secret_ref.key).ok_or_else(|| CredentialError::MissingSecret {
            identity: secret_ref.identity(),
        })?;
        decrypt_local_vault_entry(entry, &self.key_material, &secret_ref.identity()).map(SecretValue::new)
    }
}

// r[impl n8nclone.credentials.providers.local_vault]
pub fn seal_local_vault_entry(
    key: impl Into<String>,
    plaintext: &str,
    key_material: &str,
    nonce: &[u8],
) -> LocalVaultEntry {
    let plaintext_bytes = plaintext.as_bytes();
    let stream = vault_stream(key_material, nonce, plaintext_bytes.len());
    let ciphertext = xor_bytes(plaintext_bytes, &stream);
    LocalVaultEntry {
        key: key.into(),
        nonce_hex: hex::encode(nonce),
        ciphertext_hex: hex::encode(ciphertext),
        plaintext_hash: vault_plaintext_hash(plaintext_bytes),
    }
}

fn decrypt_local_vault_entry(
    entry: &LocalVaultEntry,
    key_material: &str,
    identity: &str,
) -> Result<String, CredentialError> {
    let nonce = hex::decode(&entry.nonce_hex).map_err(|error| CredentialError::ProviderFailure {
        identity: identity.to_string(),
        message: format!("invalid vault nonce: {error}"),
    })?;
    let ciphertext = hex::decode(&entry.ciphertext_hex).map_err(|error| CredentialError::ProviderFailure {
        identity: identity.to_string(),
        message: format!("invalid vault ciphertext: {error}"),
    })?;
    let stream = vault_stream(key_material, &nonce, ciphertext.len());
    let plaintext = xor_bytes(&ciphertext, &stream);
    if vault_plaintext_hash(&plaintext) != entry.plaintext_hash {
        return Err(CredentialError::TamperedSecret {
            identity: identity.to_string(),
        });
    }
    String::from_utf8(plaintext).map_err(|error| CredentialError::ProviderFailure {
        identity: identity.to_string(),
        message: format!("vault plaintext is not utf-8: {error}"),
    })
}

fn vault_stream(key_material: &str, nonce: &[u8], output_len: usize) -> Vec<u8> {
    let mut stream = Vec::with_capacity(output_len);
    let mut counter = 0_u64;
    while stream.len() < output_len {
        let mut hasher = blake3::Hasher::new();
        hasher.update(VAULT_STREAM_HASH_TAG);
        hasher.update(key_material.as_bytes());
        hasher.update(nonce);
        hasher.update(&counter.to_le_bytes());
        stream.extend_from_slice(hasher.finalize().as_bytes());
        counter = counter.saturating_add(1);
    }
    stream.truncate(output_len);
    stream
}

fn xor_bytes(left: &[u8], right: &[u8]) -> Vec<u8> {
    left.iter().zip(right).map(|(left, right)| left ^ right).collect()
}

fn vault_plaintext_hash(plaintext: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(VAULT_INTEGRITY_HASH_TAG);
    hasher.update(plaintext);
    hasher.finalize().to_hex().to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCredentialProviderConfig {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: BTreeMap<String, String>,
    pub timeout_secs: u64,
    pub output_limit_bytes: usize,
}

impl CommandCredentialProviderConfig {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env: BTreeMap::new(),
            timeout_secs: DEFAULT_PROVIDER_TIMEOUT_SECS,
            output_limit_bytes: DEFAULT_PROVIDER_OUTPUT_LIMIT_BYTES,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandCredentialProvider<R> {
    config: CommandCredentialProviderConfig,
    runner: R,
}

impl<R> CommandCredentialProvider<R> {
    pub fn new(config: CommandCredentialProviderConfig, runner: R) -> Self {
        Self { config, runner }
    }
}

#[derive(Debug, Deserialize)]
struct CommandSecretResponse {
    value: String,
}

// r[impl n8nclone.credentials.providers.command]
impl<R> CredentialProvider for CommandCredentialProvider<R>
where R: ProcessRunner + Send + Sync
{
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, CredentialError> {
        validate_secret_ref(secret_ref)?;
        if secret_ref.field.is_some() {
            return Err(CredentialError::UnsupportedFieldAccess {
                identity: secret_ref.identity(),
            });
        }
        let request = command_provider_request(&self.config, secret_ref);
        let output = self.runner.run(&request).map_err(|error| CredentialError::ProviderFailure {
            identity: secret_ref.identity(),
            message: error.to_string(),
        })?;
        command_provider_secret_from_output(secret_ref, output, self.config.output_limit_bytes).map(SecretValue::new)
    }
}

fn command_provider_request(config: &CommandCredentialProviderConfig, secret_ref: &SecretRef) -> ProcessRequest {
    ProcessRequest {
        program: config.program.clone(),
        args: config.args.clone(),
        cwd: config.cwd.clone(),
        env: config.env.clone(),
        stdin: secret_ref.identity().into_bytes(),
        timeout_secs: config.timeout_secs,
        output_limit_bytes: config.output_limit_bytes,
    }
}

fn command_provider_secret_from_output(
    secret_ref: &SecretRef,
    output: ProcessOutput,
    output_limit_bytes: usize,
) -> Result<String, CredentialError> {
    if output.status_code != Some(SUCCESS_STATUS_CODE) {
        return Err(CredentialError::ProviderFailure {
            identity: secret_ref.identity(),
            message: "command provider exited unsuccessfully".to_string(),
        });
    }
    if output.stdout.len() > output_limit_bytes {
        return Err(CredentialError::ProviderFailure {
            identity: secret_ref.identity(),
            message: "command provider output exceeded configured limit".to_string(),
        });
    }
    let stdout = String::from_utf8(output.stdout).map_err(|error| CredentialError::ProviderFailure {
        identity: secret_ref.identity(),
        message: format!("command provider output is not utf-8: {error}"),
    })?;
    parse_command_secret_stdout(secret_ref, stdout.trim())
}

fn parse_command_secret_stdout(secret_ref: &SecretRef, stdout: &str) -> Result<String, CredentialError> {
    if stdout.starts_with('{') {
        let response = serde_json::from_str::<CommandSecretResponse>(stdout).map_err(|error| {
            CredentialError::ProviderFailure {
                identity: secret_ref.identity(),
                message: format!("malformed command provider JSON: {error}"),
            }
        })?;
        if response.value.is_empty() {
            return Err(CredentialError::MissingSecret {
                identity: secret_ref.identity(),
            });
        }
        return Ok(response.value);
    }
    if stdout.is_empty() {
        return Err(CredentialError::MissingSecret {
            identity: secret_ref.identity(),
        });
    }
    Ok(stdout.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialProbeResult {
    pub identity: String,
    pub reachable: bool,
    pub message: String,
}

fn credential_probe_result(secret_ref: &SecretRef, is_reachable: bool) -> CredentialProbeResult {
    // Probe output never carries backend-provided text.
    let message = if is_reachable {
        "secret resolved"
    } else {
        "provider resolution failed"
    };
    CredentialProbeResult {
        identity: secret_ref.identity(),
        reachable: is_reachable,
        message: message.to_string(),
    }
}

// r[impl n8nclone.credentials.provider_diagnostics.redaction]
pub fn test_provider_reference(provider: &dyn CredentialProvider, secret_ref: &SecretRef) -> CredentialProbeResult {
    let is_reachable = provider.resolve(secret_ref).is_ok();
    credential_probe_result(secret_ref, is_reachable)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSecretEnv {
    pub env: HashMap<String, String>,
    pub secret_values: Vec<String>,
}

// r[impl n8nclone.credentials.validation]
pub fn validate_secret_ref(secret_ref: &SecretRef) -> Result<(), CredentialError> {
    if secret_ref.provider.trim().is_empty() {
        return Err(CredentialError::MalformedRef {
            identity: secret_ref.identity(),
            reason: "provider is empty".to_string(),
        });
    }
    if secret_ref.key.trim().is_empty() {
        return Err(CredentialError::MalformedRef {
            identity: secret_ref.identity(),
            reason: "key is empty".to_string(),
        });
    }
    if secret_ref.field.as_deref().is_some_and(|field| field.trim().is_empty()) {
        return Err(CredentialError::MalformedRef {
            identity: secret_ref.identity(),
            reason: "field is empty".to_string(),
        });
    }
    Ok(())
}

// r[impl n8nclone.credentials.provider_interface]
pub fn resolve_secret_env(
    refs: &NodeCredentialRefs,
    provider: &dyn CredentialProvider,
) -> Result<ResolvedSecretEnv, CredentialError> {
    let mut resolved_env = HashMap::new();
    let mut secret_values = Vec::new();
    let mut refs_by_env = refs.env.iter().collect::<Vec<_>>();
    refs_by_env.sort_by_key(|(name, _)| *name);

    for (name, secret_ref) in refs_by_env {
        validate_secret_ref(secret_ref)?;
        let value = provider.resolve(secret_ref)?;
        secret_values.push(value.expose_for_execution().to_string());
        resolved_env.insert(name.clone(), value.expose_for_execution().to_string());
    }

    Ok(ResolvedSecretEnv {
        env: resolved_env,
        secret_values,
    })
}

// r[impl n8nclone.credentials.redaction]
pub fn redact_text(text: &str, secret_values: &[String]) -> String {
    let mut normalized =
        secret_values.iter().map(String::as_str).filter(|secret| !secret.is_empty()).collect::<Vec<_>>();
    normalized.sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));
    normalized.dedup();

    let mut redacted = text.to_string();
    for secret in normalized {
        redacted = redacted.replace(secret, REDACTION_MARKER);
    }
    redacted
}

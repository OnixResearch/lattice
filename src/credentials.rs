//! Credential references, provider interfaces, and redaction helpers.

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

const REDACTION_MARKER: &str = "<redacted>";
const SECRET_IDENTITY_SEPARATOR: &str = ":";
const SECRET_FIELD_SEPARATOR: &str = ".";

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
}

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRef { identity, reason } => write!(f, "malformed secret reference {identity}: {reason}"),
            Self::MissingSecret { identity } => write!(f, "missing secret: {identity}"),
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
    let mut redacted = text.to_string();
    for secret in secret_values.iter().filter(|secret| !secret.is_empty()) {
        redacted = redacted.replace(secret, REDACTION_MARKER);
    }
    redacted
}

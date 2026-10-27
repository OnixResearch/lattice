//! Execution-only, material-free credential slots and current-boundary facts.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use serde::Serialize;

use super::CredentialDestination;
use super::CredentialError;
use super::CredentialProvider;
use super::CredentialResolutionContext;
use super::CredentialSubject;
use super::NodeCredentialRefs;
use super::ResolvedSecretEnv;
use super::SecretRef;

#[derive(Serialize)]
struct Binding {
    reference: SecretRef,
    resolver_policy: Option<String>,
}

/// Sealed public mappings; concrete values are obtained only at consumption.
/// The provider is a trusted shell port, not a source of effect authority.
pub(crate) struct Bindings {
    entries: BTreeMap<String, Binding>,
    destination: CredentialDestination,
    provider: Arc<dyn CredentialProvider>,
}

impl Bindings {
    pub(crate) fn new(
        references: Option<&NodeCredentialRefs>,
        destination: CredentialDestination,
        provider: Arc<dyn CredentialProvider>,
    ) -> Result<Self, CredentialError> {
        let mut entries = BTreeMap::new();
        if let Some(references) = references {
            for (name, reference) in &references.env {
                let resolver_policy = provider.resolution_policy_identity(reference)?.map(str::to_owned);
                entries.insert(name.clone(), Binding {
                    reference: reference.clone(),
                    resolver_policy,
                });
            }
        }
        Ok(Self {
            entries,
            destination,
            provider,
        })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    pub(crate) fn identity<T: Serialize>(
        &self,
        domain: &[u8],
        public_request: &T,
    ) -> Result<String, serde_json::Error> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(domain);
        serde_json::to_writer(
            &mut hasher,
            &(public_request, self.destination, &self.entries, "credential-slots-override-literal-values"),
        )?;
        Ok(format!("blake3:{}", hasher.finalize().to_hex()))
    }

    pub(crate) fn resolve(
        &self,
        subject: &CredentialSubject,
        cancellation: Option<&AtomicBool>,
    ) -> Result<ResolvedSecretEnv, CredentialError> {
        let mut resolved = ResolvedSecretEnv {
            env: HashMap::new(),
            secret_values: Vec::new(),
        };
        for (name, binding) in &self.entries {
            if cancelled(cancellation) {
                return Err(cancelled_error());
            }
            if self.provider.resolution_policy_identity(&binding.reference)? != binding.resolver_policy.as_deref() {
                return Err(CredentialError::ProviderFailure {
                    identity: "credential-policy".to_owned(),
                    message: "credential_policy_changed".to_owned(),
                });
            }
            let context = CredentialResolutionContext {
                subject: subject.clone(),
                binding: name.clone(),
                reference: binding.reference.clone(),
                destination: self.destination,
                reason: self.provider.resolution_reason(&binding.reference).unwrap_or_default().to_owned(),
            };
            let value = self.provider.resolve_in_context(&binding.reference, &context, cancellation)?;
            if cancelled(cancellation) {
                return Err(cancelled_error());
            }
            resolved.env.insert(name.clone(), value.expose_for_execution().to_owned());
            resolved.secret_values.push(value.expose_for_execution().to_owned());
        }
        Ok(resolved)
    }
}

fn cancelled(cancellation: Option<&AtomicBool>) -> bool {
    cancellation.is_some_and(|flag| flag.load(Ordering::Acquire))
}

fn cancelled_error() -> CredentialError {
    CredentialError::ProviderFailure {
        identity: "credential-resolution".to_owned(),
        message: "credential_resolution_cancelled".to_owned(),
    }
}

/// Descriptive opaque identifiers derived from the already authorized binding.
pub(crate) fn authorized_subject(binding: &crate::engine::effect::authority::IntentBinding) -> CredentialSubject {
    CredentialSubject::Node {
        workflow_id: blake3::hash(binding.workflow_revision.as_str().as_bytes()).to_hex().to_string(),
        node_id: blake3::hash(binding.node.as_str().as_bytes()).to_hex().to_string(),
    }
}

/// Explicit legacy/embedder execution has no product authority to infer here.
pub(crate) fn node_subject(
    workflow: &crate::engine::workflow::Workflow,
    node: crate::engine::workflow::NodeId,
) -> Result<CredentialSubject, &'static str> {
    let revision =
        crate::orchestration::workflow_revision(workflow).map_err(|_| "credential_workflow_identity_unavailable")?;
    Ok(CredentialSubject::Node {
        workflow_id: revision.content_hash.to_string(),
        node_id: node.0.to_string(),
    })
}

#[cfg(test)]
mod tests;

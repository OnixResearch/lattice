//! One-shot Agent requests: exact authority precedes material and transport.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use zeroize::Zeroize;

use super::AgentAdapter;
use super::AgentError;
use super::AgentOutput;
use super::AgentRequest;
use super::AgentResponse;
use super::ExecutionContext;
use crate::credentials::CredentialDestination;
use crate::credentials::CredentialProvider;
use crate::credentials::CredentialSubject;
use crate::credentials::NodeCredentialRefs;
use crate::credentials::execution::Bindings;
use crate::credentials::execution::authorized_subject;
use crate::engine::effect::dispatch::AuthorizedAdapterRequest;
use crate::engine::effect::dispatch::AuthorizedAttempt;
use crate::engine::effect::dispatch::Error;

pub(crate) struct Recipe<'a> {
    request: RefCell<Option<AgentRequest>>,
    adapter: &'a dyn AgentAdapter,
    backend: String,
    bindings: Bindings,
    cancellation: &'a AtomicBool,
    payload: String,
}

impl<'a> Recipe<'a> {
    pub(crate) fn new(
        request: AgentRequest,
        adapter: &'a dyn AgentAdapter,
        references: Option<&NodeCredentialRefs>,
        provider: Arc<dyn CredentialProvider>,
        cancellation: &'a AtomicBool,
    ) -> Result<Self, String> {
        let backend =
            adapter.execution_identity(&request).map_err(|_| "agent_runtime_identity_unavailable".to_owned())?;
        let bindings = Bindings::new(references, CredentialDestination::AgentEnvironment, provider)
            .map_err(|_| "credential_binding_rejected".to_owned())?;
        let payload = bindings
            .identity(b"lattice.effect.credential-agent-plan.v1", &(&request, &backend))
            .map_err(|_| "credential_plan_identity_failed".to_owned())?;
        Ok(Self {
            request: RefCell::new(Some(request)),
            adapter,
            backend,
            bindings,
            cancellation,
            payload,
        })
    }

    pub(crate) fn payload(&self) -> &str {
        &self.payload
    }

    fn consume(&self, subject: &CredentialSubject) -> Result<AgentResponse, Error> {
        let request = self.request.borrow_mut().take().ok_or(Error::AdapterRejected)?;
        if self.adapter.execution_identity(&request).map_err(|_| Error::AdapterRejected)? != self.backend {
            return Err(Error::AdapterRejected);
        }
        let mut resolved =
            self.bindings.resolve(subject, Some(self.cancellation)).map_err(|_| Error::AdapterRejected)?;
        let context = ExecutionContext::new(std::mem::take(&mut resolved.env));
        execute_redacted(self.adapter, &request, &context).map_err(|_| Error::AdapterRejected)
    }
}

impl AuthorizedAdapterRequest for Recipe<'_> {
    type Output = AgentResponse;

    fn execute(
        &self,
        _runtime: &crate::engine::effect::runtime::Adapters<'_>,
        authorized: &AuthorizedAttempt,
    ) -> Result<Self::Output, Error> {
        let binding = &authorized.attempt.binding;
        if !authorized.authorization.allowed
            || authorized.authorization.binding.as_ref() != Some(binding)
            || binding.operation.as_str() != "lattice/agent"
            || binding.ability.as_str() != "agent/execute"
            || binding.payload.as_str() != self.payload
            || binding.request.as_str() != crate::engine::effect::runtime::dispatch_request_identity(&self.payload)
        {
            return Err(Error::AdapterRejected);
        }
        self.consume(&authorized_subject(binding))
    }
}

pub(crate) fn execute_redacted(
    adapter: &dyn AgentAdapter,
    request: &AgentRequest,
    context: &ExecutionContext,
) -> Result<AgentResponse, AgentError> {
    let mut response = adapter.execute(request, context).map_err(|error| match error {
        AgentError::UnsupportedTarget(text) => AgentError::UnsupportedTarget(context.redact(&text)),
        AgentError::PreflightRejected(text) => AgentError::PreflightRejected(context.redact(&text)),
        AgentError::Credential(text) => AgentError::Credential(context.redact(&text)),
        AgentError::Session(text) => AgentError::Session(context.redact(&text)),
        AgentError::Adapter(text) => AgentError::Adapter(context.redact(&text)),
        other => other,
    })?;
    match &mut response.output {
        AgentOutput::Text(text) => redact_string(text, context),
        AgentOutput::Json(json) => redact_json(json, context),
        AgentOutput::Artifact(bytes) => redact_bytes(bytes, context),
        AgentOutput::Image(image) => {
            redact_bytes(&mut image.bytes, context);
            redact_string(&mut image.media_type, context);
            if let Some(label) = image.metadata.source_label.as_mut() {
                redact_string(label, context);
            }
        }
    }
    redact_json(&mut response.metadata, context);
    Ok(response)
}

fn redact_string(text: &mut String, context: &ExecutionContext) {
    let redacted = context.redact(text);
    text.zeroize();
    *text = redacted;
}

fn redact_json(value: &mut serde_json::Value, context: &ExecutionContext) {
    match value {
        serde_json::Value::String(text) => redact_string(text, context),
        serde_json::Value::Array(values) => {
            for value in values {
                redact_json(value, context);
            }
        }
        serde_json::Value::Object(values) => {
            let original = std::mem::take(values);
            for (mut key, mut value) in original {
                redact_string(&mut key, context);
                redact_json(&mut value, context);
                values.insert(key, value);
            }
        }
        _ => {}
    }
}

fn redact_bytes(bytes: &mut Vec<u8>, context: &ExecutionContext) {
    // Keep arbitrary non-UTF-8 output intact while replacing exact material spans.
    for secret in &context.secret_values {
        if secret.is_empty() {
            continue;
        }
        let needle = secret.as_bytes();
        let mut offset = 0;
        while offset + needle.len() <= bytes.len() {
            if &bytes[offset..offset + needle.len()] == needle {
                bytes[offset..offset + needle.len()].zeroize();
                bytes.splice(offset..offset + needle.len(), crate::credentials::REDACTION_MARKER.bytes());
                offset += crate::credentials::REDACTION_MARKER.len();
            } else {
                offset += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests;

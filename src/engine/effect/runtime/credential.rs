//! One-shot recipes authorize public parameters and slots, never concrete secrets.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use zeroize::Zeroize;

use crate::credentials::CredentialDestination;
use crate::credentials::CredentialProvider;
use crate::credentials::CredentialSubject;
use crate::credentials::NodeCredentialRefs;
use crate::credentials::execution::Bindings;
use crate::credentials::execution::authorized_subject;
use crate::engine::effect::dispatch::AuthorizedAdapterRequest;
use crate::engine::effect::dispatch::AuthorizedAttempt;
use crate::engine::effect::dispatch::Error;

const PROCESS_DOMAIN: &[u8] = b"lattice.effect.credential-process-plan.v1";
const NETWORK_DOMAIN: &[u8] = b"lattice.effect.credential-network-plan.v1";

// Ownership transfers from ResolvedSecretEnv into these requests. Keep the
// request buffers guarded across transport errors, cancellation, and unwinding.
struct TransientProcessRequest(crate::agent_node::ProcessRequest);

impl Drop for TransientProcessRequest {
    fn drop(&mut self) {
        for value in self.0.env.values_mut() {
            value.zeroize();
        }
    }
}

struct TransientNetworkRequest(crate::http::HttpRequestPlan);

impl Drop for TransientNetworkRequest {
    fn drop(&mut self) {
        for value in self.0.headers.values_mut() {
            value.zeroize();
        }
    }
}

pub(crate) struct ProcessRecipe {
    request: RefCell<Option<crate::agent_node::ProcessRequest>>,
    bindings: Bindings,
    payload: String,
    cancellation_observed: Cell<bool>,
}

impl ProcessRecipe {
    pub(crate) fn new(
        request: crate::agent_node::ProcessRequest,
        references: Option<&NodeCredentialRefs>,
        provider: Arc<dyn CredentialProvider>,
    ) -> Result<Self, String> {
        let bindings = Bindings::new(references, CredentialDestination::ShellEnvironment, provider)
            .map_err(|_| "credential_binding_rejected".to_owned())?;
        let payload = if bindings.is_empty() {
            super::process_plan_identity(&request)
        } else {
            bindings
                .identity(
                    PROCESS_DOMAIN,
                    &(
                        &request.program,
                        &request.args,
                        &request.cwd,
                        &request.env,
                        &request.stdin,
                        request.timeout_secs,
                        request.output_limit_bytes,
                    ),
                )
                .map_err(|_| "credential_plan_identity_failed".to_owned())?
        };
        Ok(Self {
            request: RefCell::new(Some(request)),
            bindings,
            payload,
            cancellation_observed: Cell::new(false),
        })
    }

    pub(crate) fn payload(&self) -> &str {
        &self.payload
    }

    pub(crate) fn cancellation_observed(&self) -> bool {
        self.cancellation_observed.get()
    }

    fn consume(
        &self,
        process: &crate::process::core::RuntimeConfig,
        cancellation: Option<&AtomicBool>,
        subject: &CredentialSubject,
    ) -> Result<crate::agent_node::ProcessOutput, Error> {
        let mut request = TransientProcessRequest(self.request.borrow_mut().take().ok_or(Error::AdapterRejected)?);
        let mut resolved = self.bindings.resolve(subject, cancellation).map_err(|_| Error::AdapterRejected)?;
        for (name, value) in resolved.env.drain() {
            if let Some(mut replaced) = request.0.env.insert(name, value) {
                replaced.zeroize();
            }
        }
        let mut output = crate::process::shell::run_bounded(&request.0, process, cancellation).map_err(|error| {
            self.cancellation_observed.set(matches!(error, crate::agent_node::ProcessError::Cancelled));
            Error::AdapterRejected
        })?;
        if !resolved.secret_values.is_empty() {
            let stdout =
                crate::credentials::redact_text(&String::from_utf8_lossy(&output.stdout), &resolved.secret_values);
            let stderr =
                crate::credentials::redact_text(&String::from_utf8_lossy(&output.stderr), &resolved.secret_values);
            output.stdout.zeroize();
            output.stderr.zeroize();
            output.stdout = stdout.into_bytes();
            output.stderr = stderr.into_bytes();
        }
        Ok(output)
    }
}

impl AuthorizedAdapterRequest for ProcessRecipe {
    type Output = crate::agent_node::ProcessOutput;

    fn execute(&self, runtime: &super::Adapters<'_>, authorized: &AuthorizedAttempt) -> Result<Self::Output, Error> {
        if !authorized.authorization.allowed
            || authorized.attempt.binding.ability.as_str() != super::PROCESS_ABILITY
            || authorized.attempt.binding.payload.as_str() != self.payload
        {
            return Err(Error::AdapterRejected);
        }
        self.consume(runtime.process, runtime.cancellation, &authorized_subject(&authorized.attempt.binding))
    }
}

pub(crate) struct NetworkRecipe {
    request: RefCell<Option<crate::http::HttpRequestPlan>>,
    bindings: Bindings,
    payload: String,
}

impl NetworkRecipe {
    pub(crate) fn new(
        request: crate::http::HttpRequestPlan,
        references: Option<&NodeCredentialRefs>,
        provider: Arc<dyn CredentialProvider>,
    ) -> Result<Self, String> {
        let bindings = Bindings::new(references, CredentialDestination::HttpHeader, provider)
            .map_err(|_| "credential_binding_rejected".to_owned())?;
        let mut folded_names = std::collections::BTreeSet::new();
        for name in bindings.names() {
            if reqwest::header::HeaderName::from_bytes(name.as_bytes()).is_err()
                || !folded_names.insert(name.to_ascii_lowercase())
            {
                return Err("credential_header_binding_rejected".to_owned());
            }
        }
        let payload = if bindings.is_empty() {
            super::network_plan_identity(&request)
        } else {
            let headers = request.headers.iter().collect::<BTreeMap<_, _>>();
            bindings
                .identity(
                    NETWORK_DOMAIN,
                    &(&request.method, &request.url, headers, &request.body, request.timeout_secs),
                )
                .map_err(|_| "credential_plan_identity_failed".to_owned())?
        };
        Ok(Self {
            request: RefCell::new(Some(request)),
            bindings,
            payload,
        })
    }

    pub(crate) fn payload(&self) -> &str {
        &self.payload
    }

    fn consume(
        &self,
        http: &dyn crate::http::HttpClient,
        cancellation: Option<&AtomicBool>,
        subject: &CredentialSubject,
    ) -> Result<crate::http::HttpResponse, Error> {
        let mut request = TransientNetworkRequest(self.request.borrow_mut().take().ok_or(Error::AdapterRejected)?);
        if !crate::http::validate_http_method(&request.0.method) || !crate::http::validate_http_url(&request.0.url) {
            return Err(Error::AdapterRejected);
        }
        let mut resolved = self.bindings.resolve(subject, cancellation).map_err(|_| Error::AdapterRejected)?;
        for (name, value) in resolved.env.drain() {
            request.0.headers.retain(|existing, value| {
                if existing.eq_ignore_ascii_case(&name) {
                    value.zeroize();
                    false
                } else {
                    true
                }
            });
            request.0.headers.insert(name, value);
        }
        let mut response =
            http.execute_with_cancellation(&request.0, cancellation).map_err(|_| Error::AdapterRejected)?;
        if !resolved.secret_values.is_empty() {
            let body = crate::credentials::redact_text(&response.body, &resolved.secret_values);
            response.body.zeroize();
            response.body = body;
            for value in response.headers.values_mut() {
                let redacted = crate::credentials::redact_text(value, &resolved.secret_values);
                value.zeroize();
                *value = redacted;
            }
        }
        Ok(response)
    }

    pub(crate) fn consume_for_embedding(
        &self,
        http: &dyn crate::http::HttpClient,
        cancellation: &AtomicBool,
        subject: &CredentialSubject,
    ) -> Result<crate::http::HttpResponse, String> {
        self.consume(http, Some(cancellation), subject)
            .map_err(|_| "credential_network_rejected".to_owned())
    }
}

impl AuthorizedAdapterRequest for NetworkRecipe {
    type Output = crate::http::HttpResponse;

    fn execute(&self, runtime: &super::Adapters<'_>, authorized: &AuthorizedAttempt) -> Result<Self::Output, Error> {
        if !authorized.authorization.allowed
            || authorized.attempt.binding.ability.as_str() != super::NETWORK_ABILITY
            || authorized.attempt.binding.payload.as_str() != self.payload
        {
            return Err(Error::AdapterRejected);
        }
        self.consume(runtime.http, runtime.cancellation, &authorized_subject(&authorized.attempt.binding))
    }
}

#[cfg(test)]
mod tests;

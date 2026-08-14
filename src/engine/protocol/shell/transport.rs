//! Bounded HTTP transport adapter for canonical protocol envelopes.
//!
//! r[impl lattice.role_protocol.external_roles]
//! r[impl lattice.role_protocol.authorization]
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the bounded transport adapter uses its adjacent protocol DTO and adapter vocabulary"
)]

use super::super::model::AdapterLimits;
use super::super::model::Identity;
use super::super::model::MessageEnvelope;
use super::AdapterDispatch;
use super::AdapterProfile;
use super::ProtocolAdapter;

const HTTP_METHOD: &str = "POST";
const CONTENT_TYPE_HEADER: &str = "content-type";
const CONTENT_TYPE_VALUE: &str = "application/json";
const MILLISECONDS_PER_SECOND: u64 = 1_000;
const MINIMUM_TIMEOUT_SECONDS: u64 = 1;
const SUCCESS_STATUS_START: u16 = 200;
const SUCCESS_STATUS_END: u16 = 299;

pub struct HttpAdapter<'a> {
    profile: AdapterProfile,
    endpoint: String,
    client: &'a dyn crate::http::HttpClient,
}

impl<'a> HttpAdapter<'a> {
    #[must_use]
    pub fn native(
        cohort: Identity,
        runtime_route: Identity,
        endpoint: String,
        client: &'a dyn crate::http::HttpClient,
    ) -> Self {
        Self {
            profile: AdapterProfile::Native { cohort, runtime_route },
            endpoint,
            client,
        }
    }

    #[must_use]
    pub fn external(profile: ExternalProfile, endpoint: String, client: &'a dyn crate::http::HttpClient) -> Self {
        Self {
            profile: AdapterProfile::External {
                cohort: profile.cohort,
                contract: profile.contract,
                destination_policy: profile.destination_policy,
                credential_reference: profile.credential_reference,
            },
            endpoint,
            client,
        }
    }
}

pub struct ExternalProfile {
    pub cohort: Identity,
    pub contract: Identity,
    pub destination_policy: Identity,
    pub credential_reference: Identity,
}

impl ProtocolAdapter for HttpAdapter<'_> {
    fn profile(&self) -> &AdapterProfile {
        &self.profile
    }

    fn dispatch(&self, envelope: &MessageEnvelope, limits: &AdapterLimits) -> AdapterDispatch {
        if !crate::http::validate_http_url(&self.endpoint) {
            return AdapterDispatch::RejectedBeforeStart;
        }
        let body = match serde_json::to_string(envelope) {
            Ok(body) => body,
            Err(_) => return AdapterDispatch::RejectedBeforeStart,
        };
        let body_len = match u64::try_from(body.len()) {
            Ok(body_len) => body_len,
            Err(_) => return AdapterDispatch::RejectedBeforeStart,
        };
        if body_len > limits.max_envelope_bytes {
            return AdapterDispatch::RejectedBeforeStart;
        }
        let timeout_secs = limits.timeout_ms.div_ceil(MILLISECONDS_PER_SECOND).max(MINIMUM_TIMEOUT_SECONDS);
        let request = crate::http::HttpRequestPlan {
            method: HTTP_METHOD.to_owned(),
            url: self.endpoint.clone(),
            headers: std::collections::HashMap::from([(CONTENT_TYPE_HEADER.to_owned(), CONTENT_TYPE_VALUE.to_owned())]),
            body: Some(body),
            timeout_secs,
        };
        match self.client.execute(&request) {
            Ok(response) if (SUCCESS_STATUS_START..=SUCCESS_STATUS_END).contains(&response.status) => {
                AdapterDispatch::Started
            }
            Ok(_) => AdapterDispatch::ObservedFailure,
            Err(crate::http::HttpError::InvalidRequest(_)) => AdapterDispatch::RejectedBeforeStart,
            Err(crate::http::HttpError::Timeout(_) | crate::http::HttpError::Transport(_)) => {
                AdapterDispatch::UnknownAfterStart
            }
        }
    }
}

//! Native Mantle gateway client. The producer owns wire canonicalization,
//! holder resources, durable execution and signed observations.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

use basalt::ucan::TokenSigner;
use mantle_build_contract::async_api as wire;
use serde::Deserialize;
use serde::Serialize;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use zeroize::Zeroizing;

use super::core::AttemptBinding;
use super::core::EventFacts;
use super::core::Identity;
use super::core::{self};
use super::exchange::ExchangedUcan;

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
pub const MAX_EVENT_AGE_MS: u64 = 900_000;
pub const CLOCK_SKEW_MS: u64 = 30_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Mantle request policy rejected")]
    Policy,
    #[error("Mantle authority unavailable")]
    Authority,
    #[error("Mantle transport unavailable")]
    Transport,
    #[error("Mantle response exceeds bounds")]
    Bounds,
    #[error("Mantle operation rejected")]
    Rejected,
    #[error("Mantle signed observation rejected")]
    Signature,
    #[error("Mantle observation binding rejected")]
    Binding,
}

#[derive(Clone, Debug)]
pub struct Endpoint {
    pub socket: PathBuf,
    pub peer_uid: u32,
    pub service: String,
    pub project: String,
    pub owner: String,
    pub account: String,
    pub endpoint: String,
    /// Receiver's current Basalt authority generation and exact policy reference.
    pub generation: u64,
    pub authority_policy: basalt::authority_core::BasaltPolicyRef,
    /// Independently admitted builder endpoint generation in request digests.
    pub endpoint_generation: u64,
    pub signing_key: [u8; 32],
    /// Exact source cohort admitted by the composition root, not an event claim.
    pub producer: Identity,
}
impl Endpoint {
    pub fn validate(&self) -> Result<(), Error> {
        if !self.socket.is_absolute()
            || self.socket.as_os_str().len() > 100
            || !self.service.starts_with("did:")
            || self.service.len() > 256
            || !wire::safe_segment(&self.project)
            || !self.owner.starts_with("did:")
            || self.owner.len() > 512
            || self.account.is_empty()
            || self.account.len() > 512
            || self.endpoint.is_empty()
            || self.endpoint.len() > 512
            || self.generation == 0
            || self.endpoint_generation == 0
            || self.signing_key == [0; 32]
            || self.producer.0 == [0; 32]
            || [&self.service, &self.owner, &self.account, &self.endpoint]
                .iter()
                .any(|value| !value.is_ascii() || value.chars().any(char::is_control))
        {
            return Err(Error::Policy);
        }
        Ok(())
    }
}

pub struct PlannedBuild {
    pub(super) job: core::JobPlan,
    request: wire::Request,
    pub(super) native_request_hash: Identity,
    resource: String,
}
impl PlannedBuild {
    /// Submit an already acquired native Nix derivation. Native gateway policy,
    /// not workflow hints, controls execution timeout and worker selection.
    pub fn new(
        endpoint: &Endpoint,
        job: core::JobPlan,
        drv_path: String,
        derivation_json: String,
    ) -> Result<Self, Error> {
        endpoint.validate()?;
        job.validate().map_err(|_| Error::Binding)?;
        if job.submission.audience != endpoint.service
            || job.submission.project != public_identity(endpoint.project.as_bytes())
            || job.request.derivation != public_identity(derivation_json.as_bytes())
        {
            return Err(Error::Binding);
        }
        let native_request_hash = public_identity(drv_path.as_bytes());
        let request = wire::Request::Submit {
            idempotency_key: job.idempotency.hex(),
            drv_path,
        };
        validate_request(&request)?;
        let resource = canonical_resource(endpoint, &request)?;
        Ok(Self {
            job,
            request,
            native_request_hash,
            resource,
        })
    }
    pub fn request(&self) -> &wire::Request {
        &self.request
    }
    pub fn job(&self) -> &core::JobPlan {
        &self.job
    }
    pub fn native_request_hash(&self) -> Identity {
        self.native_request_hash
    }
    pub fn resource(&self) -> &str {
        &self.resource
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeAttempt {
    pub attempt: String,
    pub fence: u64,
    pub native_request_hash: Identity,
    pub binding: AttemptBinding,
}
pub fn public_identity(bytes: &[u8]) -> Identity {
    Identity(*blake3::hash(bytes).as_bytes())
}

fn validate_request(request: &wire::Request) -> Result<(), Error> {
    let bytes = serde_json::to_vec(request).map_err(|_| Error::Policy)?;
    if bytes.len() > 4096 {
        return Err(Error::Bounds);
    }
    match request {
        wire::Request::Submit {
            drv_path,
            idempotency_key,
        } => {
            if !drv_path.starts_with("/nix/store/")
                || !drv_path.ends_with(".drv")
                || drv_path[11..].contains('/')
                || !wire::safe_segment(idempotency_key)
                || drv_path.bytes().any(|byte| !byte.is_ascii_graphic())
            {
                return Err(Error::Policy);
            }
        }
        wire::Request::Events { items, .. } if *items == 0 || *items > wire::EVENT_PAGE_MAX => {
            return Err(Error::Bounds);
        }
        wire::Request::Logs { bytes, .. } if *bytes == 0 || *bytes > wire::LOG_WINDOW_MAX => return Err(Error::Bounds),
        wire::Request::Usage { .. } => return Err(Error::Policy),
        _ => {}
    }
    if request.attempt().is_some_and(|attempt| !wire::safe_segment(attempt)) {
        return Err(Error::Policy);
    }
    Ok(())
}
pub fn attempt_request(endpoint: &Endpoint, request: wire::Request) -> Result<wire::Request, Error> {
    endpoint.validate()?;
    if request.attempt().is_none() {
        return Err(Error::Policy);
    }
    validate_request(&request)?;
    Ok(request)
}
fn native_operation(request: &wire::Request) -> basalt::mantle::Operation {
    use basalt::mantle::Operation;
    match request {
        wire::Request::Submit { .. } => Operation::BuildSubmit,
        wire::Request::Status { .. } => Operation::StatusRead,
        wire::Request::Events { .. } => Operation::EventRead,
        wire::Request::Result { .. } => Operation::ResultRead,
        wire::Request::Logs { .. } => Operation::LogRead,
        wire::Request::Cancel { .. } => Operation::OwnedAttemptCancel,
        wire::Request::Usage { .. } => Operation::UsageRead,
    }
}
fn authority_request(endpoint: &Endpoint, request: &wire::Request) -> Result<basalt::mantle::Request, Error> {
    endpoint.validate()?;
    validate_request(request)?;
    Ok(basalt::mantle::Request {
        operation: native_operation(request),
        subject: endpoint.owner.clone(),
        account: endpoint.account.clone(),
        project: endpoint.project.clone(),
        endpoint: endpoint.endpoint.clone(),
        audience: endpoint.service.clone(),
        generation: endpoint.generation,
        request_digest: wire::request_digest(request, &endpoint.project, endpoint.endpoint_generation)
            .map_err(|_| Error::Policy)?,
        target: request.target(&endpoint.project).into(),
    })
}
pub fn canonical_resource(endpoint: &Endpoint, request: &wire::Request) -> Result<String, Error> {
    let authority = authority_request(endpoint, request)?;
    let exact =
        basalt::mantle::canonical_enforcement(&authority, &endpoint.authority_policy).map_err(|_| Error::Authority)?;
    Ok(exact.resource().into())
}

#[derive(Serialize)]
struct Envelope<'a> {
    version: u32,
    authority: &'a basalt::mantle::Request,
    compact_token: &'a str,
    nonce: [u8; 32],
    holder_signature: &'a [u8],
    request: &'a wire::Request,
    signed_response: bool,
}
struct FrameSize(usize);
impl std::io::Write for FrameSize {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|count| *count <= wire::REQUEST_BYTES_MAX as usize)
            .ok_or_else(|| std::io::Error::other("bounded Mantle frame"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn encode_request(envelope: &Envelope<'_>) -> Result<Zeroizing<Vec<u8>>, Error> {
    let mut size = FrameSize(0);
    serde_json::to_writer(&mut size, envelope).map_err(|_| Error::Bounds)?;
    let mut bytes = Zeroizing::new(vec![0; size.0]);
    let mut destination = bytes.as_mut_slice();
    serde_json::to_writer(&mut destination, envelope).map_err(|_| Error::Bounds)?;
    if !destination.is_empty() {
        return Err(Error::Bounds);
    }
    Ok(bytes)
}

/// Construction is private: no caller can label an unsigned DTO as verified.
pub struct VerifiedResponse {
    snapshot: wire::SignedResponse,
}
impl VerifiedResponse {
    pub fn response(&self) -> &wire::Response {
        &self.snapshot.response
    }
    pub fn status(&self) -> Option<&wire::Status> {
        self.snapshot.status.as_ref()
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TransportResponse {
    version: u32,
    response: Option<wire::SignedResponse>,
    error: Option<String>,
}

pub struct MantleClient {
    endpoint: Endpoint,
}
impl MantleClient {
    pub fn new(endpoint: Endpoint) -> Result<Self, Error> {
        endpoint.validate()?;
        Ok(Self { endpoint })
    }
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }
    pub async fn execute(
        &self,
        request: &wire::Request,
        token: &ExchangedUcan,
        holder: &dyn TokenSigner,
        nonce: [u8; 32],
        now_ms: u64,
    ) -> Result<VerifiedResponse, Error> {
        let authority = authority_request(&self.endpoint, request)?;
        if token.holder_did() != self.endpoint.owner || nonce == [0; 32] {
            return Err(Error::Binding);
        }
        let exact = basalt::mantle::canonical_enforcement(&authority, &self.endpoint.authority_policy)
            .map_err(|_| Error::Authority)?;
        let binding = basalt::ucan_enforcement::bind_enforcement_action(authority.operation.action(), &exact)
            .map_err(|_| Error::Authority)?;
        let signature = token
            .sign_invocation(holder, exact.resource(), exact.ability(), binding, nonce)
            .map_err(|_| Error::Authority)?;
        let bytes = encode_request(&Envelope {
            version: wire::API_VERSION,
            authority: &authority,
            compact_token: token.expose_for_transport(),
            nonce,
            holder_signature: &signature,
            request,
            signed_response: true,
        })?;
        let response = tokio::time::timeout(REQUEST_TIMEOUT, async {
            let mut socket =
                tokio::time::timeout(CONNECT_TIMEOUT, tokio::net::UnixStream::connect(&self.endpoint.socket))
                    .await
                    .map_err(|_| Error::Transport)?
                    .map_err(|_| Error::Transport)?;
            if socket.peer_cred().map_err(|_| Error::Transport)?.uid() != self.endpoint.peer_uid {
                return Err(Error::Authority);
            }
            socket.write_all(&(bytes.len() as u32).to_be_bytes()).await.map_err(|_| Error::Transport)?;
            socket.write_all(&bytes).await.map_err(|_| Error::Transport)?;
            let length = socket.read_u32().await.map_err(|_| Error::Transport)?;
            if length == 0 || length > wire::FRAME_BYTES_MAX {
                return Err(Error::Bounds);
            }
            let mut bytes = Zeroizing::new(vec![0; length as usize]);
            socket.read_exact(&mut bytes).await.map_err(|_| Error::Transport)?;
            let response: TransportResponse = serde_json::from_slice(&bytes).map_err(|_| Error::Binding)?;
            if response.version != wire::API_VERSION {
                return Err(Error::Binding);
            }
            match (response.response, response.error) {
                (Some(response), None) => Ok(response),
                (None, Some(_)) => Err(Error::Rejected),
                _ => Err(Error::Binding),
            }
        })
        .await
        .map_err(|_| Error::Transport)??;
        self.verify_response(request, response, now_ms)
    }
    pub fn verify_response(
        &self,
        request: &wire::Request,
        response: wire::SignedResponse,
        now_ms: u64,
    ) -> Result<VerifiedResponse, Error> {
        let digest = wire::response_digest(&response).map_err(|_| Error::Binding)?;
        let request_digest = wire::request_digest(request, &self.endpoint.project, self.endpoint.endpoint_generation)
            .map_err(|_| Error::Policy)?;
        let created_ms = response.created_seconds.checked_mul(1_000).ok_or(Error::Bounds)?;
        if response.schema != wire::SIGNED_RESPONSE_SCHEMA
            || response.service != self.endpoint.service
            || response.project != self.endpoint.project
            || response.owner != self.endpoint.owner
            || response.request_digest != request_digest
            || response.producer != self.endpoint.signing_key
            || created_ms > now_ms.saturating_add(CLOCK_SKEW_MS)
            || now_ms.saturating_sub(created_ms) > MAX_EVENT_AGE_MS
        {
            return Err(Error::Binding);
        }
        verify_signature(&self.endpoint.signing_key, &digest, &response.signature)?;
        Ok(VerifiedResponse { snapshot: response })
    }
    fn bound_status<'a>(
        &self,
        response: &'a VerifiedResponse,
        expected: &NativeAttempt,
    ) -> Result<&'a wire::Status, Error> {
        let status = response.status().ok_or(Error::Binding)?;
        if status.attempt_id != expected.attempt
            || status.observation.fence != expected.fence
            || status.request_digest != expected.native_request_hash.0
            || status.sequence == 0
            || expected.binding.attempt != public_identity(status.attempt_id.as_bytes())
            || expected.binding.fence != status.observation.fence
            || expected.binding.producer != self.endpoint.producer
        {
            return Err(Error::Binding);
        }
        Ok(status)
    }
    fn facts(&self, status: &wire::Status, expected: &NativeAttempt) -> Result<EventFacts, Error> {
        if (status.observation.phase == wire::Phase::Succeeded) != status.observation.result_evidence.is_some() {
            return Err(Error::Binding);
        }
        let event =
            wire::event_identity(status, status.observation.phase, status.sequence, status.observation.result_evidence)
                .map_err(|_| Error::Binding)?;
        let evidence = status.observation.result_evidence.map(Identity).into_iter().collect::<BTreeSet<_>>();
        Ok(EventFacts {
            binding: expected.binding,
            event: Identity(event),
            sequence: status.sequence,
            phase: phase(status.observation.phase),
            evidence,
        })
    }
    pub fn bind_submission(
        &self,
        plan: &PlannedBuild,
        response: &VerifiedResponse,
        _now_ms: u64,
    ) -> Result<(NativeAttempt, EventFacts), Error> {
        let wire::Response::Submitted { attempt_id, .. } = response.response() else {
            return Err(Error::Binding);
        };
        let status = response.status().ok_or(Error::Binding)?;
        if status.attempt_id != *attempt_id
            || status.request_digest != plan.native_request_hash.0
            || !wire::safe_segment(attempt_id)
            || status.observation.fence == 0
        {
            return Err(Error::Binding);
        }
        let native = NativeAttempt {
            attempt: attempt_id.clone(),
            fence: status.observation.fence,
            native_request_hash: plan.native_request_hash,
            binding: AttemptBinding {
                request: plan.job.request_id,
                attempt: public_identity(attempt_id.as_bytes()),
                fence: status.observation.fence,
                producer: self.endpoint.producer,
            },
        };
        let status = self.bound_status(response, &native)?;
        Ok((native.clone(), self.facts(status, &native)?))
    }
    pub fn signed_status(
        &self,
        response: &VerifiedResponse,
        expected: &NativeAttempt,
        _now_ms: u64,
    ) -> Result<EventFacts, Error> {
        match response.response() {
            wire::Response::Status(status) if response.status() == Some(status) => {}
            wire::Response::CancelRequested { attempt_id, .. } if *attempt_id == expected.attempt => {}
            _ => return Err(Error::Binding),
        }
        self.facts(self.bound_status(response, expected)?, expected)
    }
    pub fn verify_event(
        &self,
        response: &VerifiedResponse,
        event: &wire::Event,
        expected: &NativeAttempt,
    ) -> Result<EventFacts, Error> {
        let wire::Response::Events(page) = response.response() else {
            return Err(Error::Binding);
        };
        if !page.values.contains(event) {
            return Err(Error::Binding);
        }
        let current = self.bound_status(response, expected)?;
        let observation = event.observation.as_ref().ok_or(Error::Binding)?;
        if observation.fence != expected.fence
            || observation.phase != event.phase
            || event.sequence == 0
            || event.sequence > current.sequence
            || event.phase.terminal() != event.completion.is_some()
        {
            return Err(Error::Binding);
        }
        if let Some(completion) = &event.completion {
            self.verify_completion(completion, expected, event.sequence, observation)?;
        }
        let status = wire::Status {
            attempt_id: expected.attempt.clone(),
            request_digest: expected.native_request_hash.0,
            observation: observation.clone(),
            cancel_requested: current.cancel_requested,
            sequence: event.sequence,
        };
        self.facts(&status, expected)
    }
    fn verify_completion(
        &self,
        completion: &wire::Completion,
        expected: &NativeAttempt,
        sequence: u64,
        observation: &wire::Observation,
    ) -> Result<(), Error> {
        if completion.version != wire::API_VERSION
            || completion.attempt_id != expected.attempt
            || completion.request_digest != expected.native_request_hash.0
            || !completion.terminal.terminal()
            || completion.terminal != observation.phase
            || completion.result_evidence != observation.result_evidence
            || completion.sequence != sequence
            || completion.producer != self.endpoint.signing_key
            || completion.event_id != wire::completion_identity(completion).map_err(|_| Error::Binding)?
        {
            return Err(Error::Binding);
        }
        let mut message = wire::COMPLETION_DOMAIN.to_vec();
        message.extend_from_slice(&completion.event_id);
        verify_signature(&self.endpoint.signing_key, &message, &completion.signature)
    }
    pub fn log_window(&self, response: VerifiedResponse, expected: &NativeAttempt) -> Result<(Vec<u8>, String), Error> {
        self.bound_status(&response, expected)?;
        let wire::Response::Logs(window) = response.snapshot.response else {
            return Err(Error::Binding);
        };
        Ok((window.bytes, window.next_cursor))
    }
    pub fn verify_result(
        &self,
        response: &VerifiedResponse,
        expected: &NativeAttempt,
        expected_digest: Identity,
    ) -> Result<Identity, Error> {
        let status = self.bound_status(response, expected)?;
        let wire::Response::Result(Some(completion)) = response.response() else {
            return Err(Error::Binding);
        };
        if status.observation.phase != wire::Phase::Succeeded
            || status.observation.result_evidence != Some(expected_digest.0)
        {
            return Err(Error::Binding);
        }
        self.verify_completion(completion, expected, status.sequence, &status.observation)?;
        Ok(expected_digest)
    }
}
fn verify_signature(key: &[u8; 32], message: &[u8], signature: &[u8]) -> Result<(), Error> {
    let signature: &[u8; 64] = signature.try_into().map_err(|_| Error::Signature)?;
    if !basalt::ucan::verified::verify_ed25519_signature(key, message, signature).map_err(|_| Error::Signature)? {
        return Err(Error::Signature);
    }
    Ok(())
}
fn phase(value: wire::Phase) -> core::Phase {
    match value {
        wire::Phase::Queued => core::Phase::Queued,
        wire::Phase::Running => core::Phase::Running,
        wire::Phase::Succeeded => core::Phase::Succeeded,
        wire::Phase::Failed => core::Phase::Failed,
        wire::Phase::Cancelled => core::Phase::Cancelled,
    }
}

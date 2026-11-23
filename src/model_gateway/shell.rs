//! The host side of the forwarded model broker.
//!
//! It binds the Unix socket Cloud Hypervisor connects for guest-initiated
//! vsock connections to `model_vsock_port` (`<vsock socket>_<port>`), admits
//! only connections whose peer process is the VMM (`--vmm-pid`), and serves
//! one C4 frame per connection. Each request passes the local checks, then
//! C3 `Admit`, then exactly one upstream call with the host-only credential,
//! then C3 `Settle`. The host clock decides the deadline. Nothing is retried.

// r[impl lattice.vm_profile.gateway]
// r[impl lattice.vm_profile.gateway.scenario.peer]

mod upstream;

use std::io::Write;

const MAX_RECORD_BYTES: u64 = 262_144;
const FRAME_DEADLINE: std::time::Duration = std::time::Duration::from_secs(5);
const ACCEPT_POLL: std::time::Duration = std::time::Duration::from_millis(5);

/// Everything one gateway incarnation serves, admitted before it listens.
pub struct Admitted {
    pub profile_identity: String,
    pub binding: crate::vm_profile::AttemptBinding,
    pub binding_identity: String,
    pub upstream: super::core::Upstream,
}

/// Read one bounded JSON record.
fn record<T: serde::de::DeserializeOwned>(path: &std::path::Path, name: &str) -> Result<T, String> {
    let file = std::fs::File::open(path).map_err(|_| format!("{name} is unreadable"))?;
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(file, MAX_RECORD_BYTES.saturating_add(1)), &mut bytes)
        .map_err(|_| format!("{name} is unreadable"))?;
    if u64::try_from(bytes.len()).map_or(true, |length| length > MAX_RECORD_BYTES) {
        return Err(format!("{name} exceeds its byte bound"));
    }
    serde_json::from_slice(&bytes).map_err(|_| format!("{name} is malformed"))
}

/// Admit the profile, binding and upstream of one gateway, before listening.
pub fn admit(
    profile: &std::path::Path,
    binding: &std::path::Path,
    upstream: &std::path::Path,
) -> Result<Admitted, String> {
    let profile: crate::vm_profile::ExecutionProfile = record(profile, "VM execution profile")?;
    let binding: crate::vm_profile::AttemptBinding = record(binding, "VM attempt binding")?;
    let upstream: super::core::Upstream = record(upstream, "model gateway upstream")?;
    let admitted = crate::vm_profile::admit_binding(&binding, &crate::vm_profile::Expected {
        profile: &profile,
        envelope: None,
        attempt_id: None,
        fence: None,
        guest_image: None,
        environment: None,
        execution: None,
        workspace: None,
        deadline_unix_ms: None,
    })
    .map_err(|refusal| refusal.to_string())?;
    super::admission::validate_upstream(&upstream)?;
    debug_assert_eq!(admitted.profile_identity, binding.profile);
    debug_assert!(crate::vm_profile::is_identity(&admitted.binding_identity));
    Ok(Admitted {
        profile_identity: admitted.profile_identity,
        binding,
        binding_identity: admitted.binding_identity,
        upstream,
    })
}

/// The host-only credential, resolved once from the gateway's environment.
pub fn credential(upstream: &super::core::Upstream) -> Result<Option<crate::credentials::SecretValue>, String> {
    let Some(reference) = upstream.credential.as_ref() else {
        return Ok(None);
    };
    let provider =
        crate::credentials::EnvironmentCredentialProvider::from_current_env(std::slice::from_ref(&reference.key));
    crate::credentials::CredentialProvider::resolve(&provider, reference)
        .map(Some)
        .map_err(|_| "model gateway credential is unavailable".into())
}

pub fn now_ms() -> Result<u64, String> {
    let observed = crate::operation::shell::Clock::observe_epoch_millis(&crate::operation::shell::SystemClock);
    u64::try_from(observed).map_err(|_| "host clock is unavailable".into())
}

/// One gateway incarnation: its session, gate, upstream and host clock.
pub struct Server<'a, P> {
    pub session: super::admission::Session,
    pub gate: super::authority::KilnAuthorityGate<P>,
    pub endpoint: String,
    pub credential: Option<crate::credentials::SecretValue>,
    pub vmm_pid: u32,
    pub stopped: &'a std::sync::atomic::AtomicBool,
    pub clock: &'a (dyn Fn() -> Result<u64, String> + Sync),
    /// Connections dropped before any frame was read: wrong peer or probe.
    pub dropped_connections: u64,
    /// Decided replies the guest did not receive; their outcome stays with the guest.
    pub undelivered_replies: u64,
}

impl<P: super::authority::AuthorityPort> Server<'_, P> {
    /// Serve until `stopped` is set. Past the host deadline every request is
    /// refused with `deadline`, so a guest resumed after a long pause learns
    /// why; the owner (Kiln) stops the process.
    #[cfg(unix)]
    pub fn serve(&mut self, listener: &std::os::unix::net::UnixListener) -> Result<(), String> {
        listener.set_nonblocking(true).map_err(|_| "bounded gateway listener is unavailable")?;
        while !self.stopped.load(std::sync::atomic::Ordering::Acquire) {
            match listener.accept() {
                Ok((stream, _)) => self.connection(stream)?,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(ACCEPT_POLL),
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => return Err("model gateway listener failed after start".into()),
            }
        }
        Ok(())
    }

    fn is_current(&self) -> bool {
        !self.stopped.load(std::sync::atomic::Ordering::Acquire)
            && (self.clock)().is_ok_and(|now| now < self.session.bound.deadline_unix_ms)
    }

    #[cfg(unix)]
    fn connection(&mut self, mut stream: std::os::unix::net::UnixStream) -> Result<(), String> {
        stream.set_nonblocking(false).map_err(|_| "gateway connection mode is unavailable")?;
        if !peer_is_vmm(&stream, self.vmm_pid) {
            self.dropped_connections = self.dropped_connections.saturating_add(1);
            return Ok(());
        }
        stream.set_read_timeout(Some(FRAME_DEADLINE)).map_err(|_| "gateway read deadline is unavailable")?;
        stream
            .set_write_timeout(Some(FRAME_DEADLINE))
            .map_err(|_| "gateway write deadline is unavailable")?;
        let request_bound = frame_bound(crate::http::MAX_REQUEST_BODY_BYTES)?;
        // A probe closes without a frame; a partial or malformed frame names no
        // request. Neither reaches the authority or the upstream.
        let Ok(frame) = lattice_workflow_transport::read_frame(&mut stream, request_bound) else {
            self.dropped_connections = self.dropped_connections.saturating_add(1);
            return Ok(());
        };
        let Ok(request) = serde_json::from_slice::<super::core::ForwardedRequest>(&frame) else {
            self.dropped_connections = self.dropped_connections.saturating_add(1);
            return Ok(());
        };
        let reply = self.decide(request);
        let response_bound = frame_bound(crate::http::MAX_RESPONSE_BODY_BYTES)?;
        // A lost reply leaves the guest's outcome uncertain; it is never resent.
        let is_delivered = lattice_workflow_transport::encode_frame(&reply, response_bound)
            .and_then(|frame| lattice_workflow_transport::write_frame(&mut stream, &frame))
            .is_ok();
        if !is_delivered {
            self.undelivered_replies = self.undelivered_replies.saturating_add(1);
        }
        Ok(())
    }

    /// Decide one forwarded request: local checks, `Admit`, one upstream
    /// call, `Settle`, then the reply.
    pub fn decide(&mut self, request: super::core::ForwardedRequest) -> super::core::ForwardedResponse {
        let request_index = request.request_index;
        let refused =
            |refused| super::core::ForwardedResponse::Refused(super::core::Refused { request_index, refused });
        let Ok(now_ms) = (self.clock)() else {
            return refused(super::core::ForwardedRefusal::NotCurrent);
        };
        let body = match super::admission::precheck(&self.session, &request, now_ms) {
            Ok(body) => body,
            Err(code) => return refused(code),
        };
        if self.timeout_secs(now_ms).is_none() {
            return refused(super::core::ForwardedRefusal::Deadline);
        }
        super::admission::decided(&mut self.session, request_index);
        self.gate.prepare(request_index);
        let checks = crate::execution_binding::serving::gate::Checks {
            is_exact_model: true,
            is_within_budget: self.session.forwarded_requests < self.session.bound.max_requests,
            is_current: self.is_current(),
        };
        let admitted = crate::candidate_worker::broker::ModelRequestGate::admit(&mut self.gate, checks, request_index);
        let Some(ticket) = admitted else {
            return refused(self.gate.refusal().unwrap_or(super::core::ForwardedRefusal::NotCurrent));
        };
        debug_assert!(self.gate.admission().is_some(), "an admitted ticket has an authority admission");
        self.session.forwarded_requests = self.session.forwarded_requests.saturating_add(1);
        // `Admit` takes time: the deadline and the upstream timeout follow the
        // host clock after it, never the reading before it.
        // r[impl lattice.vm_profile.gateway.scenario.admit_latency]
        let Some(timeout_secs) = (self.clock)().ok().and_then(|now_ms| self.timeout_secs(now_ms)) else {
            let cancelled = super::core::SettleOutcome::Cancelled;
            let is_recorded = self.gate.settle_outcome(ticket, cancelled, None).is_ok();
            debug_assert!(self.gate.admission().is_none(), "every admission is settled once");
            // Nothing reached the upstream; a lost settlement stays uncertain on both sides.
            if is_recorded {
                return refused(super::core::ForwardedRefusal::Deadline);
            }
            return super::admission::reply(request_index, super::admission::Observed::NotSent, String::new(), false);
        };
        let (observed, body) = self.call(body, timeout_secs);
        let usage = match observed {
            super::admission::Observed::Response { .. } => {
                super::authority::usage(&crate::candidate_worker::broker::serving::response_facts(&body))
            }
            super::admission::Observed::Withheld
            | super::admission::Observed::NotSent
            | super::admission::Observed::TimedOut
            | super::admission::Observed::Cancelled
            | super::admission::Observed::TransportFailed => None,
        };
        let outcome = super::admission::settle_outcome(observed);
        let is_recorded = self.gate.settle_outcome(ticket, outcome, usage).is_ok();
        debug_assert!(self.gate.admission().is_none(), "every admission is settled once");
        super::admission::reply(request_index, observed, body, is_recorded)
    }

    /// Seconds left before the host deadline, capped by the HTTP bound; `None`
    /// when less than one second remains.
    fn timeout_secs(&self, now_ms: u64) -> Option<u64> {
        let remaining_secs = self.session.bound.deadline_unix_ms.checked_sub(now_ms)?.checked_div(1000)?;
        let timeout_secs = remaining_secs.min(crate::http::MAX_TIMEOUT_SECS);
        (timeout_secs > 0).then_some(timeout_secs)
    }
}

fn frame_bound(body_bytes: usize) -> Result<u32, String> {
    let body_bytes = u32::try_from(body_bytes).map_err(|_| "forwarded frame bound is invalid")?;
    Ok(super::core::maximum_forwarded_frame_bytes(body_bytes))
}

/// Whether the connecting process is the admitted VMM.
#[cfg(target_os = "linux")]
fn peer_is_vmm(stream: &std::os::unix::net::UnixStream, vmm_pid: u32) -> bool {
    let Ok(expected) = i32::try_from(vmm_pid) else {
        return false;
    };
    nix::sys::socket::getsockopt(stream, nix::sys::socket::sockopt::PeerCredentials)
        .is_ok_and(|credentials| credentials.pid() == expected)
}

#[cfg(all(unix, not(target_os = "linux")))]
fn peer_is_vmm(_stream: &std::os::unix::net::UnixStream, _vmm_pid: u32) -> bool {
    false
}

/// Bind the private listener socket (mode 0600); it must not exist yet.
#[cfg(unix)]
pub fn listen(path: &std::path::Path) -> Result<std::os::unix::net::UnixListener, String> {
    let listener = std::os::unix::net::UnixListener::bind(path).map_err(|_| "model gateway socket is unavailable")?;
    std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o600))
        .map_err(|_| "model gateway socket permissions are unavailable")?;
    Ok(listener)
}

/// Print one JSON line and flush it.
pub fn announce(value: &serde_json::Value) -> Result<(), String> {
    let mut output = std::io::stdout().lock();
    writeln!(output, "{value}")
        .and_then(|()| output.flush())
        .map_err(|_| "gateway output was not written".into())
}

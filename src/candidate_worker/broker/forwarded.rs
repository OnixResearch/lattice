//! Guest side of the forwarded model broker (C4). One admitted worker
//! request becomes one framed `{binding, request_index, body}` on a new
//! AF_VSOCK connection to the host gateway; the reply decides the worker's
//! response and the gate's settlement. Nothing is resent.

// r[impl lattice.vm_profile.forwarded_upstream]
// r[impl lattice.vm_profile.forwarded_upstream.scenario.no_fallback]

use std::io::Read;
use std::io::Write;

/// How long one blocking read waits before cancellation and the deadline are rechecked.
const POLL: std::time::Duration = std::time::Duration::from_millis(100);

/// Where the guest broker reaches its host gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Target {
    /// AF_VSOCK to `VMADDR_CID_HOST` at the profile's `model_vsock_port`.
    Vsock { gateway: crate::vm_profile::VsockPort },
    /// A Unix socket standing in for the CH vsock endpoint in tests.
    #[cfg(test)]
    Unix(std::path::PathBuf),
}

impl Target {
    pub(super) const fn of(forwarded: &super::Forwarded) -> Self {
        Self::Vsock {
            gateway: forwarded.vsock_port,
        }
    }

    fn connect(&self) -> Result<std::os::unix::net::UnixStream, String> {
        match self {
            Self::Vsock { gateway } => connect_vsock(*gateway),
            #[cfg(test)]
            Self::Unix(path) => std::os::unix::net::UnixStream::connect(path)
                .map_err(|_| "model gateway stand-in is unavailable".into()),
        }
    }
}

/// The host gateway is reachable before any worker spawns; a VM-bound
/// attempt never falls back to a host-process broker.
pub(super) fn probe(target: &Target) -> Result<(), String> {
    target.connect().map(drop).map_err(|reason| {
        format!("VM-bound model broker has no host model gateway: {reason}; the worker was not started")
    })
}

/// A connected AF_VSOCK stream. Only socket reads, writes, timeouts and
/// shutdown are used, which do not depend on the address family.
#[cfg(target_os = "linux")]
fn connect_vsock(gateway: crate::vm_profile::VsockPort) -> Result<std::os::unix::net::UnixStream, String> {
    let socket = nix::sys::socket::socket(
        nix::sys::socket::AddressFamily::Vsock,
        nix::sys::socket::SockType::Stream,
        nix::sys::socket::SockFlag::SOCK_CLOEXEC,
        None,
    )
    .map_err(|_| "AF_VSOCK is unavailable")?;
    let address = nix::sys::socket::VsockAddr::new(crate::vm_profile::HOST_PEER_CID, gateway.0);
    nix::sys::socket::connect(std::os::fd::AsRawFd::as_raw_fd(&socket), &address)
        .map_err(|_| "host model gateway vsock port refused the connection")?;
    Ok(std::os::unix::net::UnixStream::from(socket))
}

#[cfg(not(target_os = "linux"))]
fn connect_vsock(_gateway: crate::vm_profile::VsockPort) -> Result<std::os::unix::net::UnixStream, String> {
    Err("AF_VSOCK is unavailable on this platform".into())
}

/// What one forwarded call produced, before gate settlement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Exchange {
    Response {
        status: u16,
        body: String,
    },
    /// Nothing reached the upstream: the gateway refused, or no frame left the guest.
    NotSent,
    /// The gateway withheld the response or the outcome is unknown.
    Uncertain,
    TimedOut,
    Cancelled,
}

pub(super) struct Call<'a> {
    pub target: &'a Target,
    pub binding: &'a str,
    pub request_index: u32,
    pub body: String,
    pub timeout: std::time::Duration,
    pub stopped: &'a std::sync::atomic::AtomicBool,
}

enum Wait {
    TimedOut,
    Cancelled,
    Lost,
}

impl Call<'_> {
    /// Forward once. A failure before the frame is complete is `NotSent`;
    /// after it, a lost or malformed reply is `Uncertain`.
    pub(super) fn run(self) -> Exchange {
        let deadline = crate::operation::shell::SystemClock.observe_monotonic_instant().checked_add(self.timeout);
        let Some(deadline) = deadline else {
            return Exchange::NotSent;
        };
        let request = crate::model_gateway::core::ForwardedRequest {
            binding: self.binding.to_owned(),
            request_index: self.request_index,
            body: self.body,
        };
        let Ok(request_bound) = bound(crate::http::MAX_REQUEST_BODY_BYTES) else {
            return Exchange::NotSent;
        };
        let Ok(frame) = lattice_workflow_transport::encode_frame(&request, request_bound) else {
            return Exchange::NotSent;
        };
        let Ok(mut stream) = self.target.connect() else {
            return Exchange::NotSent;
        };
        // A frame not fully handed to the socket cannot be acted on by the gateway.
        let is_written = stream.set_write_timeout(Some(self.timeout)).is_ok()
            && stream.write_all(&frame).and_then(|()| stream.flush()).is_ok();
        if !is_written {
            return Exchange::NotSent;
        }
        match read_reply(&mut stream, deadline, self.stopped) {
            Ok(bytes) => decode(&bytes, self.request_index),
            Err(Wait::TimedOut) => Exchange::TimedOut,
            Err(Wait::Cancelled) => Exchange::Cancelled,
            Err(Wait::Lost) => Exchange::Uncertain,
        }
    }
}

fn bound(body_bytes: usize) -> Result<u32, ()> {
    u32::try_from(body_bytes)
        .map(crate::model_gateway::core::maximum_forwarded_frame_bytes)
        .map_err(drop)
}

fn decode(bytes: &[u8], request_index: u32) -> Exchange {
    let Ok(reply) = serde_json::from_slice::<crate::model_gateway::core::ForwardedResponse>(bytes) else {
        return Exchange::Uncertain;
    };
    if !reply.answers(request_index) {
        return Exchange::Uncertain;
    }
    match reply {
        crate::model_gateway::core::ForwardedResponse::Responded(value) => Exchange::Response {
            status: value.status,
            body: value.body,
        },
        crate::model_gateway::core::ForwardedResponse::Refused(_) => Exchange::NotSent,
        crate::model_gateway::core::ForwardedResponse::Uncertain(_) => Exchange::Uncertain,
    }
}

fn read_reply(
    stream: &mut std::os::unix::net::UnixStream,
    deadline: std::time::Instant,
    stopped: &std::sync::atomic::AtomicBool,
) -> Result<Vec<u8>, Wait> {
    stream.set_read_timeout(Some(POLL)).map_err(|_| Wait::Lost)?;
    let mut header = [0_u8; lattice_workflow_transport::FRAME_HEADER_BYTES];
    read_polled(stream, &mut header, deadline, stopped)?;
    let declared_bytes = u32::from_be_bytes(header);
    let maximum_bytes = bound(crate::http::MAX_RESPONSE_BODY_BYTES).map_err(|()| Wait::Lost)?;
    if declared_bytes == 0 || declared_bytes > maximum_bytes {
        return Err(Wait::Lost);
    }
    let mut body = vec![0_u8; usize::try_from(declared_bytes).map_err(|_| Wait::Lost)?];
    read_polled(stream, &mut body, deadline, stopped)?;
    Ok(body)
}

/// Fill `buffer`, rechecking cancellation and the deadline every poll.
fn read_polled(
    stream: &mut std::os::unix::net::UnixStream,
    buffer: &mut [u8],
    deadline: std::time::Instant,
    stopped: &std::sync::atomic::AtomicBool,
) -> Result<(), Wait> {
    let mut filled = 0_usize;
    debug_assert!(!buffer.is_empty(), "frames have a header and a non-empty body");
    while filled < buffer.len() {
        if stopped.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Wait::Cancelled);
        }
        if crate::operation::shell::SystemClock.observe_monotonic_instant() >= deadline {
            return Err(Wait::TimedOut);
        }
        let remaining = buffer.get_mut(filled..).ok_or(Wait::Lost)?;
        match stream.read(remaining) {
            Ok(0) => return Err(Wait::Lost),
            Ok(count) => filled = filled.checked_add(count).ok_or(Wait::Lost)?,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted
                ) => {}
            Err(_) => return Err(Wait::Lost),
        }
    }
    debug_assert_eq!(filled, buffer.len());
    Ok(())
}

impl super::wire::Handler<'_> {
    /// The gateway and binding of a forwarding broker. A profile and handler
    /// that disagree about forwarding refuse before the gate.
    pub(super) fn forwarding_route(&self) -> Result<Option<(Target, String)>, ()> {
        match (&self.forwarding, &self.profile.forwarded) {
            (Some(target), Some(forwarded)) if self.profile.credential.is_none() => {
                Ok(Some((target.clone(), forwarded.binding.clone())))
            }
            (None, None) => Ok(None),
            (Some(_), Some(_) | None) | (None, Some(_)) => Err(()),
        }
    }

    /// Settle one forwarded call with the upstream semantics of a direct call:
    /// a refused, withheld, lost or malformed reply is a transport failure.
    // r[impl lattice.vm_profile.forwarded_upstream.scenario.settlement]
    pub(super) fn settle_forwarded(
        &mut self,
        ticket: super::GateTicket,
        exchange: Exchange,
    ) -> Result<crate::http::HttpResponse, ()> {
        let (upstream, response) = match exchange {
            Exchange::Response { status, body } => {
                let facts = super::serving::response_facts(&body);
                let response = crate::http::HttpResponse {
                    status,
                    headers: [("content-type".to_owned(), content_type(&body).to_owned())].into(),
                    body,
                };
                (crate::execution_binding::serving::gate::Upstream::Responded(facts), Some(response))
            }
            Exchange::NotSent | Exchange::Uncertain => {
                (crate::execution_binding::serving::gate::Upstream::TransportFailed, None)
            }
            Exchange::TimedOut => (crate::execution_binding::serving::gate::Upstream::TimedOut, None),
            Exchange::Cancelled => (crate::execution_binding::serving::gate::Upstream::Cancelled, None),
        };
        match self.gate.settle(ticket, upstream) {
            Ok(true) => response.ok_or(()),
            Ok(false) => Err(()),
            Err(failure) => {
                self.failure = Some(failure);
                Err(())
            }
        }
    }
}

/// C4 carries no headers: a body that is not JSON but has `data:` lines is
/// an event stream; everything else is JSON.
fn content_type(body: &str) -> &'static str {
    let is_event_stream =
        serde_json::from_str::<serde_json::Value>(body).is_err() && body.lines().any(|line| line.starts_with("data:"));
    if is_event_stream {
        "text/event-stream"
    } else {
        "application/json"
    }
}

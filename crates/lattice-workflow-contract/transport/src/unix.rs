#![cfg(target_os = "linux")]
#![allow(
    unknown_lints,
    ambient_clock,
    non_trait_imports,
    reason = "socket deadlines are explicit profile inputs; named imports keep this imperative shell readable without hiding core ownership"
)]

use std::fs::Permissions;
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use nix::sys::socket::getsockopt;
use nix::sys::socket::sockopt::PeerCredentials;

use crate::ClientFailure;
use crate::ExchangeRequest;
use crate::ExchangeResponse;
use crate::FrameFailure;
use crate::SOCKET_PERMISSION_MODE;
use crate::ServeReport;
use crate::ServerFailure;
use crate::TransportLimits;
use crate::TransportProfile;
use crate::WorkflowExchangeHandler;
use crate::WorkflowExchangeService;
use crate::encode_frame;
use crate::read_frame;
use crate::validate_exchange_request;
use crate::validate_exchange_response;
use crate::validate_transport_profile;
use crate::write_frame;

#[derive(Debug, Clone)]
pub struct UnixEndpoint<'a> {
    pub socket_path: &'a Path,
    pub profile: &'a TransportProfile,
}

#[derive(Debug, Clone)]
pub struct UnixClient<'a> {
    endpoint: UnixEndpoint<'a>,
}

impl<'a> UnixClient<'a> {
    pub const fn new(endpoint: UnixEndpoint<'a>) -> Self {
        Self { endpoint }
    }

    // r[impl lattice.workflow_interchange.transport_ambiguity]
    pub fn exchange(&self, request: &ExchangeRequest) -> Result<ExchangeResponse, ClientFailure> {
        let frame = prepare_request_frame(self.endpoint.profile, request)?;
        let stream = UnixStream::connect(self.endpoint.socket_path).map_err(|_| ClientFailure::ConnectFailed)?;
        exchange_frame(stream, self.endpoint.profile, request, &frame)
    }
}

/// Exchanges one request over a stream the caller already connected.
///
/// This is the client half for supervised channels such as a Cloud Hypervisor
/// hybrid-vsock socket after its `CONNECT` handshake, or a socketpair in tests. It
/// applies the same deadlines, framing and failure classes as [`UnixClient`].
// r[impl lattice.workflow_interchange.transport_ambiguity]
// r[impl lattice.workflow_interchange.guest_listener]
pub fn exchange_connected(
    stream: UnixStream,
    profile: &TransportProfile,
    request: &ExchangeRequest,
) -> Result<ExchangeResponse, ClientFailure> {
    let frame = prepare_request_frame(profile, request)?;
    exchange_frame(stream, profile, request, &frame)
}

fn prepare_request_frame(profile: &TransportProfile, request: &ExchangeRequest) -> Result<Vec<u8>, ClientFailure> {
    validate_transport_profile(profile).map_err(|_| ClientFailure::InvalidProfile)?;
    validate_exchange_request(request).map_err(|_| ClientFailure::InvalidRequest)?;
    encode_frame(request, profile.limits.maximum_frame_bytes).map_err(|_| ClientFailure::InvalidRequest)
}

fn exchange_frame(
    mut stream: UnixStream,
    profile: &TransportProfile,
    request: &ExchangeRequest,
    frame: &[u8],
) -> Result<ExchangeResponse, ClientFailure> {
    let limits = profile.limits;
    set_stream_deadlines(&stream, limits).map_err(|_| ClientFailure::ConnectFailed)?;
    write_frame(&mut stream, frame).map_err(classify_incomplete_write)?;
    stream.shutdown(Shutdown::Write).map_err(|_| ClientFailure::UnknownAfterWrite)?;
    let body = read_frame(&mut stream, limits.maximum_frame_bytes).map_err(|_| ClientFailure::UnknownAfterWrite)?;
    let response: ExchangeResponse = serde_json::from_slice(&body).map_err(|_| ClientFailure::UnknownAfterWrite)?;
    validate_exchange_response(request, &response).map_err(|_| ClientFailure::UnknownAfterWrite)?;
    crate::validate_response_limits(profile, &response).map_err(|_| ClientFailure::UnknownAfterWrite)?;
    Ok(response)
}

/// A bounded stream source that the exchange server drives.
///
/// The server accepts one stream, observes its peer, asks the serving profile
/// whether that peer is admitted, applies the profile deadlines, and only then
/// reads a request. Implementations own socket creation and peer observation;
/// admission policy stays with the serving profile.
// r[impl lattice.workflow_interchange.guest_listener]
pub trait Listener {
    type Stream: Read + Write;
    type Peer;

    fn accept(&self) -> Result<Self::Stream, ServerFailure>;
    fn observe_peer(&self, stream: &Self::Stream) -> Result<Self::Peer, ServerFailure>;
    fn set_deadlines(&self, stream: &mut Self::Stream, limits: TransportLimits) -> Result<(), ServerFailure>;
}

/// Kernel-observed credentials of a Unix-domain peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnixPeer {
    pub user_id: u32,
}

impl UnixPeer {
    pub fn observe(stream: &UnixStream) -> Result<Self, ServerFailure> {
        let credentials = getsockopt(stream, PeerCredentials).map_err(|_| ServerFailure::PeerObservationFailed)?;
        Ok(Self {
            user_id: credentials.uid(),
        })
    }
}

impl Listener for UnixListener {
    type Stream = UnixStream;
    type Peer = UnixPeer;

    fn accept(&self) -> Result<UnixStream, ServerFailure> {
        let (stream, _address) = Self::accept(self).map_err(|_| ServerFailure::AcceptFailed)?;
        Ok(stream)
    }

    fn observe_peer(&self, stream: &UnixStream) -> Result<UnixPeer, ServerFailure> {
        UnixPeer::observe(stream)
    }

    fn set_deadlines(&self, stream: &mut UnixStream, limits: TransportLimits) -> Result<(), ServerFailure> {
        set_stream_deadlines(stream, limits)
    }
}

// r[impl lattice.workflow_interchange.transport_bounds]
// r[impl lattice.workflow_interchange.transport_peer]
pub fn serve<H: WorkflowExchangeHandler>(
    endpoint: UnixEndpoint<'_>,
    service: &mut WorkflowExchangeService<H>,
) -> Result<ServeReport, ServerFailure> {
    validate_transport_profile(endpoint.profile).map_err(|_| ServerFailure::InvalidProfile)?;
    let listener = UnixListener::bind(endpoint.socket_path).map_err(|_| ServerFailure::BindFailed)?;
    let permissions = Permissions::from_mode(SOCKET_PERMISSION_MODE);
    if std::fs::set_permissions(endpoint.socket_path, permissions).is_err() {
        let _cleanup_result = std::fs::remove_file(endpoint.socket_path);
        return Err(ServerFailure::PermissionFailed);
    }
    let report = serve_listener(&listener, endpoint.profile, service);
    let cleanup = std::fs::remove_file(endpoint.socket_path);
    if cleanup.is_err() {
        return Err(ServerFailure::PermissionFailed);
    }
    report
}

/// Serves a listener whose peers are Unix credentials, admitting the profile's user IDs.
// r[impl lattice.workflow_interchange.guest_listener]
pub fn serve_listener<L, H>(
    listener: &L,
    profile: &TransportProfile,
    service: &mut WorkflowExchangeService<H>,
) -> Result<ServeReport, ServerFailure>
where
    L: Listener<Peer = UnixPeer>,
    H: WorkflowExchangeHandler,
{
    debug_assert!(profile.limits.maximum_connections > 0);
    debug_assert!(!profile.peers.allowed_user_ids.is_empty());
    let admits_peer = |peer: &UnixPeer| profile.peers.allowed_user_ids.binary_search(&peer.user_id).is_ok();
    let admits_response = |response: &ExchangeResponse| crate::validate_response_limits(profile, response).is_ok();
    let admission = ServeAdmission {
        limits: profile.limits,
        peer: &admits_peer,
        response: &admits_response,
    };
    serve_connections(listener, &admission, service)
}

/// Serving policy shared by every listener kind: bounds, peer admission and response bounds.
pub(crate) struct ServeAdmission<'a, P> {
    pub(crate) limits: TransportLimits,
    pub(crate) peer: &'a dyn Fn(&P) -> bool,
    pub(crate) response: &'a dyn Fn(&ExchangeResponse) -> bool,
}

pub(crate) fn serve_connections<L, H>(
    listener: &L,
    admission: &ServeAdmission<'_, L::Peer>,
    service: &mut WorkflowExchangeService<H>,
) -> Result<ServeReport, ServerFailure>
where
    L: Listener,
    H: WorkflowExchangeHandler,
{
    debug_assert!(admission.limits.maximum_connections > 0);
    debug_assert!(admission.limits.read_deadline_milliseconds > 0);
    let mut report = ServeReport {
        accepted_connections: 0,
        completed_exchanges: 0,
        rejected_peers: 0,
        failed_exchanges: 0,
    };
    while report.accepted_connections < admission.limits.maximum_connections {
        let mut stream = listener.accept()?;
        report.accepted_connections = report.accepted_connections.checked_add(1).ok_or(ServerFailure::AcceptFailed)?;
        let peer = listener.observe_peer(&stream)?;
        if !(admission.peer)(&peer) {
            report.rejected_peers = report.rejected_peers.checked_add(1).ok_or(ServerFailure::PeerDenied)?;
            continue;
        }
        if listener.set_deadlines(&mut stream, admission.limits).is_err() {
            increment_failures(&mut report)?;
            continue;
        }
        if exchange_one(&mut stream, admission, service).is_err() {
            increment_failures(&mut report)?;
            continue;
        }
        report.completed_exchanges = report.completed_exchanges.checked_add(1).ok_or(ServerFailure::WriteFailed)?;
    }
    Ok(report)
}

fn exchange_one<S, P, H>(
    stream: &mut S,
    admission: &ServeAdmission<'_, P>,
    service: &mut WorkflowExchangeService<H>,
) -> Result<(), ServerFailure>
where
    S: Read + Write,
    H: WorkflowExchangeHandler,
{
    let limits = admission.limits;
    debug_assert!(limits.maximum_frame_bytes > 0);
    debug_assert!(limits.maximum_log_bytes > 0);
    let body = read_frame(stream, limits.maximum_frame_bytes).map_err(map_read_failure)?;
    let request: ExchangeRequest = serde_json::from_slice(&body).map_err(|_| ServerFailure::ReadFailed)?;
    let candidate = service.exchange(&request).map_err(|_| ServerFailure::WriteFailed)?;
    let response = if (admission.response)(&candidate) {
        candidate
    } else {
        crate::new_response(
            request.identity.clone(),
            crate::ExchangeResult::Rejected(crate::Rejection {
                code: crate::RejectionCode::LogTooLarge,
                field: "exchange.log".to_string(),
            }),
        )
        .map_err(|_| ServerFailure::WriteFailed)?
    };
    let frame = encode_frame(&response, limits.maximum_frame_bytes).map_err(map_write_failure)?;
    write_frame(stream, &frame).map_err(map_write_failure)?;
    stream.flush().map_err(|_| ServerFailure::WriteFailed)
}

fn set_stream_deadlines(stream: &UnixStream, limits: TransportLimits) -> Result<(), ServerFailure> {
    let read = Duration::from_millis(limits.read_deadline_milliseconds);
    let write = Duration::from_millis(limits.write_deadline_milliseconds);
    stream.set_read_timeout(Some(read)).map_err(|_| ServerFailure::ReadFailed)?;
    stream.set_write_timeout(Some(write)).map_err(|_| ServerFailure::WriteFailed)
}

fn increment_failures(report: &mut ServeReport) -> Result<(), ServerFailure> {
    report.failed_exchanges = report.failed_exchanges.checked_add(1).ok_or(ServerFailure::ReadFailed)?;
    Ok(())
}

// r[impl lattice.workflow_interchange.transport_ambiguity.scenario.not-accepted]
const fn classify_incomplete_write(_failure: FrameFailure) -> ClientFailure {
    ClientFailure::NotAccepted
}

fn map_read_failure(_failure: FrameFailure) -> ServerFailure {
    ServerFailure::ReadFailed
}

fn map_write_failure(_failure: FrameFailure) -> ServerFailure {
    ServerFailure::WriteFailed
}

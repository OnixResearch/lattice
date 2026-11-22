//! Guest-side AF_VSOCK exchange endpoint (`lattice.workflow-vsock-transport-profile.v1`).
//!
//! The functional part validates the closed vsock profile and peer-CID admission
//! on every platform. The Linux shell binds `VMADDR_CID_ANY:port`, observes each
//! peer CID with `getpeername`, and drives the same bounded serve loop and wire
//! frames as the Unix endpoint.

#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the vsock boundary keeps profile, admission and wire names explicit beside the shared serve loop"
)]

use crate::AdmissionError;
use crate::MAXIMUM_ALLOWED_CIDS;
use crate::VSOCK_TRANSPORT_PROFILE_SCHEMA;
use crate::VsockTransportProfile;

/// `VMADDR_CID_ANY` from the Linux vsock ABI; never an admissible peer.
pub const VSOCK_CID_ANY: u32 = u32::MAX;
/// `VMADDR_PORT_ANY` from the Linux vsock ABI; never a valid listening port.
pub const VSOCK_PORT_ANY: u32 = u32::MAX;

// r[impl lattice.workflow_interchange.vsock_profile]
pub fn validate_vsock_transport_profile(profile: &VsockTransportProfile) -> Result<(), AdmissionError> {
    if profile.schema != VSOCK_TRANSPORT_PROFILE_SCHEMA {
        return Err(AdmissionError::ProfileSchema);
    }
    if profile.port == 0 || profile.port == VSOCK_PORT_ANY {
        return Err(AdmissionError::ProfileLimit);
    }
    crate::core::validate_transport_limits(profile.limits)?;
    let allowed = &profile.peers.allowed_cids;
    let cid_count = u32::try_from(allowed.len()).map_err(|_| AdmissionError::PeerPolicy)?;
    if cid_count == 0 || cid_count > MAXIMUM_ALLOWED_CIDS {
        return Err(AdmissionError::PeerPolicy);
    }
    if !crate::core::is_strictly_ascending(allowed) || allowed.contains(&VSOCK_CID_ANY) {
        return Err(AdmissionError::PeerPolicy);
    }
    if !crate::core::declares_required_non_claims(&profile.non_claims) {
        return Err(AdmissionError::ProfileSchema);
    }
    debug_assert!(!allowed.is_empty());
    Ok(())
}

/// Returns whether a kernel-observed peer CID is in the profile's sorted allow-list.
// r[impl lattice.workflow_interchange.vsock_profile]
#[must_use]
pub fn vsock_peer_is_admitted(profile: &VsockTransportProfile, peer_cid: u32) -> bool {
    peer_cid != VSOCK_CID_ANY && profile.peers.allowed_cids.binary_search(&peer_cid).is_ok()
}

#[cfg(target_os = "linux")]
pub use shell::VsockListener;
#[cfg(target_os = "linux")]
pub use shell::VsockPeer;
#[cfg(target_os = "linux")]
pub use shell::VsockStream;
#[cfg(target_os = "linux")]
pub use shell::serve_vsock;
#[cfg(target_os = "linux")]
pub use shell::serve_vsock_listener;

#[cfg(target_os = "linux")]
#[allow(
    unknown_lints,
    ambient_clock,
    unbounded_loop,
    numeric_units,
    reason = "socket deadlines are explicit profile inputs, enforced by a bounded number of nonblocking polls"
)]
mod shell {
    use std::io;
    use std::io::Read;
    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::os::fd::OwnedFd;
    use std::os::fd::RawFd;
    use std::time::Duration;

    use nix::errno::Errno;
    use nix::sys::socket::AddressFamily;
    use nix::sys::socket::Backlog;
    use nix::sys::socket::MsgFlags;
    use nix::sys::socket::SockFlag;
    use nix::sys::socket::SockType;
    use nix::sys::socket::VsockAddr;

    use super::VSOCK_CID_ANY;
    use super::validate_vsock_transport_profile;
    use super::vsock_peer_is_admitted;
    use crate::ExchangeResponse;
    use crate::ServeReport;
    use crate::ServerFailure;
    use crate::TransportLimits;
    use crate::VsockTransportProfile;
    use crate::WorkflowExchangeHandler;
    use crate::WorkflowExchangeService;
    use crate::local::Listener;
    use crate::local::ServeAdmission;
    use crate::local::serve_connections;

    const LISTEN_BACKLOG: i32 = 16;
    const POLL_INTERVAL: Duration = Duration::from_millis(2);
    const MAXIMUM_ACCEPT_INTERRUPTS: u32 = 16;

    /// Kernel-observed address of a vsock peer.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct VsockPeer {
        pub cid: u32,
        pub port: u32,
    }

    /// A listening `AF_VSOCK` stream socket bound to `VMADDR_CID_ANY:port`.
    #[derive(Debug)]
    pub struct VsockListener {
        socket: OwnedFd,
    }

    impl VsockListener {
        // r[impl lattice.workflow_interchange.vsock_listener]
        pub fn bind(profile: &VsockTransportProfile) -> Result<Self, ServerFailure> {
            validate_vsock_transport_profile(profile).map_err(|_| ServerFailure::InvalidProfile)?;
            let socket = nix::sys::socket::socket(AddressFamily::Vsock, SockType::Stream, SockFlag::SOCK_CLOEXEC, None)
                .map_err(|_| ServerFailure::BindFailed)?;
            let address = VsockAddr::new(VSOCK_CID_ANY, profile.port);
            nix::sys::socket::bind(socket.as_raw_fd(), &address).map_err(|_| ServerFailure::BindFailed)?;
            let backlog = Backlog::new(LISTEN_BACKLOG).map_err(|_| ServerFailure::BindFailed)?;
            nix::sys::socket::listen(&socket, backlog).map_err(|_| ServerFailure::BindFailed)?;
            Ok(Self { socket })
        }
    }

    /// One accepted vsock connection.
    ///
    /// Reads and writes are nonblocking polls bounded by the deadlines the serve
    /// loop applies; before deadlines are set every read and write is refused.
    #[derive(Debug)]
    pub struct VsockStream {
        descriptor: RawFd,
        read_deadline: Option<Duration>,
        write_deadline: Option<Duration>,
    }

    impl Drop for VsockStream {
        fn drop(&mut self) {
            let _close_result = nix::unistd::close(self.descriptor);
        }
    }

    impl Read for VsockStream {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let deadline = self.read_deadline.ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
            for _poll in 0..poll_count(deadline) {
                match nix::sys::socket::recv(self.descriptor, buffer, MsgFlags::MSG_DONTWAIT) {
                    Ok(count) => return Ok(count),
                    Err(Errno::EAGAIN | Errno::EINTR) => std::thread::sleep(POLL_INTERVAL),
                    Err(errno) => return Err(io::Error::from(errno)),
                }
            }
            Err(io::Error::from(io::ErrorKind::TimedOut))
        }
    }

    impl Write for VsockStream {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            let deadline = self.write_deadline.ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
            let flags = MsgFlags::MSG_DONTWAIT | MsgFlags::MSG_NOSIGNAL;
            for _poll in 0..poll_count(deadline) {
                match nix::sys::socket::send(self.descriptor, buffer, flags) {
                    Ok(count) => return Ok(count),
                    Err(Errno::EAGAIN | Errno::EINTR) => std::thread::sleep(POLL_INTERVAL),
                    Err(errno) => return Err(io::Error::from(errno)),
                }
            }
            Err(io::Error::from(io::ErrorKind::TimedOut))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // At least one poll; the deadline is honoured to within one poll interval.
    fn poll_count(deadline: Duration) -> u128 {
        deadline.as_millis().div_ceil(POLL_INTERVAL.as_millis()).saturating_add(1)
    }

    impl Listener for VsockListener {
        type Stream = VsockStream;
        type Peer = VsockPeer;

        fn accept(&self) -> Result<VsockStream, ServerFailure> {
            for _interrupt in 0..MAXIMUM_ACCEPT_INTERRUPTS {
                match nix::sys::socket::accept4(self.socket.as_raw_fd(), SockFlag::SOCK_CLOEXEC) {
                    Ok(descriptor) => {
                        return Ok(VsockStream {
                            descriptor,
                            read_deadline: None,
                            write_deadline: None,
                        });
                    }
                    Err(Errno::EINTR) => {}
                    Err(_) => return Err(ServerFailure::AcceptFailed),
                }
            }
            Err(ServerFailure::AcceptFailed)
        }

        fn observe_peer(&self, stream: &VsockStream) -> Result<VsockPeer, ServerFailure> {
            let address: VsockAddr =
                nix::sys::socket::getpeername(stream.descriptor).map_err(|_| ServerFailure::PeerObservationFailed)?;
            Ok(VsockPeer {
                cid: address.cid(),
                port: address.port(),
            })
        }

        fn set_deadlines(&self, stream: &mut VsockStream, limits: TransportLimits) -> Result<(), ServerFailure> {
            stream.read_deadline = Some(Duration::from_millis(limits.read_deadline_milliseconds));
            stream.write_deadline = Some(Duration::from_millis(limits.write_deadline_milliseconds));
            Ok(())
        }
    }

    /// Binds `VMADDR_CID_ANY:profile.port` and serves the profile's finite connection count.
    // r[impl lattice.workflow_interchange.vsock_listener]
    pub fn serve_vsock<H: WorkflowExchangeHandler>(
        profile: &VsockTransportProfile,
        service: &mut WorkflowExchangeService<H>,
    ) -> Result<ServeReport, ServerFailure> {
        let listener = VsockListener::bind(profile)?;
        serve_vsock_listener(&listener, profile, service)
    }

    /// Serves any listener whose peers are vsock addresses, admitting only the profile's CIDs.
    // r[impl lattice.workflow_interchange.vsock_listener]
    pub fn serve_vsock_listener<L, H>(
        listener: &L,
        profile: &VsockTransportProfile,
        service: &mut WorkflowExchangeService<H>,
    ) -> Result<ServeReport, ServerFailure>
    where
        L: Listener<Peer = VsockPeer>,
        H: WorkflowExchangeHandler,
    {
        validate_vsock_transport_profile(profile).map_err(|_| ServerFailure::InvalidProfile)?;
        let admits_peer = |peer: &VsockPeer| vsock_peer_is_admitted(profile, peer.cid);
        let admits_response =
            |response: &ExchangeResponse| crate::core::validate_log_limit(profile.limits, response).is_ok();
        let admission = ServeAdmission {
            limits: profile.limits,
            peer: &admits_peer,
            response: &admits_response,
        };
        serve_connections(listener, &admission, service)
    }
}

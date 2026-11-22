use std::collections::VecDeque;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use nix::sys::socket::setsockopt;
use nix::sys::socket::sockopt::SndBuf;

use super::reconnect::vsock_profile;
use super::*;

const TEST_ALLOWED_CID: u32 = 2;
const TEST_DENIED_CID: u32 = 9;
const TEST_PEER_PORT: u32 = 1_024;
const TEST_WRITE_DEADLINE_MILLISECONDS: u64 = 100;
const TEST_MINIMUM_SEND_BUFFER_BYTES: usize = 1;
const TEST_LARGE_LOG_BYTES: usize = 4_096;

/// A stream from a socketpair with a peer identity chosen by the test.
struct PairStream<P> {
    inner: UnixStream,
    peer: P,
}

impl<P> Read for PairStream<P> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buffer)
    }
}

impl<P> Write for PairStream<P> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buffer)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// A finite listener over pre-connected socketpairs; it exercises the same serve loop
/// as the Unix and vsock listeners without binding a socket.
struct PairListener<P> {
    pending: Mutex<VecDeque<PairStream<P>>>,
}

impl<P> PairListener<P> {
    fn new(streams: Vec<(UnixStream, P)>) -> Self {
        let pending = streams.into_iter().map(|(inner, peer)| PairStream { inner, peer }).collect();
        Self {
            pending: Mutex::new(pending),
        }
    }
}

impl<P: Copy> Listener for PairListener<P> {
    type Stream = PairStream<P>;
    type Peer = P;

    fn accept(&self) -> Result<PairStream<P>, ServerFailure> {
        let mut pending = self.pending.lock().map_err(|_| ServerFailure::AcceptFailed)?;
        pending.pop_front().ok_or(ServerFailure::AcceptFailed)
    }

    fn observe_peer(&self, stream: &PairStream<P>) -> Result<P, ServerFailure> {
        Ok(stream.peer)
    }

    fn set_deadlines(&self, stream: &mut PairStream<P>, limits: TransportLimits) -> Result<(), ServerFailure> {
        let read = Duration::from_millis(limits.read_deadline_milliseconds);
        let write = Duration::from_millis(limits.write_deadline_milliseconds);
        stream.inner.set_read_timeout(Some(read)).map_err(|_| ServerFailure::ReadFailed)?;
        stream.inner.set_write_timeout(Some(write)).map_err(|_| ServerFailure::WriteFailed)
    }
}

fn vsock_peer(cid: u32) -> VsockPeer {
    VsockPeer {
        cid,
        port: TEST_PEER_PORT,
    }
}

// r[verify lattice.workflow_interchange.vsock_listener]
// r[verify lattice.workflow_interchange.vsock_listener.scenario.denied]
#[test]
fn vsock_serving_admits_only_allowed_peer_cids_before_handler_invocation() {
    let (allowed_client, allowed_server) = UnixStream::pair().expect("allowed socketpair opens");
    let (denied_client, denied_server) = UnixStream::pair().expect("denied socketpair opens");
    let mut profile = vsock_profile(vec![TEST_ALLOWED_CID]);
    profile.limits.maximum_connections = 2;
    let listener = PairListener::new(vec![
        (denied_server, vsock_peer(TEST_DENIED_CID)),
        (allowed_server, vsock_peer(TEST_ALLOWED_CID)),
    ]);
    let client_profile = transport_profile(TEST_PROFILE_USER_ID);
    let client = thread::spawn(move || exchange_connected(allowed_client, &client_profile, &dispatch_request()));

    let mut service = WorkflowExchangeService::new(FixtureHandler { calls: 0 });
    let report = serve_vsock_listener(&listener, &profile, &mut service).expect("vsock serve loop finishes");
    let response = client.join().expect("client joins").expect("allowed peer completes an exchange");
    drop(denied_client);

    assert!(matches!(
        response.result,
        ExchangeResult::Success(success) if matches!(success.as_ref(), ExchangeSuccess::Accepted(_))
    ));
    assert_eq!(report.rejected_peers, 1);
    assert_eq!(report.completed_exchanges, 1);
    assert_eq!(service.handler().calls, 1);
}

#[test]
fn vsock_serving_refuses_an_invalid_profile_before_accepting() {
    let listener = PairListener::<VsockPeer>::new(Vec::new());
    let profile = vsock_profile(Vec::new());
    let mut service = WorkflowExchangeService::new(FixtureHandler { calls: 0 });
    assert_eq!(serve_vsock_listener(&listener, &profile, &mut service), Err(ServerFailure::InvalidProfile));
    assert_eq!(VsockListener::bind(&profile).map(|_| ()), Err(ServerFailure::InvalidProfile));
}

#[derive(Debug)]
struct LargeLogHandler;

impl WorkflowExchangeHandler for LargeLogHandler {
    fn handle(&mut self, command: &ExchangeCommand) -> Result<ExchangeSuccess, HandlerError> {
        let ExchangeCommand::ReadLog { request, accepted, .. } = command else {
            return Err(HandlerError::Rejected);
        };
        let mut observation = observation(request, accepted);
        for reference in &mut observation.logs {
            reference.bytes = u64::try_from(TEST_LARGE_LOG_BYTES).map_err(|_| HandlerError::Rejected)?;
        }
        observation.identity = observation_identity(&observation);
        Ok(ExchangeSuccess::Log {
            observation,
            bytes: vec![b'x'; TEST_LARGE_LOG_BYTES],
        })
    }
}

fn large_log_request() -> ExchangeRequest {
    let profile = contract_profile();
    let request = workflow_request(&profile);
    let accepted = accepted(&request);
    let log = observation(&request, &accepted).logs.into_iter().next().expect("test log exists");
    let log_read = LogReadRequest {
        schema: LOG_READ_SCHEMA.to_string(),
        request_identity: request.identity.clone(),
        workflow_identity: accepted.workflow_identity.clone(),
        execution_identity: accepted.execution_identity.clone(),
        node_id: log.node_id,
        stream: log.stream,
        cursor: log.cursor,
        maximum_bytes: TEST_MAXIMUM_LOG_READ_BYTES,
    };
    new_request(ExchangeCommand::ReadLog {
        profile,
        request,
        accepted,
        log_read,
    })
    .expect("log read request is admitted")
}

// r[verify lattice.workflow_interchange.transport_bounds.scenario.timeout]
// r[verify lattice.workflow_interchange.guest_listener]
// r[verify lattice.workflow_interchange.guest_listener.scenario.unix]
#[test]
fn server_write_deadline_stops_a_peer_that_never_reads_the_response() {
    let (mut client, server) = UnixStream::pair().expect("socketpair opens");
    setsockopt(&server, SndBuf, &TEST_MINIMUM_SEND_BUFFER_BYTES).expect("server send buffer shrinks");
    let user_id = nix::unistd::getuid().as_raw();
    let mut profile = transport_profile(user_id);
    profile.limits.write_deadline_milliseconds = TEST_WRITE_DEADLINE_MILLISECONDS;
    let frame =
        encode_frame(&large_log_request(), profile.limits.maximum_frame_bytes).expect("request frame is bounded");
    write_frame(&mut client, &frame).expect("request is written");
    client.shutdown(std::net::Shutdown::Write).expect("request half closes");

    let listener = PairListener::new(vec![(server, UnixPeer { user_id })]);
    let mut service = WorkflowExchangeService::new(LargeLogHandler);
    let report = serve_listener(&listener, &profile, &mut service).expect("serve loop finishes");
    drop(client);

    assert_eq!(report.failed_exchanges, 1);
    assert_eq!(report.completed_exchanges, 0);
}

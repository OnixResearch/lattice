//! Guest workflow-exchange reconnect qualification (Q1).
//!
//! Every test drives the real `ApplicationHandler`, a version-two handler profile
//! with a vsock listener, the real vsock serve loop with peer-CID admission, and
//! the real frame codec and client classification. The only substitution is the
//! socket: a fault-injecting `Listener` hands the server one end of a Unix
//! socketpair and reports the configured peer CID, because host-side AF_VSOCK
//! (`vsock_loopback`) is not available here. Real vsock is qualified by Kiln.
#![cfg(target_os = "linux")]

use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use lattice::agent_node::RuntimeOptions;
use lattice::engine::workflow::Workflow;
use lattice::engine::workflow::WorkflowTimestamp;
use lattice::store::Store;
use lattice::workflow_exchange::ApplicationHandler;
use lattice::workflow_exchange::EffectAuthorityMode;
use lattice::workflow_exchange::HANDLER_PROFILE_V2_SCHEMA;
use lattice::workflow_exchange::HandlerControls;
use lattice::workflow_exchange::HandlerListener;
use lattice::workflow_exchange::HandlerProfileV2;
use lattice::workflow_exchange::RouteNode;
use lattice::workflow_exchange::SourceBindingMode;
use lattice::workflow_exchange::WorkflowRoute;
use lattice_workflow_contract::*;
use lattice_workflow_transport::*;

const HOST_CID: u32 = 2;
const STRANGER_CID: u32 = 7;
const PEER_PORT: u32 = 1_024;
const VSOCK_PORT: u32 = 10_300;
const MAXIMUM_CONNECTIONS: u32 = 256;
const ACTIVE_EXECUTIONS: u32 = 2;
const FRAME_BYTES: u32 = 1_048_576;
const LOG_BYTES: u32 = 65_536;
const DEADLINE_MILLISECONDS: u64 = 5_000;
const SHORT_DEADLINE_MILLISECONDS: u64 = 500;
const SLOW_RESPONSE: Duration = Duration::from_millis(2_000);
const MAXIMUM_LOG_REFERENCES: u32 = 8;
const MAXIMUM_LOG_READ_BYTES: u64 = 4_096;
const MAXIMUM_TOTAL_LOG_BYTES: u64 = 16_384;
const IDENTITY_DIGITS: usize = 64;
const WORKFLOW_NAME: &str = "guest-exchange";
const QUICK_COMMAND: &str = "printf exchange";
const SLOW_COMMAND: &str = "sleep 3";
/// `{dir}` is replaced with the fixture directory; the child publishes its PID before sleeping.
const CANCELLED_COMMAND: &str = "printf '%s' $$ > '{dir}/started-pid'; exec sleep 30";
const STALE_WORKER_GRACE: Duration = Duration::from_millis(4_500);
const POLL_ATTEMPTS: usize = 150;
const RECOVERY_ATTEMPTS: usize = 20;
const POLL_DELAY: Duration = Duration::from_millis(20);
const FRAME_HEADER: usize = 4;

fn identity(digit: char) -> Identity {
    Identity::new(format!("b3:{}", digit.to_string().repeat(IDENTITY_DIGITS))).expect("test identity is valid")
}

fn label(value: &str) -> Label {
    Label::new(value.to_string()).expect("test label is valid")
}

fn contract_profile() -> CapabilityProfile {
    CapabilityProfile {
        schema: PROFILE_SCHEMA.to_string(),
        graph_schema: GRAPH_SCHEMA.to_string(),
        platforms: vec![label("linux-x86_64")],
        node_kinds: vec![label("build")],
        effect_classes: vec![EffectClass::Build],
        adapter_kinds: vec![label("local")],
        controls: ControlCapabilities {
            cancel: true,
            pause: true,
            resume: true,
        },
        limits: ContractLimits {
            maximum_nodes: 8,
            maximum_edges: 16,
            maximum_log_references: MAXIMUM_LOG_REFERENCES,
            maximum_log_read_bytes: MAXIMUM_LOG_READ_BYTES,
            maximum_total_log_bytes: MAXIMUM_TOTAL_LOG_BYTES,
        },
        non_claims: lattice_workflow_contract::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    }
}

fn workflow_request(profile: &CapabilityProfile, attempt: char) -> WorkflowRequest {
    let mut graph = WorkflowGraph {
        schema: GRAPH_SCHEMA.to_string(),
        identity: identity('0'),
        nodes: vec![WorkflowNode {
            id: label("build"),
            kind: label("build"),
            effect_class: EffectClass::Build,
            adapter_kind: label("local"),
            operation_identity: identity('1'),
        }],
        edges: Vec::new(),
    };
    graph.identity = graph_identity(&graph);
    let mut request = WorkflowRequest {
        schema: REQUEST_SCHEMA.to_string(),
        identity: identity('0'),
        idempotency_key: identity('2'),
        profile_identity: profile_identity(profile),
        platform: label("linux-x86_64"),
        kiln: KilnLinkage {
            effect_identity: identity('3'),
            attempt_identity: identity(attempt),
            candidate_identity: identity('5'),
            pipeline_revision_identity: identity('6'),
            workflow_policy_identity: identity('7'),
        },
        source: None,
        graph,
        controls: RequestedControls {
            cancel: true,
            pause: true,
            resume: true,
        },
        observation: ObservationPolicy {
            maximum_log_references: MAXIMUM_LOG_REFERENCES,
            maximum_log_read_bytes: MAXIMUM_LOG_READ_BYTES,
            maximum_total_log_bytes: MAXIMUM_TOTAL_LOG_BYTES,
        },
    };
    request.identity = lattice_workflow_contract::request_identity(&request);
    request
}

fn limits(deadline_milliseconds: u64) -> TransportLimits {
    TransportLimits {
        maximum_frame_bytes: FRAME_BYTES,
        maximum_log_bytes: LOG_BYTES,
        maximum_connections: MAXIMUM_CONNECTIONS,
        read_deadline_milliseconds: deadline_milliseconds,
        write_deadline_milliseconds: deadline_milliseconds,
    }
}

fn vsock_profile(deadline_milliseconds: u64) -> VsockTransportProfile {
    VsockTransportProfile {
        schema: VSOCK_TRANSPORT_PROFILE_SCHEMA.to_string(),
        port: VSOCK_PORT,
        limits: limits(deadline_milliseconds),
        peers: VsockPeerPolicy {
            allowed_cids: vec![HOST_CID],
        },
        non_claims: lattice_workflow_transport::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    }
}

/// The host gateway's client profile: same bounds as the guest listener.
fn client_profile(deadline_milliseconds: u64) -> TransportProfile {
    TransportProfile {
        schema: TRANSPORT_PROFILE_SCHEMA.to_string(),
        limits: limits(deadline_milliseconds),
        peers: PeerPolicy {
            allowed_user_ids: vec![nix::unistd::getuid().as_raw()],
        },
        non_claims: lattice_workflow_transport::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    }
}

struct Fixture {
    directory: tempfile::TempDir,
    store: Store,
    contract: CapabilityProfile,
    request: WorkflowRequest,
    profile: HandlerProfileV2,
}

fn fixture(command: &str, deadline_milliseconds: u64) -> Fixture {
    let directory = tempfile::tempdir().expect("temporary directory exists");
    let store = Store::new(directory.path().join("lattice.redb")).expect("fixture store opens");
    let mut workflow = Workflow::new(WORKFLOW_NAME, WorkflowTimestamp::epoch());
    let workflow_node =
        workflow.add_shell_node("build", command.replace("{dir}", &directory.path().display().to_string()));
    store.save_workflow(&workflow).expect("fixture workflow saves");
    let revision = lattice::orchestration::workflow_revision(&workflow).expect("workflow revision exists");
    let workflow_revision = Identity::new(format!("b3:{}", revision.content_hash)).expect("revision is canonical");
    let contract = contract_profile();
    let request = workflow_request(&contract, '4');
    let profile = HandlerProfileV2 {
        schema: HANDLER_PROFILE_V2_SCHEMA.to_string(),
        listener: HandlerListener::Vsock {
            transport: vsock_profile(deadline_milliseconds),
        },
        maximum_active_executions: ACTIVE_EXECUTIONS,
        controls: HandlerControls::default(),
        routes: vec![WorkflowRoute {
            profile_identity: request.profile_identity.clone(),
            graph_identity: request.graph.identity.clone(),
            workflow_name: WORKFLOW_NAME.to_string(),
            workflow_revision,
            effect_authority: EffectAuthorityMode::RouteExact,
            source_binding: SourceBindingMode::Unavailable,
            nodes: vec![RouteNode {
                request_node_id: label("build"),
                operation_identity: identity('1'),
                workflow_node_id: workflow_node.0,
            }],
        }],
        non_claims: lattice::workflow_exchange::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    };
    Fixture {
        directory,
        store,
        contract,
        request,
        profile,
    }
}

/// Fault applied to the guest side of one connection.
#[derive(Debug, Clone, Copy)]
enum Fault {
    None,
    /// The guest closes the connection before reading any request byte.
    CloseBeforeRead,
    /// The guest handles the request but its response never reaches the host.
    DropResponse,
    /// The guest's response arrives with one body byte corrupted.
    CorruptResponse,
    /// The guest's response is cut off part way through the frame.
    TruncateResponse,
    /// The guest's response is delayed past the host's read deadline.
    DelayResponse,
}

struct FaultStream {
    inner: UnixStream,
    peer: VsockPeer,
    fault: Fault,
    written: usize,
    captured: Arc<Mutex<Vec<u8>>>,
    closed: mpsc::SyncSender<()>,
}

impl Read for FaultStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if matches!(self.fault, Fault::CloseBeforeRead) {
            return Err(std::io::Error::from(std::io::ErrorKind::ConnectionReset));
        }
        self.inner.read(buffer)
    }
}

impl Write for FaultStream {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.captured.lock().expect("capture lock").extend_from_slice(buffer);
        let count = match self.fault {
            Fault::None | Fault::CloseBeforeRead => self.inner.write(buffer)?,
            Fault::DropResponse => buffer.len(),
            Fault::CorruptResponse => {
                let mut corrupted = buffer.to_vec();
                if let Some(byte) = FRAME_HEADER.checked_sub(self.written).and_then(|index| corrupted.get_mut(index)) {
                    *byte = b'!';
                }
                self.inner.write(&corrupted)?
            }
            Fault::TruncateResponse => {
                let half = buffer.len() / 2;
                self.inner.write_all(&buffer[..half])?;
                return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
            }
            Fault::DelayResponse => {
                thread::sleep(SLOW_RESPONSE);
                self.inner.write(buffer)?
            }
        };
        self.written = self.written.saturating_add(count);
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl Drop for FaultStream {
    fn drop(&mut self) {
        // Shut the socket down before signalling: `inner` is only closed after this body returns,
        // and a concurrently forked test child may briefly hold a duplicate of its descriptor.
        let _shutdown = self.inner.shutdown(std::net::Shutdown::Both);
        let _signal = self.closed.try_send(());
    }
}

/// A listener fed by the test: each connection is a socketpair end with a chosen peer CID and
/// fault.
struct FaultListener {
    incoming: mpsc::Receiver<FaultStream>,
}

impl Listener for FaultListener {
    type Stream = FaultStream;
    type Peer = VsockPeer;

    fn accept(&self) -> Result<FaultStream, ServerFailure> {
        self.incoming.recv().map_err(|_| ServerFailure::AcceptFailed)
    }

    fn observe_peer(&self, stream: &FaultStream) -> Result<VsockPeer, ServerFailure> {
        Ok(stream.peer)
    }

    fn set_deadlines(&self, stream: &mut FaultStream, limits: TransportLimits) -> Result<(), ServerFailure> {
        let read = Duration::from_millis(limits.read_deadline_milliseconds);
        let write = Duration::from_millis(limits.write_deadline_milliseconds);
        stream.inner.set_read_timeout(Some(read)).map_err(|_| ServerFailure::ReadFailed)?;
        stream.inner.set_write_timeout(Some(write)).map_err(|_| ServerFailure::WriteFailed)
    }
}

type Served = (Result<ServeReport, ServerFailure>, WorkflowExchangeService<ApplicationHandler>);

/// One guest exchange endpoint incarnation: a real handler behind the vsock serve loop.
struct Guest {
    connections: mpsc::SyncSender<FaultStream>,
    server: thread::JoinHandle<Served>,
    client: TransportProfile,
    used: u32,
}

struct Connection {
    client: UnixStream,
    captured: Arc<Mutex<Vec<u8>>>,
    closed: mpsc::Receiver<()>,
}

impl Guest {
    fn start(store: Store, profile: HandlerProfileV2) -> Self {
        let HandlerListener::Vsock { transport } = profile.listener.clone() else {
            panic!("guest fixture uses a vsock listener");
        };
        let client = client_profile(transport.limits.read_deadline_milliseconds);
        let (connections, incoming) = mpsc::sync_channel(MAXIMUM_CONNECTIONS as usize);
        let server = thread::spawn(move || {
            let handler = ApplicationHandler::new(store, profile, RuntimeOptions::default()).expect("handler starts");
            let mut service = WorkflowExchangeService::new(handler);
            let listener = FaultListener { incoming };
            let report = serve_vsock_listener(&listener, &transport, &mut service);
            (report, service)
        });
        Self {
            connections,
            server,
            client,
            used: 0,
        }
    }

    fn connect(&mut self, cid: u32, fault: Fault) -> Connection {
        let (client, server) = UnixStream::pair().expect("socketpair opens");
        let captured = Arc::new(Mutex::new(Vec::new()));
        let (closed_sender, closed) = mpsc::sync_channel(1);
        let stream = FaultStream {
            inner: server,
            peer: VsockPeer { cid, port: PEER_PORT },
            fault,
            written: 0,
            captured: Arc::clone(&captured),
            closed: closed_sender,
        };
        self.connections.send(stream).expect("guest accepts connections");
        self.used = self.used.checked_add(1).expect("connection count stays bounded");
        Connection {
            client,
            captured,
            closed,
        }
    }

    fn exchange(&mut self, fault: Fault, request: &ExchangeRequest) -> Result<ExchangeResponse, ClientFailure> {
        let connection = self.connect(HOST_CID, fault);
        exchange_connected(connection.client, &self.client, request)
    }

    fn exchange_captured(
        &mut self,
        fault: Fault,
        request: &ExchangeRequest,
    ) -> (Result<ExchangeResponse, ClientFailure>, Vec<u8>) {
        let connection = self.connect(HOST_CID, fault);
        let result = exchange_connected(connection.client, &self.client, request);
        let _closed = connection.closed.recv();
        let captured = connection.captured.lock().expect("capture lock").clone();
        (result, captured)
    }

    /// Ends the serve loop by spending the remaining connection budget on refused peers.
    fn stop(mut self) -> (ServeReport, WorkflowExchangeService<ApplicationHandler>) {
        while self.used < MAXIMUM_CONNECTIONS {
            let _refused = self.connect(STRANGER_CID, Fault::None);
        }
        let (report, service) = self.server.join().expect("guest serve thread joins");
        (report.expect("guest serve loop finishes"), service)
    }
}

fn dispatch(fixture: &Fixture) -> ExchangeRequest {
    new_request(ExchangeCommand::Dispatch {
        profile: fixture.contract.clone(),
        request: fixture.request.clone(),
    })
    .expect("dispatch envelope is valid")
}

fn recover(fixture: &Fixture) -> ExchangeRequest {
    new_request(ExchangeCommand::RecoverDispatch {
        profile: fixture.contract.clone(),
        request: fixture.request.clone(),
    })
    .expect("recovery envelope is valid")
}

fn observe(fixture: &Fixture, accepted: &WorkflowAccepted) -> ExchangeRequest {
    new_request(ExchangeCommand::Observe {
        profile: fixture.contract.clone(),
        request: fixture.request.clone(),
        accepted: accepted.clone(),
    })
    .expect("observe envelope is valid")
}

fn cancel(fixture: &Fixture, accepted: &WorkflowAccepted, sequence: u64) -> ExchangeRequest {
    let control = ControlRequest {
        schema: CONTROL_SCHEMA.to_string(),
        request_identity: fixture.request.identity.clone(),
        workflow_identity: accepted.workflow_identity.clone(),
        execution_identity: accepted.execution_identity.clone(),
        expected_sequence: sequence,
        action: ControlAction::Cancel,
    };
    new_request(ExchangeCommand::Control {
        profile: fixture.contract.clone(),
        request: fixture.request.clone(),
        accepted: accepted.clone(),
        current_sequence: sequence,
        control,
    })
    .expect("cancel envelope is valid")
}

fn read_log(fixture: &Fixture, accepted: &WorkflowAccepted, reference: &LogReference) -> ExchangeRequest {
    let log_read = LogReadRequest {
        schema: LOG_READ_SCHEMA.to_string(),
        request_identity: fixture.request.identity.clone(),
        workflow_identity: accepted.workflow_identity.clone(),
        execution_identity: accepted.execution_identity.clone(),
        node_id: reference.node_id.clone(),
        stream: reference.stream,
        cursor: reference.cursor,
        maximum_bytes: MAXIMUM_LOG_READ_BYTES,
    };
    new_request(ExchangeCommand::ReadLog {
        profile: fixture.contract.clone(),
        request: fixture.request.clone(),
        accepted: accepted.clone(),
        log_read,
    })
    .expect("log envelope is valid")
}

fn accepted(response: ExchangeResponse) -> WorkflowAccepted {
    match response.result {
        ExchangeResult::Success(success) => match *success {
            ExchangeSuccess::Accepted(accepted) => accepted,
            other => panic!("expected acceptance, got {other:?}"),
        },
        ExchangeResult::Rejected(rejection) => panic!("expected acceptance, got {rejection:?}"),
    }
}

fn observed(response: ExchangeResponse) -> ExecutionObservation {
    match response.result {
        ExchangeResult::Success(success) => match *success {
            ExchangeSuccess::Observation(observation) => observation,
            other => panic!("expected observation, got {other:?}"),
        },
        ExchangeResult::Rejected(rejection) => panic!("expected observation, got {rejection:?}"),
    }
}

fn rejection(response: &ExchangeResponse) -> Option<RejectionCode> {
    match &response.result {
        ExchangeResult::Rejected(rejection) => Some(rejection.code),
        ExchangeResult::Success(_) => None,
    }
}

/// Decodes the one response frame the guest wrote, whether or not the host received it.
fn decode_captured(captured: &[u8]) -> ExchangeResponse {
    let mut cursor = std::io::Cursor::new(captured);
    let body = read_frame(&mut cursor, FRAME_BYTES).expect("guest wrote one complete frame");
    serde_json::from_slice(&body).expect("guest response decodes")
}

fn observe_until(
    guest: &mut Guest,
    fixture: &Fixture,
    accepted: &WorkflowAccepted,
    done: impl Fn(&ExecutionObservation) -> bool,
) -> ExecutionObservation {
    let request = observe(fixture, accepted);
    for _attempt in 0..POLL_ATTEMPTS {
        let observation = observed(guest.exchange(Fault::None, &request).expect("observation exchange succeeds"));
        if done(&observation) {
            return observation;
        }
        thread::sleep(POLL_DELAY);
    }
    panic!("execution did not reach the awaited observation");
}

fn raw_exchange(guest: &mut Guest, frame: &[u8]) -> Vec<u8> {
    let mut connection = guest.connect(HOST_CID, Fault::None);
    connection.client.write_all(frame).expect("raw frame is written");
    connection.client.shutdown(std::net::Shutdown::Write).expect("raw request half closes");
    let mut response = Vec::new();
    connection.client.read_to_end(&mut response).expect("raw response is read");
    response
}

// r[verify lattice.workflow_interchange.reconnect_rules.scenario.lost-dispatch]
// r[verify lattice.workflow_interchange.transport_dispatch_recovery.scenario.lost]
#[test]
fn lost_dispatch_response_recovers_the_same_acceptance_without_redispatch() {
    let fixture = fixture(QUICK_COMMAND, DEADLINE_MILLISECONDS);
    let mut guest = Guest::start(fixture.store.clone(), fixture.profile.clone());

    let (lost, captured) = guest.exchange_captured(Fault::DropResponse, &dispatch(&fixture));
    assert_eq!(lost, Err(ClientFailure::UnknownAfterWrite));
    let outcome = ExchangeOutcome::of(&lost);
    assert_eq!(next_step(CommandKind::Dispatch, outcome, true), RecoveryStep::RecoverDispatch);

    let recovered = guest.exchange(Fault::None, &recover(&fixture));
    assert_eq!(
        next_step(CommandKind::RecoverDispatch, ExchangeOutcome::of(&recovered), true),
        RecoveryStep::Complete
    );
    let recovered = accepted(recovered.expect("recovery exchange succeeds"));
    assert_eq!(recovered, accepted(decode_captured(&captured)), "recovery returns the lost acceptance");

    let replayed = accepted(guest.exchange(Fault::None, &dispatch(&fixture)).expect("replayed dispatch succeeds"));
    assert_eq!(replayed, recovered, "a replayed identical dispatch returns the same acceptance");

    let (report, mut service) = guest.stop();
    service.handler_mut().wait_for_workers();
    let record = fixture
        .store
        .load_workflow_exchange_by_request(&fixture.request.identity)
        .expect("store reads")
        .expect("one execution exists");
    assert_eq!(record.accepted, recovered);
    assert_eq!(report.completed_exchanges, 3);
}

// r[verify lattice.workflow_interchange.application_admission.scenario.recovery]
// r[verify lattice.workflow_interchange.application_admission.scenario.conflict]
#[test]
fn duplicate_identical_request_is_accepted_once_and_changed_bodies_are_rejected() {
    let fixture = fixture(QUICK_COMMAND, DEADLINE_MILLISECONDS);
    let mut guest = Guest::start(fixture.store.clone(), fixture.profile.clone());
    let request = dispatch(&fixture);
    let frame = encode_frame(&request, FRAME_BYTES).expect("dispatch frame is bounded");

    let first = accepted(guest.exchange(Fault::None, &request).expect("dispatch succeeds"));
    let replay = raw_exchange(&mut guest, &frame);
    assert_eq!(accepted(decode_captured(&replay)), first, "identical replayed bytes return the same acceptance");

    let mut changed = fixture.request.clone();
    changed.kiln.attempt_identity = identity('a');
    changed.identity = lattice_workflow_contract::request_identity(&changed);
    let changed = new_request(ExchangeCommand::Dispatch {
        profile: fixture.contract.clone(),
        request: changed,
    })
    .expect("changed dispatch envelope is valid");
    let response = guest.exchange(Fault::None, &changed).expect("changed dispatch returns a closed response");
    assert_eq!(rejection(&response), Some(RejectionCode::InvalidContract), "idempotency key reuse is rejected");

    let mut tampered = request.clone();
    if let ExchangeCommand::Dispatch { request, .. } = &mut tampered.command {
        request.kiln.attempt_identity = identity('b');
    }
    let tampered = encode_frame(&tampered, FRAME_BYTES).expect("tampered frame is bounded");
    let response = decode_captured(&raw_exchange(&mut guest, &tampered));
    assert_eq!(rejection(&response), Some(RejectionCode::InvalidIdentity), "stale envelope identity is rejected");

    let (report, mut service) = guest.stop();
    service.handler_mut().wait_for_workers();
    assert_eq!(report.completed_exchanges, 4);
}

// r[verify lattice.workflow_interchange.transport_ambiguity.scenario.not-accepted]
// r[verify lattice.workflow_interchange.reconnect_rules.scenario.not-accepted]
#[test]
fn incomplete_request_write_is_not_accepted_and_the_identical_resend_is_admitted() {
    let fixture = fixture(QUICK_COMMAND, DEADLINE_MILLISECONDS);
    let mut guest = Guest::start(fixture.store.clone(), fixture.profile.clone());

    let connection = guest.connect(HOST_CID, Fault::CloseBeforeRead);
    connection.closed.recv().expect("guest closes the connection");
    let refused = exchange_connected(connection.client, &guest.client, &dispatch(&fixture));
    assert_eq!(refused, Err(ClientFailure::NotAccepted));
    assert_eq!(
        next_step(CommandKind::Dispatch, ExchangeOutcome::of(&refused), false),
        RecoveryStep::ResendIdentical
    );
    assert!(
        fixture
            .store
            .load_workflow_exchange_by_request(&fixture.request.identity)
            .expect("store reads")
            .is_none(),
        "an incomplete write admits nothing"
    );

    let resent = accepted(guest.exchange(Fault::None, &dispatch(&fixture)).expect("resend succeeds"));
    let (report, mut service) = guest.stop();
    service.handler_mut().wait_for_workers();
    assert_eq!(resent.request_identity, fixture.request.identity);
    assert_eq!(report.failed_exchanges, 1);
    assert_eq!(report.completed_exchanges, 1);
}

// r[verify lattice.workflow_interchange.transport_ambiguity.scenario.unknown]
#[test]
fn malformed_and_truncated_responses_stay_unknown_after_write() {
    let fixture = fixture(QUICK_COMMAND, DEADLINE_MILLISECONDS);
    let mut guest = Guest::start(fixture.store.clone(), fixture.profile.clone());

    let malformed = guest.exchange(Fault::CorruptResponse, &dispatch(&fixture));
    assert_eq!(malformed, Err(ClientFailure::UnknownAfterWrite));
    let truncated = guest.exchange(Fault::TruncateResponse, &recover(&fixture));
    assert_eq!(truncated, Err(ClientFailure::UnknownAfterWrite));
    assert_eq!(
        next_step(CommandKind::RecoverDispatch, ExchangeOutcome::of(&truncated), true),
        RecoveryStep::RetryBounded
    );
    let recovered = accepted(guest.exchange(Fault::None, &recover(&fixture)).expect("recovery succeeds"));

    let (report, mut service) = guest.stop();
    service.handler_mut().wait_for_workers();
    assert_eq!(recovered.request_identity, fixture.request.identity);
    assert_eq!(report.failed_exchanges, 1, "only the truncated write fails on the guest");
}

// r[verify lattice.workflow_interchange.transport_bounds.scenario.oversized]
#[test]
fn oversized_frame_is_refused_before_the_handler_runs() {
    let fixture = fixture(QUICK_COMMAND, DEADLINE_MILLISECONDS);
    let mut guest = Guest::start(fixture.store.clone(), fixture.profile.clone());

    let oversized = FRAME_BYTES.saturating_add(1).to_be_bytes();
    assert!(raw_exchange(&mut guest, &oversized).is_empty(), "no response to an oversized declaration");
    let missing = guest.exchange(Fault::None, &recover(&fixture)).expect("recovery returns a closed response");
    assert_eq!(rejection(&missing), Some(RejectionCode::NotFound));
    let outcome = ExchangeOutcome::of(&Ok(missing));
    assert_eq!(next_step(CommandKind::RecoverDispatch, outcome, false), RecoveryStep::StopUnknown);

    let (report, _service) = guest.stop();
    assert_eq!(report.failed_exchanges, 1);
}

// r[verify lattice.workflow_interchange.transport_bounds.scenario.timeout]
#[test]
fn read_deadlines_bound_a_stalled_host_and_a_slow_guest_response() {
    let fixture = fixture(QUICK_COMMAND, SHORT_DEADLINE_MILLISECONDS);
    let mut guest = Guest::start(fixture.store.clone(), fixture.profile.clone());

    let mut stalled = guest.connect(HOST_CID, Fault::None);
    stalled.client.write_all(&[0, 0]).expect("partial header is written");
    stalled.closed.recv().expect("guest read deadline closes the stalled connection");
    assert!(
        fixture
            .store
            .load_workflow_exchange_by_request(&fixture.request.identity)
            .expect("store reads")
            .is_none(),
        "a stalled request admits nothing"
    );

    let slow = guest.exchange(Fault::DelayResponse, &dispatch(&fixture));
    assert_eq!(slow, Err(ClientFailure::UnknownAfterWrite), "host read deadline leaves the outcome unknown");
    assert_eq!(next_step(CommandKind::Dispatch, ExchangeOutcome::of(&slow), true), RecoveryStep::RecoverDispatch);
    // The guest is still busy with the slow response, so recovery itself may time out;
    // the rule table retries a read-only recovery within the caller's bound.
    let mut recovered = None;
    for _attempt in 0..RECOVERY_ATTEMPTS {
        let attempt = guest.exchange(Fault::None, &recover(&fixture));
        match next_step(CommandKind::RecoverDispatch, ExchangeOutcome::of(&attempt), true) {
            RecoveryStep::Complete => {
                recovered = Some(accepted(attempt.expect("completed recovery has a response")));
                break;
            }
            RecoveryStep::RetryBounded => {}
            other => panic!("unexpected recovery step {other:?}"),
        }
    }
    let recovered = recovered.expect("recovery completes within the retry bound");

    let (report, mut service) = guest.stop();
    service.handler_mut().wait_for_workers();
    assert_eq!(recovered.request_identity, fixture.request.identity);
    assert!(report.failed_exchanges >= 1);
}

// r[verify lattice.workflow_interchange.vsock_listener.scenario.denied]
#[test]
fn a_peer_outside_the_allowed_cids_never_reaches_the_handler() {
    let fixture = fixture(QUICK_COMMAND, DEADLINE_MILLISECONDS);
    let mut guest = Guest::start(fixture.store.clone(), fixture.profile.clone());

    let stranger = guest.connect(STRANGER_CID, Fault::None);
    stranger.closed.recv().expect("guest closes the refused connection");
    let refused = exchange_connected(stranger.client, &guest.client, &dispatch(&fixture));
    assert!(refused.is_err());
    let missing = guest.exchange(Fault::None, &recover(&fixture)).expect("recovery returns a closed response");
    assert_eq!(rejection(&missing), Some(RejectionCode::NotFound));

    let (report, _service) = guest.stop();
    assert_eq!(report.completed_exchanges, 1);
    assert_eq!(report.rejected_peers, MAXIMUM_CONNECTIONS - 1);
}

// r[verify lattice.workflow_interchange.application_recovery.scenario.restart]
// r[verify lattice.workflow_interchange.observation_continuity.scenario.restart]
#[test]
fn handler_restart_reports_unknown_and_the_stale_incarnation_cannot_resurrect_it() {
    let fixture = fixture(SLOW_COMMAND, DEADLINE_MILLISECONDS);
    let mut first = Guest::start(fixture.store.clone(), fixture.profile.clone());
    let acceptance = accepted(first.exchange(Fault::None, &dispatch(&fixture)).expect("dispatch succeeds"));
    let running =
        observe_until(&mut first, &fixture, &acceptance, |observation| observation.state == ExecutionState::Running);
    // The first incarnation goes away with its worker still running.
    let (_report, abandoned) = first.stop();
    drop(abandoned);

    let mut second = Guest::start(fixture.store.clone(), fixture.profile.clone());
    let restarted = observed(second.exchange(Fault::None, &observe(&fixture, &acceptance)).expect("observe succeeds"));
    assert_eq!(restarted.state, ExecutionState::Unknown);
    assert!(!restarted.terminal && !restarted.successful);
    assert_eq!(continuity(Some(&running), &restarted), Continuity::Continuous);
    assert!(restarted.sequence > running.sequence, "the restart is a reported transition");
    let recovered = accepted(second.exchange(Fault::None, &recover(&fixture)).expect("recovery succeeds"));
    assert_eq!(recovered, acceptance, "restart recovery never redispatches");

    thread::sleep(STALE_WORKER_GRACE);
    let later = observed(second.exchange(Fault::None, &observe(&fixture, &acceptance)).expect("observe succeeds"));
    assert_eq!(later.state, ExecutionState::Unknown, "the abandoned worker cannot overwrite unknown");
    assert_eq!(later.sequence, restarted.sequence);
    assert_eq!(continuity(Some(&restarted), &later), Continuity::Continuous);
    let _stopped = second.stop();
}

// r[verify lattice.workflow_interchange.reconnect_rules.scenario.cancel-race]
#[test]
fn cancel_racing_reconnect_keeps_requested_and_observed_cancellation_distinct() {
    let fixture = fixture(CANCELLED_COMMAND, DEADLINE_MILLISECONDS);
    let mut guest = Guest::start(fixture.store.clone(), fixture.profile.clone());
    let acceptance = accepted(guest.exchange(Fault::None, &dispatch(&fixture)).expect("dispatch succeeds"));
    let running =
        observe_until(&mut guest, &fixture, &acceptance, |observation| observation.state == ExecutionState::Running);
    let pid_path = fixture.directory.path().join("started-pid");
    let pid = (0..POLL_ATTEMPTS)
        .find_map(|_| {
            let pid = std::fs::read_to_string(&pid_path).ok().and_then(|value| value.parse::<u32>().ok());
            if pid.is_none() {
                thread::sleep(POLL_DELAY);
            }
            pid
        })
        .expect("owned child started and published its PID");
    let process_path = std::path::PathBuf::from(format!("/proc/{pid}"));
    assert!(process_path.exists(), "the child is active before the cancel");

    let control = cancel(&fixture, &acceptance, running.sequence);
    let (lost, captured) = guest.exchange_captured(Fault::DropResponse, &control);
    assert_eq!(lost, Err(ClientFailure::UnknownAfterWrite));
    assert_eq!(next_step(CommandKind::Control, ExchangeOutcome::of(&lost), true), RecoveryStep::ObserveThenCompare);
    let applied = observed(decode_captured(&captured));
    assert_eq!(applied.sequence, running.sequence + 1, "the lost control was applied once");
    assert_eq!(applied.state, ExecutionState::Running, "requested cancellation is not observed cancellation");
    let record = fixture
        .store
        .load_workflow_exchange(&acceptance.execution_identity)
        .expect("store reads")
        .expect("execution exists");
    assert!(record.cancellation_requested, "the durable record keeps the request");

    let after = observed(guest.exchange(Fault::None, &observe(&fixture, &acceptance)).expect("observe succeeds"));
    assert_eq!(continuity(Some(&running), &after), Continuity::Continuous);
    assert_eq!(control_after_observe(running.sequence, after.sequence), RecoveryStep::StopSuperseded);
    let resent = guest.exchange(Fault::None, &control).expect("a resent control returns a closed response");
    assert!(rejection(&resent).is_some(), "the guest refuses a control whose expected sequence is behind");

    let (_report, mut service) = guest.stop();
    service.handler_mut().wait_for_workers();
    assert!(!process_path.exists(), "the owned child is stopped and reaped");
    let mut verifier = Guest::start(fixture.store.clone(), fixture.profile.clone());
    let finished = observed(verifier.exchange(Fault::None, &observe(&fixture, &acceptance)).expect("observe succeeds"));
    assert_eq!(finished.state, ExecutionState::Cancelled, "cancellation is reported once the worker stopped");
    assert!(finished.terminal && !finished.successful);
    assert_eq!(continuity(Some(&after), &finished), Continuity::Continuous);
    let _stopped = verifier.stop();
}

// r[verify lattice.workflow_interchange.observation_continuity.scenario.gap]
#[test]
fn log_cursor_resumes_after_a_lost_read_and_gaps_are_explicit() {
    let fixture = fixture(QUICK_COMMAND, DEADLINE_MILLISECONDS);
    let mut guest = Guest::start(fixture.store.clone(), fixture.profile.clone());
    let acceptance = accepted(guest.exchange(Fault::None, &dispatch(&fixture)).expect("dispatch succeeds"));
    let terminal = observe_until(&mut guest, &fixture, &acceptance, |observation| observation.terminal);
    let reference = terminal.logs.first().expect("terminal log is retained").clone();
    let log_request = read_log(&fixture, &acceptance, &reference);

    let lost = guest.exchange(Fault::DropResponse, &log_request);
    assert_eq!(lost, Err(ClientFailure::UnknownAfterWrite));
    assert_eq!(next_step(CommandKind::ReadLog, ExchangeOutcome::of(&lost), true), RecoveryStep::RetryBounded);
    let resumed = guest.exchange(Fault::None, &log_request).expect("log read resumes");
    let ExchangeResult::Success(success) = resumed.result else {
        panic!("log read is rejected");
    };
    let ExchangeSuccess::Log { observation, bytes } = *success else {
        panic!("log read returns a log");
    };
    assert_eq!(u64::try_from(bytes.len()).expect("length fits"), reference.bytes);
    assert_eq!(continuity(Some(&terminal), &observation), Continuity::Continuous);

    let mut gapped = observation.clone();
    for log in &mut gapped.logs {
        log.cursor = log.cursor.saturating_add(reference.bytes.saturating_add(1));
        log.gap_before = true;
    }
    gapped.identity = observation_identity(&gapped);
    assert_eq!(continuity(Some(&observation), &gapped), Continuity::Gap {
        nodes: vec![reference.node_id.clone()],
        streams: vec![LogStreamKey {
            node_id: reference.node_id.clone(),
            stream: reference.stream,
        }],
    });
    assert_eq!(continuity(Some(&gapped), &observation), Continuity::Reject {
        reason: ContinuityRejection::CursorRegressed
    });

    let (_report, mut service) = guest.stop();
    service.handler_mut().wait_for_workers();
}

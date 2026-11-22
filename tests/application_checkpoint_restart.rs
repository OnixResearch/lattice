//! Q1: application checkpoint restart across two state roots on one host (Lattice #12).
//!
//! The source is the real `lattice workflow-exchange serve --checkpoint-binding` process: a
//! client dispatches over its Unix socket, its worker publishes checkpoints, and it is
//! SIGKILLed; its state root is discarded and only the transferred checkpoint root survives. On
//! a second, fresh state root the real `lattice application-checkpoint restore` process adopts the
//! checkpoint, and a second `workflow-exchange serve` runs the adopted execution, which the same
//! client observes and cancels. Shell effects append to counter files outside both roots, an
//! independent oracle for replay.
#![cfg(target_os = "linux")]

use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use lattice::checkpoint::*;
use lattice_workflow_contract::*;
use lattice_workflow_transport::ExchangeCommand;
use lattice_workflow_transport::ExchangeResult;
use lattice_workflow_transport::ExchangeSuccess;
use lattice_workflow_transport::UnixClient;
use lattice_workflow_transport::UnixEndpoint;

const WORKFLOW: &str = "checkpoint-restart";
const IDENTITY_DIGITS: usize = 64;
const DEADLINE: u64 = 4_102_444_800_000;
const WAIT: Duration = Duration::from_secs(180);
const POLL: Duration = Duration::from_millis(20);
const OBSERVE_POLL: Duration = Duration::from_millis(100);
const ADMISSION: &str = "kiln-admission:q1-test";
/// Far beyond every bounded test wait, so a held gate never times out on a loaded host.
const GATE_TIMEOUT_SECS: u64 = 900;
/// Enough accepted connections for every observation poll within `WAIT`.
const MAXIMUM_CONNECTIONS: u32 = 4_096;

fn identity(digit: char) -> Identity {
    Identity::new(format!("{BLAKE3_PREFIX}{}", digit.to_string().repeat(IDENTITY_DIGITS))).expect("test identity")
}

fn label(value: &str) -> Label {
    Label::new(value.to_string()).expect("test label")
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
            pause: false,
            resume: false,
        },
        limits: ContractLimits {
            maximum_nodes: 8,
            maximum_edges: 8,
            maximum_log_references: 8,
            maximum_log_read_bytes: 4_096,
            maximum_total_log_bytes: 16_384,
        },
        non_claims: lattice_workflow_contract::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    }
}

fn workflow_request() -> WorkflowRequest {
    keyed_request('2')
}

/// A request of the same route under its own idempotency key, so its execution is distinct.
fn keyed_request(key: char) -> WorkflowRequest {
    let node = |id: &str, operation: char| WorkflowNode {
        id: label(id),
        kind: label("build"),
        effect_class: EffectClass::Build,
        adapter_kind: label("local"),
        operation_identity: identity(operation),
    };
    let mut graph = WorkflowGraph {
        schema: GRAPH_SCHEMA.to_string(),
        identity: identity('0'),
        nodes: vec![node("a", '1'), node("b", '2')],
        edges: vec![WorkflowEdge {
            source: label("a"),
            target: label("b"),
        }],
    };
    graph.identity = graph_identity(&graph);
    let mut request = WorkflowRequest {
        schema: REQUEST_SCHEMA.to_string(),
        identity: identity('0'),
        idempotency_key: identity(key),
        profile_identity: profile_identity(&contract_profile()),
        platform: label("linux-x86_64"),
        kiln: KilnLinkage {
            effect_identity: identity('3'),
            attempt_identity: identity('4'),
            candidate_identity: identity('5'),
            pipeline_revision_identity: identity('6'),
            workflow_policy_identity: identity('7'),
        },
        source: None,
        graph,
        controls: RequestedControls {
            cancel: true,
            pause: false,
            resume: false,
        },
        observation: ObservationPolicy {
            maximum_log_references: 8,
            maximum_log_read_bytes: 4_096,
            maximum_total_log_bytes: 16_384,
        },
    };
    request.identity = lattice_workflow_contract::request_identity(&request);
    request
}

/// Each node drains its piped stdin, appends once to `effects/<name>` and then waits for
/// `effects/release-<name>`. Draining first keeps a released node from exiting before the
/// runtime has written its stdin, which bounded-exec reports as a broken pipe and so a failure.
/// A gate also ends once `effects` is gone, so a node orphaned by a SIGKILLed exchange cannot
/// outlive its test's temporary directory.
fn gated(effects: &Path, name: &str) -> String {
    let effects = effects.display();
    format!(
        "cat > /dev/null; printf {name} >> '{effects}/{name}'; while [ ! -e '{effects}/release-{name}' ] && [ -d '{effects}' ]; do sleep 0.05; done; printf {name}"
    )
}

fn gated_node(
    workflow: &mut lattice::engine::workflow::Workflow,
    effects: &Path,
    name: &str,
) -> lattice::engine::workflow::NodeId {
    workflow
        .try_add_configured_node(name, lattice::engine::workflow::NodeConfig::Shell {
            command: gated(effects, name),
            cwd: None,
            env: std::collections::HashMap::new(),
            timeout_secs: GATE_TIMEOUT_SECS,
            pipe_stdin: true,
        })
        .expect("gated shell node")
}

fn transport() -> lattice_workflow_transport::TransportProfile {
    lattice_workflow_transport::TransportProfile {
        schema: lattice_workflow_transport::TRANSPORT_PROFILE_SCHEMA.to_string(),
        limits: lattice_workflow_transport::TransportLimits {
            maximum_frame_bytes: 1_048_576,
            maximum_log_bytes: 65_536,
            maximum_connections: MAXIMUM_CONNECTIONS,
            read_deadline_milliseconds: 5_000,
            write_deadline_milliseconds: 5_000,
        },
        peers: lattice_workflow_transport::PeerPolicy {
            allowed_user_ids: vec![nix::unistd::getuid().as_raw()],
        },
        non_claims: lattice_workflow_transport::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    }
}

/// One state root: a data directory with the saved workflow, a checkpoint root, the handler
/// profile and the route's checkpoint binding.
struct Root {
    directory: PathBuf,
    data: PathBuf,
    checkpoint: PathBuf,
    profile: lattice::workflow_exchange::HandlerProfile,
    binding: CheckpointBinding,
}

/// Both roots of one test share `socket`: it is part of the handler profile identity, which a
/// restart must reproduce exactly.
fn root(directory: &Path, effects: &Path, socket: &Path, checkpoint: CheckpointProfile) -> Root {
    let mut workflow =
        lattice::engine::workflow::Workflow::new(WORKFLOW, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let node_a = gated_node(&mut workflow, effects, "a");
    let node_b = gated_node(&mut workflow, effects, "b");
    let stdout = workflow.graph.node(node_a).expect("node a").output_ports[0].id;
    let stdin = workflow.graph.node(node_b).expect("node b").input_ports[0].id;
    workflow.graph.add_edge(stdout, stdin).expect("a feeds b");
    let revision = lattice::orchestration::workflow_revision(&workflow).expect("workflow revision");
    let revision = Identity::new(format!("{BLAKE3_PREFIX}{}", revision.content_hash)).expect("revision identity");
    let request = workflow_request();
    let route = lattice::workflow_exchange::WorkflowRoute {
        profile_identity: request.profile_identity.clone(),
        graph_identity: request.graph.identity.clone(),
        workflow_name: WORKFLOW.to_string(),
        workflow_revision: revision.clone(),
        effect_authority: lattice::workflow_exchange::EffectAuthorityMode::RouteExact,
        source_binding: lattice::workflow_exchange::SourceBindingMode::Unavailable,
        nodes: vec![
            lattice::workflow_exchange::RouteNode {
                request_node_id: label("a"),
                operation_identity: identity('1'),
                workflow_node_id: node_a.0,
            },
            lattice::workflow_exchange::RouteNode {
                request_node_id: label("b"),
                operation_identity: identity('2'),
                workflow_node_id: node_b.0,
            },
        ],
    };
    let profile = lattice::workflow_exchange::HandlerProfile {
        schema: lattice::workflow_exchange::HANDLER_PROFILE_SCHEMA.to_string(),
        socket_path: socket.to_path_buf(),
        transport: transport(),
        maximum_active_executions: 2,
        controls: lattice::workflow_exchange::HandlerControls::default(),
        routes: vec![route],
        non_claims: lattice::workflow_exchange::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    };
    let data = directory.join("data");
    let checkpoint_root = directory.join("checkpoint");
    std::fs::create_dir_all(&data).expect("data directory");
    let binding = CheckpointBinding {
        schema: BINDING_SCHEMA.to_string(),
        vm_profile: identity('a'),
        checkpoint,
        graph_identity: request.graph.identity,
        workflow_revision: revision,
        root: checkpoint_root.clone(),
        workspace: None,
        binding: Some(identity('b')),
        environment: None,
        deadline_unix_ms: DEADLINE,
        non_claims: lattice::checkpoint::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    };
    let store = lattice::store::Store::new(lattice::config::store_path(&data)).expect("store opens");
    store.save_workflow(&workflow).expect("workflow saves");
    Root {
        directory: directory.to_path_buf(),
        data,
        checkpoint: checkpoint_root,
        profile,
        binding,
    }
}

fn store(root: &Root) -> lattice::store::Store {
    lattice::store::Store::new(lattice::config::store_path(&root.data)).expect("store opens")
}

fn wait_for(what: &str, mut ready: impl FnMut() -> bool) {
    let started = Instant::now();
    while !ready() {
        assert!(started.elapsed() < WAIT, "timed out waiting for {what}");
        std::thread::sleep(POLL);
    }
}

fn count(effects: &Path, name: &str) -> usize {
    std::fs::read_to_string(effects.join(name)).map_or(0, |text| text.len())
}

fn release(effects: &Path, name: &str) {
    std::fs::write(effects.join(format!("release-{name}")), b"").expect("release gate");
}

/// Command inputs of one root: a CLI configuration whose data directory is the root's, the
/// handler profile and the checkpoint binding, all written into the root directory.
struct Inputs {
    config: PathBuf,
    profile: PathBuf,
    binding: PathBuf,
}

fn inputs(root: &Root) -> Inputs {
    let config = root.directory.join("config.ncl");
    std::fs::write(
        &config,
        format!(
            "{{ data_dir = {:?}, default_timeout_secs = 60, shell = \"sh\", max_parallelism = 1 }}\n",
            root.data.display().to_string()
        ),
    )
    .expect("config");
    std::fs::write(root.directory.join("profile.json"), serde_json::to_vec(&root.profile).expect("profile JSON"))
        .expect("profile data");
    let profile = root.directory.join("profile.ncl");
    std::fs::write(&profile, "import \"profile.json\"\n").expect("profile");
    let binding = root.directory.join("binding.json");
    std::fs::write(&binding, serde_json::to_vec(&root.binding).expect("binding JSON")).expect("binding");
    Inputs {
        config,
        profile,
        binding,
    }
}

/// One real `lattice workflow-exchange serve --checkpoint-binding` process and its client.
struct Exchange {
    child: Child,
    socket: PathBuf,
    transport: lattice_workflow_transport::TransportProfile,
}

impl Exchange {
    fn start(root: &Root) -> Self {
        let inputs = inputs(root);
        let socket = root.profile.socket_path.clone();
        let stderr = std::fs::File::create(root.directory.join("exchange.stderr")).expect("stderr file");
        let mut child = Command::new(env!("CARGO_BIN_EXE_lattice"))
            .arg("--config")
            .arg(&inputs.config)
            .args(["workflow-exchange", "serve", "--profile"])
            .arg(&inputs.profile)
            .arg("--checkpoint-binding")
            .arg(&inputs.binding)
            .stdout(Stdio::null())
            .stderr(stderr)
            .spawn()
            .expect("workflow exchange starts");
        wait_for("the exchange to listen", || {
            assert!(child.try_wait().expect("exchange status").is_none(), "the exchange exited early");
            socket.exists()
        });
        Self {
            child,
            socket,
            transport: root.profile.transport.clone(),
        }
    }

    fn call(&self, command: ExchangeCommand) -> ExchangeSuccess {
        let request = lattice_workflow_transport::new_request(command).expect("exchange request");
        let client = UnixClient::new(UnixEndpoint {
            socket_path: &self.socket,
            profile: &self.transport,
        });
        match client.exchange(&request).expect("exchange answers").result {
            ExchangeResult::Success(success) => *success,
            ExchangeResult::Rejected(rejection) => panic!("exchange rejected the command: {rejection:?}"),
        }
    }

    fn dispatch(&self) -> WorkflowAccepted {
        self.dispatch_as(&workflow_request())
    }

    fn dispatch_as(&self, request: &WorkflowRequest) -> WorkflowAccepted {
        match self.call(ExchangeCommand::Dispatch {
            profile: contract_profile(),
            request: request.clone(),
        }) {
            ExchangeSuccess::Accepted(accepted) => accepted,
            other => panic!("dispatch must be accepted: {other:?}"),
        }
    }

    fn observe(&self, accepted: &WorkflowAccepted) -> ExecutionObservation {
        self.observe_as(&workflow_request(), accepted)
    }

    fn observe_as(&self, request: &WorkflowRequest, accepted: &WorkflowAccepted) -> ExecutionObservation {
        match self.call(ExchangeCommand::Observe {
            profile: contract_profile(),
            request: request.clone(),
            accepted: accepted.clone(),
        }) {
            ExchangeSuccess::Observation(observation) => observation,
            other => panic!("observe must answer an observation: {other:?}"),
        }
    }

    fn cancel(&self, accepted: &WorkflowAccepted, sequence: u64) -> ExecutionObservation {
        let control = ControlRequest {
            schema: CONTROL_SCHEMA.to_string(),
            request_identity: workflow_request().identity,
            workflow_identity: accepted.workflow_identity.clone(),
            execution_identity: accepted.execution_identity.clone(),
            expected_sequence: sequence,
            action: ControlAction::Cancel,
        };
        match self.call(ExchangeCommand::Control {
            profile: contract_profile(),
            request: workflow_request(),
            accepted: accepted.clone(),
            current_sequence: sequence,
            control,
        }) {
            ExchangeSuccess::Observation(observation) => observation,
            other => panic!("cancel must answer an observation: {other:?}"),
        }
    }

    fn wait_terminal(&self, accepted: &WorkflowAccepted) -> ExecutionObservation {
        self.wait_terminal_as(&workflow_request(), accepted)
    }

    fn wait_terminal_as(&self, request: &WorkflowRequest, accepted: &WorkflowAccepted) -> ExecutionObservation {
        let started = Instant::now();
        loop {
            let observation = self.observe_as(request, accepted);
            if observation.terminal {
                return observation;
            }
            assert!(started.elapsed() < WAIT, "timed out waiting for a terminal observation: {observation:?}");
            std::thread::sleep(OBSERVE_POLL);
        }
    }

    /// SIGKILL the process and remove its socket, as a host discards a dead guest.
    fn kill(mut self) {
        self.stop();
    }

    fn stop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _killed = self.child.kill();
        }
        let _reaped = self.child.wait();
        let _removed = std::fs::remove_file(&self.socket);
    }
}

impl Drop for Exchange {
    fn drop(&mut self) {
        self.stop();
    }
}

struct Killed {
    directory: tempfile::TempDir,
    effects: PathBuf,
    socket: PathBuf,
    transfer: CheckpointRoot,
    accepted: WorkflowAccepted,
}

/// Serve the source root, dispatch through its socket, run `until` against it, SIGKILL it, keep
/// only a copy of its checkpoint root, and discard its state root.
fn run_and_kill_source(prepare: impl FnOnce(&Path), until: impl FnOnce(&Path, &Path)) -> Killed {
    let directory = tempfile::tempdir().expect("temporary directory");
    let effects = directory.path().join("effects");
    std::fs::create_dir_all(&effects).expect("effects directory");
    let socket = directory.path().join("exchange.sock");
    let source = root(&directory.path().join("source"), &effects, &socket, CheckpointProfile::WorkflowNodeBoundaryV1);
    std::fs::create_dir_all(&source.checkpoint).expect("source checkpoint root");
    prepare(&source.checkpoint);
    let exchange = Exchange::start(&source);
    let accepted = exchange.dispatch();
    until(&source.checkpoint, &effects);
    exchange.kill();
    let transfer = directory.path().join("transfer");
    copy_tree(&source.checkpoint, &transfer);
    std::fs::remove_dir_all(&source.directory).expect("source state root discarded");
    Killed {
        directory,
        effects,
        socket,
        transfer: CheckpointRoot::new(transfer),
        accepted,
    }
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("copy target");
    for entry in std::fs::read_dir(from).expect("copy source") {
        let entry = entry.expect("entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy file");
        }
    }
}

fn latest(root: &CheckpointRoot) -> (Identity, Manifest) {
    let (_, identity) = root.committed().expect("committed").pop().expect("a committed manifest");
    let bytes = root.manifest_bytes(&identity).expect("manifest read").expect("manifest present");
    (identity, serde_json::from_slice(&bytes).expect("manifest parses"))
}

fn restart_request(checkpoint: &Identity, manifest: &Manifest, incarnation: u32, admission: &str) -> RestartRequest {
    RestartRequest {
        schema: RESTART_REQUEST_SCHEMA.to_string(),
        checkpoint: checkpoint.clone(),
        admission: admission.to_string(),
        restarted_incarnation: incarnation,
        request: manifest.execution.request.clone(),
        execution: manifest.execution.accepted.execution_identity.clone(),
        binding: manifest.binding.clone(),
        cancellation_requested: false,
    }
}

/// Run the real `lattice application-checkpoint restore` against a target state root.
fn cli_restore(target: &Root, source: &CheckpointRoot, request: &RestartRequest) -> (bool, serde_json::Value) {
    let inputs = inputs(target);
    let request_path = target.directory.join("request.json");
    std::fs::write(&request_path, serde_json::to_vec(request).expect("request JSON")).expect("request");
    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&inputs.config)
        .args(["application-checkpoint", "restore", "--profile"])
        .arg(&inputs.profile)
        .arg("--binding")
        .arg(&inputs.binding)
        .arg("--source")
        .arg(source.path())
        .arg("--request")
        .arg(&request_path)
        .output()
        .expect("restore command runs");
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|_| panic!("restore prints JSON: {}", String::from_utf8_lossy(&output.stderr)));
    (output.status.success(), report)
}

fn target(killed: &Killed, name: &str, checkpoint: CheckpointProfile) -> Root {
    root(&killed.directory.path().join(name), &killed.effects, &killed.socket, checkpoint)
}

/// The target's durable execution and per-node run details, for assertion messages.
fn run_diagnostics(root: &Root, execution: &Identity) -> String {
    let store = store(root);
    let Ok(Some(record)) = store.load_workflow_exchange(execution) else {
        return "no execution record".to_string();
    };
    let nodes = record.nodes.iter().map(|node| (node.request_node_id.as_str(), node.state)).collect::<Vec<_>>();
    let details = match store.load_run_details(lattice::engine::RunId(record.run_id)) {
        Ok(Some(details)) => details
            .nodes
            .iter()
            .map(|node| format!("{} {:?} {:?}", node.node_label, node.status, node.error))
            .collect::<Vec<_>>(),
        Ok(None) => vec!["no run details".to_string()],
        Err(error) => vec![format!("run details unreadable: {error}")],
    };
    format!("state {:?}, nodes {nodes:?}, run details {details:?}", record.state)
}

/// A source drained after `a` committed: one completed node, nothing uncertain, `b` held.
fn drained_after_a() -> (Killed, Identity, Manifest) {
    let killed = run_and_kill_source(
        |_| {},
        |checkpoint, effects| {
            wait_for("node a to start", || count(effects, "a") == 1);
            std::fs::write(checkpoint.join("drain-request"), b"").expect("drain requested");
            release(effects, "a");
            wait_for("drain acknowledgement", || checkpoint.join("drained.json").exists());
        },
    );
    let drained = killed.transfer.drain_acknowledgement().expect("drain read").expect("drained");
    let (checkpoint, manifest) = latest(&killed.transfer);
    assert_eq!(drained.manifest, checkpoint, "the drained manifest is the latest commit");
    assert_eq!(manifest.completed_nodes.len(), 1);
    assert!(manifest.uncertain_nodes.is_empty(), "a drained manifest is quiescent");
    assert!(!manifest.cancellation_requested);
    assert_eq!(count(&killed.effects, "b"), 0, "b never started on the source");
    (killed, checkpoint, manifest)
}

// r[verify lattice.application_checkpoint.restore]
// r[verify lattice.application_checkpoint.resume]
// r[verify lattice.application_checkpoint.fence]
// r[verify lattice.application_checkpoint.receipt]
/// Crash after an external-effect node: the source drains after `a`, is SIGKILLed and loses its
/// state root; the target adopts the drained manifest, and its served exchange never re-runs
/// `a`, runs `b` once and keeps publishing.
#[test]
fn drained_source_restarts_on_a_second_root_after_sigkill() {
    let (killed, checkpoint, manifest) = drained_after_a();
    release(&killed.effects, "b");
    let target_root = target(&killed, "target", CheckpointProfile::WorkflowNodeBoundaryV1);
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);
    let (success, report) = cli_restore(&target_root, &killed.transfer, &request);
    assert!(success, "restore succeeds: {report}");
    assert_eq!(report["receipt"]["kind"], "application_restart");
    assert_eq!(report["receipt"]["restarted_incarnation"], 1);
    assert_eq!(report["receipt"]["admission"], ADMISSION);
    assert_eq!(report["receipt"]["checkpoint"], checkpoint.as_str());
    assert_eq!(report["execution_state"], "queued", "restore adopts; it runs nothing");
    assert_eq!(count(&killed.effects, "b"), 0);

    let exchange = Exchange::start(&target_root);
    let observation = exchange.wait_terminal(&killed.accepted);
    exchange.kill();
    assert_eq!(
        observation.state,
        ExecutionState::Completed,
        "{}",
        run_diagnostics(&target_root, &request.execution)
    );
    assert_eq!(count(&killed.effects, "a"), 1, "a is restored, not replayed");
    assert_eq!(count(&killed.effects, "b"), 1, "b ran exactly once, on the target");

    let target_store = store(&target_root);
    let adopted = target_store.load_workflow_exchange(&request.execution).expect("load").expect("adopted record");
    assert_eq!(adopted.accepted, manifest.execution.accepted, "execution identity survives the restart");
    let adoption = target_store.load_workflow_exchange_adoption(&request.execution).expect("load").expect("adoption");
    assert_eq!(adoption.deadline_unix_ms, DEADLINE);
    assert_eq!(adoption.consumed.elapsed_ms, manifest.consumed.elapsed_ms);
    let (_, newest) = latest(&CheckpointRoot::new(&target_root.checkpoint));
    assert_eq!(newest.producing_incarnation, 1, "the restarted incarnation publishes its own checkpoints");
    assert!(newest.consumed.elapsed_ms >= manifest.consumed.elapsed_ms, "elapsed budget is never reset");
    drop(target_store);

    let (repeat, again) = cli_restore(&target_root, &killed.transfer, &request);
    assert!(repeat, "an identical restart is idempotent: {again}");
    assert_eq!(again["receipt"], report["receipt"]);
    assert_eq!(count(&killed.effects, "b"), 1, "an idempotent restart dispatches nothing");
    let stale = restart_request(&checkpoint, &manifest, 1, "kiln-admission:q1-other");
    let (refused, stale_report) = cli_restore(&target_root, &killed.transfer, &stale);
    assert!(!refused);
    assert_eq!(stale_report["refused"], "stale_admission");
}

// r[verify lattice.application_checkpoint.resume]
/// A restored execution is observable and cancellable through the served exchange, and exactly
/// one process can run it: a second exchange on the same state root cannot start.
#[test]
fn restored_execution_is_observed_and_cancelled_through_the_exchange() {
    let (killed, checkpoint, manifest) = drained_after_a();
    let target_root = target(&killed, "target", CheckpointProfile::WorkflowNodeBoundaryV1);
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);
    let (success, report) = cli_restore(&target_root, &killed.transfer, &request);
    assert!(success, "restore succeeds: {report}");

    let exchange = Exchange::start(&target_root);
    wait_for("node b to start on the target", || count(&killed.effects, "b") == 1);
    let inputs = inputs(&target_root);
    let second = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&inputs.config)
        .args(["workflow-exchange", "serve", "--profile"])
        .arg(&inputs.profile)
        .arg("--checkpoint-binding")
        .arg(&inputs.binding)
        .output()
        .expect("second exchange runs");
    assert!(!second.status.success(), "a second process cannot serve the same state root");
    let running = exchange.observe(&killed.accepted);
    assert_eq!(running.state, ExecutionState::Running);
    let cancelling = exchange.cancel(&killed.accepted, running.sequence);
    assert!(cancelling.sequence > running.sequence);
    let observation = exchange.wait_terminal(&killed.accepted);
    exchange.kill();
    release(&killed.effects, "b");
    assert_eq!(
        observation.state,
        ExecutionState::Cancelled,
        "{}",
        run_diagnostics(&target_root, &request.execution)
    );
    assert_eq!(count(&killed.effects, "a"), 1, "a is restored, not replayed");
    let (_, newest) = latest(&CheckpointRoot::new(&target_root.checkpoint));
    assert_eq!(newest.producing_incarnation, 1, "the served exchange publishes the restarted incarnation");
}

/// A source of `request` that drains before its first effect, killed and transferred: its
/// manifest completes nothing and leaves nothing uncertain.
fn drained_before_dispatch(
    directory: &Path,
    effects: &Path,
    socket: &Path,
    request: &WorkflowRequest,
) -> (CheckpointRoot, WorkflowAccepted) {
    let name = request.idempotency_key.as_str().trim_start_matches(BLAKE3_PREFIX)[..1].to_string();
    let source = root(
        &directory.join(format!("source-{name}")),
        effects,
        socket,
        CheckpointProfile::WorkflowNodeBoundaryV1,
    );
    std::fs::create_dir_all(&source.checkpoint).expect("source checkpoint root");
    std::fs::write(source.checkpoint.join("drain-request"), b"").expect("drain requested");
    let exchange = Exchange::start(&source);
    let accepted = exchange.dispatch_as(request);
    wait_for("drain acknowledgement", || source.checkpoint.join("drained.json").exists());
    exchange.kill();
    let transfer = directory.join(format!("transfer-{name}"));
    copy_tree(&source.checkpoint, &transfer);
    std::fs::remove_dir_all(&source.directory).expect("source state root discarded");
    (CheckpointRoot::new(transfer), accepted)
}

// r[verify lattice.application_checkpoint.resume]
/// More adopted executions than the active-execution limit: the served exchange runs at most the
/// limit, lowest run id first; the extra one observes as `queued` and starts on the first
/// exchange request after a slot frees.
#[test]
fn adopted_executions_wait_for_the_active_execution_limit() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let effects = directory.path().join("effects");
    std::fs::create_dir_all(&effects).expect("effects directory");
    let socket = directory.path().join("exchange.sock");
    let target_root =
        root(&directory.path().join("target"), &effects, &socket, CheckpointProfile::WorkflowNodeBoundaryV1);
    let limit = usize::try_from(target_root.profile.maximum_active_executions).expect("limit");
    let executions = ['5', '8', '9']
        .into_iter()
        .take(limit + 1)
        .map(|key| {
            let request = keyed_request(key);
            let (transfer, accepted) = drained_before_dispatch(directory.path(), &effects, &socket, &request);
            let (checkpoint, manifest) = latest(&transfer);
            let (ok, report) =
                cli_restore(&target_root, &transfer, &restart_request(&checkpoint, &manifest, 1, ADMISSION));
            assert!(ok, "restore succeeds: {report}");
            assert_eq!(report["execution_state"], "queued");
            (request, accepted)
        })
        .collect::<Vec<_>>();
    assert_eq!(executions.len(), limit + 1);

    let exchange = Exchange::start(&target_root);
    wait_for("the limit's executions to start node a", || count(&effects, "a") == limit);
    // A bounded stability check: an execution past the limit must not start while the others
    // hold their gates, however many exchange requests arrive.
    for _ in 0..20 {
        let states = executions
            .iter()
            .map(|(request, accepted)| exchange.observe_as(request, accepted).state)
            .collect::<Vec<_>>();
        assert_eq!(states, [vec![ExecutionState::Running; limit], vec![ExecutionState::Queued]].concat());
        assert_eq!(count(&effects, "a"), limit, "no execution past the limit started");
        std::thread::sleep(OBSERVE_POLL);
    }
    release(&effects, "a");
    release(&effects, "b");
    let (waiting_request, waiting_accepted) = executions.last().expect("the waiting execution");
    let waiting = exchange.wait_terminal_as(waiting_request, waiting_accepted);
    let others = executions
        .iter()
        .map(|(request, accepted)| exchange.wait_terminal_as(request, accepted).state)
        .collect::<Vec<_>>();
    exchange.kill();
    assert_eq!(
        waiting.state,
        ExecutionState::Completed,
        "{}",
        run_diagnostics(&target_root, &waiting_accepted.execution_identity)
    );
    assert_eq!(others, vec![ExecutionState::Completed; limit + 1]);
    assert_eq!((count(&effects, "a"), count(&effects, "b")), (limit + 1, limit + 1), "every execution ran once");
}

// r[verify lattice.application_checkpoint.resume]
/// An adopted execution that was running when its exchange process died is `Unknown` to the
/// next exchange process, which never runs it again.
#[test]
fn crashed_running_adoption_is_unknown_to_the_next_exchange() {
    let (killed, checkpoint, manifest) = drained_after_a();
    let target_root = target(&killed, "target", CheckpointProfile::WorkflowNodeBoundaryV1);
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);
    let (success, report) = cli_restore(&target_root, &killed.transfer, &request);
    assert!(success, "restore succeeds: {report}");
    let exchange = Exchange::start(&target_root);
    wait_for("node b to start on the target", || count(&killed.effects, "b") == 1);
    assert_eq!(exchange.observe(&killed.accepted).state, ExecutionState::Running);
    exchange.kill();
    let restarted = Exchange::start(&target_root);
    assert_eq!(restarted.observe(&killed.accepted).state, ExecutionState::Unknown);
    restarted.kill();
    release(&killed.effects, "b");
    assert_eq!(count(&killed.effects, "b"), 1, "the restarted exchange never runs it again");
}

// r[verify lattice.application_checkpoint.cancellation]
/// A cancel Kiln acknowledged after the drained manifest travels in the restart request: the
/// restarted run starts cancelled and dispatches no further effect.
#[test]
fn restart_request_cancel_starts_the_restored_run_cancelled() {
    let (killed, checkpoint, manifest) = drained_after_a();
    let target_root = target(&killed, "target", CheckpointProfile::WorkflowNodeBoundaryV1);
    let mut request = restart_request(&checkpoint, &manifest, 1, ADMISSION);
    request.cancellation_requested = true;
    let (success, report) = cli_restore(&target_root, &killed.transfer, &request);
    assert!(success, "restore succeeds: {report}");
    let adoption = store(&target_root)
        .load_workflow_exchange_adoption(&request.execution)
        .expect("load")
        .expect("adoption");
    assert!(adoption.cancellation_requested, "the adoption keeps Kiln's cancel");

    let exchange = Exchange::start(&target_root);
    let observation = exchange.wait_terminal(&killed.accepted);
    exchange.kill();
    release(&killed.effects, "b");
    assert_eq!(
        observation.state,
        ExecutionState::Cancelled,
        "{}",
        run_diagnostics(&target_root, &request.execution)
    );
    assert_eq!(count(&killed.effects, "b"), 0, "a cancelled restart dispatches no further effect");
    assert_eq!(count(&killed.effects, "a"), 1);
}

// r[verify lattice.application_checkpoint.fence]
/// Crash mid-node: `b` started its effect when the source died; the target adopts it as
/// `Unknown` and never replays it.
#[test]
fn sigkill_mid_effect_restores_unknown_without_replay() {
    let killed = run_and_kill_source(
        |_| {},
        |_, effects| {
            release(effects, "a");
            wait_for("node b to start", || count(effects, "b") == 1);
        },
    );
    let (checkpoint, manifest) = latest(&killed.transfer);
    assert!(!manifest.uncertain_nodes.is_empty(), "b was committed as uncertain before dispatch");
    let target_root = target(&killed, "target", CheckpointProfile::WorkflowNodeBoundaryV1);
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);
    let (success, report) = cli_restore(&target_root, &killed.transfer, &request);
    assert!(success, "restore issues a receipt: {report}");
    assert_eq!(report["receipt"]["kind"], "application_restart");
    assert_eq!(report["execution_state"], "unknown");
    let states = report["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .map(|node| node["state"].clone())
        .collect::<Vec<_>>();
    assert!(states.contains(&serde_json::json!("unknown")), "{states:?}");
    let exchange = Exchange::start(&target_root);
    assert_eq!(exchange.observe(&killed.accepted).state, ExecutionState::Unknown);
    exchange.kill();
    release(&killed.effects, "b");
    assert_eq!(count(&killed.effects, "a"), 1);
    assert_eq!(count(&killed.effects, "b"), 1, "the uncertain effect is never replayed");
}

/// Crash before dispatch: a drain requested before the first effect parks it; the target runs
/// every node exactly once.
#[test]
fn sigkill_before_dispatch_runs_every_node_once_on_the_target() {
    let killed = run_and_kill_source(
        |checkpoint| std::fs::write(checkpoint.join("drain-request"), b"").expect("drain requested"),
        |checkpoint, _| wait_for("drain acknowledgement", || checkpoint.join("drained.json").exists()),
    );
    let (checkpoint, manifest) = latest(&killed.transfer);
    assert!(manifest.completed_nodes.is_empty() && manifest.uncertain_nodes.is_empty());
    assert_eq!(count(&killed.effects, "a"), 0);
    release(&killed.effects, "a");
    release(&killed.effects, "b");
    let target_root = target(&killed, "target", CheckpointProfile::WorkflowNodeBoundaryV1);
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);
    let (success, report) = cli_restore(&target_root, &killed.transfer, &request);
    assert!(success, "restore succeeds: {report}");
    let exchange = Exchange::start(&target_root);
    let observation = exchange.wait_terminal(&killed.accepted);
    exchange.kill();
    assert_eq!(
        observation.state,
        ExecutionState::Completed,
        "{}",
        run_diagnostics(&target_root, &request.execution)
    );
    assert_eq!(count(&killed.effects, "a"), 1);
    assert_eq!(count(&killed.effects, "b"), 1);
}

/// A completed source run through the served exchange, gates released.
struct Completed {
    directory: tempfile::TempDir,
    effects: PathBuf,
    socket: PathBuf,
    source: Root,
    accepted: WorkflowAccepted,
}

fn completed_source(checkpoint: CheckpointProfile, workspace: Option<&[(&str, &str)]>) -> Completed {
    let directory = tempfile::tempdir().expect("temporary directory");
    let effects = directory.path().join("effects");
    std::fs::create_dir_all(&effects).expect("effects directory");
    release(&effects, "a");
    release(&effects, "b");
    let socket = directory.path().join("exchange.sock");
    let mut source = root(&directory.path().join("source"), &effects, &socket, checkpoint);
    if let Some(files) = workspace {
        let path = directory.path().join("source-workspace");
        for (name, content) in files {
            std::fs::create_dir_all(&path).expect("workspace");
            std::fs::write(path.join(name), content).expect("workspace file");
        }
        source.binding.workspace = Some(path);
    }
    let exchange = Exchange::start(&source);
    let accepted = exchange.dispatch();
    let observation = exchange.wait_terminal(&accepted);
    exchange.kill();
    assert_eq!(observation.state, ExecutionState::Completed);
    Completed {
        directory,
        effects,
        socket,
        source,
        accepted,
    }
}

// r[verify lattice.application_checkpoint.commit]
// r[verify lattice.application_checkpoint.admission]
// r[verify lattice.application_checkpoint.carry_forward]
// r[verify lattice.application_checkpoint.unavailable]
/// Partial publication, missing objects, crossed identities, a moved deadline or attempt
/// binding, unavailable profiles and an older checkpoint, all against real roots and the real
/// restore process.
#[test]
fn partial_publication_crossed_identities_and_older_checkpoints_stay_honest() {
    let run = completed_source(CheckpointProfile::WorkflowNodeBoundaryV1, None);
    let source_root = CheckpointRoot::new(&run.source.checkpoint);
    let committed = source_root.committed().expect("committed");
    assert!(committed.len() >= 3, "fence and boundary publications: {committed:?}");
    let (checkpoint, manifest) = latest(&source_root);
    assert_eq!(manifest.completed_nodes.len(), 2);

    let check = |name: &str, root: &CheckpointRoot, request: &RestartRequest, change: &dyn Fn(&mut Root)| {
        let mut target_root = self::root(
            &run.directory.path().join(name),
            &run.effects,
            &run.socket,
            CheckpointProfile::WorkflowNodeBoundaryV1,
        );
        change(&mut target_root);
        let (ok, report) = cli_restore(&target_root, root, request);
        let adopted = store(&target_root).load_workflow_exchange(&request.execution).expect("load").is_some();
        assert_eq!(ok, adopted, "a restore adopts exactly when it succeeds: {report}");
        (ok, report)
    };
    let unchanged = |_: &mut Root| {};
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);

    // A manifest whose commit marker was never written (crash between objects and marker).
    let partial = run.directory.path().join("partial");
    copy_tree(&run.source.checkpoint, &partial);
    let marker = std::fs::read_dir(partial.join("commits"))
        .expect("commits")
        .map(|entry| entry.expect("entry").path())
        .find(|path| path.to_string_lossy().contains(checkpoint.as_str().trim_start_matches("b3:")))
        .expect("latest marker");
    std::fs::remove_file(marker).expect("marker removed");
    let (ok, report) = check("partial-target", &CheckpointRoot::new(&partial), &request, &unchanged);
    assert!(!ok);
    assert_eq!(report["refused"], "not_committed");

    // A missing node object.
    let missing = run.directory.path().join("missing");
    copy_tree(&run.source.checkpoint, &missing);
    let object = manifest.completed_nodes[0].outputs_object.as_str().trim_start_matches("b3:").to_string();
    std::fs::remove_file(missing.join("objects").join(object)).expect("object removed");
    let (ok, report) = check("missing-target", &CheckpointRoot::new(&missing), &request, &unchanged);
    assert!(!ok);
    assert_eq!(report["refused"], "object_missing");

    // Crossed execution and binding, a wrong schema, a stale incarnation.
    let mut crossed = request.clone();
    crossed.execution = identity('c');
    assert_eq!(check("crossed-execution", &source_root, &crossed, &unchanged).1["refused"], "crossed_execution");
    let mut crossed = request.clone();
    crossed.binding = Some(identity('c'));
    assert_eq!(check("crossed-binding", &source_root, &crossed, &unchanged).1["refused"], "crossed_binding");
    let mut wrong = request.clone();
    wrong.schema = "lattice.application-restart-request.v0".to_string();
    assert_eq!(check("wrong-schema", &source_root, &wrong, &unchanged).1["refused"], "schema");
    let mut stale = request.clone();
    stale.restarted_incarnation = 0;
    assert_eq!(check("stale", &source_root, &stale, &unchanged).1["refused"], "stale_admission");

    // A target binding may not extend the deadline or move the attempt binding.
    let later = |root: &mut Root| root.binding.deadline_unix_ms = DEADLINE + 1;
    assert_eq!(check("later-deadline", &source_root, &request, &later).1["refused"], "carry_forward");
    let moved = |root: &mut Root| root.binding.binding = Some(identity('d'));
    assert_eq!(check("moved-binding", &source_root, &request, &moved).1["refused"], "crossed_binding");

    // A target whose profile declares no checkpoint cannot restore.
    let unavailable = |root: &mut Root| root.binding.checkpoint = CheckpointProfile::None;
    assert_eq!(check("unavailable", &source_root, &request, &unavailable).1["refused"], "checkpoint_unavailable");

    // An older checkpoint keeps the newer budget and marks what later manifests started as
    // uncertain, so nothing is replayed.
    let (older_identity, older) = committed
        .iter()
        .rev()
        .filter_map(|(_, identity)| {
            let bytes = source_root.manifest_bytes(identity).ok()??;
            Some((identity.clone(), serde_json::from_slice::<Manifest>(&bytes).ok()?))
        })
        .find(|(_, manifest)| manifest.completed_nodes.len() == 1)
        .expect("an older manifest with one completed node");
    let before = (count(&run.effects, "a"), count(&run.effects, "b"));
    let request = restart_request(&older_identity, &older, 1, ADMISSION);
    let (ok, report) = check("older", &source_root, &request, &unchanged);
    assert!(ok, "older checkpoint restores: {report}");
    assert_eq!(report["execution_state"], "unknown");
    assert_eq!((count(&run.effects, "a"), count(&run.effects, "b")), before, "nothing replayed");
    let older_store = lattice::store::Store::new(lattice::config::store_path(&run.directory.path().join("older/data")))
        .expect("older store");
    let adoption = older_store.load_workflow_exchange_adoption(&request.execution).expect("load").expect("adoption");
    assert!(
        adoption.consumed.elapsed_ms >= manifest.consumed.elapsed_ms,
        "elapsed budget of the newest manifest is kept"
    );
    assert_eq!(adoption.deadline_unix_ms, DEADLINE);
}

// r[verify lattice.application_checkpoint.restore]
// r[verify lattice.application_checkpoint.workspace]
/// A target workspace holding different content refuses and adopts nothing, on every retry. A
/// restore that committed and was never followed by an exchange (a crash after the commit) is
/// complete: the identical retry succeeds and the served exchange runs it to completion.
#[test]
fn workspace_conflict_adopts_nothing_and_a_committed_adoption_is_complete() {
    let files: &[(&str, &str)] = &[("note.txt", "from the source")];
    let run = completed_source(CheckpointProfile::WorkflowNodeBoundaryV1, Some(files));
    let source_root = CheckpointRoot::new(&run.source.checkpoint);
    let (checkpoint, manifest) = latest(&source_root);
    assert_eq!(manifest.workspace_objects.len(), 1);
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);
    let mut target_root = root(
        &run.directory.path().join("target"),
        &run.effects,
        &run.socket,
        CheckpointProfile::WorkflowNodeBoundaryV1,
    );
    let workspace = run.directory.path().join("target-workspace");
    std::fs::create_dir_all(&workspace).expect("target workspace");
    std::fs::write(workspace.join("note.txt"), "later edits").expect("conflicting file");
    target_root.binding.workspace = Some(workspace.clone());

    for attempt in 0..2 {
        let (ok, report) = cli_restore(&target_root, &source_root, &request);
        assert!(!ok, "attempt {attempt}: {report}");
        assert_eq!(report["refused"], "workspace_path");
        let target_store = store(&target_root);
        assert!(target_store.load_workflow_exchange(&request.execution).expect("load").is_none(), "nothing adopted");
        assert!(target_store.load_workflow_exchange_adoption(&request.execution).expect("load").is_none());
    }
    assert_eq!(std::fs::read_to_string(workspace.join("note.txt")).expect("note"), "later edits");

    std::fs::remove_file(workspace.join("note.txt")).expect("conflict cleared");
    let (ok, report) = cli_restore(&target_root, &source_root, &request);
    assert!(ok, "restore succeeds: {report}");
    assert_eq!(std::fs::read_to_string(workspace.join("note.txt")).expect("note"), "from the source");
    let (repeat, again) = cli_restore(&target_root, &source_root, &request);
    assert!(repeat, "the identical retry after a crash is idempotent: {again}");
    assert_eq!(again["receipt"], report["receipt"]);
    assert_eq!(again["execution_state"], "queued", "the retry does not claim the run finished");
    let before = (count(&run.effects, "a"), count(&run.effects, "b"));
    let exchange = Exchange::start(&target_root);
    let observation = exchange.wait_terminal(&run.accepted);
    exchange.kill();
    assert_eq!(observation.state, ExecutionState::Completed);
    assert_eq!((count(&run.effects, "a"), count(&run.effects, "b")), before, "nothing replayed");
}

// r[verify lattice.application_checkpoint.hook]
/// Without a declared checkpoint the route publishes nothing and reports `Unavailable`.
#[test]
fn route_without_checkpoint_publishes_nothing() {
    let run = completed_source(CheckpointProfile::None, None);
    let handler = lattice::workflow_exchange::ApplicationHandler::new(
        store(&run.source),
        run.source.profile.clone(),
        lattice::agent_node::RuntimeOptions::default(),
    )
    .expect("handler")
    .with_application_checkpoint(run.source.binding.clone())
    .expect("checkpoint binding");
    assert_eq!(handler.checkpoint_offer(), CheckpointOffer::Unavailable);
    assert!(!run.source.checkpoint.exists(), "no checkpoint root is written without hooks");
    assert_eq!((count(&run.effects, "a"), count(&run.effects, "b")), (1, 1));
}

/// `inspect` lists committed manifests through the process boundary.
#[test]
fn inspect_lists_committed_manifests() {
    let run = completed_source(CheckpointProfile::WorkflowNodeBoundaryV1, None);
    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(inputs(&run.source).config)
        .args(["application-checkpoint", "inspect", "--root"])
        .arg(&run.source.checkpoint)
        .output()
        .expect("inspect runs");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("inspect JSON");
    let committed = report["committed"].as_array().expect("committed");
    assert_eq!(committed.len(), CheckpointRoot::new(&run.source.checkpoint).committed().expect("committed").len());
    assert_eq!(committed.last().expect("latest")["completed_nodes"], 2);
}

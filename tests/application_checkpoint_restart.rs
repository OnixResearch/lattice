//! Q1: application checkpoint restart across two state roots on one host (Lattice #12).
//!
//! The source is a real process (this test binary re-executed in the `source` role) running the
//! production `ApplicationHandler`, worker and executor with real shell effects. It is SIGKILLed,
//! its state root is discarded, and only the transferred checkpoint root survives. The target is
//! the real `lattice application-checkpoint restore` process on a second, fresh state root.
//! Shell effects append to counter files outside both roots, an independent oracle for replay.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use lattice::checkpoint::*;
use lattice_workflow_contract::*;
use lattice_workflow_transport::ExchangeCommand;
use lattice_workflow_transport::ExchangeSuccess;
use lattice_workflow_transport::WorkflowExchangeHandler as _;

const ROLE_ENV: &str = "LATTICE_APPLICATION_CHECKPOINT_TEST_ROLE";
const DIR_ENV: &str = "LATTICE_APPLICATION_CHECKPOINT_TEST_DIR";
const ROLE_SOURCE: &str = "source";
const WORKFLOW: &str = "checkpoint-restart";
const IDENTITY_DIGITS: usize = 64;
const DEADLINE: u64 = 4_102_444_800_000;
const WAIT: Duration = Duration::from_secs(180);
const POLL: Duration = Duration::from_millis(20);
const ADMISSION: &str = "kiln-admission:q1-test";
/// Far beyond every bounded test wait, so a held gate never times out on a loaded host.
const GATE_TIMEOUT_SECS: u64 = 900;

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
        idempotency_key: identity('2'),
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
fn gated(effects: &Path, name: &str) -> String {
    let effects = effects.display();
    format!(
        "cat > /dev/null; printf {name} >> '{effects}/{name}'; while [ ! -e '{effects}/release-{name}' ]; do sleep 0.05; done; printf {name}"
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

struct Root {
    data: PathBuf,
    checkpoint: PathBuf,
    profile: lattice::workflow_exchange::HandlerProfile,
    binding: CheckpointBinding,
}

fn root(directory: &Path, effects: &Path, checkpoint: CheckpointProfile) -> Root {
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
        // Part of the handler profile identity: both roots must serve the identical profile.
        socket_path: PathBuf::from("/run/lattice/workflow-exchange.sock"),
        transport: lattice_workflow_transport::TransportProfile {
            schema: lattice_workflow_transport::TRANSPORT_PROFILE_SCHEMA.to_string(),
            limits: lattice_workflow_transport::TransportLimits {
                maximum_frame_bytes: 1_048_576,
                maximum_log_bytes: 65_536,
                maximum_connections: 8,
                read_deadline_milliseconds: 5_000,
                write_deadline_milliseconds: 5_000,
            },
            peers: lattice_workflow_transport::PeerPolicy {
                allowed_user_ids: vec![1_000],
            },
            non_claims: lattice_workflow_transport::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
        },
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
        data,
        checkpoint: checkpoint_root,
        profile,
        binding,
    }
}

fn store(root: &Root) -> lattice::store::Store {
    lattice::store::Store::new(lattice::config::store_path(&root.data)).expect("store opens")
}

fn handler(root: &Root) -> lattice::workflow_exchange::ApplicationHandler {
    lattice::workflow_exchange::ApplicationHandler::new(
        store(root),
        root.profile.clone(),
        lattice::agent_node::RuntimeOptions::default(),
    )
    .expect("handler")
    .with_application_checkpoint(root.binding.clone())
    .expect("checkpoint binding")
}

fn dispatch(handler: &mut lattice::workflow_exchange::ApplicationHandler) -> WorkflowAccepted {
    match handler.handle(&ExchangeCommand::Dispatch {
        profile: contract_profile(),
        request: workflow_request(),
    }) {
        Ok(ExchangeSuccess::Accepted(accepted)) => accepted,
        other => panic!("dispatch must be accepted: {other:?}"),
    }
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

/// Re-executed entry point; a no-op in an ordinary test run.
#[test]
fn application_checkpoint_role_entry() {
    let Ok(role) = std::env::var(ROLE_ENV) else {
        return;
    };
    assert_eq!(role, ROLE_SOURCE);
    let directory = PathBuf::from(std::env::var(DIR_ENV).expect("role directory"));
    let source = root(&directory.join("source"), &directory.join("effects"), CheckpointProfile::WorkflowNodeBoundaryV1);
    let mut handler = handler(&source);
    let accepted = dispatch(&mut handler);
    std::fs::write(directory.join("accepted.json"), serde_json::to_vec(&accepted).expect("accepted JSON"))
        .expect("accepted written");
    loop {
        std::thread::park();
    }
}

struct Killed {
    directory: tempfile::TempDir,
    effects: PathBuf,
    transfer: CheckpointRoot,
}

/// Start the source process, run `until` against it, SIGKILL it, keep only a copy of its
/// checkpoint root, and discard its state root.
fn run_and_kill_source(prepare: impl FnOnce(&Path), until: impl FnOnce(&Path, &Path)) -> Killed {
    let directory = tempfile::tempdir().expect("temporary directory");
    let effects = directory.path().join("effects");
    std::fs::create_dir_all(&effects).expect("effects directory");
    let source_checkpoint = directory.path().join("source/checkpoint");
    std::fs::create_dir_all(&source_checkpoint).expect("source checkpoint root");
    prepare(&source_checkpoint);
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "application_checkpoint_role_entry",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env(ROLE_ENV, ROLE_SOURCE)
        .env(DIR_ENV, directory.path())
        .spawn()
        .expect("source process starts");
    until(&source_checkpoint, &effects);
    child.kill().expect("SIGKILL source");
    child.wait().expect("source reaped");
    let transfer = directory.path().join("transfer");
    copy_tree(&source_checkpoint, &transfer);
    std::fs::remove_dir_all(directory.path().join("source")).expect("source state root discarded");
    Killed {
        directory,
        effects,
        transfer: CheckpointRoot::new(transfer),
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
    }
}

/// A CLI configuration whose data directory is `data`, so no command touches the operator's.
fn write_config(directory: &Path, data: &Path) -> PathBuf {
    let config = directory.join("config.ncl");
    std::fs::write(
        &config,
        format!(
            "{{ data_dir = {:?}, default_timeout_secs = 60, shell = \"sh\", max_parallelism = 1 }}\n",
            data.display().to_string()
        ),
    )
    .expect("config");
    config
}

/// Run the real `lattice application-checkpoint restore` against a target state root.
fn cli_restore(
    target: &Root,
    directory: &Path,
    source: &CheckpointRoot,
    request: &RestartRequest,
) -> (bool, serde_json::Value) {
    let profile = directory.join("profile.ncl");
    let binding = directory.join("binding.json");
    let request_path = directory.join("request.json");
    std::fs::write(directory.join("profile.json"), serde_json::to_vec(&target.profile).expect("profile JSON"))
        .expect("profile data");
    std::fs::write(&profile, "import \"profile.json\"\n").expect("profile");
    std::fs::write(&binding, serde_json::to_vec(&target.binding).expect("binding JSON")).expect("binding");
    std::fs::write(&request_path, serde_json::to_vec(request).expect("request JSON")).expect("request");
    let config = write_config(directory, &target.data);
    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config)
        .args(["application-checkpoint", "restore", "--profile"])
        .arg(&profile)
        .arg("--binding")
        .arg(&binding)
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

fn target(killed: &Killed, name: &str, checkpoint: CheckpointProfile) -> (Root, PathBuf) {
    let directory = killed.directory.path().join(name);
    (root(&directory, &killed.effects, checkpoint), directory)
}

// r[verify lattice.application_checkpoint.restore]
// r[verify lattice.application_checkpoint.fence]
// r[verify lattice.application_checkpoint.receipt]
/// Crash after an external-effect node: the source drains after `a`, is SIGKILLed and loses its
/// state root; the target restarts from the drained manifest, never re-runs `a`, runs `b` once.
#[test]
fn drained_source_restarts_on_a_second_root_after_sigkill() {
    let killed = run_and_kill_source(
        |_| {},
        |checkpoint, effects| {
            wait_for("node a to start", || count(effects, "a") == 1);
            std::fs::write(checkpoint.join("drain-request"), b"").expect("drain requested");
            release(effects, "a");
            wait_for("drain acknowledgement", || checkpoint.join("drained.json").exists());
        },
    );
    let transfer = &killed.transfer;
    let drained = transfer.drain_acknowledgement().expect("drain read").expect("drained");
    let (checkpoint, manifest) = latest(transfer);
    assert_eq!(drained.manifest, checkpoint, "the drained manifest is the latest commit");
    assert_eq!(manifest.completed_nodes.len(), 1);
    assert!(manifest.uncertain_nodes.is_empty(), "a drained manifest is quiescent");
    assert_eq!(count(&killed.effects, "b"), 0, "b never started on the source");

    release(&killed.effects, "b");
    let (target_root, directory) = target(&killed, "target", CheckpointProfile::WorkflowNodeBoundaryV1);
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);
    let (success, report) = cli_restore(&target_root, &directory, transfer, &request);
    assert!(success, "restore succeeds: {report}");
    assert_eq!(report["receipt"]["kind"], "application_restart");
    assert_eq!(report["receipt"]["restarted_incarnation"], 1);
    assert_eq!(report["receipt"]["admission"], ADMISSION);
    assert_eq!(report["receipt"]["checkpoint"], checkpoint.as_str());
    assert_eq!(report["execution_state"], "completed", "{}", run_diagnostics(&target_root, &request.execution));
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

    let (repeat, again) = cli_restore(&target_root, &directory, transfer, &request);
    assert!(repeat, "an identical restart is idempotent: {again}");
    assert_eq!(again["receipt"], report["receipt"]);
    assert_eq!(count(&killed.effects, "b"), 1, "an idempotent restart dispatches nothing");
    let stale = restart_request(&checkpoint, &manifest, 1, "kiln-admission:q1-other");
    let (refused, stale_report) = cli_restore(&target_root, &directory, transfer, &stale);
    assert!(!refused);
    assert_eq!(stale_report["refused"], "stale_admission");
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
    let (target_root, directory) = target(&killed, "target", CheckpointProfile::WorkflowNodeBoundaryV1);
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);
    let (success, report) = cli_restore(&target_root, &directory, &killed.transfer, &request);
    release(&killed.effects, "b");
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
    let (target_root, directory) = target(&killed, "target", CheckpointProfile::WorkflowNodeBoundaryV1);
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);
    let (success, report) = cli_restore(&target_root, &directory, &killed.transfer, &request);
    assert!(success, "restore succeeds: {report}");
    assert_eq!(report["execution_state"], "completed", "{}", run_diagnostics(&target_root, &request.execution));
    assert_eq!(count(&killed.effects, "a"), 1);
    assert_eq!(count(&killed.effects, "b"), 1);
}

// r[verify lattice.application_checkpoint.commit]
// r[verify lattice.application_checkpoint.admission]
// r[verify lattice.application_checkpoint.carry_forward]
// r[verify lattice.application_checkpoint.unavailable]
/// Partial publication, missing objects, crossed identities, unavailable profiles and an older
/// checkpoint, all against real roots and the real restore process.
#[test]
fn partial_publication_crossed_identities_and_older_checkpoints_stay_honest() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let effects = directory.path().join("effects");
    std::fs::create_dir_all(&effects).expect("effects directory");
    release(&effects, "a");
    release(&effects, "b");
    let source = root(&directory.path().join("source"), &effects, CheckpointProfile::WorkflowNodeBoundaryV1);
    let mut source_handler = handler(&source);
    dispatch(&mut source_handler);
    source_handler.wait_for_workers();
    drop(source_handler);
    let source_root = CheckpointRoot::new(&source.checkpoint);
    let committed = source_root.committed().expect("committed");
    assert!(committed.len() >= 3, "fence and boundary publications: {committed:?}");
    let (checkpoint, manifest) = latest(&source_root);
    assert_eq!(manifest.completed_nodes.len(), 2);

    let check = |name: &str, root: &CheckpointRoot, request: &RestartRequest, profile: CheckpointProfile| {
        let target_directory = directory.path().join(name);
        let target_root = self::root(&target_directory, &effects, profile);
        cli_restore(&target_root, &target_directory, root, request)
    };
    let request = restart_request(&checkpoint, &manifest, 1, ADMISSION);

    // A manifest whose commit marker was never written (crash between objects and marker).
    let partial = directory.path().join("partial");
    copy_tree(&source.checkpoint, &partial);
    let marker = std::fs::read_dir(partial.join("commits"))
        .expect("commits")
        .map(|entry| entry.expect("entry").path())
        .find(|path| path.to_string_lossy().contains(checkpoint.as_str().trim_start_matches("b3:")))
        .expect("latest marker");
    std::fs::remove_file(marker).expect("marker removed");
    let (ok, report) = check(
        "partial-target",
        &CheckpointRoot::new(&partial),
        &request,
        CheckpointProfile::WorkflowNodeBoundaryV1,
    );
    assert!(!ok);
    assert_eq!(report["refused"], "not_committed");

    // A missing node object.
    let missing = directory.path().join("missing");
    copy_tree(&source.checkpoint, &missing);
    let object = manifest.completed_nodes[0].outputs_object.as_str().trim_start_matches("b3:").to_string();
    std::fs::remove_file(missing.join("objects").join(object)).expect("object removed");
    let (ok, report) = check(
        "missing-target",
        &CheckpointRoot::new(&missing),
        &request,
        CheckpointProfile::WorkflowNodeBoundaryV1,
    );
    assert!(!ok);
    assert_eq!(report["refused"], "object_missing");

    // Crossed execution and binding, a wrong schema, a stale incarnation.
    let mut crossed = request.clone();
    crossed.execution = identity('c');
    assert_eq!(
        check("crossed-execution", &source_root, &crossed, CheckpointProfile::WorkflowNodeBoundaryV1).1["refused"],
        "crossed_execution"
    );
    let mut crossed = request.clone();
    crossed.binding = Some(identity('c'));
    assert_eq!(
        check("crossed-binding", &source_root, &crossed, CheckpointProfile::WorkflowNodeBoundaryV1).1["refused"],
        "crossed_binding"
    );
    let mut wrong = request.clone();
    wrong.schema = "lattice.application-restart-request.v0".to_string();
    assert_eq!(
        check("wrong-schema", &source_root, &wrong, CheckpointProfile::WorkflowNodeBoundaryV1).1["refused"],
        "schema"
    );
    let mut stale = request.clone();
    stale.restarted_incarnation = 0;
    assert_eq!(
        check("stale", &source_root, &stale, CheckpointProfile::WorkflowNodeBoundaryV1).1["refused"],
        "stale_admission"
    );

    // A target whose profile declares no checkpoint cannot restore.
    assert_eq!(
        check("unavailable", &source_root, &request, CheckpointProfile::None).1["refused"],
        "checkpoint_unavailable"
    );

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
    let before = (count(&effects, "a"), count(&effects, "b"));
    let request = restart_request(&older_identity, &older, 1, ADMISSION);
    let (ok, report) = check("older", &source_root, &request, CheckpointProfile::WorkflowNodeBoundaryV1);
    assert!(ok, "older checkpoint restores: {report}");
    assert_eq!(report["execution_state"], "unknown");
    assert_eq!((count(&effects, "a"), count(&effects, "b")), before, "nothing replayed");
    let older_store = lattice::store::Store::new(lattice::config::store_path(&directory.path().join("older/data")))
        .expect("older store");
    let adoption = older_store.load_workflow_exchange_adoption(&request.execution).expect("load").expect("adoption");
    assert!(
        adoption.consumed.elapsed_ms >= manifest.consumed.elapsed_ms,
        "budget of the newest manifest is kept"
    );
    assert_eq!(adoption.deadline_unix_ms, DEADLINE);
}

// r[verify lattice.application_checkpoint.hook]
/// Without a declared checkpoint the route publishes nothing and reports `Unavailable`.
#[test]
fn route_without_checkpoint_publishes_nothing() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let effects = directory.path().join("effects");
    std::fs::create_dir_all(&effects).expect("effects directory");
    release(&effects, "a");
    release(&effects, "b");
    let source = root(&directory.path().join("source"), &effects, CheckpointProfile::None);
    let mut handler = handler(&source);
    assert_eq!(handler.checkpoint_offer(), CheckpointOffer::Unavailable);
    dispatch(&mut handler);
    handler.wait_for_workers();
    assert!(!source.checkpoint.exists(), "no checkpoint root is written without hooks");
    assert_eq!((count(&effects, "a"), count(&effects, "b")), (1, 1));
}

/// `inspect` lists committed manifests through the process boundary.
#[test]
fn inspect_lists_committed_manifests() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let effects = directory.path().join("effects");
    std::fs::create_dir_all(&effects).expect("effects directory");
    release(&effects, "a");
    release(&effects, "b");
    let source = root(&directory.path().join("source"), &effects, CheckpointProfile::WorkflowNodeBoundaryV1);
    let mut handler = handler(&source);
    dispatch(&mut handler);
    handler.wait_for_workers();
    let config = write_config(directory.path(), &directory.path().join("inspect-data"));
    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config)
        .args(["application-checkpoint", "inspect", "--root"])
        .arg(&source.checkpoint)
        .output()
        .expect("inspect runs");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("inspect JSON");
    let committed = report["committed"].as_array().expect("committed");
    assert_eq!(committed.len(), CheckpointRoot::new(&source.checkpoint).committed().expect("committed").len());
    assert_eq!(committed.last().expect("latest")["completed_nodes"], 2);
}

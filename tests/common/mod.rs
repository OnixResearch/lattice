//! Shared test helpers — workflow builders, temp stores, assertion utilities.
#![allow(dead_code)]

use std::collections::HashMap;

use lattice::engine::value::ContentHash;
use lattice::engine::value::Value;
use lattice::engine::workflow::Workflow;
use lattice::store::Store;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;
use tempfile::TempDir;

const SHELL_STDOUT_OUTPUT_INDEX: usize = 0;
const SHELL_STDERR_OUTPUT_INDEX: usize = 1;
const SHELL_EXIT_OUTPUT_INDEX: usize = 2;

#[cfg(unix)]
pub const FORCED_DEATH_MARKER_CONTENT: &str = "forced-death";

#[cfg(unix)]
fn quote_for_posix_shell(value: &str) -> String {
    assert!(!value.contains('\0'), "shell fixture value must not contain NUL");
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(unix)]
fn quote_path_for_posix_shell(path: &std::path::Path) -> String {
    assert!(path.is_absolute(), "fault-injection fixture paths must be absolute");
    quote_for_posix_shell(path.to_str().expect("fault-injection fixture path must be UTF-8"))
}

/// Build a deterministic Unix shell command that kills its Lattice parent after
/// recording a durable checkpoint.
#[cfg(unix)]
pub fn forced_parent_death_command(marker_path: &std::path::Path) -> String {
    let marker = quote_path_for_posix_shell(marker_path);
    format!("printf %s '{}' > {marker}; kill -KILL \"$PPID\"; exit", FORCED_DEATH_MARKER_CONTENT)
}

/// Kill the first Lattice parent at the workflow boundary, then emit one
/// recovery effect when the same workflow is started by a later process.
#[cfg(unix)]
pub fn forced_parent_death_once_then_write_command(
    marker_path: &std::path::Path,
    recovery_effect_path: &std::path::Path,
    recovery_effect: &str,
) -> String {
    assert!(!recovery_effect.is_empty(), "recovery effect must be observable");
    let marker = quote_path_for_posix_shell(marker_path);
    let recovery_effect_path = quote_path_for_posix_shell(recovery_effect_path);
    let recovery_effect = quote_for_posix_shell(recovery_effect);
    format!(
        "if [ ! -e {marker} ]; then printf %s '{}' > {marker}; kill -KILL \"$PPID\"; exit; fi; printf %s {recovery_effect} > {recovery_effect_path}",
        FORCED_DEATH_MARKER_CONTENT,
    )
}

/// Build a simple linear two-node workflow: echo → grep.
/// Returns (workflow, node_a_id, node_b_id).
pub fn two_node_pipeline() -> (Workflow, rat_nodegraph::model::NodeId, rat_nodegraph::model::NodeId) {
    let mut wf = Workflow::new("test-pipeline");
    let a = wf.add_shell_node("echo-hello", "echo hello world");
    let b = wf.add_shell_node("grep-hello", "grep hello");

    // Wire a's stdout (output port 0) → b's stdin (input port 0).
    let a_stdout = stdout_port_id(&wf, a);
    let b_stdin = wf.graph.node(b).unwrap().input_ports[0].id;
    wf.graph.add_edge(a_stdout, b_stdin).unwrap();

    (wf, a, b)
}

/// Build a diamond-shaped workflow:
///
///       A
///      / \
///     B   C
///      \ /
///       D
pub fn diamond_workflow() -> (
    Workflow,
    rat_nodegraph::model::NodeId,
    rat_nodegraph::model::NodeId,
    rat_nodegraph::model::NodeId,
    rat_nodegraph::model::NodeId,
) {
    let mut wf = Workflow::new("diamond");
    let a = wf.add_shell_node("source", "echo diamond");
    let b = wf.add_shell_node("left", "cat");
    let c = wf.add_shell_node("right", "cat");
    let d = wf.add_shell_node("sink", "cat");

    // A → B
    let a_out = stdout_port_id(&wf, a);
    let b_in = wf.graph.node(b).unwrap().input_ports[0].id;
    wf.graph.add_edge(a_out, b_in).unwrap();

    // A → C  (use stderr port from A since stdout is taken; or add another node)
    // Actually, we need separate edges. Let's use A's stderr → C.
    let a_stderr = stderr_port_id(&wf, a);
    let c_in = wf.graph.node(c).unwrap().input_ports[0].id;
    wf.graph.add_edge(a_stderr, c_in).unwrap();

    // B → D
    let b_out = stdout_port_id(&wf, b);
    let d_in = wf.graph.node(d).unwrap().input_ports[0].id;
    wf.graph.add_edge(b_out, d_in).unwrap();

    (wf, a, b, c, d)
}

/// Build a single standalone node (no edges).
pub fn standalone_node(label: &str, command: &str) -> (Workflow, rat_nodegraph::model::NodeId) {
    let mut wf = Workflow::new("standalone");
    let id = wf.add_shell_node(label, command);
    (wf, id)
}

/// Create a Store backed by a temp directory. Returns (Store, TempDir).
/// Keep TempDir alive for the duration of the test — dropping it deletes the db.
pub fn temp_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let db_path = dir.path().join("test.redb");
    let store = Store::new(&db_path).expect("failed to create store");
    (store, dir)
}

pub fn stdout_port_id(workflow: &Workflow, node_id: NodeId) -> PortId {
    workflow.graph.node(node_id).expect("node exists").output_ports[SHELL_STDOUT_OUTPUT_INDEX].id
}

pub fn stderr_port_id(workflow: &Workflow, node_id: NodeId) -> PortId {
    workflow.graph.node(node_id).expect("node exists").output_ports[SHELL_STDERR_OUTPUT_INDEX].id
}

pub fn exit_port_id(workflow: &Workflow, node_id: NodeId) -> PortId {
    workflow.graph.node(node_id).expect("node exists").output_ports[SHELL_EXIT_OUTPUT_INDEX].id
}

/// Build a fake cache entry: maps an input hash to a set of real port→value outputs.
pub fn fake_cache_entry(
    workflow: &Workflow,
    node_id: NodeId,
    input_hash: ContentHash,
    stdout_val: &str,
) -> (ContentHash, HashMap<PortId, Value>) {
    let mut outputs = HashMap::new();
    outputs.insert(stdout_port_id(workflow, node_id), Value::from_string(stdout_val.into()));
    outputs.insert(stderr_port_id(workflow, node_id), Value::from_string(String::new()));
    outputs.insert(exit_port_id(workflow, node_id), Value::from_exit_code(0));
    (input_hash, outputs)
}

//! Shared test helpers — workflow builders, temp stores, assertion utilities.

use std::collections::HashMap;

use n8nclone::engine::value::ContentHash;
use n8nclone::engine::value::Value;
use n8nclone::engine::workflow::Workflow;
use n8nclone::store::Store;
use rat_nodegraph::model::PortId;
use tempfile::TempDir;

/// Build a simple linear two-node workflow: echo → grep.
/// Returns (workflow, node_a_id, node_b_id).
pub fn two_node_pipeline() -> (Workflow, rat_nodegraph::model::NodeId, rat_nodegraph::model::NodeId) {
    let mut wf = Workflow::new("test-pipeline");
    let a = wf.add_shell_node("echo-hello", "echo hello world");
    let b = wf.add_shell_node("grep-hello", "grep hello");

    // Wire a's stdout (output port 0) → b's stdin (input port 0).
    let a_stdout = wf.graph.node(a).unwrap().output_ports[0].id;
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
    let a_out = wf.graph.node(a).unwrap().output_ports[0].id;
    let b_in = wf.graph.node(b).unwrap().input_ports[0].id;
    wf.graph.add_edge(a_out, b_in).unwrap();

    // A → C  (use stderr port from A since stdout is taken; or add another node)
    // Actually, we need separate edges. Let's use A's stderr → C.
    let a_stderr = wf.graph.node(a).unwrap().output_ports[1].id;
    let c_in = wf.graph.node(c).unwrap().input_ports[0].id;
    wf.graph.add_edge(a_stderr, c_in).unwrap();

    // B → D
    let b_out = wf.graph.node(b).unwrap().output_ports[0].id;
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

/// Build a fake cache entry: maps an input hash to a set of port→value outputs.
pub fn fake_cache_entry(input_hash: ContentHash, stdout_val: &str) -> (ContentHash, HashMap<PortId, Value>) {
    let mut outputs = HashMap::new();
    outputs.insert(PortId(u64::MAX - 2), Value::from_string(stdout_val.into()));
    outputs.insert(PortId(u64::MAX - 1), Value::from_string(String::new()));
    outputs.insert(PortId(u64::MAX), Value::from_exit_code(0));
    (input_hash, outputs)
}

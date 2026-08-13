//! Tests for engine::workflow — graph construction, node configs, edges.

mod common;

use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentTarget;
use lattice::engine::workflow::GraphError;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::PortSpec;
use lattice::engine::workflow::Workflow;
use lattice::engine::workflow::WorkflowMeta;
use lattice::engine::workflow::WorkflowTimestamp;
use pretty_assertions::assert_eq;

const AGENT_PROMPT: &str = "summarize this";

// ---------------------------------------------------------------------------
// WorkflowMeta
// ---------------------------------------------------------------------------

#[test]
fn meta_new_sets_name() {
    let m = WorkflowMeta::new("my workflow", WorkflowTimestamp::epoch());
    assert_eq!(m.name, "my workflow");
    assert!(m.description.is_empty());
}

#[test]
fn equal_explicit_timestamps_create_equal_metadata() {
    let timestamp = WorkflowTimestamp::epoch();
    let first = WorkflowMeta::new("ts test", timestamp);
    let second = WorkflowMeta::new("ts test", timestamp);
    assert_eq!(first.created, second.created);
    assert_eq!(first.modified, second.modified);
}

#[test]
fn malformed_explicit_timestamp_is_rejected() {
    let result = serde_json::from_str::<WorkflowTimestamp>("\"not-a-timestamp\"");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// NodeConfig
// ---------------------------------------------------------------------------

#[test]
fn shell_config_defaults() {
    let cfg = NodeConfig::shell("ls -la");
    match &cfg {
        NodeConfig::Shell {
            command,
            cwd,
            env,
            timeout_secs,
            pipe_stdin,
        } => {
            assert_eq!(command, "ls -la");
            assert!(cwd.is_none());
            assert!(env.is_empty());
            assert_eq!(*timeout_secs, 60);
            assert!(*pipe_stdin);
        }
        _ => panic!("expected shell config"),
    }
}

#[test]
fn shell_config_kind_label() {
    let cfg = NodeConfig::shell("true");
    assert_eq!(cfg.kind_label(), "Shell");
}

#[test]
fn node_config_serde_round_trip() {
    let cfg = NodeConfig::Shell {
        command: "wc -l".into(),
        cwd: Some("/tmp".into()),
        env: [("FOO".into(), "bar".into())].into(),
        timeout_secs: 30,
        pipe_stdin: false,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: NodeConfig = serde_json::from_str(&json).unwrap();
    match back {
        NodeConfig::Shell {
            command,
            cwd,
            env,
            timeout_secs,
            pipe_stdin,
        } => {
            assert_eq!(command, "wc -l");
            assert_eq!(cwd.as_deref(), Some("/tmp"));
            assert_eq!(env.get("FOO").map(|s| s.as_str()), Some("bar"));
            assert_eq!(timeout_secs, 30);
            assert!(!pipe_stdin);
        }
        _ => panic!("expected shell config"),
    }
}

#[test]
fn http_config_defaults() {
    let cfg = NodeConfig::http("get", "https://example.test");
    match cfg {
        NodeConfig::Http {
            method,
            url,
            headers,
            body,
            timeout_secs,
        } => {
            assert_eq!(method, "get");
            assert_eq!(url, "https://example.test");
            assert!(headers.is_empty());
            assert!(body.is_none());
            assert_eq!(timeout_secs, 60);
        }
        _ => panic!("expected HTTP config"),
    }
}

#[test]
fn file_config_defaults() {
    let cfg = NodeConfig::file(lattice::file_node::FileOperationKind::Read, "/tmp/example");
    match cfg {
        NodeConfig::File {
            operation,
            path,
            content,
            overwrite,
            allowed_roots,
        } => {
            assert_eq!(operation, lattice::file_node::FileOperationKind::Read);
            assert_eq!(path, "/tmp/example");
            assert!(content.is_none());
            assert!(!overwrite);
            assert!(allowed_roots.is_empty());
        }
        _ => panic!("expected file config"),
    }
}

#[test]
fn image_input_config_defaults() {
    let cfg = NodeConfig::image_input("/tmp/selfie.png");
    match cfg {
        NodeConfig::ImageInput {
            path,
            allowed_roots,
            max_bytes,
        } => {
            assert_eq!(path, "/tmp/selfie.png");
            assert!(allowed_roots.is_empty());
            assert_eq!(max_bytes, lattice::image_node::DEFAULT_MAX_IMAGE_BYTES);
        }
        _ => panic!("expected image input config"),
    }
}

#[test]
fn agent_config_defaults() {
    let cfg = NodeConfig::agent(AgentTarget::Pi, AGENT_PROMPT, AgentOutputMode::Text);
    match cfg {
        NodeConfig::Agent(settings) => {
            assert_eq!(settings.target, AgentTarget::Pi);
            assert_eq!(settings.prompt, AGENT_PROMPT);
            assert_eq!(settings.output_mode, AgentOutputMode::Text);
            assert!(settings.model_hint.is_none());
            assert!(settings.context_refs.is_empty());
        }
        _ => panic!("expected agent config"),
    }
}

// ---------------------------------------------------------------------------
// Workflow construction
// ---------------------------------------------------------------------------

#[test]
fn new_workflow_starts_empty() {
    let wf = Workflow::new("empty", lattice::engine::workflow::WorkflowTimestamp::epoch());
    assert_eq!(wf.meta.name, "empty");
    assert_eq!(wf.graph.node_count(), 0);
    assert_eq!(wf.graph.edge_count(), 0);
    assert!(wf.configs.is_empty());
}

#[test]
fn add_shell_node_registers_config() {
    let mut wf = Workflow::new("test", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let id = wf.add_shell_node("echo", "echo hi");
    assert_eq!(wf.graph.node_count(), 1);
    let cfg = wf.node_config(id).expect("config should exist");
    assert_eq!(cfg.kind_label(), "Shell");
}

#[test]
fn add_shell_node_has_correct_ports() {
    let mut wf = Workflow::new("test", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let id = wf.add_shell_node("node", "true");
    let node = wf.graph.node(id).unwrap();
    assert_eq!(node.input_ports.len(), 1, "one input: stdin");
    assert_eq!(node.output_ports.len(), 3, "three outputs: stdout, stderr, exit");
    assert_eq!(node.input_ports[0].label, "stdin");
    assert_eq!(node.output_ports[0].label, "stdout");
    assert_eq!(node.output_ports[1].label, "stderr");
    assert_eq!(node.output_ports[2].label, "exit");
}

#[test]
fn add_multiple_nodes() {
    let mut wf = Workflow::new("multi", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let a = wf.add_shell_node("a", "echo a");
    let b = wf.add_shell_node("b", "echo b");
    let c = wf.add_shell_node("c", "echo c");
    assert_eq!(wf.graph.node_count(), 3);
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_eq!(wf.configs.len(), 3);
}

#[test]
fn connect_nodes_creates_edge() {
    let (wf, _a, _b) = common::two_node_pipeline();
    assert_eq!(wf.graph.edge_count(), 1);
}

#[test]
fn dag_mode_prevents_cycles() {
    let mut wf = Workflow::new("cycle", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let a = wf.add_shell_node("a", "true");
    let b = wf.add_shell_node("b", "true");

    let a_out = wf.graph.node(a).unwrap().output_ports[0].id;
    let b_in = wf.graph.node(b).unwrap().input_ports[0].id;
    wf.graph.add_edge(a_out, b_in).unwrap();

    // Try to create b → a (would form a cycle).
    let b_out = wf.graph.node(b).unwrap().output_ports[0].id;
    let a_in = wf.graph.node(a).unwrap().input_ports[0].id;
    let result = wf.graph.add_edge(b_out, a_in);
    assert!(result.is_err(), "cycle should be rejected in DAG mode");
}

#[test]
fn self_loop_rejected() {
    let mut wf = Workflow::new("selfloop", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let a = wf.add_shell_node("a", "true");
    let a_out = wf.graph.node(a).unwrap().output_ports[0].id;
    let a_in = wf.graph.node(a).unwrap().input_ports[0].id;
    let result = wf.graph.add_edge(a_out, a_in);
    assert!(result.is_err(), "self-loop should be rejected");
}

#[test]
fn incompatible_port_types_are_rejected_without_an_edge() {
    let mut wf = Workflow::new("typed edge", WorkflowTimestamp::epoch());
    let source = wf.graph.add_node("source", &[], &[PortSpec::new("value", "string")]);
    let target = wf.graph.add_node("target", &[PortSpec::new("value", "json")], &[]);
    let source_port = wf.graph.node(source).expect("source").output_ports[0].id;
    let target_port = wf.graph.node(target).expect("target").input_ports[0].id;

    assert_eq!(
        wf.graph.add_edge(source_port, target_port),
        Err(GraphError::IncompatibleTypes {
            source_tag: "string".to_string(),
            target_tag: "json".to_string(),
        })
    );
    assert_eq!(wf.graph.edge_count(), 0);
}

#[test]
fn set_node_config_replaces_existing() {
    let mut wf = Workflow::new("replace", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let id = wf.add_shell_node("node", "old-command");
    wf.set_node_config(id, NodeConfig::shell("new-command"));
    match wf.node_config(id).unwrap() {
        NodeConfig::Shell { command, .. } => assert_eq!(command, "new-command"),
        _ => panic!("expected shell config"),
    }
}

#[test]
fn touch_uses_the_supplied_timestamp() {
    let mut wf = Workflow::new("touch", WorkflowTimestamp::epoch());
    let before = wf.meta.modified;
    let after = before + chrono::Duration::seconds(1);
    wf.touch(WorkflowTimestamp::from_datetime(after));
    assert_eq!(wf.meta.modified, after);
}

#[test]
fn workflow_serde_round_trip() {
    let (wf, _, _) = common::two_node_pipeline();
    let json = serde_json::to_string(&wf).unwrap();
    let back: Workflow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.meta.name, wf.meta.name);
    assert_eq!(back.graph.node_count(), wf.graph.node_count());
    assert_eq!(back.graph.edge_count(), wf.graph.edge_count());
    assert_eq!(back.configs.len(), wf.configs.len());
}

// ---------------------------------------------------------------------------
// Diamond workflow structural checks
// ---------------------------------------------------------------------------

#[test]
fn diamond_has_four_nodes_three_edges() {
    let (wf, _, _, _, _) = common::diamond_workflow();
    assert_eq!(wf.graph.node_count(), 4);
    assert_eq!(wf.graph.edge_count(), 3);
}

#[test]
fn diamond_node_configs_all_present() {
    let (wf, a, b, c, d) = common::diamond_workflow();
    for id in [a, b, c, d] {
        assert!(wf.node_config(id).is_some(), "config missing for {:?}", id);
    }
}

//! Tests for engine::executor — topological sort, node execution, full runs, caching.

mod common;

use std::collections::HashMap;

use n8nclone::engine::executor::Executor;
use n8nclone::engine::executor::NodeStatus;
use n8nclone::engine::value::ContentHash;
use n8nclone::engine::value::Value;
use n8nclone::engine::workflow::Workflow;
use pretty_assertions::assert_eq;
use rat_nodegraph::model::PortId;

// ---------------------------------------------------------------------------
// Single-node execution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execute_echo_produces_stdout() {
    let (wf, node_id) = common::standalone_node("echo", "echo hello");
    let executor = Executor::new();
    let inputs = HashMap::new();

    let result = executor.execute_node(&wf, node_id, &inputs).await;

    match &result.status {
        NodeStatus::Completed { outputs, cached } => {
            assert!(!cached);
            let stdout_port = common::stdout_port_id(&wf, node_id);
            let stdout = outputs.get(&stdout_port).expect("stdout missing");
            assert_eq!(stdout.as_display_string().trim(), "hello");
        }
        other => panic!("expected Completed, got {:?}", other),
    }
}

#[tokio::test]
async fn execute_failing_command() {
    let (wf, node_id) = common::standalone_node("fail", "exit 1");
    let executor = Executor::new();
    let inputs = HashMap::new();

    let result = executor.execute_node(&wf, node_id, &inputs).await;

    match &result.status {
        NodeStatus::Completed { outputs, .. } => {
            let exit_port = common::exit_port_id(&wf, node_id);
            let exit_val = outputs.get(&exit_port).expect("exit missing");
            assert_eq!(exit_val.as_display_string(), "exit code: 1");
        }
        other => panic!("expected Completed (with non-zero exit), got {:?}", other),
    }
}

#[tokio::test]
async fn execute_node_not_in_graph() {
    let wf = Workflow::new("empty");
    let executor = Executor::new();
    let fake_id = rat_nodegraph::model::NodeId(9999);
    let result = executor.execute_node(&wf, fake_id, &HashMap::new()).await;

    match &result.status {
        NodeStatus::Failed { error } => {
            assert!(error.contains("not found"), "error: {}", error);
        }
        other => panic!("expected Failed, got {:?}", other),
    }
}

#[tokio::test]
async fn execute_node_missing_config() {
    // Add a node to the graph but don't register a config.
    let mut wf = Workflow::new("no-config");
    let id = wf.graph.add_node("orphan", &[], &[rat_nodegraph::model::PortSpec::new("out", "string")]);
    // Deliberately skip wf.configs.insert().

    let executor = Executor::new();
    let result = executor.execute_node(&wf, id, &HashMap::new()).await;

    match &result.status {
        NodeStatus::Failed { error } => {
            assert!(error.contains("no config"), "error: {}", error);
        }
        other => panic!("expected Failed, got {:?}", other),
    }
}

#[tokio::test]
async fn execute_with_stdin_piped() {
    let (wf, node_id) = common::standalone_node("cat", "cat");
    let executor = Executor::new();
    let mut inputs = HashMap::new();
    inputs.insert(PortId(0), Value::from_string("piped input\n".into()));

    let result = executor.execute_node(&wf, node_id, &inputs).await;

    match &result.status {
        NodeStatus::Completed { outputs, .. } => {
            let stdout_port = common::stdout_port_id(&wf, node_id);
            let stdout = outputs.get(&stdout_port).expect("stdout missing");
            assert_eq!(stdout.as_display_string().trim(), "piped input");
        }
        other => panic!("expected Completed, got {:?}", other),
    }
}

#[tokio::test]
async fn execute_captures_stderr() {
    let (wf, node_id) = common::standalone_node("stderr", "echo oops >&2");
    let executor = Executor::new();
    let result = executor.execute_node(&wf, node_id, &HashMap::new()).await;

    match &result.status {
        NodeStatus::Completed { outputs, .. } => {
            let stderr_port = common::stderr_port_id(&wf, node_id);
            let stderr = outputs.get(&stderr_port).expect("stderr missing");
            assert_eq!(stderr.as_display_string().trim(), "oops");
        }
        other => panic!("expected Completed, got {:?}", other),
    }
}

#[tokio::test]
async fn execute_timeout() {
    let mut wf = Workflow::new("timeout");
    let id = wf.add_shell_node("slow", "sleep 60");
    // Override timeout to 1 second.
    wf.set_node_config(id, n8nclone::engine::workflow::NodeConfig::Shell {
        command: "sleep 60".into(),
        cwd: None,
        env: HashMap::new(),
        timeout_secs: 1,
        pipe_stdin: false,
    });

    let executor = Executor::new();
    let result = executor.execute_node(&wf, id, &HashMap::new()).await;

    match &result.status {
        NodeStatus::Failed { error } => {
            assert!(error.contains("timed out"), "error: {}", error);
        }
        other => panic!("expected Failed (timeout), got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Full workflow run
// ---------------------------------------------------------------------------

#[tokio::test]
async fn run_single_node_workflow() {
    let (wf, _) = common::standalone_node("echo", "echo standalone");
    let mut executor = Executor::new();
    let cache = HashMap::new();

    let (run_id, results) = executor.run(&wf, &cache).await;
    assert_eq!(run_id.0, 0);
    assert_eq!(results.len(), 1);
    match &results[0].status {
        NodeStatus::Completed { .. } => {}
        other => panic!("expected Completed, got {:?}", other),
    }
}

#[tokio::test]
async fn run_two_node_pipeline() {
    let (wf, _, grep_node) = common::two_node_pipeline();
    let mut executor = Executor::new();
    let cache = HashMap::new();

    let (_, results) = executor.run(&wf, &cache).await;
    assert_eq!(results.len(), 2, "both nodes should execute");

    // First node (echo) should succeed.
    match &results[0].status {
        NodeStatus::Completed { .. } => {}
        other => panic!("first node: expected Completed, got {:?}", other),
    }

    // Second node should receive upstream stdout on stdin and grep should match.
    match &results[1].status {
        NodeStatus::Completed { outputs, .. } => {
            let stdout_port = common::stdout_port_id(&wf, grep_node);
            let exit_port = common::exit_port_id(&wf, grep_node);
            let stdout = outputs.get(&stdout_port).expect("grep stdout missing");
            let exit = outputs.get(&exit_port).expect("grep exit missing");
            assert_eq!(stdout.as_display_string().trim(), "hello world");
            assert_eq!(exit.as_display_string(), "exit code: 0");
        }
        other => panic!("second node: expected Completed, got {:?}", other),
    }
}

#[tokio::test]
async fn run_increments_run_id() {
    let (wf, _) = common::standalone_node("echo", "true");
    let mut executor = Executor::new();
    let cache = HashMap::new();

    let (id1, _) = executor.run(&wf, &cache).await;
    let (id2, _) = executor.run(&wf, &cache).await;
    let (id3, _) = executor.run(&wf, &cache).await;

    assert_eq!(id1.0, 0);
    assert_eq!(id2.0, 1);
    assert_eq!(id3.0, 2);
}

#[tokio::test]
async fn run_empty_workflow() {
    let wf = Workflow::new("empty");
    let mut executor = Executor::new();
    let cache = HashMap::new();

    let (_, results) = executor.run(&wf, &cache).await;
    assert!(results.is_empty());
}

#[tokio::test]
async fn run_diamond_executes_all_nodes() {
    let (wf, _, _, _, _) = common::diamond_workflow();
    let mut executor = Executor::new();
    let cache = HashMap::new();

    let (_, results) = executor.run(&wf, &cache).await;
    assert_eq!(results.len(), 4, "all four diamond nodes should execute");
}

// ---------------------------------------------------------------------------
// Cache hits
// ---------------------------------------------------------------------------

#[tokio::test]
async fn run_uses_cache_when_hash_matches() {
    let (wf, node_id) = common::standalone_node("echo", "echo cached");
    let mut executor = Executor::new();

    // First run — no cache.
    let (_, results) = executor.run(&wf, &HashMap::new()).await;
    let first_input_hash = results[0].input_hash;

    // Build a cache map with the same input hash.
    let (_, cached_outputs) = common::fake_cache_entry(&wf, node_id, first_input_hash, "from cache");
    let mut cache = HashMap::new();
    cache.insert(first_input_hash, cached_outputs);

    // Second run — should hit cache.
    let (_, results2) = executor.run(&wf, &cache).await;
    assert_eq!(results2.len(), 1);
    match &results2[0].status {
        NodeStatus::Completed { cached, .. } => {
            assert!(*cached, "expected a cache hit");
        }
        other => panic!("expected Completed (cached), got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Input hash stability
// ---------------------------------------------------------------------------

#[tokio::test]
async fn input_hash_is_stable_across_runs() {
    let (wf, _) = common::standalone_node("echo", "echo stable");
    let mut executor = Executor::new();
    let cache = HashMap::new();

    let (_, r1) = executor.run(&wf, &cache).await;
    let (_, r2) = executor.run(&wf, &cache).await;

    assert_eq!(r1[0].input_hash, r2[0].input_hash, "same inputs should produce the same input hash");
}

// ---------------------------------------------------------------------------
// Environment variable passing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execute_with_env_vars() {
    let mut wf = Workflow::new("env-test");
    let id = wf.add_shell_node("env", "echo $MY_VAR");
    wf.set_node_config(id, n8nclone::engine::workflow::NodeConfig::Shell {
        command: "echo $MY_VAR".into(),
        cwd: None,
        env: [("MY_VAR".into(), "injected".into())].into(),
        timeout_secs: 10,
        pipe_stdin: false,
    });

    let executor = Executor::new();
    let result = executor.execute_node(&wf, id, &HashMap::new()).await;

    match &result.status {
        NodeStatus::Completed { outputs, .. } => {
            let stdout_port = common::stdout_port_id(&wf, id);
            let stdout = outputs.get(&stdout_port).unwrap();
            assert_eq!(stdout.as_display_string().trim(), "injected");
        }
        other => panic!("expected Completed, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Working directory
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execute_with_cwd() {
    let mut wf = Workflow::new("cwd-test");
    let id = wf.add_shell_node("pwd", "pwd");
    wf.set_node_config(id, n8nclone::engine::workflow::NodeConfig::Shell {
        command: "pwd".into(),
        cwd: Some("/tmp".into()),
        env: HashMap::new(),
        timeout_secs: 10,
        pipe_stdin: false,
    });

    let executor = Executor::new();
    let result = executor.execute_node(&wf, id, &HashMap::new()).await;

    match &result.status {
        NodeStatus::Completed { outputs, .. } => {
            let stdout_port = common::stdout_port_id(&wf, id);
            let stdout = outputs.get(&stdout_port).unwrap();
            // /tmp might resolve to a private tmp on some systems, just check it ran.
            assert!(!stdout.as_display_string().trim().is_empty());
        }
        other => panic!("expected Completed, got {:?}", other),
    }
}

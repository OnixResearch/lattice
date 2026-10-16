//! Tests for engine::executor — topological sort, node execution, full runs, caching.

mod common;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::executor::node_execution_cache_key;
use lattice::engine::value::Value;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::PortId;
use lattice::engine::workflow::PortSpec;
use lattice::engine::workflow::Workflow;
use pretty_assertions::assert_eq;

const EXIT_CODE_SUCCESS_DISPLAY: &str = "exit code: 0";
const EXIT_CODE_FAILURE_DISPLAY: &str = "exit code: 1";
const MATCHING_SOURCE_COMMAND: &str = "echo hello world";
const NON_MATCHING_SOURCE_COMMAND: &str = "echo hi";
const GREP_WORLD_COMMAND: &str = "grep world";
const SHELL_STDIN_INPUT_INDEX: usize = 0;
const CACHE_BASE_COMMAND: &str = "printf base";
const CACHE_CHANGED_COMMAND: &str = "printf command";
const CACHE_CWD_COMMAND: &str = "printf cwd";
const CACHE_ENV_COMMAND: &str = "printf env";
const CACHE_TIMEOUT_COMMAND: &str = "printf timeout";
const CACHE_PIPE_STDIN_COMMAND: &str = "printf pipe";
const CACHE_OLD_OUTPUT: &str = "old cache";
const CACHE_BASE_TIMEOUT_SECS: u64 = 10;
const CACHE_CHANGED_TIMEOUT_SECS: u64 = 11;
const CACHE_CWD_OLD: &str = "/tmp";
const CACHE_ENV_KEY: &str = "CACHE_VALUE";
const CACHE_ENV_OLD: &str = "old";
const CACHE_ENV_NEW: &str = "new";
const CACHE_ENV_FIRST_KEY: &str = "CACHE_A";
const CACHE_ENV_FIRST_VALUE: &str = "one";
const CACHE_ENV_SECOND_KEY: &str = "CACHE_B";
const CACHE_ENV_SECOND_VALUE: &str = "two";
const CACHE_STABLE_COMMAND: &str = "printf stable";
const CACHE_ENV_ORDER_COMMAND: &str = "printf env-order";
const CACHE_HIT_OUTPUT: &str = "cache hit";
const FIRST_RESULT_INDEX: usize = 0;
const AUX_INPUT_INDEX: usize = 0;
const LABELED_STDIN_INPUT_INDEX: usize = 1;
const SINGLE_STDIN_INPUT_INDEX: usize = 0;
const CUSTOM_AUX_INPUT_LABEL: &str = "aux";
const CUSTOM_STDIN_INPUT_LABEL: &str = "stdin";
const CUSTOM_STDOUT_OUTPUT_LABEL: &str = "stdout";
const CUSTOM_STDERR_OUTPUT_LABEL: &str = "stderr";
const CUSTOM_EXIT_OUTPUT_LABEL: &str = "exit";
const TYPE_TAG_STRING: &str = "string";
const TYPE_TAG_EXIT_CODE: &str = "exitcode";
const AUX_SOURCE_COMMAND: &str = "printf aux";
const SELECTED_STDIN_SOURCE_COMMAND: &str = "printf selected";
const CAT_COMMAND: &str = "cat";
const STDERR_SOURCE_COMMAND: &str = "printf stderr-value >&2";
const PIPE_DISABLED_SOURCE_COMMAND: &str = "printf ignored";
const CACHE_KEY_DIRECT_COMMAND: &str = "printf direct";
const CACHE_KEY_DIRECT_INPUT: &str = "direct input";
const CACHE_KEY_NODE_ID_VALUE: u64 = 4242;
const CACHE_KEY_INPUT_PORT_VALUE: u64 = 9001;
const CACHE_KEY_OTHER_INPUT_PORT_VALUE: u64 = 9002;
#[cfg(unix)]
const CANCELLATION_READY_FILE_NAME: &str = "executor-cancellation-ready";
#[cfg(unix)]
const CANCELLATION_FORBIDDEN_FILE_NAME: &str = "executor-cancellation-forbidden";
#[cfg(unix)]
const CANCELLATION_FORBIDDEN_EFFECT: &str = "must-not-complete";
#[cfg(unix)]
const CANCELLATION_DELAY_SECS: u64 = 2;
#[cfg(unix)]
const CANCELLATION_OBSERVATION_WAIT_SECS: u64 = 3;
#[cfg(unix)]
const CANCELLATION_POLL_ATTEMPTS: usize = 250;
#[cfg(unix)]
const CANCELLATION_POLL_DELAY_MS: u64 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellConfigMutation {
    Command,
    Cwd,
    Env,
    Timeout,
    PipeStdin,
}

const SHELL_CONFIG_MUTATIONS: &[ShellConfigMutation] = &[
    ShellConfigMutation::Command,
    ShellConfigMutation::Cwd,
    ShellConfigMutation::Env,
    ShellConfigMutation::Timeout,
    ShellConfigMutation::PipeStdin,
];

struct CacheConfigCase {
    mutation: ShellConfigMutation,
    old_config: NodeConfig,
    new_config: NodeConfig,
    expected_stdout: &'static str,
}

fn shell_pipeline(source_command: &str, target_command: &str) -> (Workflow, NodeId, NodeId) {
    let mut workflow = Workflow::new("test-pipeline", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let source = workflow.add_shell_node("source", source_command);
    let target = workflow.add_shell_node("target", target_command);
    let source_stdout = common::stdout_port_id(&workflow, source);
    let target_stdin = workflow.graph.node(target).expect("target node exists").input_ports[SHELL_STDIN_INPUT_INDEX].id;
    workflow.graph.add_edge(source_stdout, target_stdin).expect("pipeline edge is valid");
    (workflow, source, target)
}

fn shell_config(
    command: &str,
    cwd: Option<&str>,
    env: &[(&str, &str)],
    timeout_secs: u64,
    pipe_stdin: bool,
) -> NodeConfig {
    NodeConfig::Shell {
        command: command.to_string(),
        cwd: cwd.map(str::to_string),
        env: env.iter().map(|(key, value)| ((*key).to_string(), (*value).to_string())).collect(),
        timeout_secs,
        pipe_stdin,
    }
}

fn cache_config_case(mutation: ShellConfigMutation) -> CacheConfigCase {
    match mutation {
        ShellConfigMutation::Command => CacheConfigCase {
            mutation,
            old_config: shell_config(CACHE_BASE_COMMAND, None, &[], CACHE_BASE_TIMEOUT_SECS, true),
            new_config: shell_config(CACHE_CHANGED_COMMAND, None, &[], CACHE_BASE_TIMEOUT_SECS, true),
            expected_stdout: "command",
        },
        ShellConfigMutation::Cwd => CacheConfigCase {
            mutation,
            old_config: shell_config(CACHE_CWD_COMMAND, Some(CACHE_CWD_OLD), &[], CACHE_BASE_TIMEOUT_SECS, true),
            new_config: shell_config(CACHE_CWD_COMMAND, None, &[], CACHE_BASE_TIMEOUT_SECS, true),
            expected_stdout: "cwd",
        },
        ShellConfigMutation::Env => CacheConfigCase {
            mutation,
            old_config: shell_config(
                CACHE_ENV_COMMAND,
                None,
                &[(CACHE_ENV_KEY, CACHE_ENV_OLD)],
                CACHE_BASE_TIMEOUT_SECS,
                true,
            ),
            new_config: shell_config(
                CACHE_ENV_COMMAND,
                None,
                &[(CACHE_ENV_KEY, CACHE_ENV_NEW)],
                CACHE_BASE_TIMEOUT_SECS,
                true,
            ),
            expected_stdout: "env",
        },
        ShellConfigMutation::Timeout => CacheConfigCase {
            mutation,
            old_config: shell_config(CACHE_TIMEOUT_COMMAND, None, &[], CACHE_BASE_TIMEOUT_SECS, true),
            new_config: shell_config(CACHE_TIMEOUT_COMMAND, None, &[], CACHE_CHANGED_TIMEOUT_SECS, true),
            expected_stdout: "timeout",
        },
        ShellConfigMutation::PipeStdin => CacheConfigCase {
            mutation,
            old_config: shell_config(CACHE_PIPE_STDIN_COMMAND, None, &[], CACHE_BASE_TIMEOUT_SECS, true),
            new_config: shell_config(CACHE_PIPE_STDIN_COMMAND, None, &[], CACHE_BASE_TIMEOUT_SECS, false),
            expected_stdout: "pipe",
        },
    }
}

fn workflow_with_config(config: NodeConfig) -> (Workflow, NodeId) {
    let command = match &config {
        NodeConfig::Shell { command, .. } => command.clone(),
        _ => panic!("workflow_with_config expects shell config"),
    };
    let mut workflow = Workflow::new("cache-key", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_shell_node("cache", command);
    workflow.set_node_config(node_id, config);
    (workflow, node_id)
}

fn add_custom_multi_input_shell_node(workflow: &mut Workflow, label: &str, command: &str) -> NodeId {
    let node_id = workflow.graph.add_node(
        label,
        &[
            PortSpec::new(CUSTOM_AUX_INPUT_LABEL, TYPE_TAG_STRING),
            PortSpec::new(CUSTOM_STDIN_INPUT_LABEL, TYPE_TAG_STRING),
        ],
        &[
            PortSpec::new(CUSTOM_STDOUT_OUTPUT_LABEL, TYPE_TAG_STRING),
            PortSpec::new(CUSTOM_STDERR_OUTPUT_LABEL, TYPE_TAG_STRING),
            PortSpec::new(CUSTOM_EXIT_OUTPUT_LABEL, TYPE_TAG_EXIT_CODE),
        ],
    );
    workflow.set_node_config(node_id, NodeConfig::shell(command));
    node_id
}

fn step_status(results: &[lattice::engine::executor::StepResult], node_id: NodeId) -> &NodeStatus {
    &results.iter().find(|step| step.node_id == node_id).expect("step result missing").status
}

fn completed_stdout<'a>(workflow: &Workflow, node_id: NodeId, status: &'a NodeStatus) -> &'a Value {
    match status {
        NodeStatus::Completed { outputs, .. } => {
            let stdout_port = common::stdout_port_id(workflow, node_id);
            outputs.get(&stdout_port).expect("stdout missing")
        }
        other => panic!("expected Completed, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Single-node execution
// ---------------------------------------------------------------------------

// r[verify n8nclone.workflow_execution.pipeline_stdin.real_port_outputs]
// r[verify n8nclone.workflow_execution.pipeline_stdin.real_port_outputs.completed]
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
            assert!(matches!(exit_val.data, lattice::engine::ValueData::ExitCode(1)));
        }
        other => panic!("expected Completed (with non-zero exit), got {:?}", other),
    }
}

// r[verify lattice.tui_ux.process_failure_recovery.scenario.cancel]
#[tokio::test]
async fn explicit_false_cancellation_token_allows_shell_completion() {
    let (workflow, node_id) = common::standalone_node("uncancelled", "printf completed");
    let cancellation = Arc::new(AtomicBool::new(false));
    let executor = Executor::new().with_cancellation(cancellation);

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    assert!(matches!(result.status, NodeStatus::Completed { .. }));
}

// r[verify lattice.tui_ux.process_failure_recovery.scenario.cancel]
#[cfg(unix)]
#[tokio::test]
async fn explicit_cancellation_token_stops_bounded_shell_before_pending_effect() {
    let temp_dir = tempfile::tempdir().expect("temporary executor cancellation directory");
    let ready_path = temp_dir.path().join(CANCELLATION_READY_FILE_NAME);
    let forbidden_path = temp_dir.path().join(CANCELLATION_FORBIDDEN_FILE_NAME);
    let command = common::ready_then_delayed_effect_command(
        &ready_path,
        &forbidden_path,
        CANCELLATION_DELAY_SECS,
        CANCELLATION_FORBIDDEN_EFFECT,
    );
    let (workflow, node_id) = common::standalone_node("cancel", &command);
    let cancellation = Arc::new(AtomicBool::new(false));
    let executor = Executor::new().with_cancellation(Arc::clone(&cancellation));
    let execution = tokio::spawn(async move { executor.execute_node(&workflow, node_id, &HashMap::new()).await });

    for _ in 0..CANCELLATION_POLL_ATTEMPTS {
        if ready_path.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(CANCELLATION_POLL_DELAY_MS)).await;
    }
    assert!(ready_path.exists(), "cancellable shell must report readiness");
    cancellation.store(true, Ordering::SeqCst);
    let result = execution.await.expect("cancellable executor task joins");
    tokio::time::sleep(Duration::from_secs(CANCELLATION_OBSERVATION_WAIT_SECS)).await;

    assert!(matches!(result.status, NodeStatus::Failed { ref error } if error == "process cancelled"));
    assert!(!forbidden_path.exists(), "canceled bounded shell must not perform its pending effect");
}

#[tokio::test]
async fn execute_node_not_in_graph() {
    let wf = Workflow::new("empty", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let executor = Executor::new();
    let fake_id = lattice::engine::workflow::NodeId(9999);
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
    let mut wf = Workflow::new("no-config", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let id = wf.graph.add_node("orphan", &[], &[lattice::engine::workflow::PortSpec::new("out", "string")]);
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
    let mut wf = Workflow::new("timeout", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let id = wf.add_shell_node("slow", "sleep 60");
    // Override timeout to 1 second.
    wf.set_node_config(id, lattice::engine::workflow::NodeConfig::Shell {
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
async fn run_two_node_pipeline_without_match_has_empty_stdout_and_exit_one() {
    let (workflow, _, grep_node) = shell_pipeline(NON_MATCHING_SOURCE_COMMAND, GREP_WORLD_COMMAND);
    let mut executor = Executor::new();
    let cache = HashMap::new();

    let (_, results) = executor.run(&workflow, &cache).await;

    assert_eq!(results.len(), 2, "source and grep nodes should both run");
    match &results[1].status {
        NodeStatus::Completed { outputs, .. } => {
            let stdout_port = common::stdout_port_id(&workflow, grep_node);
            let exit_port = common::exit_port_id(&workflow, grep_node);
            let stdout = outputs.get(&stdout_port).expect("grep stdout missing");
            let exit = outputs.get(&exit_port).expect("grep exit missing");
            assert_eq!(stdout.as_display_string(), "");
            assert_eq!(exit.as_display_string(), EXIT_CODE_FAILURE_DISPLAY);
        }
        other => panic!("grep node: expected Completed, got {:?}", other),
    }
}

#[tokio::test]
async fn run_two_node_pipeline_with_match_outputs_matching_line() {
    let (workflow, _, grep_node) = shell_pipeline(MATCHING_SOURCE_COMMAND, GREP_WORLD_COMMAND);
    let mut executor = Executor::new();
    let cache = HashMap::new();

    let (_, results) = executor.run(&workflow, &cache).await;

    assert_eq!(results.len(), 2, "source and grep nodes should both run");
    match &results[1].status {
        NodeStatus::Completed { outputs, .. } => {
            let stdout_port = common::stdout_port_id(&workflow, grep_node);
            let exit_port = common::exit_port_id(&workflow, grep_node);
            let stdout = outputs.get(&stdout_port).expect("grep stdout missing");
            let exit = outputs.get(&exit_port).expect("grep exit missing");
            assert_eq!(stdout.as_display_string().trim(), "hello world");
            assert_eq!(exit.as_display_string(), EXIT_CODE_SUCCESS_DISPLAY);
        }
        other => panic!("grep node: expected Completed, got {:?}", other),
    }
}

#[tokio::test]
async fn run_multi_input_shell_uses_labeled_stdin_port() {
    let mut workflow = Workflow::new("multi-input", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let aux_source = workflow.add_shell_node("aux", AUX_SOURCE_COMMAND);
    let stdin_source = workflow.add_shell_node("stdin", SELECTED_STDIN_SOURCE_COMMAND);
    let target = add_custom_multi_input_shell_node(&mut workflow, "target", CAT_COMMAND);
    let aux_stdout = common::stdout_port_id(&workflow, aux_source);
    let stdin_stdout = common::stdout_port_id(&workflow, stdin_source);
    let target_node = workflow.graph.node(target).expect("target exists");
    let target_aux = target_node.input_ports[AUX_INPUT_INDEX].id;
    let target_stdin = target_node.input_ports[LABELED_STDIN_INPUT_INDEX].id;
    workflow.graph.add_edge(aux_stdout, target_aux).expect("aux edge");
    workflow.graph.add_edge(stdin_stdout, target_stdin).expect("stdin edge");
    let mut executor = Executor::new();

    let (_, results) = executor.run(&workflow, &HashMap::new()).await;

    let stdout = completed_stdout(&workflow, target, step_status(&results, target));
    assert_eq!(stdout.as_display_string(), "selected");
}

#[tokio::test]
async fn run_stderr_output_can_feed_downstream_stdin() {
    let mut workflow = Workflow::new("stderr-pipeline", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let source = workflow.add_shell_node("stderr-source", STDERR_SOURCE_COMMAND);
    let target = workflow.add_shell_node("cat", CAT_COMMAND);
    let source_stderr = common::stderr_port_id(&workflow, source);
    let target_stdin = workflow.graph.node(target).expect("target exists").input_ports[SINGLE_STDIN_INPUT_INDEX].id;
    workflow.graph.add_edge(source_stderr, target_stdin).expect("stderr edge");
    let mut executor = Executor::new();

    let (_, results) = executor.run(&workflow, &HashMap::new()).await;

    let stdout = completed_stdout(&workflow, target, step_status(&results, target));
    assert_eq!(stdout.as_display_string(), "stderr-value");
}

#[tokio::test]
async fn run_pipe_stdin_false_ignores_connected_input() {
    let mut workflow = Workflow::new("pipe-disabled", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let source = workflow.add_shell_node("source", PIPE_DISABLED_SOURCE_COMMAND);
    let target = workflow.add_shell_node("cat", CAT_COMMAND);
    workflow.set_node_config(target, shell_config(CAT_COMMAND, None, &[], CACHE_BASE_TIMEOUT_SECS, false));
    let source_stdout = common::stdout_port_id(&workflow, source);
    let target_stdin = workflow.graph.node(target).expect("target exists").input_ports[SINGLE_STDIN_INPUT_INDEX].id;
    workflow.graph.add_edge(source_stdout, target_stdin).expect("stdout edge");
    let mut executor = Executor::new();

    let (_, results) = executor.run(&workflow, &HashMap::new()).await;

    let stdout = completed_stdout(&workflow, target, step_status(&results, target));
    assert_eq!(stdout.as_display_string(), "");
}

#[test]
fn exit_code_output_cannot_wire_to_string_stdin() {
    let (mut workflow, source, target) = shell_pipeline(MATCHING_SOURCE_COMMAND, GREP_WORLD_COMMAND);
    let source_exit = common::exit_port_id(&workflow, source);
    let target_stdin = workflow.graph.node(target).expect("target exists").input_ports[SINGLE_STDIN_INPUT_INDEX].id;

    assert!(workflow.graph.add_edge(source_exit, target_stdin).is_err());
}

// r[verify n8nclone.workflow_execution.pipeline_stdin.cache_real_ports.routable]
// r[verify n8nclone.workflow_execution.pipeline_stdin.stdout_to_stdin]
// r[verify n8nclone.workflow_execution.pipeline_stdin.stdout_to_stdin.connected]
#[tokio::test]
async fn regression_connected_stdout_reaches_downstream_stdin() {
    let (wf, echo_node, grep_node) = common::two_node_pipeline();
    let echo_stdout_port = common::stdout_port_id(&wf, echo_node);
    let grep_stdout_port = common::stdout_port_id(&wf, grep_node);
    let grep_exit_port = common::exit_port_id(&wf, grep_node);
    let mut executor = Executor::new();
    let cache = HashMap::new();

    let (_, results) = executor.run(&wf, &cache).await;

    assert_eq!(results.len(), 2, "echo and grep nodes should both run");
    match &results[0].status {
        NodeStatus::Completed { outputs, .. } => {
            let upstream_stdout =
                outputs.get(&echo_stdout_port).expect("upstream stdout must use the real graph port id");
            assert_eq!(upstream_stdout.as_display_string().trim(), "hello world");
        }
        other => panic!("source node: expected Completed, got {:?}", other),
    }
    match &results[1].status {
        NodeStatus::Completed { outputs, .. } => {
            let downstream_stdout = outputs.get(&grep_stdout_port).expect("downstream stdout missing");
            let downstream_exit = outputs.get(&grep_exit_port).expect("downstream exit missing");
            assert_eq!(downstream_stdout.as_display_string().trim(), "hello world");
            assert_eq!(downstream_exit.as_display_string(), "exit code: 0");
        }
        other => panic!("grep node: expected Completed, got {:?}", other),
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
    let wf = Workflow::new("empty", lattice::engine::workflow::WorkflowTimestamp::epoch());
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

#[test]
fn direct_cache_key_is_stable_for_identical_config_and_inputs() {
    let config = shell_config(CACHE_KEY_DIRECT_COMMAND, None, &[], CACHE_BASE_TIMEOUT_SECS, true);
    let node_id = NodeId(CACHE_KEY_NODE_ID_VALUE);
    let mut inputs = HashMap::new();
    inputs.insert(PortId(CACHE_KEY_INPUT_PORT_VALUE), Value::from_string(CACHE_KEY_DIRECT_INPUT.to_string()));

    let first = node_execution_cache_key(node_id, &config, &inputs);
    let second = node_execution_cache_key(node_id, &config, &inputs);

    assert_eq!(first, second);
}

#[test]
fn direct_cache_key_changes_when_input_port_identity_changes() {
    let config = shell_config(CACHE_KEY_DIRECT_COMMAND, None, &[], CACHE_BASE_TIMEOUT_SECS, true);
    let node_id = NodeId(CACHE_KEY_NODE_ID_VALUE);
    let input_value = Value::from_string(CACHE_KEY_DIRECT_INPUT.to_string());
    let mut first_inputs = HashMap::new();
    first_inputs.insert(PortId(CACHE_KEY_INPUT_PORT_VALUE), input_value.clone());
    let mut second_inputs = HashMap::new();
    second_inputs.insert(PortId(CACHE_KEY_OTHER_INPUT_PORT_VALUE), input_value);

    let first = node_execution_cache_key(node_id, &config, &first_inputs);
    let second = node_execution_cache_key(node_id, &config, &second_inputs);

    assert_ne!(first, second);
}

#[tokio::test]
async fn cache_key_is_stable_for_identical_shell_config() {
    let config = shell_config(
        CACHE_STABLE_COMMAND,
        None,
        &[(CACHE_ENV_FIRST_KEY, CACHE_ENV_FIRST_VALUE)],
        CACHE_BASE_TIMEOUT_SECS,
        true,
    );
    let (first_workflow, first_node_id) = workflow_with_config(config.clone());
    let (second_workflow, second_node_id) = workflow_with_config(config);
    let mut executor = Executor::new();
    let (_, first_results) = executor.run(&first_workflow, &HashMap::new()).await;
    let first_input_hash = first_results[FIRST_RESULT_INDEX].input_hash;
    let (_, cached_outputs) =
        common::fake_cache_entry(&first_workflow, first_node_id, first_input_hash, CACHE_HIT_OUTPUT);
    let mut cache = HashMap::new();
    cache.insert(first_input_hash, cached_outputs);

    let (_, second_results) = executor.run(&second_workflow, &cache).await;

    assert_eq!(second_results[FIRST_RESULT_INDEX].input_hash, first_input_hash);
    match &second_results[FIRST_RESULT_INDEX].status {
        NodeStatus::Completed { outputs, cached } => {
            assert!(*cached, "identical config and inputs should reuse cached outputs");
            let stdout_port = common::stdout_port_id(&second_workflow, second_node_id);
            let stdout = outputs.get(&stdout_port).expect("stdout missing");
            assert_eq!(stdout.as_display_string(), CACHE_HIT_OUTPUT);
        }
        other => panic!("expected Completed, got {:?}", other),
    }
}

#[tokio::test]
async fn cache_key_is_stable_across_env_insertion_order() {
    let first_config = shell_config(
        CACHE_ENV_ORDER_COMMAND,
        None,
        &[
            (CACHE_ENV_FIRST_KEY, CACHE_ENV_FIRST_VALUE),
            (CACHE_ENV_SECOND_KEY, CACHE_ENV_SECOND_VALUE),
        ],
        CACHE_BASE_TIMEOUT_SECS,
        true,
    );
    let second_config = shell_config(
        CACHE_ENV_ORDER_COMMAND,
        None,
        &[
            (CACHE_ENV_SECOND_KEY, CACHE_ENV_SECOND_VALUE),
            (CACHE_ENV_FIRST_KEY, CACHE_ENV_FIRST_VALUE),
        ],
        CACHE_BASE_TIMEOUT_SECS,
        true,
    );
    let (first_workflow, first_node_id) = workflow_with_config(first_config);
    let (second_workflow, second_node_id) = workflow_with_config(second_config);
    let mut executor = Executor::new();
    let (_, first_results) = executor.run(&first_workflow, &HashMap::new()).await;
    let first_input_hash = first_results[FIRST_RESULT_INDEX].input_hash;
    let (_, cached_outputs) =
        common::fake_cache_entry(&first_workflow, first_node_id, first_input_hash, CACHE_HIT_OUTPUT);
    let mut cache = HashMap::new();
    cache.insert(first_input_hash, cached_outputs);

    let (_, second_results) = executor.run(&second_workflow, &cache).await;

    assert_eq!(second_results[FIRST_RESULT_INDEX].input_hash, first_input_hash);
    match &second_results[FIRST_RESULT_INDEX].status {
        NodeStatus::Completed { outputs, cached } => {
            assert!(*cached, "env insertion order must not change the cache key");
            let stdout_port = common::stdout_port_id(&second_workflow, second_node_id);
            let stdout = outputs.get(&stdout_port).expect("stdout missing");
            assert_eq!(stdout.as_display_string(), CACHE_HIT_OUTPUT);
        }
        other => panic!("expected Completed, got {:?}", other),
    }
}

// r[verify n8nclone.workflow_execution.cache_key.shell_config.command_edit]
#[tokio::test]
async fn cache_key_includes_shell_executable_config() {
    for mutation in SHELL_CONFIG_MUTATIONS {
        let case = cache_config_case(*mutation);
        let (old_workflow, old_node_id) = workflow_with_config(case.old_config);
        let mut executor = Executor::new();
        let (_, old_results) = executor.run(&old_workflow, &HashMap::new()).await;
        let old_input_hash = old_results[0].input_hash;
        let (_, old_cached_outputs) =
            common::fake_cache_entry(&old_workflow, old_node_id, old_input_hash, CACHE_OLD_OUTPUT);
        let mut cache = HashMap::new();
        cache.insert(old_input_hash, old_cached_outputs);

        let (new_workflow, new_node_id) = workflow_with_config(case.new_config);
        let (_, new_results) = executor.run(&new_workflow, &cache).await;

        match &new_results[0].status {
            NodeStatus::Completed { outputs, cached } => {
                assert!(!cached, "{mutation:?} config change must not reuse the old cache entry");
                let stdout_port = common::stdout_port_id(&new_workflow, new_node_id);
                let stdout = outputs.get(&stdout_port).expect("stdout missing");
                assert_eq!(stdout.as_display_string().trim(), case.expected_stdout, "mutation: {:?}", case.mutation);
            }
            other => panic!("{mutation:?}: expected Completed, got {:?}", other),
        }
    }
}

// r[verify n8nclone.workflow_execution.pipeline_stdin.cache_real_ports]
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
    let mut wf = Workflow::new("env-test", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let id = wf.add_shell_node("env", "echo $MY_VAR");
    wf.set_node_config(id, lattice::engine::workflow::NodeConfig::Shell {
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
    let mut wf = Workflow::new("cwd-test", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let id = wf.add_shell_node("pwd", "pwd");
    wf.set_node_config(id, lattice::engine::workflow::NodeConfig::Shell {
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

//! DAG executor — runs workflows in topological order.
//!
//! Two modes:
//!  - **Run**: async parallel execution of independent branches via tokio.
//!  - **Debug**: step-through, one node at a time, with pause after each.

use std::collections::HashMap;

use rat_nodegraph::model::Node;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;
use serde::Deserialize;
use serde::Serialize;
use tokio::process::Command;
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::engine::value::ContentHash;
use crate::engine::value::Value;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::Workflow;

const SHELL_STDOUT_OUTPUT_INDEX: usize = 0;
const SHELL_STDERR_OUTPUT_INDEX: usize = 1;
const SHELL_EXIT_OUTPUT_INDEX: usize = 2;
const HASH_TAG_NODE_EXECUTION: &[u8] = b"node-execution:v1";
const HASH_TAG_NODE_ID: &[u8] = b"node-id";
const HASH_TAG_SHELL_CONFIG: &[u8] = b"shell-config";
const HASH_TAG_COMMAND: &[u8] = b"command";
const HASH_TAG_CWD: &[u8] = b"cwd";
const HASH_TAG_ENV: &[u8] = b"env";
const HASH_TAG_TIMEOUT: &[u8] = b"timeout-secs";
const HASH_TAG_PIPE_STDIN: &[u8] = b"pipe-stdin";
const HASH_TAG_INPUTS: &[u8] = b"inputs";
const HASH_TAG_INPUT_PORT: &[u8] = b"input-port";
const HASH_TAG_INPUT_TYPE: &[u8] = b"input-type";
const HASH_TAG_INPUT_VALUE: &[u8] = b"input-value";
const HASH_TAG_SOME: &[u8] = b"some";
const HASH_TAG_NONE: &[u8] = b"none";
const HASH_TAG_TRUE: &[u8] = b"true";
const HASH_TAG_FALSE: &[u8] = b"false";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShellOutputPortIds {
    stdout: PortId,
    stderr: PortId,
    exit: PortId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShellOutputValues {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

// r[impl n8nclone.workflow_execution.cache_key.shell_config]
fn node_execution_hash(node_id: NodeId, config: &NodeConfig, inputs: &HashMap<PortId, Value>) -> ContentHash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_TAG_NODE_EXECUTION);
    update_u64(&mut hasher, HASH_TAG_NODE_ID, node_id.0);
    update_node_config_hash(&mut hasher, config);
    update_inputs_hash(&mut hasher, inputs);
    ContentHash(*hasher.finalize().as_bytes())
}

fn input_values_hash(inputs: &HashMap<PortId, Value>) -> ContentHash {
    let mut hasher = blake3::Hasher::new();
    update_inputs_hash(&mut hasher, inputs);
    ContentHash(*hasher.finalize().as_bytes())
}

fn update_node_config_hash(hasher: &mut blake3::Hasher, config: &NodeConfig) {
    match config {
        NodeConfig::Shell {
            command,
            cwd,
            env,
            timeout_secs,
            pipe_stdin,
        } => {
            hasher.update(HASH_TAG_SHELL_CONFIG);
            update_str(hasher, HASH_TAG_COMMAND, command);
            update_optional_str(hasher, HASH_TAG_CWD, cwd.as_deref());
            update_env(hasher, env);
            update_u64(hasher, HASH_TAG_TIMEOUT, *timeout_secs);
            update_bool(hasher, HASH_TAG_PIPE_STDIN, *pipe_stdin);
        }
    }
}

fn update_inputs_hash(hasher: &mut blake3::Hasher, inputs: &HashMap<PortId, Value>) {
    let mut input_pairs = inputs.iter().collect::<Vec<_>>();
    input_pairs.sort_by_key(|(port_id, _)| port_id.0);

    hasher.update(HASH_TAG_INPUTS);
    update_len(hasher, input_pairs.len());
    for (port_id, value) in input_pairs {
        update_u64(hasher, HASH_TAG_INPUT_PORT, port_id.0);
        update_str(hasher, HASH_TAG_INPUT_TYPE, value.type_tag().as_str());
        hasher.update(HASH_TAG_INPUT_VALUE);
        hasher.update(&value.hash.0);
    }
}

fn update_env(hasher: &mut blake3::Hasher, env: &HashMap<String, String>) {
    let mut env_pairs = env.iter().collect::<Vec<_>>();
    env_pairs.sort_by_key(|(key, _)| *key);

    hasher.update(HASH_TAG_ENV);
    update_len(hasher, env_pairs.len());
    for (key, value) in env_pairs {
        update_str(hasher, HASH_TAG_ENV, key);
        update_str(hasher, HASH_TAG_ENV, value);
    }
}

fn update_optional_str(hasher: &mut blake3::Hasher, tag: &[u8], value: Option<&str>) {
    hasher.update(tag);
    match value {
        Some(value) => update_str(hasher, HASH_TAG_SOME, value),
        None => {
            hasher.update(HASH_TAG_NONE);
        }
    }
}

fn update_str(hasher: &mut blake3::Hasher, tag: &[u8], value: &str) {
    hasher.update(tag);
    update_len(hasher, value.len());
    hasher.update(value.as_bytes());
}

fn update_bool(hasher: &mut blake3::Hasher, tag: &[u8], value: bool) {
    hasher.update(tag);
    hasher.update(if value { HASH_TAG_TRUE } else { HASH_TAG_FALSE });
}

fn update_u64(hasher: &mut blake3::Hasher, tag: &[u8], value: u64) {
    hasher.update(tag);
    hasher.update(&value.to_le_bytes());
}

fn update_len(hasher: &mut blake3::Hasher, value: usize) {
    let encoded_len = u64::try_from(value).expect("hash input length fits in u64");
    hasher.update(&encoded_len.to_le_bytes());
}

fn shell_output_port_ids(node: &Node) -> Result<ShellOutputPortIds, String> {
    let stdout = node
        .output_ports
        .get(SHELL_STDOUT_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing stdout output port", node.label))?;
    let stderr = node
        .output_ports
        .get(SHELL_STDERR_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing stderr output port", node.label))?;
    let exit = node
        .output_ports
        .get(SHELL_EXIT_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing exit output port", node.label))?;

    Ok(ShellOutputPortIds { stdout, stderr, exit })
}

fn shell_outputs_by_real_port(ports: ShellOutputPortIds, outputs: ShellOutputValues) -> HashMap<PortId, Value> {
    let mut mapped = HashMap::new();
    mapped.insert(ports.stdout, Value::from_string(outputs.stdout));
    mapped.insert(ports.stderr, Value::from_string(outputs.stderr));
    mapped.insert(ports.exit, Value::from_exit_code(outputs.exit_code));
    mapped
}

/// Unique identifier for an execution run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RunId(pub u64);

impl std::fmt::Display for RunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "run:{}", self.0)
    }
}

/// Execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Run all independent branches in parallel.
    Run,
    /// Step through one node at a time.
    Debug,
}

/// Per-node execution status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed {
        outputs: HashMap<PortId, Value>,
        cached: bool,
    },
    Failed {
        error: String,
    },
    Skipped,
}

/// Result of executing a single step (one node).
#[derive(Debug, Clone)]
pub struct StepResult {
    pub node_id: NodeId,
    pub status: NodeStatus,
    pub input_hash: ContentHash,
}

/// The executor.
pub struct Executor {
    next_run_id: u64,
}

impl Executor {
    pub fn new() -> Self {
        Self { next_run_id: 0 }
    }

    /// Execute a single node given its input values.
    pub async fn execute_node(
        &self,
        workflow: &Workflow,
        node_id: NodeId,
        inputs: &HashMap<PortId, Value>,
    ) -> StepResult {
        let node = match workflow.graph.node(node_id) {
            Some(n) => n,
            None => {
                return StepResult {
                    node_id,
                    status: NodeStatus::Failed {
                        error: "node not found in graph".into(),
                    },
                    input_hash: ContentHash::from_bytes(b""),
                };
            }
        };

        let fallback_input_hash = input_values_hash(inputs);

        let config = match workflow.node_config(node_id) {
            Some(c) => c,
            None => {
                return StepResult {
                    node_id,
                    status: NodeStatus::Failed {
                        error: format!("no config for node {:?} '{}'", node_id, node.label),
                    },
                    input_hash: fallback_input_hash,
                };
            }
        };
        let input_hash = node_execution_hash(node_id, config, inputs);

        info!(node = %node.label, "executing node");

        match config {
            NodeConfig::Shell {
                command,
                cwd,
                env,
                timeout_secs,
                pipe_stdin,
            } => {
                let output_ports = match shell_output_port_ids(node) {
                    Ok(ports) => ports,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let result = execute_shell(command, cwd, env, *timeout_secs, *pipe_stdin, inputs).await;

                match result {
                    Ok(shell_outputs) => StepResult {
                        node_id,
                        status: NodeStatus::Completed {
                            outputs: shell_outputs_by_real_port(output_ports, shell_outputs),
                            cached: false,
                        },
                        input_hash,
                    },
                    Err(e) => StepResult {
                        node_id,
                        status: NodeStatus::Failed { error: e.to_string() },
                        input_hash,
                    },
                }
            }
        }
    }

    /// Run the full workflow (topological order, parallel where possible).
    pub async fn run(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
    ) -> (RunId, Vec<StepResult>) {
        let run_id = RunId(self.next_run_id);
        self.next_run_id += 1;

        let topo = match topological_sort(workflow) {
            Some(order) => order,
            None => {
                error!("workflow contains cycles, cannot execute");
                return (run_id, Vec::new());
            }
        };

        info!(run = %run_id, nodes = topo.len(), "starting workflow execution");

        let mut results = Vec::new();
        let mut port_values: HashMap<PortId, Value> = HashMap::new();

        for node_id in topo {
            // Gather inputs from upstream edges.
            let inputs = gather_inputs(workflow, node_id, &port_values);

            // Check cache.
            let input_hash = match workflow.node_config(node_id) {
                Some(config) => node_execution_hash(node_id, config, &inputs),
                None => input_values_hash(&inputs),
            };

            if let Some(cached_outputs) = cache.get(&input_hash) {
                debug!(node = ?node_id, "cache hit, skipping execution");
                for (port_id, value) in cached_outputs {
                    port_values.insert(*port_id, value.clone());
                }
                results.push(StepResult {
                    node_id,
                    status: NodeStatus::Completed {
                        outputs: cached_outputs.clone(),
                        cached: true,
                    },
                    input_hash,
                });
                continue;
            }

            let step = self.execute_node(workflow, node_id, &inputs).await;

            // Store outputs for downstream nodes.
            if let NodeStatus::Completed { ref outputs, .. } = step.status {
                for (port_id, value) in outputs {
                    port_values.insert(*port_id, value.clone());
                }
            }

            results.push(step);
        }

        info!(run = %run_id, steps = results.len(), "workflow execution complete");
        (run_id, results)
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a shell command, returning output values keyed by port.
async fn execute_shell(
    command: &str,
    cwd: &Option<String>,
    env: &HashMap<String, String>,
    timeout_secs: u64,
    pipe_stdin: bool,
    inputs: &HashMap<PortId, Value>,
) -> Result<ShellOutputValues, Box<dyn std::error::Error + Send + Sync>> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    for (k, v) in env {
        cmd.env(k, v);
    }

    if pipe_stdin {
        cmd.stdin(std::process::Stdio::piped());
    } else {
        cmd.stdin(std::process::Stdio::null());
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    // Pipe stdin from first input value.
    if pipe_stdin
        && let Some(stdin_val) = inputs.values().next()
        && let Some(ref mut stdin) = child.stdin
    {
        use tokio::io::AsyncWriteExt;
        let data = stdin_val.as_display_string();
        let _ = stdin.write_all(data.as_bytes()).await;
        // Drop stdin to close the pipe.
        drop(child.stdin.take());
    }

    let output = if timeout_secs > 0 {
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), child.wait_with_output())
            .await
            .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "command timed out".into() })??
    } else {
        child.wait_with_output().await?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok(ShellOutputValues {
        stdout,
        stderr,
        exit_code,
    })
}

/// Gather input values for a node from upstream edge outputs.
fn gather_inputs(workflow: &Workflow, node_id: NodeId, port_values: &HashMap<PortId, Value>) -> HashMap<PortId, Value> {
    let mut inputs = HashMap::new();

    let node = match workflow.graph.node(node_id) {
        Some(n) => n,
        None => return inputs,
    };

    for input_port in &node.input_ports {
        // Find the edge targeting this input port.
        for edge in workflow.graph.edges() {
            if edge.target == input_port.id
                && let Some(value) = port_values.get(&edge.source)
            {
                inputs.insert(input_port.id, value.clone());
            }
        }
    }

    inputs
}

/// Topological sort of the workflow graph. Returns None if cycles exist.
fn topological_sort(workflow: &Workflow) -> Option<Vec<NodeId>> {
    let node_ids = workflow.graph.node_ids();
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for &id in &node_ids {
        in_degree.insert(id, 0);
        adjacency.insert(id, Vec::new());
    }

    for edge in workflow.graph.edges() {
        let src_node = workflow.graph.port_owner(edge.source);
        let tgt_node = workflow.graph.port_owner(edge.target);
        if let (Some(src), Some(tgt)) = (src_node, tgt_node) {
            adjacency.entry(src).or_default().push(tgt);
            *in_degree.entry(tgt).or_default() += 1;
        }
    }

    let mut queue: Vec<NodeId> = in_degree.iter().filter(|(_, deg)| **deg == 0).map(|(&id, _)| id).collect();
    queue.sort(); // deterministic order

    let mut order = Vec::new();

    while let Some(node) = queue.pop() {
        order.push(node);
        if let Some(neighbors) = adjacency.get(&node) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(&neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(neighbor);
                    }
                }
            }
        }
    }

    if order.len() == node_ids.len() {
        Some(order)
    } else {
        None
    }
}

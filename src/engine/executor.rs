//! DAG executor — runs workflows in topological order.
//!
//! Two modes:
//!  - **Run**: async parallel execution of independent branches via tokio.
//!  - **Debug**: step-through, one node at a time, with pause after each.

use std::collections::HashMap;

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

        // Compute input hash from all input values.
        let mut input_hashes: Vec<ContentHash> = inputs.values().map(|v| v.hash).collect();
        input_hashes.sort();
        let input_hash = ContentHash::combine(&input_hashes);

        let config = match workflow.node_config(node_id) {
            Some(c) => c,
            None => {
                return StepResult {
                    node_id,
                    status: NodeStatus::Failed {
                        error: format!("no config for node {:?} '{}'", node_id, node.label),
                    },
                    input_hash,
                };
            }
        };

        info!(node = %node.label, "executing node");

        match config {
            NodeConfig::Shell {
                command,
                cwd,
                env,
                timeout_secs,
                pipe_stdin,
            } => {
                let result = execute_shell(command, cwd, env, *timeout_secs, *pipe_stdin, inputs).await;

                match result {
                    Ok(outputs) => StepResult {
                        node_id,
                        status: NodeStatus::Completed { outputs, cached: false },
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
            let mut input_hashes: Vec<ContentHash> = inputs.values().map(|v| v.hash).collect();
            input_hashes.sort();
            let input_hash = ContentHash::combine(&input_hashes);

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
) -> Result<HashMap<PortId, Value>, Box<dyn std::error::Error + Send + Sync>> {
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
    if pipe_stdin {
        if let Some(stdin_val) = inputs.values().next() {
            if let Some(ref mut stdin) = child.stdin {
                use tokio::io::AsyncWriteExt;
                let data = stdin_val.as_display_string();
                let _ = stdin.write_all(data.as_bytes()).await;
                // Drop stdin to close the pipe.
                drop(child.stdin.take());
            }
        }
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

    // Build output map. Port IDs are assigned by the graph — we use
    // positional convention: output[0]=stdout, output[1]=stderr, output[2]=exit.
    // The caller maps these to actual PortIds from the node.
    //
    // For now, we return a synthetic map. The executor resolves port IDs.
    let mut outputs = HashMap::new();

    // We use sentinel PortIds here — the executor maps them to real
    // port IDs when integrating with the workflow graph.
    outputs.insert(PortId(u64::MAX - 2), Value::from_string(stdout));
    outputs.insert(PortId(u64::MAX - 1), Value::from_string(stderr));
    outputs.insert(PortId(u64::MAX), Value::from_exit_code(exit_code));

    Ok(outputs)
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
            if edge.target == input_port.id {
                if let Some(value) = port_values.get(&edge.source) {
                    inputs.insert(input_port.id, value.clone());
                }
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

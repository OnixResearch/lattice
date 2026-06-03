//! Workflow definition — the node graph + per-node configuration.
//!
//! A Workflow wraps a rat-nodegraph Graph and attaches execution config
//! (shell commands, environment, etc.) to each node.

use std::collections::HashMap;

use rat_nodegraph::model::Graph;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortSpec;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::value::TypeTag;

/// Metadata about a workflow (stored alongside the graph).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMeta {
    pub name: String,
    pub description: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
}

impl WorkflowMeta {
    pub fn new(name: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.into(),
            description: String::new(),
            created: now,
            modified: now,
        }
    }
}

/// Per-node execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeConfig {
    Shell {
        command: String,
        /// Working directory (None = inherit).
        cwd: Option<String>,
        /// Extra environment variables.
        env: HashMap<String, String>,
        /// Timeout in seconds (0 = no timeout).
        timeout_secs: u64,
        /// If true, pipe previous node's stdout into this command's stdin.
        pipe_stdin: bool,
    },
}

impl NodeConfig {
    pub fn shell(command: impl Into<String>) -> Self {
        Self::Shell {
            command: command.into(),
            cwd: None,
            env: HashMap::new(),
            timeout_secs: 60,
            pipe_stdin: true,
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::Shell { .. } => "Shell",
        }
    }
}

/// A complete workflow: graph topology + node configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub meta: WorkflowMeta,
    pub graph: Graph,
    pub configs: HashMap<u64, NodeConfig>,
}

impl Workflow {
    pub fn new(name: impl Into<String>) -> Self {
        let mut graph = Graph::new();
        graph.dag_mode = true;
        // Accept "any" type on either side of an edge.
        graph.set_compatibility(Box::new(|src, tgt| src == tgt || src == "any" || tgt == "any"));

        Self {
            meta: WorkflowMeta::new(name),
            graph,
            configs: HashMap::new(),
        }
    }

    /// Add a shell node with the given label and command.
    /// Returns the NodeId.
    pub fn add_shell_node(&mut self, label: impl Into<String>, command: impl Into<String>) -> NodeId {
        let id = self.graph.add_node(label, &[PortSpec::new("stdin", TypeTag::String.as_str())], &[
            PortSpec::new("stdout", TypeTag::String.as_str()),
            PortSpec::new("stderr", TypeTag::String.as_str()),
            PortSpec::new("exit", TypeTag::ExitCode.as_str()),
        ]);
        self.configs.insert(id.0, NodeConfig::shell(command));
        id
    }

    /// Get the config for a node.
    pub fn node_config(&self, id: NodeId) -> Option<&NodeConfig> {
        self.configs.get(&id.0)
    }

    /// Set or replace the config for a node.
    pub fn set_node_config(&mut self, id: NodeId, config: NodeConfig) {
        self.configs.insert(id.0, config);
    }

    /// Touch the modified timestamp.
    pub fn touch(&mut self) {
        self.meta.modified = chrono::Utc::now();
    }
}

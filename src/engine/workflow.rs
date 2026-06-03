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

use crate::credentials::NodeCredentialRefs;
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
    Http {
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<String>,
        timeout_secs: u64,
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

    pub fn http(method: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Http {
            method: method.into(),
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            timeout_secs: 60,
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::Shell { .. } => "Shell",
            Self::Http { .. } => "HTTP",
        }
    }
}

/// A complete workflow: graph topology + node configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub meta: WorkflowMeta,
    pub graph: Graph,
    pub configs: HashMap<u64, NodeConfig>,
    #[serde(default)]
    pub credentials: HashMap<u64, NodeCredentialRefs>,
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
            credentials: HashMap::new(),
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

    /// Add an HTTP request node with default status, headers, and body outputs.
    /// Returns the NodeId.
    pub fn add_http_node(
        &mut self,
        label: impl Into<String>,
        method: impl Into<String>,
        url: impl Into<String>,
    ) -> NodeId {
        let id = self.graph.add_node(label, &[PortSpec::new("body", TypeTag::String.as_str())], &[
            PortSpec::new("status", TypeTag::String.as_str()),
            PortSpec::new("headers", TypeTag::Json.as_str()),
            PortSpec::new("body", TypeTag::String.as_str()),
        ]);
        self.configs.insert(id.0, NodeConfig::http(method, url));
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

    /// Get credential references for a node.
    pub fn node_credentials(&self, id: NodeId) -> Option<&NodeCredentialRefs> {
        self.credentials.get(&id.0)
    }

    /// Set or replace credential references for a node.
    pub fn set_node_credentials(&mut self, id: NodeId, refs: NodeCredentialRefs) {
        if refs.is_empty() {
            self.credentials.remove(&id.0);
        } else {
            self.credentials.insert(id.0, refs);
        }
    }

    /// Touch the modified timestamp.
    pub fn touch(&mut self) {
        self.meta.modified = chrono::Utc::now();
    }
}

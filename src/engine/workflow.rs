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

use crate::agent_node::AgentNodeSettings;
use crate::agent_node::AgentOutputMode;
use crate::agent_node::AgentTarget;
use crate::control_flow::ConditionNodeConfig;
use crate::control_flow::MergeNodeConfig;
use crate::control_flow::SubworkflowNodeConfig;
use crate::credentials::NodeCredentialRefs;
use crate::engine::value::TypeTag;
use crate::file_node::FileOperationKind;
use crate::image_node::DEFAULT_MAX_IMAGE_BYTES;
use crate::triggers::TriggerDefinition;

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
    File {
        operation: FileOperationKind,
        path: String,
        content: Option<String>,
        overwrite: bool,
        allowed_roots: Vec<String>,
    },
    ImageInput {
        path: String,
        allowed_roots: Vec<String>,
        max_bytes: usize,
    },
    Agent(AgentNodeSettings),
    Condition(ConditionNodeConfig),
    Merge(MergeNodeConfig),
    Subworkflow(SubworkflowNodeConfig),
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

    pub fn file(operation: FileOperationKind, path: impl Into<String>) -> Self {
        Self::File {
            operation,
            path: path.into(),
            content: None,
            overwrite: false,
            allowed_roots: Vec::new(),
        }
    }

    pub fn image_input(path: impl Into<String>) -> Self {
        Self::ImageInput {
            path: path.into(),
            allowed_roots: Vec::new(),
            max_bytes: DEFAULT_MAX_IMAGE_BYTES,
        }
    }

    pub fn agent(target: AgentTarget, prompt: impl Into<String>, output_mode: AgentOutputMode) -> Self {
        Self::Agent(AgentNodeSettings::new(target, prompt, output_mode))
    }

    pub fn condition(config: ConditionNodeConfig) -> Self {
        Self::Condition(config)
    }

    pub fn merge(config: MergeNodeConfig) -> Self {
        Self::Merge(config)
    }

    pub fn subworkflow(config: SubworkflowNodeConfig) -> Self {
        Self::Subworkflow(config)
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::Shell { .. } => "Shell",
            Self::Http { .. } => "HTTP",
            Self::File { .. } => "File",
            Self::ImageInput { .. } => "Image Input",
            Self::Agent(_) => "Agent",
            Self::Condition(_) => "Condition",
            Self::Merge(_) => "Merge",
            Self::Subworkflow(_) => "Subworkflow",
        }
    }
}

/// Optional per-node generic runner metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeRunnerSettings {
    pub runtime_kind: String,
    pub runtime_key: String,
    #[serde(default)]
    pub effect_policy: String,
    #[serde(default)]
    pub cache_policy: String,
    #[serde(default)]
    pub retry_policy: String,
    #[serde(default)]
    pub artifact_policy: String,
}

impl NodeRunnerSettings {
    pub fn new(runtime_kind: impl Into<String>, runtime_key: impl Into<String>) -> Self {
        Self {
            runtime_kind: runtime_kind.into(),
            runtime_key: runtime_key.into(),
            effect_policy: String::new(),
            cache_policy: String::new(),
            retry_policy: String::new(),
            artifact_policy: String::new(),
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
    #[serde(default)]
    pub runner: HashMap<u64, NodeRunnerSettings>,
    #[serde(default)]
    pub triggers: Vec<TriggerDefinition>,
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
            runner: HashMap::new(),
            triggers: Vec::new(),
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

    /// Add a file operation node with content, metadata, and bytes-written outputs.
    /// Returns the NodeId.
    pub fn add_file_node(
        &mut self,
        label: impl Into<String>,
        operation: FileOperationKind,
        path: impl Into<String>,
    ) -> NodeId {
        let id = self.graph.add_node(label, &[PortSpec::new("content", TypeTag::String.as_str())], &[
            PortSpec::new("content", TypeTag::String.as_str()),
            PortSpec::new("metadata", TypeTag::Json.as_str()),
            PortSpec::new("bytes_written", TypeTag::String.as_str()),
        ]);
        self.configs.insert(id.0, NodeConfig::file(operation, path));
        id
    }

    /// Add an image input node with image and metadata outputs.
    /// Returns the NodeId.
    pub fn add_image_input_node(&mut self, label: impl Into<String>, path: impl Into<String>) -> NodeId {
        let id = self.graph.add_node(label, &[], &[
            PortSpec::new("image", TypeTag::Image.as_str()),
            PortSpec::new("metadata", TypeTag::Json.as_str()),
        ]);
        self.configs.insert(id.0, NodeConfig::image_input(path));
        id
    }

    /// Add a generic agent node with media-aware inputs and typed outputs.
    /// Returns the NodeId.
    pub fn add_agent_node(
        &mut self,
        label: impl Into<String>,
        target: AgentTarget,
        prompt: impl Into<String>,
        output_mode: AgentOutputMode,
    ) -> NodeId {
        let id = self.graph.add_node(
            label,
            &[
                PortSpec::new("prompt", TypeTag::String.as_str()),
                PortSpec::new("image", TypeTag::Image.as_str()),
                PortSpec::new("json", TypeTag::Json.as_str()),
            ],
            &[
                PortSpec::new("text", TypeTag::String.as_str()),
                PortSpec::new("json", TypeTag::Json.as_str()),
                PortSpec::new("image", TypeTag::Image.as_str()),
                PortSpec::new("artifact", TypeTag::Bytes.as_str()),
                PortSpec::new("metadata", TypeTag::Json.as_str()),
            ],
        );
        self.configs.insert(id.0, NodeConfig::agent(target, prompt, output_mode));
        id
    }

    /// Add a condition node with one input and one output per branch.
    pub fn add_condition_node(&mut self, label: impl Into<String>, config: ConditionNodeConfig) -> NodeId {
        let output_specs = config
            .branches
            .iter()
            .map(|branch| PortSpec::new(branch.label.clone(), TypeTag::Any.as_str()))
            .collect::<Vec<_>>();
        let id = self.graph.add_node(label, &[PortSpec::new("value", TypeTag::Any.as_str())], &output_specs);
        self.configs.insert(id.0, NodeConfig::condition(config));
        id
    }

    /// Add a merge node with explicit input labels and one merged output.
    pub fn add_merge_node(&mut self, label: impl Into<String>, config: MergeNodeConfig) -> NodeId {
        let input_specs = config
            .required_inputs
            .iter()
            .map(|input| PortSpec::new(input.clone(), TypeTag::Any.as_str()))
            .collect::<Vec<_>>();
        let id = self.graph.add_node(label, &input_specs, &[PortSpec::new(
            crate::control_flow::merge_output_port_label(),
            TypeTag::Any.as_str(),
        )]);
        self.configs.insert(id.0, NodeConfig::merge(config));
        id
    }

    /// Add a subworkflow invocation node with declared parent-facing ports.
    pub fn add_subworkflow_node(&mut self, label: impl Into<String>, config: SubworkflowNodeConfig) -> NodeId {
        let input_specs = config
            .input_mappings
            .iter()
            .map(|mapping| PortSpec::new(mapping.parent_input.clone(), TypeTag::Any.as_str()))
            .collect::<Vec<_>>();
        let output_specs = if config.output_mappings.is_empty() {
            vec![PortSpec::new(
                crate::control_flow::subworkflow_default_output_port_label(),
                TypeTag::Any.as_str(),
            )]
        } else {
            config
                .output_mappings
                .iter()
                .map(|mapping| PortSpec::new(mapping.parent_output.clone(), TypeTag::Any.as_str()))
                .collect::<Vec<_>>()
        };
        let id = self.graph.add_node(label, &input_specs, &output_specs);
        self.configs.insert(id.0, NodeConfig::subworkflow(config));
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

    /// Get generic runner settings for a node.
    pub fn node_runner_settings(&self, id: NodeId) -> Option<&NodeRunnerSettings> {
        self.runner.get(&id.0)
    }

    /// Set or replace generic runner settings for a node.
    pub fn set_node_runner_settings(&mut self, id: NodeId, settings: NodeRunnerSettings) {
        self.runner.insert(id.0, settings);
    }

    /// Touch the modified timestamp.
    pub fn touch(&mut self) {
        self.meta.modified = chrono::Utc::now();
    }
}

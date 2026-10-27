//! Workflow definition: Lattice-owned graph state and per-node configuration.
//!
//! The workflow core owns deterministic product state and aggregate mutations.
//! Choregraph owns graph admission. Subwayrat stays behind the TUI adapter.

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::agent_node::AgentNodeSettings;
use crate::agent_node::AgentOutputMode;
use crate::agent_node::AgentTarget;
use crate::control_flow::ConditionNodeConfig;
use crate::control_flow::MergeNodeConfig;
use crate::control_flow::SubworkflowNodeConfig;
use crate::credentials::NodeCredentialRefs;
pub use crate::engine::domain::Edge;
pub use crate::engine::domain::GraphError;
pub use crate::engine::domain::Node;
pub use crate::engine::domain::NodeId;
pub use crate::engine::domain::Port;
pub use crate::engine::domain::PortDirection;
pub use crate::engine::domain::PortId;
pub use crate::engine::domain::PortSpec;
pub use crate::engine::domain::WorkflowGraph;
pub use crate::engine::domain::WorkflowNodeIdentity;
pub use crate::engine::domain::WorkflowPortIdentity;
use crate::engine::value::TypeTag;
use crate::file_node::FileOperationKind;
use crate::image_node::DEFAULT_MAX_IMAGE_BYTES;
use crate::triggers::TriggerDefinition;

// r[impl lattice.workflow_validation.domain_boundary.time]
/// A validated explicit UTC timestamp for workflow-domain decisions.
#[allow(unknown_lints)]
#[allow(
    path_segment_repetition,
    reason = "the public type names its workflow-owned time domain explicitly"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowTimestamp(chrono::DateTime<chrono::Utc>);

impl WorkflowTimestamp {
    pub const fn from_datetime(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self(value)
    }

    pub const fn into_datetime(self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }

    pub const fn epoch() -> Self {
        Self(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
    }
}

/// Metadata about a workflow (stored alongside the graph).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMeta {
    pub name: String,
    pub description: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
}

impl WorkflowMeta {
    pub fn new(name: impl Into<String>, timestamp: WorkflowTimestamp) -> Self {
        let timestamp = timestamp.into_datetime();
        Self {
            name: name.into(),
            description: String::new(),
            created: timestamp,
            modified: timestamp,
        }
    }
}

/// Per-node execution configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    Operation(Box<crate::operation::InvocationPlan>),
    Condition(ConditionNodeConfig),
    Merge(MergeNodeConfig),
    Subworkflow(SubworkflowNodeConfig),
    Approval(crate::workflow::gate::ApprovalNodeConfig),
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

    pub fn operation(plan: crate::operation::InvocationPlan) -> Self {
        Self::Operation(Box::new(plan))
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

    pub fn approval(config: crate::workflow::gate::ApprovalNodeConfig) -> Self {
        Self::Approval(config)
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::Shell { .. } => "Shell",
            Self::Http { .. } => "HTTP",
            Self::File { .. } => "File",
            Self::ImageInput { .. } => "Image Input",
            Self::Agent(_) => "Agent",
            Self::Operation(_) => "Operation",
            Self::Condition(_) => "Condition",
            Self::Merge(_) => "Merge",
            Self::Subworkflow(_) => "Subworkflow",
            Self::Approval(_) => "Approval",
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

// r[impl lattice.workflow_validation.domain_boundary.application_model]
/// A complete workflow: graph topology + node configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub meta: WorkflowMeta,
    pub graph: WorkflowGraph,
    pub configs: HashMap<u64, NodeConfig>,
    #[serde(default)]
    pub credentials: HashMap<u64, NodeCredentialRefs>,
    #[serde(default)]
    pub runner: HashMap<u64, NodeRunnerSettings>,
    #[serde(default)]
    pub triggers: Vec<TriggerDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationError {
    NodeNotFound(NodeId),
    StaleEdit(NodeId),
    ConditionConfigNotFound(NodeId),
    InvalidConditionConfig(crate::control_flow::ControlFlowError),
    InvalidAggregate(String),
    Graph(GraphError),
}

impl From<GraphError> for MutationError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

/// A removal is valid only for the exact owner and incident topology observed.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeRemovalPlan {
    pub node_id: NodeId,
    node: Node,
    config: NodeConfig,
    credentials: Option<NodeCredentialRefs>,
    runner: Option<NodeRunnerSettings>,
    incident_edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionBranchReconciliationPlan {
    pub node_id: NodeId,
    pub config: ConditionNodeConfig,
    pub requested_specs: Vec<PortSpec>,
}

impl Workflow {
    pub fn new(name: impl Into<String>, timestamp: WorkflowTimestamp) -> Self {
        let graph = WorkflowGraph::new();

        Self {
            meta: WorkflowMeta::new(name, timestamp),
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
        self.add_configured_node(label, NodeConfig::shell(command))
    }

    /// Add an HTTP request node with default status, headers, and body outputs.
    /// Returns the NodeId.
    pub fn add_http_node(
        &mut self,
        label: impl Into<String>,
        method: impl Into<String>,
        url: impl Into<String>,
    ) -> NodeId {
        self.add_configured_node(label, NodeConfig::http(method, url))
    }

    /// Add a file operation node with content, metadata, and bytes-written outputs.
    /// Returns the NodeId.
    pub fn add_file_node(
        &mut self,
        label: impl Into<String>,
        operation: FileOperationKind,
        path: impl Into<String>,
    ) -> NodeId {
        self.add_configured_node(label, NodeConfig::file(operation, path))
    }

    /// Add an image input node with image and metadata outputs.
    /// Returns the NodeId.
    pub fn add_image_input_node(&mut self, label: impl Into<String>, path: impl Into<String>) -> NodeId {
        self.add_configured_node(label, NodeConfig::image_input(path))
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
        self.add_configured_node(label, NodeConfig::agent(target, prompt, output_mode))
    }

    /// Add a typed operation node with one validated JSON result.
    pub fn add_operation_node(&mut self, label: impl Into<String>, plan: crate::operation::InvocationPlan) -> NodeId {
        self.add_configured_node(label, NodeConfig::operation(plan))
    }

    /// Add a condition node with one input and one output per branch.
    pub fn add_condition_node(&mut self, label: impl Into<String>, config: ConditionNodeConfig) -> NodeId {
        self.add_configured_node(label, NodeConfig::condition(config))
    }

    /// Add a merge node with explicit input labels and one merged output.
    pub fn add_merge_node(&mut self, label: impl Into<String>, config: MergeNodeConfig) -> NodeId {
        self.add_configured_node(label, NodeConfig::merge(config))
    }

    /// Add a subworkflow invocation node with declared parent-facing ports.
    pub fn add_subworkflow_node(&mut self, label: impl Into<String>, config: SubworkflowNodeConfig) -> NodeId {
        self.add_configured_node(label, NodeConfig::subworkflow(config))
    }

    /// Add a durable approval gate with declared selected inputs and resume outputs.
    pub fn add_approval_node(
        &mut self,
        label: impl Into<String>,
        config: crate::workflow::gate::ApprovalNodeConfig,
    ) -> NodeId {
        self.add_configured_node(label, NodeConfig::approval(config))
    }

    #[allow(
        unknown_lints,
        no_unwrap,
        reason = "existing infallible constructors retain their admitted-template contract; authoring uses try_add_configured_node"
    )]
    fn add_configured_node(&mut self, label: impl Into<String>, config: NodeConfig) -> NodeId {
        self.try_add_configured_node(label, config)
            .expect("Lattice-owned node templates require valid bounded graph identities and port declarations")
    }

    /// Add a validated config and its canonical typed ports atomically.
    /// The caller validates config semantics; graph declaration/allocation errors
    /// preserve the complete workflow, including allocator counters and config maps.
    pub fn try_add_configured_node(
        &mut self,
        label: impl Into<String>,
        config: NodeConfig,
    ) -> Result<NodeId, MutationError> {
        let id =
            match &config {
                NodeConfig::Shell { .. } => {
                    self.graph.try_add_node(label, &[PortSpec::new("stdin", TypeTag::String.as_str())], &[
                        PortSpec::new("stdout", TypeTag::String.as_str()),
                        PortSpec::new("stderr", TypeTag::String.as_str()),
                        PortSpec::new("exit", TypeTag::ExitCode.as_str()),
                    ])
                }
                NodeConfig::Http { .. } => {
                    self.graph.try_add_node(label, &[PortSpec::new("body", TypeTag::String.as_str())], &[
                        PortSpec::new("status", TypeTag::String.as_str()),
                        PortSpec::new("headers", TypeTag::Json.as_str()),
                        PortSpec::new("body", TypeTag::String.as_str()),
                    ])
                }
                NodeConfig::File { .. } => {
                    self.graph.try_add_node(label, &[PortSpec::new("content", TypeTag::String.as_str())], &[
                        PortSpec::new("content", TypeTag::String.as_str()),
                        PortSpec::new("metadata", TypeTag::Json.as_str()),
                        PortSpec::new("bytes_written", TypeTag::String.as_str()),
                    ])
                }
                NodeConfig::ImageInput { .. } => self.graph.try_add_node(label, &[], &[
                    PortSpec::new("image", TypeTag::Image.as_str()),
                    PortSpec::new("metadata", TypeTag::Json.as_str()),
                ]),
                NodeConfig::Agent(_) => self.graph.try_add_node(
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
                ),
                NodeConfig::Operation(_) => {
                    self.graph.try_add_node(label, &[], &[PortSpec::new("result", TypeTag::Json.as_str())])
                }
                NodeConfig::Condition(config) => {
                    let outputs = config
                        .branches
                        .iter()
                        .map(|branch| PortSpec::new(branch.label.clone(), TypeTag::Any.as_str()))
                        .collect::<Vec<_>>();
                    self.graph.try_add_node(label, &[PortSpec::new("value", TypeTag::Any.as_str())], &outputs)
                }
                NodeConfig::Merge(config) => {
                    let inputs = config
                        .required_inputs
                        .iter()
                        .map(|input| PortSpec::new(input.clone(), TypeTag::Any.as_str()))
                        .collect::<Vec<_>>();
                    self.graph.try_add_node(label, &inputs, &[PortSpec::new(
                        crate::control_flow::merge_output_port_label(),
                        TypeTag::Any.as_str(),
                    )])
                }
                NodeConfig::Subworkflow(config) => {
                    let inputs = config
                        .input_mappings
                        .iter()
                        .map(|mapping| PortSpec::new(mapping.parent_input.clone(), TypeTag::Any.as_str()))
                        .collect::<Vec<_>>();
                    let outputs = if config.output_mappings.is_empty() {
                        vec![PortSpec::new(
                            crate::control_flow::subworkflow_default_output_port_label(),
                            TypeTag::Any.as_str(),
                        )]
                    } else {
                        config
                            .output_mappings
                            .iter()
                            .map(|mapping| PortSpec::new(mapping.parent_output.clone(), TypeTag::Any.as_str()))
                            .collect()
                    };
                    self.graph.try_add_node(label, &inputs, &outputs)
                }
                NodeConfig::Approval(config) => {
                    const CREDENTIAL_OUTPUT_PREFIX: &str = "credential:";
                    let inputs = config
                        .selected_input_ports
                        .iter()
                        .map(|input| PortSpec::new(input.clone(), TypeTag::Any.as_str()))
                        .collect::<Vec<_>>();
                    let mut outputs = config
                        .resume
                        .values
                        .iter()
                        .map(|field| PortSpec::new(field.name.as_str(), field.type_tag.clone()))
                        .collect::<Vec<_>>();
                    outputs.extend(config.resume.credential_bindings.iter().map(|name| {
                        PortSpec::new(format!("{CREDENTIAL_OUTPUT_PREFIX}{name}"), TypeTag::String.as_str())
                    }));
                    self.graph.try_add_node(label, &inputs, &outputs)
                }
            }?;
        self.configs.insert(id.0, config);
        Ok(id)
    }

    pub fn plan_node_removal(&self, node_id: impl WorkflowNodeIdentity) -> Result<NodeRemovalPlan, MutationError> {
        let node_id = node_id.workflow_node_id();
        let node = self.graph.node(node_id).ok_or(MutationError::NodeNotFound(node_id))?;
        self.validate_aggregate_ownership()?;
        let config = self
            .configs
            .get(&node_id.0)
            .ok_or_else(|| MutationError::InvalidAggregate(format!("node {node_id} has no execution config")))?;
        Ok(NodeRemovalPlan {
            node_id,
            node: node.clone(),
            config: config.clone(),
            credentials: self.credentials.get(&node_id.0).cloned(),
            runner: self.runner.get(&node_id.0).cloned(),
            incident_edges: self.incident_edges(node_id).cloned().collect(),
        })
    }

    /// Reject stale owners before validating and committing the complete candidate.
    pub fn apply_node_removal(&mut self, plan: NodeRemovalPlan) -> Result<(), MutationError> {
        if self.graph.node(plan.node_id) != Some(&plan.node)
            || self.configs.get(&plan.node_id.0) != Some(&plan.config)
            || self.credentials.get(&plan.node_id.0) != plan.credentials.as_ref()
            || self.runner.get(&plan.node_id.0) != plan.runner.as_ref()
            || !self.incident_edges(plan.node_id).eq(plan.incident_edges.iter())
        {
            return Err(MutationError::StaleEdit(plan.node_id));
        }
        self.validate_aggregate_ownership()?;
        let mut candidate = self.clone();
        candidate.graph.remove_node(plan.node_id)?;
        candidate.configs.remove(&plan.node_id.0);
        candidate.credentials.remove(&plan.node_id.0);
        candidate.runner.remove(&plan.node_id.0);
        candidate.validate_aggregate_ownership()?;
        candidate.validate_expression_ownership()?;
        *self = candidate;
        Ok(())
    }

    /// Remove a node and all node-owned workflow state as one lifecycle operation.
    pub fn remove_node(&mut self, node_id: impl WorkflowNodeIdentity) -> Result<(), MutationError> {
        let plan = self.plan_node_removal(node_id)?;
        self.apply_node_removal(plan)
    }

    fn incident_edges(&self, node_id: NodeId) -> impl Iterator<Item = &Edge> {
        self.graph.edges().iter().filter(move |edge| {
            self.graph.port_owner(edge.source) == Some(node_id) || self.graph.port_owner(edge.target) == Some(node_id)
        })
    }

    /// Structural admission deliberately excludes unrelated runtime/provider policy.
    pub(crate) fn validate_aggregate_ownership(&self) -> Result<(), MutationError> {
        for node_id in self.configs.keys().chain(self.credentials.keys()).chain(self.runner.keys()) {
            if self.graph.node(NodeId(*node_id)).is_none() {
                return Err(MutationError::InvalidAggregate(format!("state belongs to missing node {node_id}")));
            }
        }
        let mut ports = std::collections::BTreeSet::new();
        for node in self.graph.nodes() {
            if !self.configs.contains_key(&node.id.0) {
                return Err(MutationError::InvalidAggregate(format!("node {} has no execution config", node.id)));
            }
            for (direction, declarations) in [
                (PortDirection::Input, &node.input_ports),
                (PortDirection::Output, &node.output_ports),
            ] {
                let mut labels = std::collections::BTreeSet::new();
                for port in declarations {
                    if !ports.insert(port.id)
                        || !labels.insert(&port.label)
                        || port.direction != direction
                        || self.graph.port_owner(port.id) != Some(node.id)
                        || self.graph.port(port.id) != Some(port)
                    {
                        return Err(MutationError::InvalidAggregate(format!("invalid ownership of port {}", port.id)));
                    }
                }
            }
        }
        for edge in self.graph.edges() {
            let source = self.graph.port(edge.source).ok_or(GraphError::PortNotFound(edge.source))?;
            let target = self.graph.port(edge.target).ok_or(GraphError::PortNotFound(edge.target))?;
            if source.direction != PortDirection::Output || target.direction != PortDirection::Input {
                return Err(GraphError::WrongDirection.into());
            }
            if self.graph.port_owner(edge.source) == self.graph.port_owner(edge.target) {
                return Err(GraphError::SelfLoop.into());
            }
            if source.type_tag != target.type_tag
                && source.type_tag != TypeTag::Any.as_str()
                && target.type_tag != TypeTag::Any.as_str()
            {
                return Err(GraphError::IncompatibleTypes {
                    source_tag: source.type_tag.clone(),
                    target_tag: target.type_tag.clone(),
                }
                .into());
            }
        }
        Ok(())
    }

    fn validate_expression_ownership(&self) -> Result<(), MutationError> {
        if let Err(issues) = crate::expression_dependencies::workflow_projection(self) {
            for issue in issues {
                if matches!(
                    issue.code,
                    crate::expression_dependencies::ExpressionDependencyIssueCode::MissingProducerNode
                        | crate::expression_dependencies::ExpressionDependencyIssueCode::MissingProducerOutputPort
                ) {
                    return Err(MutationError::InvalidAggregate(issue.message));
                }
            }
        }
        Ok(())
    }

    pub fn plan_condition_branch_reconciliation(
        &self,
        node_id: impl WorkflowNodeIdentity,
        branches: Vec<crate::control_flow::ConditionBranch>,
    ) -> Result<ConditionBranchReconciliationPlan, MutationError> {
        let node_id = node_id.workflow_node_id();
        let node = self.graph.node(node_id).ok_or(MutationError::NodeNotFound(node_id))?;
        if !matches!(self.configs.get(&node_id.0), Some(NodeConfig::Condition(_))) {
            return Err(MutationError::ConditionConfigNotFound(node_id));
        }
        let config = ConditionNodeConfig { branches };
        crate::control_flow::validate_condition_config(&config).map_err(MutationError::InvalidConditionConfig)?;
        self.validate_aggregate_ownership()?;
        let requested_specs = config
            .branches
            .iter()
            .map(|branch| {
                let type_tag = node
                    .output_ports
                    .iter()
                    .find(|port| port.label == branch.label)
                    .map_or(TypeTag::Any.as_str(), |port| port.type_tag.as_str());
                PortSpec::new(branch.label.clone(), type_tag)
            })
            .collect();
        Ok(ConditionBranchReconciliationPlan {
            node_id,
            config,
            requested_specs,
        })
    }

    /// Reconcile condition branches and output ports without stale ownership or edges.
    pub fn replace_condition_branches(
        &mut self,
        node_id: impl WorkflowNodeIdentity,
        branches: Vec<crate::control_flow::ConditionBranch>,
    ) -> Result<(), MutationError> {
        let plan = self.plan_condition_branch_reconciliation(node_id, branches)?;
        let mut candidate = self.clone();
        candidate.graph.reconcile_node_ports(plan.node_id, PortDirection::Output, &plan.requested_specs)?;
        candidate.configs.insert(plan.node_id.0, NodeConfig::Condition(plan.config));
        candidate.validate_aggregate_ownership()?;
        candidate.validate_expression_ownership()?;
        *self = candidate;
        Ok(())
    }

    /// Get the config for a node.
    pub fn node_config(&self, id: impl WorkflowNodeIdentity) -> Option<&NodeConfig> {
        self.configs.get(&id.workflow_node_id().0)
    }

    /// Set or replace the config for a node.
    pub fn set_node_config(&mut self, id: impl WorkflowNodeIdentity, config: NodeConfig) {
        self.configs.insert(id.workflow_node_id().0, config);
    }

    /// Get credential references for a node.
    pub fn node_credentials(&self, id: impl WorkflowNodeIdentity) -> Option<&NodeCredentialRefs> {
        self.credentials.get(&id.workflow_node_id().0)
    }

    /// Set or replace credential references for a node.
    pub fn set_node_credentials(&mut self, id: impl WorkflowNodeIdentity, refs: NodeCredentialRefs) {
        let id = id.workflow_node_id();
        if refs.is_empty() {
            self.credentials.remove(&id.0);
        } else {
            self.credentials.insert(id.0, refs);
        }
    }

    /// Get generic runner settings for a node.
    pub fn node_runner_settings(&self, id: impl WorkflowNodeIdentity) -> Option<&NodeRunnerSettings> {
        self.runner.get(&id.workflow_node_id().0)
    }

    /// Set or replace generic runner settings for a node.
    pub fn set_node_runner_settings(&mut self, id: impl WorkflowNodeIdentity, settings: NodeRunnerSettings) {
        self.runner.insert(id.workflow_node_id().0, settings);
    }

    // r[impl lattice.workflow_validation.domain_boundary.validation.shell_time]
    /// Set the modified timestamp from one shell-owned clock observation.
    pub fn touch(&mut self, timestamp: WorkflowTimestamp) {
        self.meta.modified = timestamp.into_datetime();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control_flow::ConditionPredicate;
    use crate::credentials::SecretRef;

    fn branch(label: &str) -> crate::control_flow::ConditionBranch {
        crate::control_flow::ConditionBranch {
            label: label.to_string(),
            predicate: ConditionPredicate::Always,
        }
    }

    #[test]
    fn remove_node_cleans_all_owned_state_and_incident_edges() {
        let mut workflow = Workflow::new("aggregate deletion", crate::engine::workflow::WorkflowTimestamp::epoch());
        let source = workflow.add_shell_node("source", "printf source");
        let target = workflow.add_shell_node("target", "cat");
        let source_port = workflow.graph.node(source).expect("source node").output_ports[0].id;
        let target_port = workflow.graph.node(target).expect("target node").input_ports[0].id;
        workflow.graph.add_edge(source_port, target_port).expect("compatible edge");

        let mut credentials = NodeCredentialRefs::default();
        credentials.env.insert("TOKEN".to_string(), SecretRef::new("env", "TOKEN", None));
        workflow.set_node_credentials(source, credentials);
        workflow.set_node_runner_settings(source, NodeRunnerSettings::new("host", "shell"));

        workflow.remove_node(source).expect("aggregate removal succeeds");
        // r[verify lattice.workflow_validation.domain_boundary.application_model.accepted]

        assert!(workflow.graph.node(source).is_none());
        assert!(workflow.graph.node(target).is_some());
        assert!(workflow.node_config(source).is_none());
        assert!(workflow.node_credentials(source).is_none());
        assert!(workflow.node_runner_settings(source).is_none());
        assert_eq!(workflow.graph.edge_count(), 0);
    }

    #[test]
    fn remove_node_cleans_owned_state_for_every_builtin_config_kind() {
        let mut workflow = Workflow::new("all built-in deletion", crate::engine::workflow::WorkflowTimestamp::epoch());
        let nodes = [
            workflow.add_shell_node("shell", "true"),
            workflow.add_http_node("http", "GET", "https://example.com"),
            workflow.add_file_node("file", FileOperationKind::Read, "./input.txt"),
            workflow.add_image_input_node("image", "./input.png"),
            workflow.add_agent_node("agent", AgentTarget::Pi, "prompt", AgentOutputMode::Text),
            workflow.add_condition_node("condition", ConditionNodeConfig {
                branches: vec![branch("match")],
            }),
            workflow.add_merge_node("merge", MergeNodeConfig {
                policy: crate::control_flow::MergePolicy::All,
                required_inputs: vec!["input".to_string()],
            }),
            workflow.add_subworkflow_node("subworkflow", SubworkflowNodeConfig {
                workflow_name: "child".to_string(),
                input_mappings: Vec::new(),
                output_mappings: Vec::new(),
            }),
        ];
        for node_id in nodes {
            let mut credentials = NodeCredentialRefs::default();
            credentials.env.insert("TOKEN".to_string(), SecretRef::new("env", "TOKEN", None));
            workflow.set_node_credentials(node_id, credentials);
            workflow.set_node_runner_settings(node_id, NodeRunnerSettings::new("host", "builtin"));
        }

        for node_id in nodes {
            workflow.remove_node(node_id).expect("built-in node removal succeeds");
        }

        assert_eq!(workflow.graph.node_count(), 0);
        assert!(workflow.configs.is_empty());
        assert!(workflow.credentials.is_empty());
        assert!(workflow.runner.is_empty());
    }

    #[test]
    fn remove_missing_node_fails_without_mutating_workflow() {
        let mut workflow = Workflow::new("failed deletion", crate::engine::workflow::WorkflowTimestamp::epoch());
        let node = workflow.add_shell_node("kept", "true");
        let missing = NodeId(u64::MAX);

        assert_eq!(workflow.remove_node(missing), Err(MutationError::NodeNotFound(missing)));
        assert!(workflow.graph.node(node).is_some());
        assert!(workflow.node_config(node).is_some());
        assert_eq!(workflow.graph.node_count(), 1);
    }

    #[test]
    fn stale_removal_plan_fails_without_mutating_current_state() {
        let mut workflow = Workflow::new("stale deletion", crate::engine::workflow::WorkflowTimestamp::epoch());
        let stale_node = workflow.add_shell_node("stale", "true");
        let retained_node = workflow.add_shell_node("retained", "true");
        let plan = workflow.plan_node_removal(stale_node).expect("removal plan");
        workflow.remove_node(stale_node).expect("initial removal");
        let node_count = workflow.graph.node_count();

        assert_eq!(workflow.apply_node_removal(plan), Err(MutationError::StaleEdit(stale_node)));
        assert_eq!(workflow.graph.node_count(), node_count);
        assert!(workflow.graph.node(retained_node).is_some());
        assert!(workflow.node_config(retained_node).is_some());
    }

    #[test]
    fn condition_reconciliation_preserves_matching_ports_and_drops_removed_edges() {
        let mut workflow =
            Workflow::new("condition reconciliation", crate::engine::workflow::WorkflowTimestamp::epoch());
        let condition = workflow.add_condition_node("route", ConditionNodeConfig {
            branches: vec![branch("keep"), branch("remove")],
        });
        let keep_target = workflow.add_shell_node("keep target", "cat");
        let remove_target = workflow.add_shell_node("remove target", "cat");
        let condition_node = workflow.graph.node(condition).expect("condition node");
        let keep_port = condition_node.output_ports[0].id;
        let remove_port = condition_node.output_ports[1].id;
        let keep_input = workflow.graph.node(keep_target).expect("keep target").input_ports[0].id;
        let remove_input = workflow.graph.node(remove_target).expect("remove target").input_ports[0].id;
        workflow.graph.add_edge(keep_port, keep_input).expect("keep edge");
        workflow.graph.add_edge(remove_port, remove_input).expect("remove edge");

        workflow
            .replace_condition_branches(condition, vec![branch("added"), branch("keep")])
            .expect("valid branch reconciliation");

        let reconciled = workflow.graph.node(condition).expect("reconciled condition");
        assert_eq!(reconciled.output_ports.len(), 2);
        assert_eq!(reconciled.output_ports[0].label, "added");
        assert_eq!(reconciled.output_ports[1].label, "keep");
        assert_eq!(reconciled.output_ports[1].id, keep_port);
        assert!(workflow.graph.port(remove_port).is_none());
        assert_eq!(workflow.graph.edges().len(), 1);
        assert_eq!(workflow.graph.edges()[0].source, keep_port);
        assert_eq!(workflow.graph.edges()[0].target, keep_input);
    }

    #[test]
    fn invalid_condition_reconciliation_preserves_config_ports_and_edges() {
        let mut workflow =
            Workflow::new("rejected condition reconciliation", crate::engine::workflow::WorkflowTimestamp::epoch());
        let condition = workflow.add_condition_node("route", ConditionNodeConfig {
            branches: vec![branch("first"), branch("second")],
        });
        let prior_config = workflow.node_config(condition).cloned().expect("condition config");
        let prior_ports = workflow.graph.node(condition).expect("condition node").output_ports.clone();
        let prior_edges = workflow.graph.edges().to_vec();

        let result = workflow.replace_condition_branches(condition, vec![branch("duplicate"), branch("duplicate")]);
        // r[verify lattice.workflow_validation.domain_boundary.validation]
        // r[verify lattice.workflow_validation.domain_boundary.validation.paired]

        assert_eq!(
            result,
            Err(MutationError::InvalidConditionConfig(crate::control_flow::ControlFlowError::DuplicateBranchLabel(
                "duplicate".to_string()
            )))
        );
        assert_eq!(workflow.graph.node(condition).expect("condition node").output_ports, prior_ports);
        assert_eq!(workflow.graph.edges(), prior_edges);
        match (workflow.node_config(condition), prior_config) {
            (Some(NodeConfig::Condition(actual)), NodeConfig::Condition(expected)) => assert_eq!(actual, &expected),
            _ => panic!("expected unchanged condition config"),
        }
    }

    fn snapshot(workflow: &Workflow) -> serde_json::Value {
        serde_json::to_value(workflow).expect("workflow snapshot")
    }

    #[test]
    fn removal_plan_rejects_changed_or_replaced_owner_and_incident_topology() {
        for change in 0..7 {
            let mut workflow = Workflow::new("stale owners", WorkflowTimestamp::epoch());
            let owner = workflow.add_shell_node("owner", "true");
            let other = workflow.add_shell_node("other", "cat");
            let plan = workflow.plan_node_removal(owner).expect("plan");
            match change {
                0 => workflow.graph.node_mut(owner).expect("owner").label = "replacement".to_string(),
                1 => workflow.set_node_config(owner, NodeConfig::http("POST", "https://example.com")),
                2 => {
                    let mut refs = NodeCredentialRefs::default();
                    refs.env.insert("TOKEN".to_string(), SecretRef::new("env", "NEW_TOKEN", None));
                    workflow.set_node_credentials(owner, refs);
                }
                3 => workflow.set_node_runner_settings(owner, NodeRunnerSettings::new("shell", "new-binding")),
                4 => {
                    let source = workflow.graph.node(owner).expect("owner").output_ports[0].id;
                    let target = workflow.graph.node(other).expect("other").input_ports[0].id;
                    workflow.graph.add_edge(source, target).expect("incident edge");
                }
                5 => {
                    workflow.graph.remove_node(owner).expect("remove old owner");
                    let mut stored = serde_json::to_value(&workflow.graph).expect("graph value");
                    stored["next_node_id"] = serde_json::json!(owner.0);
                    workflow.graph = serde_json::from_value(stored).expect("restore reusable id fixture");
                    let replacement = workflow.add_shell_node("new owner", "false");
                    assert_eq!(replacement, owner);
                }
                6 => workflow.graph.node_mut(owner).expect("owner").x = 42,
                _ => unreachable!("bounded test cases"),
            }
            let before = snapshot(&workflow);
            assert_eq!(workflow.apply_node_removal(plan), Err(MutationError::StaleEdit(owner)));
            assert_eq!(snapshot(&workflow), before, "owner change {change}");
        }
    }

    #[test]
    fn removal_plan_preserves_intervening_unrelated_edits() {
        let mut workflow = Workflow::new("unrelated edits", WorkflowTimestamp::epoch());
        let owner = workflow.add_shell_node("owner", "true");
        let other = workflow.add_shell_node("other", "true");
        let plan = workflow.plan_node_removal(owner).expect("plan");
        workflow.set_node_config(other, NodeConfig::shell("printf changed"));
        workflow.meta.description = "keep new description".to_string();
        let other_node = workflow.graph.node(other).cloned().expect("other");
        let other_config = workflow.node_config(other).cloned().expect("config");
        workflow.apply_node_removal(plan).expect("unchanged owner");
        assert_eq!(workflow.graph.node(other), Some(&other_node));
        assert_eq!(workflow.node_config(other), Some(&other_config));
        assert_eq!(workflow.meta.description, "keep new description");
    }

    #[test]
    fn orphan_maps_and_malformed_ports_reject_removal_without_partial_cleanup() {
        for corruption in 0..6 {
            let mut workflow = Workflow::new("orphan rejection", WorkflowTimestamp::epoch());
            let owner = workflow.add_shell_node("owner", "true");
            let plan = workflow.plan_node_removal(owner).expect("plan before corruption");
            let orphan = NodeId(u64::MAX);
            match corruption {
                0 => {
                    workflow.configs.insert(orphan.0, NodeConfig::shell("orphan"));
                }
                1 => {
                    workflow.credentials.insert(orphan.0, NodeCredentialRefs::default());
                }
                2 => {
                    workflow.runner.insert(orphan.0, NodeRunnerSettings::new("shell", "orphan"));
                }
                3 => {
                    let other = workflow.add_shell_node("missing config", "true");
                    workflow.configs.remove(&other.0);
                }
                4 => {
                    let other = workflow.add_shell_node("malformed direction", "true");
                    workflow.graph.node_mut(other).expect("other").output_ports[0].direction = PortDirection::Input;
                }
                5 => {
                    let other = workflow.add_shell_node("duplicate port", "true");
                    let duplicate = workflow.graph.node(other).expect("other").output_ports[0].clone();
                    workflow.graph.node_mut(other).expect("other").output_ports.push(duplicate);
                }
                _ => unreachable!("bounded test cases"),
            }
            let before = snapshot(&workflow);
            assert!(matches!(workflow.apply_node_removal(plan), Err(MutationError::InvalidAggregate(_))));
            assert_eq!(snapshot(&workflow), before, "corruption {corruption}");
        }
    }

    #[test]
    fn dangling_edge_rejects_aggregate_removal_and_condition_reconciliation() {
        let mut workflow = Workflow::new("dangling edge", WorkflowTimestamp::epoch());
        let condition = workflow.add_condition_node("route", ConditionNodeConfig {
            branches: vec![branch("keep")],
        });
        let target = workflow.add_shell_node("target", "cat");
        let source = workflow.graph.node(condition).expect("route").output_ports[0].id;
        let input = workflow.graph.node(target).expect("target").input_ports[0].id;
        workflow.graph.add_edge(source, input).expect("edge");
        workflow.graph.node_mut(target).expect("target").input_ports.clear();
        let before = snapshot(&workflow);
        assert!(workflow.remove_node(target).is_err());
        assert_eq!(snapshot(&workflow), before);
        assert!(workflow.replace_condition_branches(condition, vec![branch("new")]).is_err());
        assert_eq!(snapshot(&workflow), before);
    }

    #[test]
    fn condition_reorder_unchanged_rename_and_removal_preserve_semantic_port_identity() {
        let mut workflow = Workflow::new("condition transitions", WorkflowTimestamp::epoch());
        let condition = workflow.add_condition_node("route", ConditionNodeConfig {
            branches: vec![branch("left"), branch("right")],
        });
        let target = workflow.add_shell_node("target", "cat");
        workflow.graph.node_mut(condition).expect("condition").output_ports[0].type_tag = "string".to_string();
        let original = workflow.graph.node(condition).expect("condition").output_ports.clone();
        let input = workflow.graph.node(target).expect("target").input_ports[0].id;
        workflow.graph.add_edge(original[0].id, input).expect("left edge");
        let before = snapshot(&workflow);
        workflow
            .replace_condition_branches(condition, vec![branch("left"), branch("right")])
            .expect("unchanged");
        assert_eq!(snapshot(&workflow), before);
        workflow
            .replace_condition_branches(condition, vec![branch("right"), branch("left")])
            .expect("reorder");
        assert_eq!(workflow.graph.node(condition).expect("condition").output_ports, vec![
            original[1].clone(),
            original[0].clone()
        ]);
        assert_eq!(workflow.graph.edges(), &[Edge {
            source: original[0].id,
            target: input
        }]);
        workflow
            .replace_condition_branches(condition, vec![branch("renamed"), branch("right")])
            .expect("rename");
        let renamed = workflow.graph.node(condition).expect("condition").output_ports[0].clone();
        assert_ne!(renamed.id, original[0].id);
        assert_ne!(renamed.id, original[1].id);
        assert_eq!(renamed.type_tag, "any");
        assert!(workflow.graph.port(original[0].id).is_none());
        assert!(workflow.graph.edges().is_empty());
        workflow.replace_condition_branches(condition, vec![branch("right")]).expect("remove");
        assert_eq!(workflow.graph.node(condition).expect("condition").output_ports, vec![original[1].clone()]);
        assert!(workflow.graph.port(renamed.id).is_none());
    }

    #[test]
    fn malformed_condition_edits_and_exhaustion_are_atomic() {
        let mut workflow = Workflow::new("invalid condition edits", WorkflowTimestamp::epoch());
        let condition = workflow.add_condition_node("route", ConditionNodeConfig {
            branches: vec![branch("keep")],
        });
        let invalid = [
            vec![branch("")],
            vec![branch("  ")],
            vec![branch("duplicate"), branch("duplicate")],
            vec![crate::control_flow::ConditionBranch {
                label: "nan".to_string(),
                predicate: ConditionPredicate::NumberGreaterThan(f64::NAN),
            }],
            vec![crate::control_flow::ConditionBranch {
                label: "infinity".to_string(),
                predicate: ConditionPredicate::NumberGreaterThan(f64::INFINITY),
            }],
            vec![crate::control_flow::ConditionBranch {
                label: "pointer".to_string(),
                predicate: ConditionPredicate::JsonPointerExists("not/a/pointer".to_string()),
            }],
            vec![crate::control_flow::ConditionBranch {
                label: "escape".to_string(),
                predicate: ConditionPredicate::JsonPointerExists("/bad~2escape".to_string()),
            }],
            vec![crate::control_flow::ConditionBranch {
                label: "trailing".to_string(),
                predicate: ConditionPredicate::JsonPointerExists("/bad~".to_string()),
            }],
        ];
        for branches in invalid {
            let before = snapshot(&workflow);
            assert!(matches!(
                workflow.replace_condition_branches(condition, branches),
                Err(MutationError::InvalidConditionConfig(_))
            ));
            assert_eq!(snapshot(&workflow), before);
        }
        let mut stored = serde_json::to_value(&workflow.graph).expect("graph");
        stored["next_port_id"] = serde_json::json!(u64::MAX);
        workflow.graph = serde_json::from_value(stored).expect("exhausted allocator fixture");
        let before = snapshot(&workflow);
        assert_eq!(
            workflow.replace_condition_branches(condition, vec![branch("keep"), branch("new")]),
            Err(MutationError::Graph(GraphError::PortIdExhausted))
        );
        assert_eq!(snapshot(&workflow), before);
        workflow
            .replace_condition_branches(condition, vec![branch("keep")])
            .expect("unchanged needs no fresh id");
        assert_eq!(snapshot(&workflow), before);
    }

    #[test]
    fn valid_json_pointer_predicates_include_root_and_escaped_tokens() {
        let mut workflow = Workflow::new("valid predicates", WorkflowTimestamp::epoch());
        let condition = workflow.add_condition_node("route", ConditionNodeConfig {
            branches: vec![branch("keep")],
        });
        for pointer in ["", "/", "/a~1b/~0key", "/unicode/λ"] {
            let edited = crate::control_flow::ConditionBranch {
                label: "keep".to_string(),
                predicate: ConditionPredicate::JsonPointerExists(pointer.to_string()),
            };
            workflow.replace_condition_branches(condition, vec![edited.clone()]).expect("RFC 6901 pointer");
            assert_eq!(
                workflow.node_config(condition),
                Some(&NodeConfig::condition(ConditionNodeConfig { branches: vec![edited] }))
            );
        }
    }

    #[test]
    fn finite_condition_thresholds_preserve_matching_output_identity() {
        let mut workflow = Workflow::new("finite predicate bounds", WorkflowTimestamp::epoch());
        let condition = workflow.add_condition_node("route", ConditionNodeConfig {
            branches: vec![branch("keep")],
        });
        let original = workflow.graph.node(condition).expect("condition").output_ports[0].clone();
        for threshold in [f64::MIN, -0.0, 0.0, f64::MAX] {
            let edited = crate::control_flow::ConditionBranch {
                label: "keep".to_string(),
                predicate: ConditionPredicate::NumberGreaterThan(threshold),
            };
            workflow.replace_condition_branches(condition, vec![edited.clone()]).expect("finite threshold");
            assert_eq!(workflow.graph.node(condition).expect("condition").output_ports, vec![original.clone()]);
            assert_eq!(
                workflow.node_config(condition),
                Some(&NodeConfig::condition(ConditionNodeConfig { branches: vec![edited] }))
            );
        }
    }

    #[test]
    fn condition_reconciliation_rejects_missing_or_noncondition_owner() {
        let mut workflow = Workflow::new("wrong condition owner", WorkflowTimestamp::epoch());
        let shell = workflow.add_shell_node("shell", "true");
        let before = snapshot(&workflow);
        assert_eq!(
            workflow.replace_condition_branches(shell, vec![branch("new")]),
            Err(MutationError::ConditionConfigNotFound(shell))
        );
        assert_eq!(snapshot(&workflow), before);
        let missing = NodeId(u64::MAX);
        assert_eq!(
            workflow.replace_condition_branches(missing, vec![branch("new")]),
            Err(MutationError::NodeNotFound(missing))
        );
        assert_eq!(snapshot(&workflow), before);
    }
}

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
    Graph(GraphError),
}

impl From<GraphError> for MutationError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeRemovalPlan {
    pub node_id: NodeId,
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

    /// Add a typed operation node with one validated JSON result.
    pub fn add_operation_node(&mut self, label: impl Into<String>, plan: crate::operation::InvocationPlan) -> NodeId {
        let id = self.graph.add_node(label, &[], &[PortSpec::new("result", TypeTag::Json.as_str())]);
        self.configs.insert(id.0, NodeConfig::operation(plan));
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

    /// Add a durable approval gate with declared selected inputs and resume outputs.
    pub fn add_approval_node(
        &mut self,
        label: impl Into<String>,
        config: crate::workflow::gate::ApprovalNodeConfig,
    ) -> NodeId {
        const CREDENTIAL_OUTPUT_PREFIX: &str = "credential:";
        let input_specs = config
            .selected_input_ports
            .iter()
            .map(|input| PortSpec::new(input.clone(), TypeTag::Any.as_str()))
            .collect::<Vec<_>>();
        let mut output_specs = config
            .resume
            .values
            .iter()
            .map(|field| PortSpec::new(field.name.as_str(), field.type_tag.clone()))
            .collect::<Vec<_>>();
        output_specs.extend(
            config
                .resume
                .credential_bindings
                .iter()
                .map(|name| PortSpec::new(format!("{CREDENTIAL_OUTPUT_PREFIX}{name}"), TypeTag::String.as_str())),
        );
        let id = self.graph.add_node(label, &input_specs, &output_specs);
        self.configs.insert(id.0, NodeConfig::approval(config));
        id
    }

    pub fn plan_node_removal(&self, node_id: impl WorkflowNodeIdentity) -> Result<NodeRemovalPlan, MutationError> {
        let node_id = node_id.workflow_node_id();
        if self.graph.node(node_id).is_none() {
            return Err(MutationError::NodeNotFound(node_id));
        }
        Ok(NodeRemovalPlan { node_id })
    }

    /// Apply a planned removal only while its selected node still exists.
    pub fn apply_node_removal(&mut self, plan: NodeRemovalPlan) -> Result<(), MutationError> {
        if self.graph.node(plan.node_id).is_none() {
            return Err(MutationError::StaleEdit(plan.node_id));
        }
        self.graph.remove_node(plan.node_id)?;
        self.configs.remove(&plan.node_id.0);
        self.credentials.remove(&plan.node_id.0);
        self.runner.remove(&plan.node_id.0);
        Ok(())
    }

    /// Remove a node and all node-owned workflow state as one lifecycle operation.
    pub fn remove_node(&mut self, node_id: impl WorkflowNodeIdentity) -> Result<(), MutationError> {
        let plan = self.plan_node_removal(node_id)?;
        self.apply_node_removal(plan)
    }

    pub fn plan_condition_branch_reconciliation(
        &self,
        node_id: impl WorkflowNodeIdentity,
        branches: Vec<crate::control_flow::ConditionBranch>,
    ) -> Result<ConditionBranchReconciliationPlan, MutationError> {
        let node_id = node_id.workflow_node_id();
        if self.graph.node(node_id).is_none() {
            return Err(MutationError::NodeNotFound(node_id));
        }
        if !matches!(self.configs.get(&node_id.0), Some(NodeConfig::Condition(_))) {
            return Err(MutationError::ConditionConfigNotFound(node_id));
        }
        let config = ConditionNodeConfig { branches };
        crate::control_flow::validate_condition_config(&config).map_err(MutationError::InvalidConditionConfig)?;
        let requested_specs = config
            .branches
            .iter()
            .map(|branch| PortSpec::new(branch.label.clone(), TypeTag::Any.as_str()))
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
        self.graph.reconcile_node_ports(plan.node_id, PortDirection::Output, &plan.requested_specs)?;
        self.configs.insert(plan.node_id.0, NodeConfig::Condition(plan.config));
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
}

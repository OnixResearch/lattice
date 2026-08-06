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
    /// Shell convenience constructor that reads the wall clock.
    pub fn new(name: impl Into<String>) -> Self {
        Self::new_at(name, chrono::Utc::now())
    }

    /// Deterministic constructor that derives created and modified from a supplied
    /// temporal fact instead of reading an ambient clock.
    pub fn new_at(name: impl Into<String>, now: chrono::DateTime<chrono::Utc>) -> Self {
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

/// Error for stale or contradictory temporal observations supplied to core constructors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalError {
    /// The supplied timestamp precedes the workflow creation time and is contradictory.
    StaleModified {
        created: chrono::DateTime<chrono::Utc>,
        supplied: chrono::DateTime<chrono::Utc>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationError {
    NodeNotFound(NodeId),
    ConditionConfigNotFound(NodeId),
    InvalidConditionConfig(crate::control_flow::ControlFlowError),
    Graph(rat_nodegraph::model::GraphError),
}

impl From<rat_nodegraph::model::GraphError> for MutationError {
    fn from(error: rat_nodegraph::model::GraphError) -> Self {
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
    /// Shell convenience constructor that reads the wall clock.
    pub fn new(name: impl Into<String>) -> Self {
        Self::new_at(name, chrono::Utc::now())
    }

    /// Deterministic constructor that builds an empty workflow from supplied temporal facts.
    pub fn new_at(name: impl Into<String>, now: chrono::DateTime<chrono::Utc>) -> Self {
        let mut graph = Graph::new();
        graph.dag_mode = true;
        // Accept "any" type on either side of an edge.
        graph.set_compatibility(Box::new(|src, tgt| src == tgt || src == "any" || tgt == "any"));

        Self {
            meta: WorkflowMeta::new_at(name, now),
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

    pub fn plan_node_removal(&self, node_id: NodeId) -> Result<NodeRemovalPlan, MutationError> {
        if self.graph.node(node_id).is_none() {
            return Err(MutationError::NodeNotFound(node_id));
        }
        Ok(NodeRemovalPlan { node_id })
    }

    /// Remove a node and all node-owned workflow state as one lifecycle operation.
    pub fn remove_node(&mut self, node_id: NodeId) -> Result<(), MutationError> {
        let plan = self.plan_node_removal(node_id)?;
        self.graph.remove_node(plan.node_id)?;
        self.configs.remove(&plan.node_id.0);
        self.credentials.remove(&plan.node_id.0);
        self.runner.remove(&plan.node_id.0);
        Ok(())
    }

    pub fn plan_condition_branch_reconciliation(
        &self,
        node_id: NodeId,
        branches: Vec<crate::control_flow::ConditionBranch>,
    ) -> Result<ConditionBranchReconciliationPlan, MutationError> {
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
        node_id: NodeId,
        branches: Vec<crate::control_flow::ConditionBranch>,
    ) -> Result<(), MutationError> {
        let plan = self.plan_condition_branch_reconciliation(node_id, branches)?;
        self.graph.reconcile_node_ports(
            plan.node_id,
            rat_nodegraph::model::PortDirection::Output,
            &plan.requested_specs,
        )?;
        self.configs.insert(plan.node_id.0, NodeConfig::Condition(plan.config));
        Ok(())
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

    /// Shell convenience that updates the modified timestamp from the wall clock.
    pub fn touch(&mut self) {
        let _ = self.touch_at(chrono::Utc::now());
    }

    /// Update the modified timestamp from an explicit supplied observation.
    /// Rejects stale or contradictory facts instead of silently applying them.
    pub fn touch_at(&mut self, now: chrono::DateTime<chrono::Utc>) -> Result<(), TemporalError> {
        if now < self.meta.created {
            return Err(TemporalError::StaleModified {
                created: self.meta.created,
                supplied: now,
            });
        }
        self.meta.modified = now;
        Ok(())
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

    fn utc(rfc3339: &str) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339(rfc3339).expect("valid rfc3339").with_timezone(&chrono::Utc)
    }

    #[test]
    fn workflow_meta_create_and_modify_derive_from_supplied_time() {
        let fixed = utc("2026-01-02T03:04:05Z");
        let meta = WorkflowMeta::new_at("timed", fixed);
        assert_eq!(meta.created, fixed);
        assert_eq!(meta.modified, fixed);
    }

    #[test]
    fn workflow_new_at_builds_from_supplied_time_without_a_clock() {
        let fixed = utc("2026-02-03T04:05:06Z");
        let workflow = Workflow::new_at("timed", fixed);
        assert_eq!(workflow.meta.created, fixed);
        assert_eq!(workflow.meta.modified, fixed);
    }

    #[test]
    fn touch_at_applies_the_supplied_observation() {
        let created = utc("2026-01-02T03:04:05Z");
        let touched = utc("2026-01-02T04:00:00Z");
        let mut workflow = Workflow::new_at("timed", created);
        workflow.touch_at(touched).expect("later timestamp is valid");
        assert_eq!(workflow.meta.modified, touched);
    }

    #[test]
    fn touch_at_rejects_a_stale_timestamp_without_mutating() {
        let created = utc("2026-01-02T03:04:05Z");
        let stale = created - chrono::Duration::seconds(5);
        let mut workflow = Workflow::new_at("guarded", created);
        let before = workflow.meta.modified;

        assert_eq!(
            workflow.touch_at(stale),
            Err(TemporalError::StaleModified {
                created,
                supplied: stale,
            })
        );
        assert_eq!(workflow.meta.modified, before);
        assert_eq!(workflow.meta.modified, created);
    }

    #[test]
    fn remove_node_cleans_all_owned_state_and_incident_edges() {
        let mut workflow = Workflow::new("aggregate deletion");
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

        assert!(workflow.graph.node(source).is_none());
        assert!(workflow.graph.node(target).is_some());
        assert!(workflow.node_config(source).is_none());
        assert!(workflow.node_credentials(source).is_none());
        assert!(workflow.node_runner_settings(source).is_none());
        assert_eq!(workflow.graph.edge_count(), 0);
    }

    #[test]
    fn remove_node_cleans_owned_state_for_every_builtin_config_kind() {
        let mut workflow = Workflow::new("all built-in deletion");
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
        let mut workflow = Workflow::new("failed deletion");
        let node = workflow.add_shell_node("kept", "true");
        let missing = NodeId(u64::MAX);

        assert_eq!(workflow.remove_node(missing), Err(MutationError::NodeNotFound(missing)));
        assert!(workflow.graph.node(node).is_some());
        assert!(workflow.node_config(node).is_some());
        assert_eq!(workflow.graph.node_count(), 1);
    }

    #[test]
    fn condition_reconciliation_preserves_matching_ports_and_drops_removed_edges() {
        let mut workflow = Workflow::new("condition reconciliation");
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
        let mut workflow = Workflow::new("rejected condition reconciliation");
        let condition = workflow.add_condition_node("route", ConditionNodeConfig {
            branches: vec![branch("first"), branch("second")],
        });
        let prior_config = workflow.node_config(condition).cloned().expect("condition config");
        let prior_ports = workflow.graph.node(condition).expect("condition node").output_ports.clone();
        let prior_edges = workflow.graph.edges().to_vec();

        let result = workflow.replace_condition_branches(condition, vec![branch("duplicate"), branch("duplicate")]);

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

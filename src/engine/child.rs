//! Functional subworkflow invocation and result validation around an injected runner shell.

type OrderedMap<K, V> = std::collections::BTreeMap<K, V>;
type OrderedSet<T> = std::collections::BTreeSet<T>;
type HashMap<K, V> = std::collections::HashMap<K, V>;
type Node = rat_nodegraph::model::Node;
type NodeId = rat_nodegraph::model::NodeId;
type PortId = rat_nodegraph::model::PortId;
type SubworkflowNodeConfig = crate::control_flow::SubworkflowNodeConfig;
type Value = crate::engine::Value;
type Workflow = crate::engine::Workflow;
type RunId = crate::engine::executor::RunId;

mod error;
mod runtime;

pub use runtime::RepositorySubworkflowRunnerShell;
pub use runtime::WorkflowCatalogRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubworkflowCorrelation {
    pub parent_run_id: RunId,
    pub subworkflow_node_id: NodeId,
}

#[derive(Debug, Clone)]
pub struct SubworkflowInvocation {
    pub workflow_name: String,
    pub ancestry: Vec<String>,
    pub inputs: OrderedMap<String, Value>,
    pub requested_outputs: Vec<String>,
    pub correlation: SubworkflowCorrelation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubworkflowEventKind {
    Started,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubworkflowEventMetadata {
    pub parent_run_id: RunId,
    pub subworkflow_node_id: NodeId,
    pub child_run_id: RunId,
    pub workflow_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubworkflowEvent {
    pub kind: SubworkflowEventKind,
    pub metadata: SubworkflowEventMetadata,
}

#[derive(Debug, Clone)]
pub struct SubworkflowExecution {
    pub workflow_name: String,
    pub child_run_id: RunId,
    pub outputs: OrderedMap<String, Value>,
    pub events: Vec<SubworkflowEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubworkflowExecutionError {
    RunnerUnavailable,
    MissingParentInput(String),
    MissingParentInputPort(String),
    DuplicateChildInput(String),
    MissingParentOutputPort(String),
    DuplicateParentOutput(String),
    MissingChildOutput(String),
    MissingChildInputPort(String),
    AmbiguousChildInputPort(String),
    AmbiguousChildOutput(String),
    ChildWorkflowMissing(String),
    ChildWorkflowLoadFailed,
    ChildWorkflowExecutionFailed,
    WorkflowIdentityMismatch,
    MissingLifecycleEvents,
    LifecycleOrderMismatch,
    EventCorrelationMismatch,
}

#[async_trait::async_trait]
pub trait SubworkflowRunnerShell: Send + Sync {
    /// Load and execute the named saved child workflow under the supplied bounded ancestry and
    /// correlation context.
    async fn execute(
        &self,
        invocation: &SubworkflowInvocation,
    ) -> Result<SubworkflowExecution, SubworkflowExecutionError>;
}

#[derive(Debug, Default)]
pub struct UnavailableSubworkflowRunnerShell;

#[async_trait::async_trait]
impl SubworkflowRunnerShell for UnavailableSubworkflowRunnerShell {
    async fn execute(
        &self,
        _invocation: &SubworkflowInvocation,
    ) -> Result<SubworkflowExecution, SubworkflowExecutionError> {
        Err(SubworkflowExecutionError::RunnerUnavailable)
    }
}

pub fn build_invocation(
    parent_workflow_name: &str,
    parent_run_id: RunId,
    node: &Node,
    config: &SubworkflowNodeConfig,
    inputs: &HashMap<PortId, Value>,
) -> Result<SubworkflowInvocation, SubworkflowExecutionError> {
    debug_assert!(!parent_workflow_name.is_empty());
    debug_assert!(config.input_mappings.len() <= node.input_ports.len());
    let mut child_inputs = OrderedMap::new();
    for mapping in &config.input_mappings {
        let parent_input_definition = node
            .input_ports
            .iter()
            .find(|port| port.label == mapping.parent_input)
            .ok_or_else(|| SubworkflowExecutionError::MissingParentInputPort(mapping.parent_input.clone()))?;
        let value = inputs
            .get(&parent_input_definition.id)
            .ok_or_else(|| SubworkflowExecutionError::MissingParentInput(mapping.parent_input.clone()))?;
        assert!(child_inputs.len() < config.input_mappings.len());
        if child_inputs.insert(mapping.child_input.clone(), value.clone()).is_some() {
            return Err(SubworkflowExecutionError::DuplicateChildInput(mapping.child_input.clone()));
        }
    }

    Ok(SubworkflowInvocation {
        workflow_name: config.workflow_name.clone(),
        ancestry: vec![parent_workflow_name.to_string()],
        inputs: child_inputs,
        requested_outputs: config.output_mappings.iter().map(|mapping| mapping.child_output.clone()).collect(),
        correlation: SubworkflowCorrelation {
            parent_run_id,
            subworkflow_node_id: node.id,
        },
    })
}

pub fn validate_and_map_execution(
    node: &Node,
    config: &SubworkflowNodeConfig,
    invocation: &SubworkflowInvocation,
    execution: SubworkflowExecution,
) -> Result<HashMap<PortId, Value>, SubworkflowExecutionError> {
    if execution.workflow_name != invocation.workflow_name {
        return Err(SubworkflowExecutionError::WorkflowIdentityMismatch);
    }
    validate_events(invocation, &execution)?;

    debug_assert_eq!(invocation.correlation.subworkflow_node_id, node.id);
    debug_assert!(config.output_mappings.len() <= node.output_ports.len());
    let mut mapped_parent_labels = OrderedSet::new();
    let mut mapped_outputs = HashMap::with_capacity(config.output_mappings.len());
    for mapping in &config.output_mappings {
        if !mapped_parent_labels.insert(mapping.parent_output.clone()) {
            return Err(SubworkflowExecutionError::DuplicateParentOutput(mapping.parent_output.clone()));
        }
        let parent_output_definition = node
            .output_ports
            .iter()
            .find(|port| port.label == mapping.parent_output)
            .ok_or_else(|| SubworkflowExecutionError::MissingParentOutputPort(mapping.parent_output.clone()))?;
        let child_value = execution
            .outputs
            .get(&mapping.child_output)
            .ok_or_else(|| SubworkflowExecutionError::MissingChildOutput(mapping.child_output.clone()))?;
        mapped_outputs.insert(parent_output_definition.id, child_value.clone());
    }
    Ok(mapped_outputs)
}

pub fn bind_named_inputs(
    workflow: &Workflow,
    named_inputs: &OrderedMap<String, Value>,
) -> Result<HashMap<PortId, Value>, SubworkflowExecutionError> {
    let total_child_inputs = workflow.graph.nodes().map(|node| node.input_ports.len()).sum::<usize>();
    debug_assert!(named_inputs.len() <= total_child_inputs);
    debug_assert!(workflow.graph.edge_count() <= total_child_inputs);
    let mut bound_inputs = HashMap::with_capacity(named_inputs.len());
    for (label, value) in named_inputs {
        let matching_input_definitions = workflow
            .graph
            .nodes()
            .flat_map(|node| node.input_ports.iter())
            .filter(|port| port.label == *label)
            .filter(|port| !workflow.graph.edges().iter().any(|edge| edge.target == port.id))
            .collect::<Vec<_>>();
        let Some(port) = matching_input_definitions.first() else {
            return Err(SubworkflowExecutionError::MissingChildInputPort(label.clone()));
        };
        if matching_input_definitions.len() != 1 {
            return Err(SubworkflowExecutionError::AmbiguousChildInputPort(label.clone()));
        }
        bound_inputs.insert(port.id, value.clone());
    }
    Ok(bound_inputs)
}

pub(super) fn collect_requested_outputs(
    workflow: &Workflow,
    steps: &[crate::engine::executor::StepResult],
    requested_outputs: &[String],
) -> Result<OrderedMap<String, Value>, SubworkflowExecutionError> {
    let total_child_outputs = workflow.graph.nodes().map(|node| node.output_ports.len()).sum::<usize>();
    debug_assert!(requested_outputs.len() <= total_child_outputs);
    debug_assert!(steps.len() <= workflow.graph.node_count());
    let mut output_pairs = Vec::with_capacity(requested_outputs.len());
    for requested_output in requested_outputs {
        let mut matches = Vec::with_capacity(steps.len());
        for step in steps {
            let Some(node) = workflow.graph.node(step.node_id) else {
                continue;
            };
            let crate::engine::executor::NodeStatus::Completed { outputs, .. } = &step.status else {
                continue;
            };
            for port in &node.output_ports {
                if port.label == *requested_output
                    && let Some(value) = outputs.get(&port.id)
                {
                    matches.push(value.clone());
                }
            }
        }
        let Some(value) = matches.pop() else {
            return Err(SubworkflowExecutionError::MissingChildOutput(requested_output.clone()));
        };
        if !matches.is_empty() {
            return Err(SubworkflowExecutionError::AmbiguousChildOutput(requested_output.clone()));
        }
        assert!(output_pairs.len() < requested_outputs.len());
        output_pairs.push((requested_output.clone(), value));
    }
    Ok(output_pairs.into_iter().collect())
}

fn validate_events(
    invocation: &SubworkflowInvocation,
    execution: &SubworkflowExecution,
) -> Result<(), SubworkflowExecutionError> {
    debug_assert_eq!(execution.workflow_name, invocation.workflow_name);
    debug_assert!(!invocation.workflow_name.is_empty());
    let Some(first_event) = execution.events.first() else {
        return Err(SubworkflowExecutionError::MissingLifecycleEvents);
    };
    let Some(last_event) = execution.events.last() else {
        return Err(SubworkflowExecutionError::MissingLifecycleEvents);
    };
    if first_event.kind != SubworkflowEventKind::Started || last_event.kind != SubworkflowEventKind::Completed {
        return Err(SubworkflowExecutionError::LifecycleOrderMismatch);
    }
    for event in &execution.events {
        let metadata = &event.metadata;
        let is_parent_run_mismatch = metadata.parent_run_id != invocation.correlation.parent_run_id;
        let is_parent_node_mismatch = metadata.subworkflow_node_id != invocation.correlation.subworkflow_node_id;
        let is_child_run_mismatch = metadata.child_run_id != execution.child_run_id;
        let is_workflow_mismatch = metadata.workflow_name != invocation.workflow_name;
        if is_parent_run_mismatch || is_parent_node_mismatch {
            return Err(SubworkflowExecutionError::EventCorrelationMismatch);
        }
        if is_child_run_mismatch || is_workflow_mismatch {
            return Err(SubworkflowExecutionError::EventCorrelationMismatch);
        }
    }
    Ok(())
}

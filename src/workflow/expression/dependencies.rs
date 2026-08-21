//! Pure adapters from Lattice workflow values to expression dependency facts.

#[path = "inventory/portable.rs"]
mod portable_inventory;
#[path = "inventory/runtime.rs"]
mod runtime_inventory;

pub use lattice_core::expression_dependencies::ExecutionDependency;
pub use lattice_core::expression_dependencies::ExpressionDependency;
pub use lattice_core::expression_dependencies::ExpressionDependencyIssue;
pub use lattice_core::expression_dependencies::ExpressionDependencyIssueCode;
pub use lattice_core::expression_dependencies::ExpressionDependencyProjection;

// r[impl lattice.workflow_validation.explicit_dependencies.projection]
pub fn workflow_projection(
    workflow: &crate::engine::workflow::Workflow,
) -> Result<ExpressionDependencyProjection, Vec<ExpressionDependencyIssue>> {
    let fields = runtime_inventory::fields(workflow)?;
    let node_ids = workflow.graph.nodes().map(|node| node.id.0).collect::<std::collections::BTreeSet<_>>();
    let output_ports = workflow
        .graph
        .nodes()
        .flat_map(|node| {
            node.output_ports.iter().map(move |port| lattice_core::expression_dependencies::OutputPortFact {
                producer_node: node.id.0,
                output_port: lattice_core::expressions::ExpressionPortId(port.id.0),
            })
        })
        .collect::<Vec<_>>();
    let data_dependencies = workflow
        .graph
        .edges()
        .iter()
        .filter_map(|edge| {
            let producer_node = workflow.graph.port_owner(edge.source)?;
            let consumer_node = workflow.graph.port_owner(edge.target)?;
            Some(ExecutionDependency {
                producer_node: producer_node.0,
                consumer_node: consumer_node.0,
            })
        })
        .collect::<Vec<_>>();
    let available = workflow
        .graph
        .nodes()
        .map(|node| node.input_ports.len().saturating_add(node.output_ports.len()))
        .sum::<usize>();
    debug_assert!(output_ports.len() <= available);
    debug_assert!(data_dependencies.len() <= workflow.graph.edges().len());
    lattice_core::expression_dependencies::project_expression_dependencies(
        &fields,
        &node_ids,
        &output_ports,
        &data_dependencies,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextError {
    pub consumer_node: u64,
    pub config_field_path: String,
    pub producer_node: u64,
    pub producer_output_port: lattice_core::expressions::ExpressionPortId,
}

pub fn context_for_consumer(
    dependencies: &[ExpressionDependency],
    consumer_node: u64,
    port_values: &std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::value::Value>,
) -> Result<crate::expressions::ExpressionContext, ContextError> {
    let selected = dependencies.iter().filter(|dependency| dependency.consumer_node == consumer_node);
    let values = selected
        .map(|dependency| {
            let value = port_values
                .get(&crate::engine::workflow::PortId(dependency.producer_output_port.0))
                .ok_or_else(|| ContextError {
                    consumer_node,
                    config_field_path: dependency.config_field_path.clone(),
                    producer_node: dependency.producer_node,
                    producer_output_port: dependency.producer_output_port,
                })?;
            Ok(((dependency.producer_node, dependency.producer_output_port), value.clone()))
        })
        .collect::<Result<std::collections::HashMap<_, _>, ContextError>>()?;
    Ok(crate::expressions::ExpressionContext::new(values))
}

pub fn portable_projection(
    portable: &crate::workflow_format::PortableWorkflow,
) -> Result<ExpressionDependencyProjection, Vec<ExpressionDependencyIssue>> {
    let fields = portable_inventory::fields(&portable.configs);
    let node_ids = portable.nodes.iter().map(|node| node.id).collect::<std::collections::BTreeSet<_>>();
    let output_ports = portable
        .nodes
        .iter()
        .flat_map(|node| {
            node.output_ports.iter().map(move |port| lattice_core::expression_dependencies::OutputPortFact {
                producer_node: node.id,
                output_port: lattice_core::expressions::ExpressionPortId(port.id),
            })
        })
        .collect::<Vec<_>>();
    let port_owners = portable
        .nodes
        .iter()
        .flat_map(|node| node.input_ports.iter().chain(&node.output_ports).map(move |port| (port.id, node.id)))
        .collect::<std::collections::BTreeMap<_, _>>();
    let data_dependencies = portable
        .edges
        .iter()
        .filter_map(|edge| {
            Some(ExecutionDependency {
                producer_node: *port_owners.get(&edge.source)?,
                consumer_node: *port_owners.get(&edge.target)?,
            })
        })
        .collect::<Vec<_>>();
    debug_assert!(output_ports.len() <= port_owners.len());
    debug_assert!(data_dependencies.len() <= portable.edges.len());
    lattice_core::expression_dependencies::project_expression_dependencies(
        &fields,
        &node_ids,
        &output_ports,
        &data_dependencies,
    )
}

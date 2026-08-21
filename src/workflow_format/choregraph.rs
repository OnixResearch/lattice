//! Pure conversion from decoded Lattice workflow values to Choregraph input.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

const LIFECYCLE_EVENT_COUNT: usize = 5;
const LIFECYCLE_EVENTS: [&str; LIFECYCLE_EVENT_COUNT] = ["cancel", "complete", "fail", "mark-ready", "start"];
const REQUEST_SLOT: &str = "dispatch-request";
const PAYLOAD_SLOT: &str = "resolved-payload";
const REQUEST_SOURCE_SUFFIX: &str = "/request";
const PAYLOAD_SOURCE_SUFFIX: &str = "/payload";
const DECLARATION_PREFIX: &str = "lattice-effect";
const RESOURCE_FILE_PREFIX: &str = "lattice://file/";
const RESOURCE_PROCESS_PREFIX: &str = "lattice://process/";
const RESOURCE_NETWORK_PREFIX: &str = "lattice://network/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SyntheticPortIdentity(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SyntheticExpressionEdge {
    consumer_node: u64,
    source: SyntheticPortIdentity,
    target: SyntheticPortIdentity,
}

fn limit(value: usize) -> Result<u64, super::WorkflowFormatError> {
    u64::try_from(value).map_err(|_| super::WorkflowFormatError::ChoregraphLimitEncoding)
}

fn limits() -> Result<choregraph_nickel::EvaluatedLimits, super::WorkflowFormatError> {
    Ok(choregraph_nickel::EvaluatedLimits {
        max_nodes: limit(choregraph_core::DEFAULT_MAX_NODES)?,
        max_edges: limit(choregraph_core::DEFAULT_MAX_EDGES)?,
        max_operations: limit(choregraph_core::DEFAULT_MAX_OPERATIONS)?,
        max_effect_declarations: limit(choregraph_core::DEFAULT_MAX_EFFECT_DECLARATIONS)?,
        max_identifier_bytes: limit(choregraph_core::DEFAULT_MAX_IDENTIFIER_BYTES)?,
        max_reference_bytes: limit(choregraph_core::DEFAULT_MAX_REFERENCE_BYTES)?,
        max_event_bytes: limit(choregraph_core::DEFAULT_MAX_EVENT_BYTES)?,
        max_intent_bytes: limit(choregraph_core::DEFAULT_MAX_INTENT_BYTES)?,
        max_intents_per_transition: limit(choregraph_core::DEFAULT_MAX_INTENTS_PER_TRANSITION)?,
    })
}

fn node(input: &super::PortableNode, synthetic_inputs: &[u64]) -> choregraph_nickel::LatticeNode {
    let inputs = input.input_ports.iter().map(|port| port.id).chain(synthetic_inputs.iter().copied()).map(|id| {
        choregraph_nickel::LatticePort {
            id,
            direction: choregraph_nickel::LatticePortDirection::Input,
        }
    });
    let outputs = input.output_ports.iter().map(|port| choregraph_nickel::LatticePort {
        id: port.id,
        direction: choregraph_nickel::LatticePortDirection::Output,
    });
    choregraph_nickel::LatticeNode {
        id: input.id,
        ports: inputs.chain(outputs).collect(),
    }
}

fn failure(error: choregraph_nickel::FrontendFailure) -> super::WorkflowFormatError {
    error
        .issues
        .first()
        .map(|issue| super::WorkflowFormatError::ChoregraphLowering(issue.code()))
        .unwrap_or(super::WorkflowFormatError::ChoregraphLoweringContract)
}

fn effect_binding(slot: &str, source: String) -> choregraph_nickel::EvaluatedReferenceBinding {
    choregraph_nickel::EvaluatedReferenceBinding::Slot {
        slot_id: slot.to_string(),
        source_ref: source,
    }
}

fn effect_contract(config: &super::PortableNodeConfig) -> Option<(&'static str, &'static str)> {
    match config.kind.as_str() {
        "shell" | "operation" => Some((RESOURCE_PROCESS_PREFIX, "process/execute")),
        "http" => Some((RESOURCE_NETWORK_PREFIX, "network/request")),
        "file" => Some((RESOURCE_FILE_PREFIX, match config.file_operation.as_str() {
            "read" => "file/read",
            "append" => "file/append",
            _ => "file/write",
        })),
        _ => None,
    }
}

fn effect(config: &super::PortableNodeConfig) -> Option<choregraph_nickel::EvaluatedLatticeEffectV2> {
    let (resource_prefix, ability) = effect_contract(config)?;
    let source_prefix = format!("node/{}/input", config.node_id);
    Some(choregraph_nickel::EvaluatedLatticeEffectV2 {
        declaration_key: format!("{DECLARATION_PREFIX}-{}", config.node_id),
        node_id: config.node_id,
        from_state: "ready".to_string(),
        action: "start".to_string(),
        resource: format!("{resource_prefix}{}", config.node_id),
        ability: ability.to_string(),
        request_binding: effect_binding(REQUEST_SLOT, format!("{source_prefix}{REQUEST_SOURCE_SUFFIX}")),
        payload_binding: effect_binding(PAYLOAD_SLOT, format!("{source_prefix}{PAYLOAD_SOURCE_SUFFIX}")),
    })
}

fn expression_issue(
    error: Vec<crate::expression_dependencies::ExpressionDependencyIssue>,
) -> super::WorkflowFormatError {
    match error.into_iter().next() {
        Some(issue) => super::WorkflowFormatError::InvalidExpressionDependency {
            node_id: issue.consumer_node,
            field_path: issue.config_field_path,
            message: issue.message,
        },
        None => super::WorkflowFormatError::ChoregraphLoweringContract,
    }
}

fn synthetic_expression_edges(
    portable: &super::PortableWorkflow,
) -> Result<Vec<SyntheticExpressionEdge>, super::WorkflowFormatError> {
    let projection = crate::expression_dependencies::portable_projection(portable).map_err(expression_issue)?;
    debug_assert!(projection.expression_dependencies.len() <= choregraph_core::DEFAULT_MAX_EDGES);
    debug_assert!(projection.execution_dependencies.len() <= choregraph_core::DEFAULT_MAX_EDGES);
    let port_owners = portable
        .nodes
        .iter()
        .flat_map(|node| node.input_ports.iter().chain(&node.output_ports).map(move |port| (port.id, node.id)))
        .collect::<BTreeMap<_, _>>();
    let data_pairs = portable
        .edges
        .iter()
        .filter_map(|edge| Some((*port_owners.get(&edge.source)?, *port_owners.get(&edge.target)?)))
        .collect::<BTreeSet<_>>();
    let mut expression_pair_sources = BTreeMap::new();
    for dependency in projection.expression_dependencies {
        let pair = (dependency.producer_node, dependency.consumer_node);
        if data_pairs.contains(&pair) {
            continue;
        }
        expression_pair_sources
            .entry(pair)
            .and_modify(|source: &mut SyntheticPortIdentity| {
                source.0 = source.0.min(dependency.producer_output_port.0);
            })
            .or_insert(SyntheticPortIdentity(dependency.producer_output_port.0));
    }

    let maximum_port = portable
        .nodes
        .iter()
        .flat_map(|node| node.input_ports.iter().chain(&node.output_ports))
        .map(|port| port.id)
        .max();
    let first_synthetic = SyntheticPortIdentity(match maximum_port {
        Some(port_id) => port_id.checked_add(1).ok_or(super::WorkflowFormatError::ChoregraphDependencyPortExhausted)?,
        None => 0,
    });

    expression_pair_sources
        .into_iter()
        .enumerate()
        .map(|(index, ((_producer_node, consumer_node), source))| {
            let target = first_synthetic
                .0
                .checked_add(
                    u64::try_from(index).map_err(|_| super::WorkflowFormatError::ChoregraphDependencyPortExhausted)?,
                )
                .map(SyntheticPortIdentity)
                .ok_or(super::WorkflowFormatError::ChoregraphDependencyPortExhausted)?;
            Ok(SyntheticExpressionEdge {
                consumer_node,
                source,
                target,
            })
        })
        .collect()
}

// r[impl lattice.choregraph_adoption.lowering]
// r[impl lattice.choregraph_effect_authority.binding]
pub fn lower_portable(
    portable: &super::PortableWorkflow,
) -> Result<choregraph_nickel::LoweringOutput, super::WorkflowFormatError> {
    let synthetic_edges = synthetic_expression_edges(portable)?;
    let mut synthetic_inputs = BTreeMap::<u64, Vec<u64>>::new();
    for edge in &synthetic_edges {
        synthetic_inputs.entry(edge.consumer_node).or_default().push(edge.target.0);
    }
    let evaluated = choregraph_nickel::EvaluatedSource::LatticeWorkflowV2 {
        schema: choregraph_nickel::LATTICE_V2_EVALUATED_SCHEMA.to_string(),
        lifecycle_profile: choregraph_core::LINEAR_LIFECYCLE_V1.to_string(),
        events: LIFECYCLE_EVENTS.map(str::to_string).to_vec(),
        limits: limits()?,
        imports: Vec::new(),
        nodes: portable
            .nodes
            .iter()
            .map(|input| node(input, synthetic_inputs.get(&input.id).map_or(&[], Vec::as_slice)))
            .collect(),
        edges: portable
            .edges
            .iter()
            .map(|edge| choregraph_nickel::LatticePortEdge {
                source: edge.source,
                target: edge.target,
            })
            .chain(synthetic_edges.iter().map(|edge| choregraph_nickel::LatticePortEdge {
                source: edge.source.0,
                target: edge.target.0,
            }))
            .collect(),
        configs: portable
            .configs
            .iter()
            .map(|config| choregraph_nickel::LatticeNodeConfigV2 {
                node_id: config.node_id,
                operation: config.kind.clone(),
                effectful: effect_contract(config).is_some(),
            })
            .collect(),
        effects: portable.configs.iter().filter_map(effect).collect(),
    };
    let evaluated_json =
        serde_json::to_vec(&evaluated).map_err(|_| super::WorkflowFormatError::ChoregraphInputEncoding)?;
    let lowering =
        choregraph_nickel::lower_evaluated_json(&evaluated_json, &[], choregraph_nickel::reviewed_frontend_limits())
            .map_err(failure)?;
    debug_assert_eq!(lowering.machine.node_ids().len(), portable.nodes.len());
    let expected_edge_count = portable
        .edges
        .len()
        .checked_add(synthetic_edges.len())
        .ok_or(super::WorkflowFormatError::ChoregraphLimitEncoding)?;
    debug_assert_eq!(lowering.machine.edges().len(), expected_edge_count);
    Ok(lowering)
}

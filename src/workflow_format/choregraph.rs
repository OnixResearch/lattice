//! Pure conversion from decoded Lattice workflow values to Choregraph input.

use choregraph_core::AdmissionLimits;
use choregraph_nickel::EvaluatedLimits;
use choregraph_nickel::EvaluatedSource;
use choregraph_nickel::LatticeNode;
use choregraph_nickel::LatticeNodeConfig;
use choregraph_nickel::LatticePort;
use choregraph_nickel::LatticePortDirection;
use choregraph_nickel::LatticePortEdge;
use choregraph_nickel::LoweringOutput;

use super::PortableNode;
use super::PortableWorkflow;
use super::WorkflowFormatError;

const CHOREGRAPH_LIFECYCLE_EVENT_COUNT: usize = 5;
const CHOREGRAPH_LIFECYCLE_EVENTS: [&str; CHOREGRAPH_LIFECYCLE_EVENT_COUNT] =
    ["cancel", "complete", "fail", "mark-ready", "start"];

fn choregraph_limit(value: usize) -> Result<u64, WorkflowFormatError> {
    u64::try_from(value).map_err(|_| WorkflowFormatError::ChoregraphLimitEncoding)
}

fn choregraph_limits() -> Result<EvaluatedLimits, WorkflowFormatError> {
    let ceiling = AdmissionLimits::default();
    Ok(EvaluatedLimits {
        max_nodes: choregraph_limit(ceiling.max_nodes)?,
        max_edges: choregraph_limit(ceiling.max_edges)?,
        max_operations: choregraph_limit(ceiling.max_operations)?,
        max_effect_declarations: choregraph_limit(ceiling.max_effect_declarations)?,
        max_identifier_bytes: choregraph_limit(ceiling.max_identifier_bytes)?,
        max_reference_bytes: choregraph_limit(ceiling.max_reference_bytes)?,
        max_event_bytes: choregraph_limit(ceiling.max_event_bytes)?,
        max_intent_bytes: choregraph_limit(ceiling.max_intent_bytes)?,
        max_intents_per_transition: choregraph_limit(ceiling.max_intents_per_transition)?,
    })
}

fn choregraph_node(node: &PortableNode) -> LatticeNode {
    let input_ports = node.input_ports.iter().map(|port| LatticePort {
        id: port.id,
        direction: LatticePortDirection::Input,
    });
    let output_ports = node.output_ports.iter().map(|port| LatticePort {
        id: port.id,
        direction: LatticePortDirection::Output,
    });
    LatticeNode {
        id: node.id,
        ports: input_ports.chain(output_ports).collect(),
    }
}

fn choregraph_failure(error: choregraph_nickel::FrontendFailure) -> WorkflowFormatError {
    error
        .issues
        .first()
        .map(|issue| WorkflowFormatError::ChoregraphLowering(issue.code()))
        .unwrap_or(WorkflowFormatError::ChoregraphLoweringContract)
}

// r[impl lattice.choregraph_adoption.lowering]
pub fn lower_portable_workflow_with_choregraph(
    portable: &PortableWorkflow,
) -> Result<LoweringOutput, WorkflowFormatError> {
    let evaluated = EvaluatedSource::LatticeWorkflow {
        schema: choregraph_nickel::LATTICE_EVALUATED_SCHEMA.to_string(),
        lifecycle_profile: choregraph_core::LINEAR_LIFECYCLE_V1.to_string(),
        events: CHOREGRAPH_LIFECYCLE_EVENTS.map(str::to_string).to_vec(),
        limits: choregraph_limits()?,
        imports: Vec::new(),
        nodes: portable.nodes.iter().map(choregraph_node).collect(),
        edges: portable
            .edges
            .iter()
            .map(|edge| LatticePortEdge {
                source: edge.source,
                target: edge.target,
            })
            .collect(),
        configs: portable
            .configs
            .iter()
            .map(|config| LatticeNodeConfig {
                node_id: config.node_id,
                operation: config.kind.clone(),
            })
            .collect(),
    };
    let evaluated_json = serde_json::to_vec(&evaluated).map_err(|_| WorkflowFormatError::ChoregraphInputEncoding)?;
    let lowering =
        choregraph_nickel::lower_evaluated_json(&evaluated_json, &[], choregraph_nickel::reviewed_frontend_limits())
            .map_err(choregraph_failure)?;
    debug_assert_eq!(lowering.machine.node_ids().len(), portable.nodes.len());
    debug_assert_eq!(lowering.machine.edges().len(), portable.edges.len());
    Ok(lowering)
}

//! Pure conversion from decoded Lattice workflow values to Choregraph input.

const LIFECYCLE_EVENT_COUNT: usize = 5;
const LIFECYCLE_EVENTS: [&str; LIFECYCLE_EVENT_COUNT] = ["cancel", "complete", "fail", "mark-ready", "start"];

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

fn node(input: &super::PortableNode) -> choregraph_nickel::LatticeNode {
    let inputs = input.input_ports.iter().map(|port| choregraph_nickel::LatticePort {
        id: port.id,
        direction: choregraph_nickel::LatticePortDirection::Input,
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

// r[impl lattice.choregraph_adoption.lowering]
pub fn lower_portable(
    portable: &super::PortableWorkflow,
) -> Result<choregraph_nickel::LoweringOutput, super::WorkflowFormatError> {
    let evaluated = choregraph_nickel::EvaluatedSource::LatticeWorkflow {
        schema: choregraph_nickel::LATTICE_EVALUATED_SCHEMA.to_string(),
        lifecycle_profile: choregraph_core::LINEAR_LIFECYCLE_V1.to_string(),
        events: LIFECYCLE_EVENTS.map(str::to_string).to_vec(),
        limits: limits()?,
        imports: Vec::new(),
        nodes: portable.nodes.iter().map(node).collect(),
        edges: portable
            .edges
            .iter()
            .map(|edge| choregraph_nickel::LatticePortEdge {
                source: edge.source,
                target: edge.target,
            })
            .collect(),
        configs: portable
            .configs
            .iter()
            .map(|config| choregraph_nickel::LatticeNodeConfig {
                node_id: config.node_id,
                operation: config.kind.clone(),
            })
            .collect(),
    };
    let evaluated_json =
        serde_json::to_vec(&evaluated).map_err(|_| super::WorkflowFormatError::ChoregraphInputEncoding)?;
    let lowering =
        choregraph_nickel::lower_evaluated_json(&evaluated_json, &[], choregraph_nickel::reviewed_frontend_limits())
            .map_err(failure)?;
    debug_assert_eq!(lowering.machine.node_ids().len(), portable.nodes.len());
    debug_assert_eq!(lowering.machine.edges().len(), portable.edges.len());
    Ok(lowering)
}

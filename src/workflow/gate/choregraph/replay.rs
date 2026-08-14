pub fn replay_request(
    workflow: &crate::engine::workflow::Workflow,
    request: &crate::workflow::gate::model::ApprovalRequest,
) -> Result<
    std::collections::BTreeMap<crate::engine::workflow::NodeId, crate::workflow::gate::model::PersistedNodeState>,
    AdapterError,
> {
    require_profile(request)?;
    let machine = approval_machine(workflow)?;
    let state = replay(&machine, &request.graph_events)?;
    let node_count = workflow.graph.nodes().count();
    let mut result = Vec::with_capacity(node_count);
    for node in workflow.graph.nodes() {
        let machine_node = machine_node_id(&machine, node.id)?;
        let Some(local) = state.state(&machine_node) else {
            return Err(error(AdapterErrorCode::UnknownNode, "Choregraph replay omitted a workflow node"));
        };
        let persisted = match local {
            choregraph_core::LocalState::Completed => Some(crate::workflow::gate::model::PersistedNodeState::Completed),
            choregraph_core::LocalState::Failed => Some(crate::workflow::gate::model::PersistedNodeState::Failed),
            choregraph_core::LocalState::Cancelled => Some(crate::workflow::gate::model::PersistedNodeState::Cancelled),
            choregraph_core::LocalState::Pending
            | choregraph_core::LocalState::Ready
            | choregraph_core::LocalState::Running
            | choregraph_core::LocalState::Suspended => None,
        };
        if let Some(persisted) = persisted {
            result.push((node.id, persisted));
        }
    }
    debug_assert!(result.len() <= node_count);
    debug_assert!(result.iter().all(|(node_id, _)| workflow.graph.node(*node_id).is_some()));
    Ok(result.into_iter().collect())
}

fn approval_machine(
    workflow: &crate::engine::workflow::Workflow,
) -> Result<choregraph_core::AdmittedMachine, AdapterError> {
    let nodes = workflow
        .graph
        .nodes()
        .map(|node| choregraph_core::NodeSpec::new(node_reference(node.id), Vec::new()))
        .collect::<Vec<_>>();
    let node_count = nodes.len();
    let port_owners = workflow
        .graph
        .nodes()
        .flat_map(|node| node.input_ports.iter().chain(&node.output_ports).map(move |port| (port.id, node.id)))
        .collect::<std::collections::BTreeMap<_, _>>();
    let edge_count = workflow.graph.edges().len();
    let mut edge_pairs = Vec::with_capacity(edge_count);
    for edge in workflow.graph.edges() {
        let source = port_owners
            .get(&edge.source)
            .ok_or_else(|| error(AdapterErrorCode::UnknownNode, "approval graph edge has no source owner"))?;
        let target = port_owners
            .get(&edge.target)
            .ok_or_else(|| error(AdapterErrorCode::UnknownNode, "approval graph edge has no target owner"))?;
        edge_pairs.push((*source, *target));
    }
    edge_pairs.sort_unstable();
    edge_pairs.dedup();
    let edges = edge_pairs
        .into_iter()
        .map(|(source, target)| choregraph_core::EdgeSpec::new(node_reference(source), node_reference(target)))
        .collect::<Vec<_>>();
    let mut spec = choregraph_core::MachineSpec::new(nodes, edges);
    spec.lifecycle_profile = crate::workflow::gate::model::CHOREGRAPH_SUSPENSION_PROFILE.to_owned();
    let limits = choregraph_core::AdmissionLimits {
        max_nodes: choregraph_core::DEFAULT_MAX_NODES,
        max_edges: choregraph_core::DEFAULT_MAX_EDGES,
        max_operations: choregraph_core::DEFAULT_MAX_OPERATIONS,
        max_effect_declarations: choregraph_core::DEFAULT_MAX_EFFECT_DECLARATIONS,
        max_identifier_bytes: choregraph_core::DEFAULT_MAX_IDENTIFIER_BYTES,
        max_reference_bytes: choregraph_core::DEFAULT_MAX_REFERENCE_BYTES,
        max_event_bytes: choregraph_core::DEFAULT_MAX_EVENT_BYTES,
        max_intent_bytes: choregraph_core::DEFAULT_MAX_INTENT_BYTES,
        max_intents_per_transition: choregraph_core::DEFAULT_MAX_INTENTS_PER_TRANSITION,
    };
    debug_assert_eq!(spec.nodes.len(), node_count);
    debug_assert!(spec.edges.len() <= edge_count);
    choregraph_core::AdmittedMachine::admit(spec, limits)
        .map_err(|_| error(AdapterErrorCode::GraphAdmission, "workflow approval graph failed Choregraph admission"))
}

fn replay(
    machine: &choregraph_core::AdmittedMachine,
    events: &[crate::workflow::gate::model::ApprovalGraphEvent],
) -> Result<choregraph_core::MachineState, AdapterError> {
    let mut state = machine.initial_state();
    for approval_event in events {
        state = apply_event(machine, &state, approval_event)?;
    }
    Ok(state)
}

fn apply_event(
    machine: &choregraph_core::AdmittedMachine,
    state: &choregraph_core::MachineState,
    event: &crate::workflow::gate::model::ApprovalGraphEvent,
) -> Result<choregraph_core::MachineState, AdapterError> {
    let event = event_to_core(event)?;
    match choregraph_core::step(machine, state, &event) {
        choregraph_core::Decision::Accept { next, intents } => {
            if !intents.is_empty() {
                return Err(error(
                    AdapterErrorCode::UnexpectedIntent,
                    "workflow approval suspension transition emitted a product effect intent",
                ));
            }
            Ok(next)
        }
        choregraph_core::Decision::Deny { .. } => Err(error(
            AdapterErrorCode::TransitionDenied,
            "Choregraph denied the exact workflow approval suspension transition",
        )),
    }
}

fn event_to_core(
    event: &crate::workflow::gate::model::ApprovalGraphEvent,
) -> Result<choregraph_core::Event, AdapterError> {
    let node = node_reference(event.node_id);
    debug_assert!(!node.is_empty());
    debug_assert!(node.starts_with(CHOREGRAPH_NODE_PREFIX));
    let action = match event.action {
        crate::workflow::gate::model::ApprovalGraphAction::MarkReady => choregraph_core::Action::MarkReady,
        crate::workflow::gate::model::ApprovalGraphAction::Start => choregraph_core::Action::Start,
        crate::workflow::gate::model::ApprovalGraphAction::Complete => choregraph_core::Action::Complete,
        crate::workflow::gate::model::ApprovalGraphAction::Fail => choregraph_core::Action::Fail,
        crate::workflow::gate::model::ApprovalGraphAction::Cancel => choregraph_core::Action::Cancel,
        crate::workflow::gate::model::ApprovalGraphAction::Suspend => choregraph_core::Action::Suspend,
        crate::workflow::gate::model::ApprovalGraphAction::ResumeSignal => choregraph_core::Action::ResumeSignal,
    };
    if matches!(
        event.action,
        crate::workflow::gate::model::ApprovalGraphAction::Suspend
            | crate::workflow::gate::model::ApprovalGraphAction::ResumeSignal
    ) {
        let class = event
            .suspension_class
            .as_deref()
            .ok_or_else(|| error(AdapterErrorCode::StateMismatch, "approval suspension event is missing its class"))?;
        let reference = event.suspension_reference.as_deref().ok_or_else(|| {
            error(AdapterErrorCode::StateMismatch, "approval suspension event is missing its reference")
        })?;
        Ok(choregraph_core::Event::suspension(node, action, class, reference))
    } else {
        if event.suspension_class.is_some() || event.suspension_reference.is_some() {
            return Err(error(
                AdapterErrorCode::StateMismatch,
                "non-suspension approval event contains suspension facts",
            ));
        }
        Ok(choregraph_core::Event::new(node, action))
    }
}

fn machine_node_id(
    machine: &choregraph_core::AdmittedMachine,
    node_id: crate::engine::workflow::NodeId,
) -> Result<choregraph_core::NodeId, AdapterError> {
    let expected = node_reference(node_id);
    machine
        .node_ids()
        .iter()
        .find(|candidate| candidate.as_str() == expected)
        .cloned()
        .ok_or_else(|| error(AdapterErrorCode::UnknownNode, "workflow approval node is missing from Choregraph"))
}

fn require_profile(request: &crate::workflow::gate::model::ApprovalRequest) -> Result<(), AdapterError> {
    if request.choregraph_profile != crate::workflow::gate::model::CHOREGRAPH_SUSPENSION_PROFILE
        || request.suspension_class != crate::workflow::gate::model::CHOREGRAPH_SUSPENSION_CLASS
    {
        return Err(error(
            AdapterErrorCode::ProfileMismatch,
            "workflow approval request does not bind the pinned Choregraph suspension profile",
        ));
    }
    Ok(())
}

fn event(
    node_id: crate::engine::workflow::NodeId,
    action: crate::workflow::gate::model::ApprovalGraphAction,
) -> crate::workflow::gate::model::ApprovalGraphEvent {
    crate::workflow::gate::model::ApprovalGraphEvent {
        node_id,
        action,
        suspension_class: None,
        suspension_reference: None,
    }
}

fn node_reference(node_id: crate::engine::workflow::NodeId) -> String {
    format!("{CHOREGRAPH_NODE_PREFIX}{}", node_id.0)
}

fn error(code: AdapterErrorCode, message: impl Into<String>) -> AdapterError {
    AdapterError {
        code,
        message: message.into(),
    }
}

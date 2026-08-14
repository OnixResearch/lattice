struct AuthorityIdentityInput<'a> {
    request: &'a crate::workflow::gate::model::ApprovalRequest,
    actor: &'a crate::workflow::gate::identity::ActorId,
    ability: crate::workflow::gate::model::DecisionAbility,
    observed_epoch_millis: i64,
    profile: crate::workflow::gate::model::AuthorityProfile,
    verification_receipt_references: &'a [String],
}

pub fn delegated_authority_input_id(
    request: &crate::workflow::gate::model::ApprovalRequest,
    actor: &crate::workflow::gate::identity::ActorId,
    ability: crate::workflow::gate::model::DecisionAbility,
    observed_epoch_millis: i64,
    verification_receipt_references: &[String],
) -> Result<crate::workflow::gate::identity::ApprovalAuthorityInputId, AdmissionError> {
    authority_input_identity(AuthorityIdentityInput {
        request,
        actor,
        ability,
        observed_epoch_millis,
        profile: crate::workflow::gate::model::AuthorityProfile::BasaltUcanVerified,
        verification_receipt_references,
    })
}

fn authority_input_identity(
    input: AuthorityIdentityInput<'_>,
) -> Result<crate::workflow::gate::identity::ApprovalAuthorityInputId, AdmissionError> {
    let mut receipt_refs = input.verification_receipt_references.to_vec();
    receipt_refs.sort();
    let material = AuthorityInputMaterial {
        request_id: &input.request.id,
        workflow_revision: &input.request.workflow_revision,
        node_id: input.request.node_id,
        actor: input.actor,
        ability: input.ability,
        authority_policy_id: &input.request.authority_policy_id,
        observed_epoch_millis: input.observed_epoch_millis,
        profile: input.profile,
        verification_receipt_references: &receipt_refs,
    };
    Ok(crate::workflow::gate::identity::ApprovalAuthorityInputId::from_generated(hash(
        AUTHORITY_INPUT_HASH_DOMAIN,
        &material,
    )?))
}

fn validate_delegated_facts(
    request: &crate::workflow::gate::model::ApprovalRequest,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
    command: &crate::workflow::gate::model::DecisionCommand,
    ability: crate::workflow::gate::model::DecisionAbility,
    facts: &crate::workflow::gate::model::VerifiedDelegatedAuthorityFacts,
) -> Result<(), AdmissionError> {
    let expected_policy = config.authority.delegated_policy_reference.as_deref();
    if facts.request_id != request.id || facts.workflow_revision != request.workflow_revision {
        return Err(error(AdmissionCode::AuthorityDenied, "verified authority targets another request revision"));
    }
    if facts.node_id != request.node_id || facts.actor != command.actor {
        return Err(error(AdmissionCode::AuthorityDenied, "verified authority targets another node or actor"));
    }
    if facts.ability != ability || Some(facts.policy_reference.as_str()) != expected_policy {
        return Err(error(AdmissionCode::AuthorityDenied, "verified authority targets another ability or policy"));
    }
    if facts.verification_receipt_references.is_empty() || has_duplicates(&facts.verification_receipt_references) {
        return Err(error(AdmissionCode::AuthorityDenied, "verified authority receipts are missing or duplicated"));
    }
    if facts.verification_receipt_references.iter().any(|reference| !is_valid_reference(reference)) {
        return Err(error(AdmissionCode::AuthorityDenied, "verified authority receipt reference is invalid"));
    }
    if !facts.caveats_allow || !facts.replay_allow {
        return Err(error(AdmissionCode::AuthorityDenied, "verified authority caveat or replay policy denied"));
    }
    if !facts.current {
        return Err(error(AdmissionCode::AuthorityDenied, "verified authority facts are not current"));
    }
    debug_assert_eq!(facts.request_id, request.id);
    debug_assert_eq!(facts.actor, command.actor);
    Ok(())
}

fn validate_reason(
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
    reason: Option<&str>,
) -> Result<Option<String>, AdmissionError> {
    let normalized = reason.filter(|value| !value.is_empty());
    if config.reason.required && normalized.is_none() {
        return Err(error(AdmissionCode::ReasonDenied, "approval reason is required by policy"));
    }
    if normalized.is_some_and(|value| value.len() > config.reason.max_bytes || value.chars().any(char::is_control)) {
        return Err(error(AdmissionCode::ReasonDenied, "approval reason violates the bounded text policy"));
    }
    if config.reason.identity_bound {
        normalized.map(|value| hash(REASON_HASH_DOMAIN, &value)).transpose()
    } else {
        Ok(None)
    }
}

fn require_request_binding(
    request: &crate::workflow::gate::model::ApprovalRequest,
    request_id: &str,
    workflow_revision: &crate::orchestration::WorkflowRevision,
) -> Result<(), AdmissionError> {
    if request.id.as_str() != request_id {
        return Err(error(AdmissionCode::RequestMismatch, "approval request identity does not match"));
    }
    if &request.workflow_revision != workflow_revision {
        return Err(error(
            AdmissionCode::WorkflowRevisionMismatch,
            "approval workflow revision does not match current persisted state",
        ));
    }
    Ok(())
}

fn selected_input_facts(
    workflow: &crate::engine::workflow::Workflow,
    node_id: crate::engine::workflow::NodeId,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
    inputs: &std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::value::Value>,
) -> Result<Vec<crate::workflow::gate::model::SelectedInputFact>, AdmissionError> {
    let node = workflow
        .graph
        .node(node_id)
        .ok_or_else(|| error(AdmissionCode::IdentityFailure, "approval node is missing from workflow graph"))?;
    let by_label = node
        .input_ports
        .iter()
        .map(|port| (port.label.as_str(), port))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut facts = Vec::with_capacity(config.selected_input_ports.len());
    for label in &config.selected_input_ports {
        let port = by_label.get(label.as_str()).ok_or_else(|| {
            error(AdmissionCode::RequestMismatch, "approval selected input port is not declared by the node")
        })?;
        let value = inputs
            .get(&port.id)
            .ok_or_else(|| error(AdmissionCode::RequestMismatch, "approval selected input value is not available"))?;
        facts.push(crate::workflow::gate::model::SelectedInputFact {
            port_id: port.id,
            port_label: port.label.clone(),
            value_hash: crate::workflow::gate::identity::ApprovalInputId::from_generated(value.hash.to_hex()),
            type_tag: value.type_tag().as_str().to_owned(),
        });
    }
    facts.sort_by_key(|fact| fact.port_id);
    debug_assert_eq!(facts.len(), config.selected_input_ports.len());
    debug_assert!(facts.windows(2).all(|pair| pair[0].port_id < pair[1].port_id));
    Ok(facts)
}

fn validate_selected_input_checkpoint(
    request: &crate::workflow::gate::model::ApprovalRequest,
    workflow: &crate::engine::workflow::Workflow,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
) -> Result<(), AdmissionError> {
    let node = workflow.graph.node(request.node_id).ok_or_else(|| {
        error(AdmissionCode::UnsafeCheckpoint, "approval gate is missing from the checkpoint workflow")
    })?;
    if request.selected_inputs.len() != config.selected_input_ports.len() {
        return Err(error(AdmissionCode::UnsafeCheckpoint, "approval selected-input checkpoint count changed"));
    }
    let outputs = request
        .completed_nodes
        .iter()
        .flat_map(|node| node.outputs.iter())
        .map(|(port_id, value)| (*port_id, value))
        .collect::<std::collections::BTreeMap<_, _>>();
    for fact in &request.selected_inputs {
        let port = node
            .input_ports
            .iter()
            .find(|port| port.id == fact.port_id && port.label == fact.port_label)
            .ok_or_else(|| error(AdmissionCode::UnsafeCheckpoint, "approval selected-input port changed"))?;
        if !config.selected_input_ports.contains(&port.label) {
            return Err(error(AdmissionCode::UnsafeCheckpoint, "approval selected-input configuration changed"));
        }
        if let Some(edge) = workflow.graph.edges().iter().find(|edge| edge.target == port.id) {
            let value = outputs.get(&edge.source).ok_or_else(|| {
                error(AdmissionCode::UnsafeCheckpoint, "approval selected-input source output is missing")
            })?;
            if fact.value_hash.as_str() != value.hash.to_hex() || fact.type_tag != value.type_tag().as_str() {
                return Err(error(AdmissionCode::UnsafeCheckpoint, "approval selected-input identity changed"));
            }
        }
    }
    debug_assert_eq!(request.selected_inputs.len(), config.selected_input_ports.len());
    debug_assert!(request.selected_inputs.iter().all(|fact| config.selected_input_ports.contains(&fact.port_label)));
    Ok(())
}

fn validate_checkpoint(nodes: &[crate::workflow::gate::model::PersistedNodeOutcome]) -> Result<(), AdmissionError> {
    if nodes.len() > MAX_CHECKPOINT_NODES {
        return Err(error(AdmissionCode::UnsafeCheckpoint, "approval checkpoint node count exceeds its bound"));
    }
    let mut node_ids = std::collections::BTreeSet::new();
    for node in nodes {
        if !node_ids.insert(node.node_id) {
            return Err(error(AdmissionCode::UnsafeCheckpoint, "approval checkpoint repeats a node identity"));
        }
        if node.outputs.values().any(|value| !value.sensitive_fragments.is_empty()) {
            return Err(error(
                AdmissionCode::UnsafeCheckpoint,
                "approval checkpoint cannot persist resolved secret values",
            ));
        }
    }
    debug_assert_eq!(node_ids.len(), nodes.len());
    debug_assert!(nodes.len() <= MAX_CHECKPOINT_NODES);
    Ok(())
}

fn validate_graph_checkpoint(
    events: &[crate::workflow::gate::model::ApprovalGraphEvent],
) -> Result<(), AdmissionError> {
    if events.len() > MAX_GRAPH_EVENTS {
        return Err(error(AdmissionCode::UnsafeCheckpoint, "approval graph event count exceeds its bound"));
    }
    debug_assert!(events.len() <= MAX_GRAPH_EVENTS);
    const { assert!(MAX_GRAPH_EVENTS >= MAX_CHECKPOINT_NODES) };
    Ok(())
}

fn approval_config(
    workflow: &crate::engine::workflow::Workflow,
    node_id: crate::engine::workflow::NodeId,
) -> Result<&crate::workflow::gate::model::ApprovalNodeConfig, AdmissionError> {
    let config = workflow
        .node_config(node_id)
        .ok_or_else(|| error(AdmissionCode::RequestMismatch, "workflow node is missing its configuration"))?;
    if let crate::engine::workflow::NodeConfig::Approval(config) = config {
        return Ok(config);
    }
    Err(error(AdmissionCode::RequestMismatch, "workflow node is not an approval gate"))
}

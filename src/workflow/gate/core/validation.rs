impl crate::workflow::gate::model::DecisionAbility {
    const VARIANT_COUNT: usize = 2;
}

fn policy_identities(
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
) -> Result<
    (
        crate::workflow::gate::identity::ApprovalAuthorityPolicyId,
        crate::workflow::gate::identity::ApprovalExpiryPolicyId,
        crate::workflow::gate::identity::ApprovalResumeSchemaId,
    ),
    AdmissionError,
> {
    let identities = (
        crate::workflow::gate::identity::ApprovalAuthorityPolicyId::from_generated(hash(
            POLICY_HASH_DOMAIN,
            &config.authority,
        )?),
        crate::workflow::gate::identity::ApprovalExpiryPolicyId::from_generated(hash(
            EXPIRY_HASH_DOMAIN,
            &config.expiry,
        )?),
        crate::workflow::gate::identity::ApprovalResumeSchemaId::from_generated(hash(
            RESUME_SCHEMA_HASH_DOMAIN,
            &config.resume,
        )?),
    );
    debug_assert!(!identities.0.as_str().is_empty());
    debug_assert!(!identities.2.as_str().is_empty());
    Ok(identities)
}

pub fn validate_request_integrity(
    request: &crate::workflow::gate::model::ApprovalRequest,
    workflow: &crate::engine::workflow::Workflow,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
) -> Result<(), AdmissionError> {
    let validation = validate_node_config(config);
    if !validation.is_empty() {
        return Err(error(AdmissionCode::IdentityFailure, "approval node configuration is not admitted"));
    }
    validate_checkpoint(&request.completed_nodes)?;
    validate_graph_checkpoint(&request.graph_events)?;
    validate_selected_input_checkpoint(request, workflow, config)?;
    let revision = crate::orchestration::workflow_revision(workflow)
        .map_err(|_| error(AdmissionCode::IdentityFailure, "workflow revision identity failed"))?;
    let (authority_policy_id, expiry_policy_id, resume_schema_id) = policy_identities(config)?;
    let has_configuration_changed = [
        request.workflow_name != workflow.meta.name,
        request.workflow_revision != revision.content_hash,
        request.gate_id != config.gate_id,
        request.required_artifacts != config.required_artifacts,
        request.required_non_claims != config.required_non_claims,
        request.authority_policy_id != authority_policy_id,
        request.expiry_policy_id != expiry_policy_id,
        request.resume_schema_id != resume_schema_id,
        request.choregraph_profile != crate::workflow::gate::model::CHOREGRAPH_SUSPENSION_PROFILE,
        request.suspension_class != crate::workflow::gate::model::CHOREGRAPH_SUSPENSION_CLASS,
        request.suspension_reference != format!("blake3:{}", request.id),
    ]
    .into_iter()
    .any(std::convert::identity);
    if has_configuration_changed {
        return Err(error(
            AdmissionCode::IdentityFailure,
            "approval request does not match the current typed configuration",
        ));
    }
    validate_decision_integrity(request, config)?;
    let material = RequestIdentityMaterial {
        run_id: request.run_id,
        workflow_name: &request.workflow_name,
        workflow_revision: &request.workflow_revision,
        node_id: request.node_id,
        gate_id: &request.gate_id,
        requester: &request.requester,
        selected_inputs: &request.selected_inputs,
        required_artifacts: &request.required_artifacts,
        authority_policy_id: &request.authority_policy_id,
        expiry_policy_id: &request.expiry_policy_id,
        resume_schema_id: &request.resume_schema_id,
        requested_epoch_millis: request.requested_epoch_millis,
        decision_deadline_epoch_millis: request.decision_deadline_epoch_millis,
    };
    let expected_request_id =
        crate::workflow::gate::identity::ApprovalRequestId::from_generated(hash(REQUEST_HASH_DOMAIN, &material)?);
    if expected_request_id != request.id {
        return Err(error(AdmissionCode::IdentityFailure, "approval request identity is inconsistent"));
    }
    debug_assert_eq!(request.gate_id, config.gate_id);
    debug_assert_eq!(request.workflow_revision, revision.content_hash);
    validate_event_chain(request)
}

// r[impl lattice.workflow_approval.node]
// r[impl lattice.workflow_approval.node.valid]
// r[impl lattice.workflow_approval.node.invalid]
pub fn build_request(
    input: RequestBuildInput<'_>,
) -> Result<crate::workflow::gate::model::ApprovalRequest, AdmissionError> {
    let config = approval_config(input.workflow, input.node_id)?;
    let validation = validate_node_config(config);
    if !validation.is_empty() {
        return Err(error(AdmissionCode::IdentityFailure, "approval node configuration is not admitted"));
    }
    validate_checkpoint(&input.completed_nodes)?;
    validate_graph_checkpoint(&input.graph_events)?;
    let revision = crate::orchestration::workflow_revision(input.workflow)
        .map_err(|_| error(AdmissionCode::IdentityFailure, "workflow revision identity failed"))?;
    let selected_inputs = selected_input_facts(input.workflow, input.node_id, config, input.gate_inputs)?;
    let (authority_policy_id, expiry_policy_id, resume_schema_id) = policy_identities(config)?;
    let decision_deadline_epoch_millis = deadline(input.observed_epoch_millis, config.expiry.decision_ttl_millis)?;
    let material = RequestIdentityMaterial {
        run_id: input.run_id,
        workflow_name: &input.workflow.meta.name,
        workflow_revision: &revision.content_hash,
        node_id: input.node_id,
        gate_id: &config.gate_id,
        requester: &input.requester,
        selected_inputs: &selected_inputs,
        required_artifacts: &config.required_artifacts,
        authority_policy_id: &authority_policy_id,
        expiry_policy_id: &expiry_policy_id,
        resume_schema_id: &resume_schema_id,
        requested_epoch_millis: input.observed_epoch_millis,
        decision_deadline_epoch_millis,
    };
    let request_id =
        crate::workflow::gate::identity::ApprovalRequestId::from_generated(hash(REQUEST_HASH_DOMAIN, &material)?);
    let suspension_reference = format!("blake3:{request_id}");
    let request = crate::workflow::gate::model::ApprovalRequest {
        id: request_id,
        run_id: input.run_id,
        workflow_name: input.workflow.meta.name.clone(),
        workflow_revision: revision.content_hash,
        node_id: input.node_id,
        gate_id: config.gate_id.clone(),
        requester: input.requester,
        safe_summary: config.safe_summary.clone(),
        selected_inputs,
        required_artifacts: config.required_artifacts.clone(),
        authority_policy_id,
        expiry_policy_id,
        resume_schema_id,
        required_non_claims: config.required_non_claims.clone(),
        requested_epoch_millis: input.observed_epoch_millis,
        decision_deadline_epoch_millis,
        status: crate::workflow::gate::model::ApprovalRequestStatus::Suspended,
        decision: None,
        active_resume_owner: None,
        completed_resume_id: None,
        completed_nodes: input.completed_nodes,
        graph_events: input.graph_events,
        events: Vec::new(),
        choregraph_profile: crate::workflow::gate::model::CHOREGRAPH_SUSPENSION_PROFILE.to_owned(),
        suspension_class: crate::workflow::gate::model::CHOREGRAPH_SUSPENSION_CLASS.to_owned(),
        suspension_reference,
        version: REQUEST_VERSION_INITIAL,
    };
    debug_assert_eq!(request.status, crate::workflow::gate::model::ApprovalRequestStatus::Suspended);
    debug_assert_eq!(request.version, REQUEST_VERSION_INITIAL);
    Ok(request)
}

pub fn record_suspension(
    request: &crate::workflow::gate::model::ApprovalRequest,
) -> Result<crate::workflow::gate::model::ApprovalRequest, AdmissionError> {
    if request.status != crate::workflow::gate::model::ApprovalRequestStatus::Suspended || !request.events.is_empty() {
        return Err(error(AdmissionCode::WrongStatus, "approval suspension can only seal a new suspended request"));
    }
    let mut updated = request.clone();
    append_event(&mut updated, crate::workflow::gate::model::ApprovalEventKind::Suspended)?;
    Ok(updated)
}

// r[impl lattice.workflow_approval.decision]
// r[impl lattice.workflow_approval.decision.approve]
// r[impl lattice.workflow_approval.decision.reject]
// r[impl lattice.workflow_approval.authority]
// r[impl lattice.workflow_approval.authority.valid]
// r[impl lattice.workflow_approval.authority.denied]
pub fn plan_decision(
    request: &crate::workflow::gate::model::ApprovalRequest,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
    command: &crate::workflow::gate::model::DecisionCommand,
) -> Result<DecisionPlan, AdmissionError> {
    require_request_binding(request, command.request_id.as_str(), &command.expected_workflow_revision)?;
    if command.observed_epoch_millis > request.decision_deadline_epoch_millis {
        let mut expired = request.clone();
        if matches!(expired.status, crate::workflow::gate::model::ApprovalRequestStatus::Suspended) {
            expired.status = crate::workflow::gate::model::ApprovalRequestStatus::Expired;
            increment_version(&mut expired)?;
            append_event(&mut expired, crate::workflow::gate::model::ApprovalEventKind::Expired)?;
            return Ok(DecisionPlan {
                request: expired,
                outcome: crate::workflow::gate::model::DecisionOutcome::Expired {
                    terminal: config.terminal.on_expiry,
                },
            });
        }
        return Err(error(AdmissionCode::Expired, "approval request is expired"));
    }
    let ability = crate::workflow::gate::model::DecisionAbility::for_decision(command.decision);
    if !config.authority.accepted_abilities.contains(&ability) {
        return Err(error(AdmissionCode::AuthorityDenied, "approval ability is not admitted by current policy"));
    }
    let decision = make_decision(request, config, command, ability)?;
    let decision_id = decision.id.clone();
    if let Some(existing) = &request.decision {
        if existing.id == decision_id {
            return Ok(DecisionPlan {
                request: request.clone(),
                outcome: crate::workflow::gate::model::DecisionOutcome::Idempotent { decision_id },
            });
        }
        return Err(error(AdmissionCode::ConflictingDecision, "approval request already has a different decision"));
    }
    if request.status != crate::workflow::gate::model::ApprovalRequestStatus::Suspended {
        return Err(error(AdmissionCode::WrongStatus, "approval decision requires suspended request state"));
    }
    let mut updated = request.clone();
    updated.decision = Some(decision);
    updated.status = match command.decision {
        crate::workflow::gate::model::DecisionClass::Approve => {
            crate::workflow::gate::model::ApprovalRequestStatus::Approved
        }
        crate::workflow::gate::model::DecisionClass::Reject => {
            crate::workflow::gate::model::ApprovalRequestStatus::Rejected
        }
    };
    increment_version(&mut updated)?;
    append_event(&mut updated, match command.decision {
        crate::workflow::gate::model::DecisionClass::Approve => {
            crate::workflow::gate::model::ApprovalEventKind::Approved
        }
        crate::workflow::gate::model::DecisionClass::Reject => {
            crate::workflow::gate::model::ApprovalEventKind::Rejected
        }
    })?;
    debug_assert!(updated.decision.is_some());
    debug_assert!(matches!(
        updated.status,
        crate::workflow::gate::model::ApprovalRequestStatus::Approved
            | crate::workflow::gate::model::ApprovalRequestStatus::Rejected
    ));
    Ok(DecisionPlan {
        request: updated,
        outcome: crate::workflow::gate::model::DecisionOutcome::Recorded { decision_id },
    })
}

fn make_decision(
    request: &crate::workflow::gate::model::ApprovalRequest,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
    command: &crate::workflow::gate::model::DecisionCommand,
    ability: crate::workflow::gate::model::DecisionAbility,
) -> Result<crate::workflow::gate::model::ApprovalDecision, AdmissionError> {
    let authority_input_id = authority_input(request, config, command, ability)?;
    let (authority_profile, delegated_policy_reference, verification_receipt_references) =
        persisted_authority_references(&command.evidence);
    let reason_identity = validate_reason(config, command.reason.as_deref())?;
    let material = DecisionIdentityMaterial {
        request_id: &request.id,
        workflow_revision: &request.workflow_revision,
        actor: &command.actor,
        decision: command.decision,
        ability,
        reason_identity: &reason_identity,
        decided_epoch_millis: command.observed_epoch_millis,
        authority_input_id: &authority_input_id,
    };
    let id =
        crate::workflow::gate::identity::ApprovalDecisionId::from_generated(hash(DECISION_HASH_DOMAIN, &material)?);
    debug_assert_eq!(ability, crate::workflow::gate::model::DecisionAbility::for_decision(command.decision));
    debug_assert!(!authority_input_id.as_str().is_empty());
    Ok(crate::workflow::gate::model::ApprovalDecision {
        id,
        actor: command.actor.clone(),
        decision: command.decision,
        ability,
        reason_identity,
        decided_epoch_millis: command.observed_epoch_millis,
        authority_input_id,
        authority_profile,
        delegated_policy_reference,
        verification_receipt_references,
    })
}

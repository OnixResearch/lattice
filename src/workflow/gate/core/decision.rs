// r[impl lattice.workflow_approval.resume]
// r[impl lattice.workflow_approval.resume.valid]
// r[impl lattice.workflow_approval.resume.inputs]
// r[impl lattice.workflow_approval.resume.denied]
// r[impl lattice.workflow_approval.terminal.duplicate]
pub fn plan_resume_claim(
    request: &crate::workflow::gate::model::ApprovalRequest,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
    command: &crate::workflow::gate::model::ResumeCommand,
) -> Result<ResumeClaimPlan, AdmissionError> {
    require_request_binding(request, command.request_id.as_str(), &command.expected_workflow_revision)?;
    if request.status == crate::workflow::gate::model::ApprovalRequestStatus::Resumed {
        let resume_id = request.completed_resume_id.clone().ok_or_else(|| {
            error(AdmissionCode::IdentityFailure, "resumed approval request is missing resume identity")
        })?;
        let plan = resume_plan(request, command, config)?;
        if plan.id != resume_id {
            return Err(error(
                AdmissionCode::ConflictingDecision,
                "completed approval resume facts do not match the repeated command",
            ));
        }
        return Ok(ResumeClaimPlan {
            request: request.clone(),
            plan,
            outcome: crate::workflow::gate::model::ResumeOutcome::Idempotent { resume_id },
        });
    }
    let expected_head = request
        .events
        .last()
        .map(|event| &event.id)
        .ok_or_else(|| error(AdmissionCode::EventHeadMismatch, "approval request has no event-chain head"))?;
    if expected_head != &command.expected_event_head {
        return Err(error(AdmissionCode::EventHeadMismatch, "approval event-chain head changed"));
    }
    if request.status == crate::workflow::gate::model::ApprovalRequestStatus::Resuming
        || request.status == crate::workflow::gate::model::ApprovalRequestStatus::ResumeUncertain
    {
        return Err(error(
            AdmissionCode::ResumeUncertain,
            "approval resume outcome is active or uncertain and cannot be retried automatically",
        ));
    }
    if request.status != crate::workflow::gate::model::ApprovalRequestStatus::Approved {
        return Err(error(AdmissionCode::WrongStatus, "workflow resume requires one accepted approval decision"));
    }
    if request.active_resume_owner.is_some() {
        return Err(error(AdmissionCode::ActiveOwner, "approval request has another active resume owner"));
    }
    validate_resume_expiry(request, config, command.observed_epoch_millis)?;
    let plan = resume_plan(request, command, config)?;
    let mut updated = request.clone();
    updated.status = crate::workflow::gate::model::ApprovalRequestStatus::Resuming;
    updated.active_resume_owner = Some(command.owner.clone());
    increment_version(&mut updated)?;
    append_event(&mut updated, crate::workflow::gate::model::ApprovalEventKind::ResumeClaimed)?;
    debug_assert_eq!(updated.status, crate::workflow::gate::model::ApprovalRequestStatus::Resuming);
    debug_assert_eq!(updated.active_resume_owner.as_ref(), Some(&command.owner));
    Ok(ResumeClaimPlan {
        request: updated,
        outcome: crate::workflow::gate::model::ResumeOutcome::Claimed {
            resume_id: plan.id.clone(),
        },
        plan,
    })
}

// r[impl lattice.workflow_approval.terminal]
// r[impl lattice.workflow_approval.terminal.expired]
pub fn expire_before_resume(
    request: &crate::workflow::gate::model::ApprovalRequest,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
    observed_epoch_millis: i64,
) -> Result<crate::workflow::gate::model::ApprovalRequest, AdmissionError> {
    if request.status != crate::workflow::gate::model::ApprovalRequestStatus::Approved {
        return Err(error(AdmissionCode::WrongStatus, "resume expiry requires an approved request"));
    }
    if validate_resume_expiry(request, config, observed_epoch_millis).is_ok() {
        return Err(error(AdmissionCode::WrongStatus, "approval resume window is not expired"));
    }
    let mut updated = request.clone();
    updated.status = crate::workflow::gate::model::ApprovalRequestStatus::Expired;
    increment_version(&mut updated)?;
    append_event(&mut updated, crate::workflow::gate::model::ApprovalEventKind::Expired)?;
    Ok(updated)
}

pub fn record_resume_signal(
    request: &crate::workflow::gate::model::ApprovalRequest,
    owner: &crate::workflow::gate::identity::ResumeOwnerId,
) -> Result<crate::workflow::gate::model::ApprovalRequest, AdmissionError> {
    require_active_resume_owner(request, owner)?;
    let mut updated = request.clone();
    increment_version(&mut updated)?;
    append_event(&mut updated, crate::workflow::gate::model::ApprovalEventKind::ResumeSignalled)?;
    Ok(updated)
}

pub fn complete_resume(
    request: &crate::workflow::gate::model::ApprovalRequest,
    owner: &crate::workflow::gate::identity::ResumeOwnerId,
    resume_id: crate::workflow::gate::identity::ApprovalResumeId,
) -> Result<crate::workflow::gate::model::ApprovalRequest, AdmissionError> {
    require_active_resume_owner(request, owner)?;
    let mut updated = request.clone();
    updated.status = crate::workflow::gate::model::ApprovalRequestStatus::Resumed;
    updated.active_resume_owner = None;
    updated.completed_resume_id = Some(resume_id);
    increment_version(&mut updated)?;
    append_event(&mut updated, crate::workflow::gate::model::ApprovalEventKind::Resumed)?;
    Ok(updated)
}

pub fn mark_resume_uncertain(
    request: &crate::workflow::gate::model::ApprovalRequest,
    owner: &crate::workflow::gate::identity::ResumeOwnerId,
) -> Result<crate::workflow::gate::model::ApprovalRequest, AdmissionError> {
    require_active_resume_owner(request, owner)?;
    let mut updated = request.clone();
    updated.status = crate::workflow::gate::model::ApprovalRequestStatus::ResumeUncertain;
    increment_version(&mut updated)?;
    append_event(&mut updated, crate::workflow::gate::model::ApprovalEventKind::ResumeUncertain)?;
    Ok(updated)
}

fn receipt_event_head(
    request: &crate::workflow::gate::model::ApprovalRequest,
) -> Result<crate::workflow::gate::identity::ApprovalEventId, AdmissionError> {
    let event_head = request
        .events
        .last()
        .map(|event| event.id.clone())
        .ok_or_else(|| error(AdmissionCode::EventHeadMismatch, "approval receipt requires an event-chain head"))?;
    debug_assert!(!request.events.is_empty());
    debug_assert_eq!(request.events.last().map(|event| &event.id), Some(&event_head));
    Ok(event_head)
}

pub fn approval_receipt(
    request: &crate::workflow::gate::model::ApprovalRequest,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
) -> Result<crate::workflow::gate::model::ApprovalReceipt, AdmissionError> {
    let authority_policy_id = crate::workflow::gate::identity::ApprovalAuthorityPolicyId::from_generated(hash(
        POLICY_HASH_DOMAIN,
        &config.authority,
    )?);
    let expiry_policy_id = crate::workflow::gate::identity::ApprovalExpiryPolicyId::from_generated(hash(
        EXPIRY_HASH_DOMAIN,
        &config.expiry,
    )?);
    if authority_policy_id != request.authority_policy_id || expiry_policy_id != request.expiry_policy_id {
        return Err(error(
            AdmissionCode::AuthorityDenied,
            "approval receipt policy does not match the persisted request",
        ));
    }
    debug_assert_eq!(authority_policy_id, request.authority_policy_id);
    debug_assert_eq!(expiry_policy_id, request.expiry_policy_id);
    let event_head = receipt_event_head(request)?;
    let delegated_policy_reference = request
        .decision
        .as_ref()
        .and_then(|decision| decision.delegated_policy_reference.clone())
        .or_else(|| config.authority.delegated_policy_reference.clone());
    let verification_receipt_references = request
        .decision
        .as_ref()
        .map(|decision| decision.verification_receipt_references.clone())
        .unwrap_or_default();
    let material = ReceiptIdentityMaterial {
        request_id: &request.id,
        run_id: request.run_id,
        workflow_revision: &request.workflow_revision,
        node_id: request.node_id,
        gate_id: &request.gate_id,
        status: request.status,
        authority_profile: config.authority.profile,
        authority_policy_id: &request.authority_policy_id,
        expiry_policy_id: &request.expiry_policy_id,
        delegated_policy_reference: &delegated_policy_reference,
        verification_receipt_references: &verification_receipt_references,
        clock_source: config.expiry.clock_source,
        event_head: &event_head,
        event_count: request.events.len(),
        choregraph_profile: &request.choregraph_profile,
        suspension_class: &request.suspension_class,
        required_non_claims: &request.required_non_claims,
    };
    Ok(crate::workflow::gate::model::ApprovalReceipt {
        id: crate::workflow::gate::identity::ApprovalReceiptId::from_generated(hash(RECEIPT_HASH_DOMAIN, &material)?),
        request_id: request.id.clone(),
        run_id: request.run_id,
        workflow_revision: request.workflow_revision.clone(),
        node_id: request.node_id,
        gate_id: request.gate_id.clone(),
        status: request.status,
        authority_profile: config.authority.profile,
        authority_policy_id: request.authority_policy_id.clone(),
        expiry_policy_id: request.expiry_policy_id.clone(),
        delegated_policy_reference,
        verification_receipt_references,
        clock_source: config.expiry.clock_source,
        event_head,
        event_count: request.events.len(),
        choregraph_profile: request.choregraph_profile.clone(),
        suspension_class: request.suspension_class.clone(),
        required_non_claims: request.required_non_claims.clone(),
    })
}

// r[impl lattice.workflow_approval.terminal]
pub fn terminal_outcome(
    request: &crate::workflow::gate::model::ApprovalRequest,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
) -> Option<crate::workflow::gate::model::TerminalOutcome> {
    match request.status {
        crate::workflow::gate::model::ApprovalRequestStatus::Rejected => Some(config.terminal.on_rejection),
        crate::workflow::gate::model::ApprovalRequestStatus::Expired => Some(config.terminal.on_expiry),
        crate::workflow::gate::model::ApprovalRequestStatus::Suspended
        | crate::workflow::gate::model::ApprovalRequestStatus::Approved
        | crate::workflow::gate::model::ApprovalRequestStatus::Resuming
        | crate::workflow::gate::model::ApprovalRequestStatus::Resumed
        | crate::workflow::gate::model::ApprovalRequestStatus::ResumeUncertain => None,
    }
}

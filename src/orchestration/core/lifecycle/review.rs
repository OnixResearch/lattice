pub(in crate::orchestration::core) fn submit_proposal(
    instance: &mut super::super::Instance,
    proposal: &super::super::RevisionProposal,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    if instance.status != super::super::Status::WaitingForApproval || instance.pending_proposal.is_some() {
        return Err(super::super::model::failures::invalid_transition(
            "a proposal requires an empty waiting-for-approval state",
        ));
    }
    validate_proposal(instance, proposal)?;
    instance.pending_proposal = Some(proposal.clone());
    Ok(super::super::EventKind::ProposalSubmitted)
}

pub(in crate::orchestration::core) fn decide_proposal(
    instance: &mut super::super::Instance,
    approval: &super::super::ApprovalDecision,
    is_successor_validated: bool,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    if instance.status != super::super::Status::WaitingForApproval {
        return Err(super::super::model::failures::invalid_transition(
            "proposal decisions require waiting-for-approval state",
        ));
    }
    let proposal = instance.pending_proposal.clone().ok_or_else(|| {
        super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::ProposalMismatch,
            "no proposal is pending for approval",
        )
    })?;
    validate_approval(instance, &proposal, approval)?;
    debug_assert_eq!(approval.proposal_id, proposal.id);
    debug_assert_eq!(approval.epoch, instance.epoch);
    if !approval.approved {
        instance.pending_proposal = None;
        return Ok(super::super::EventKind::ProposalRejected);
    }
    if !is_successor_validated {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::SuccessorValidationFailed,
            "approval cannot bypass successor workflow validation",
        ));
    }
    instance.workflow_revision = proposal.successor;
    instance.pending_proposal = None;
    instance.epoch = instance.epoch.saturating_add(super::super::EPOCH_INCREMENT);
    instance.status = super::super::Status::Ready;
    Ok(super::super::EventKind::ProposalApproved)
}

pub(in crate::orchestration::core) fn reject_proposal(
    instance: &mut super::super::Instance,
    actor: &str,
    proposal_id: &crate::orchestration::identity::ProposalId,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    authorize_actor(instance, actor)?;
    if instance.status != super::super::Status::WaitingForApproval {
        return Err(super::super::model::failures::invalid_transition(
            "proposal rejection requires waiting-for-approval state",
        ));
    }
    let proposal = instance.pending_proposal.as_ref().ok_or_else(|| {
        super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::ProposalMismatch,
            "no proposal is pending for rejection",
        )
    })?;
    if &proposal.id != proposal_id {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::ProposalMismatch,
            "proposal rejection identity does not match the pending proposal",
        ));
    }
    debug_assert_eq!(instance.status, super::super::Status::WaitingForApproval);
    debug_assert_eq!(&proposal.id, proposal_id);
    instance.pending_proposal = None;
    Ok(super::super::EventKind::ProposalRejected)
}

pub(super) fn validate_proposal(
    instance: &super::super::Instance,
    proposal: &super::super::RevisionProposal,
) -> Result<(), super::super::Diagnostic> {
    let is_instance_base_match =
        proposal.instance_id == instance.id && proposal.base_definition_revision == instance.definition_revision;
    let is_revision_base_match =
        proposal.base_epoch == instance.epoch && proposal.base_workflow_revision == instance.workflow_revision;
    if !is_instance_base_match || !is_revision_base_match {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::ProposalMismatch,
            "proposal does not bind the current orchestration base state",
        ));
    }
    if super::super::proposal_identity(proposal)? != proposal.id {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::InvalidIdentity,
            "proposal BLAKE3 identity does not match its canonical content",
        ));
    }
    debug_assert_eq!(proposal.instance_id, instance.id);
    debug_assert_eq!(proposal.base_workflow_revision, instance.workflow_revision);
    let mut diagnostics = Vec::new();
    super::super::model::admission::validate_workflow_ref(&proposal.successor, &mut diagnostics);
    if let Some(diagnostic) = diagnostics.into_iter().next() {
        return Err(diagnostic);
    }
    Ok(())
}

pub(super) fn validate_approval(
    instance: &super::super::Instance,
    proposal: &super::super::RevisionProposal,
    approval: &super::super::ApprovalDecision,
) -> Result<(), super::super::Diagnostic> {
    authorize_actor(instance, &approval.actor)?;
    let expected_authority_hash = super::super::authority_policy_hash(&instance.definition.authority)?;
    let is_instance_revision_match = approval.instance_id == instance.id
        && approval.epoch == instance.epoch
        && approval.base_workflow_revision == instance.workflow_revision;
    let is_proposal_authority_match =
        approval.proposal_id == proposal.id && approval.authority_policy_hash == expected_authority_hash;
    if !is_instance_revision_match || !is_proposal_authority_match {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::StaleApproval,
            "approval does not bind the current instance, epoch, revision, proposal, and authority policy",
        ));
    }
    if super::super::approval_identity(approval)? != approval.id {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::InvalidIdentity,
            "approval BLAKE3 identity does not match its canonical content",
        ));
    }
    debug_assert_eq!(approval.proposal_id, proposal.id);
    debug_assert_eq!(approval.authority_policy_hash, expected_authority_hash);
    Ok(())
}

pub fn authorize_definition_actor(
    definition: &super::super::Definition,
    actor: &str,
) -> Result<(), super::super::Diagnostic> {
    if definition.authority.approvers.contains(actor) {
        Ok(())
    } else {
        Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::UnauthorizedActor,
            format!("actor '{actor}' is not authorized for this orchestration"),
        ))
    }
}

pub(in crate::orchestration::core) fn authorize_actor(
    instance: &super::super::Instance,
    actor: &str,
) -> Result<(), super::super::Diagnostic> {
    if instance.definition.authority.approvers.contains(actor) {
        Ok(())
    } else {
        Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::UnauthorizedActor,
            format!("actor '{actor}' is not authorized for this orchestration"),
        ))
    }
}

pub fn make_proposal(
    instance: &super::super::Instance,
    successor: super::super::WorkflowRevisionRef,
    safe_summary: impl Into<String>,
) -> Result<super::super::RevisionProposal, super::super::Diagnostic> {
    let mut proposal = super::super::RevisionProposal {
        id: crate::orchestration::identity::ProposalId::from_generated(
            "0".repeat(blake3::OUT_LEN.saturating_mul(super::super::HEX_CHARS_PER_BYTE)),
        ),
        instance_id: instance.id.clone(),
        base_definition_revision: instance.definition_revision.clone(),
        base_epoch: instance.epoch,
        base_workflow_revision: instance.workflow_revision.clone(),
        successor,
        safe_summary: safe_summary.into(),
    };
    proposal.id = super::super::proposal_identity(&proposal)?;
    Ok(proposal)
}

pub fn make_approval(
    instance: &super::super::Instance,
    proposal: &super::super::RevisionProposal,
    actor: impl Into<String>,
    approved: bool,
) -> Result<super::super::ApprovalDecision, super::super::Diagnostic> {
    let mut approval = super::super::ApprovalDecision {
        id: crate::orchestration::identity::ApprovalId::from_generated(
            "0".repeat(blake3::OUT_LEN.saturating_mul(super::super::HEX_CHARS_PER_BYTE)),
        ),
        instance_id: instance.id.clone(),
        epoch: instance.epoch,
        base_workflow_revision: instance.workflow_revision.clone(),
        proposal_id: proposal.id.clone(),
        authority_policy_hash: super::super::authority_policy_hash(&instance.definition.authority)?,
        actor: actor.into(),
        approved,
    };
    approval.id = super::super::approval_identity(&approval)?;
    Ok(approval)
}

pub async fn approve_with_execution(
    store: &crate::store::Store,
    request: super::super::ProductApproval<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let super::super::ProductApproval {
        instance_id,
        proposal,
        actor,
        execution,
    } = request;
    let instance_id = crate::orchestration::InstanceId::try_from_hex(instance_id)
        .map_err(super::super::durability::persistence::identity_error)?;
    let instance = super::super::durability::persistence::load_instance_by_id(store, &instance_id)?;
    debug_assert_eq!(instance.id, instance_id);
    debug_assert_eq!(proposal.instance_id, instance_id);
    let instance = submit_if_absent(store, instance, &proposal)?;
    decide_approved_proposal(store, instance, proposal, &actor, execution).await
}

pub(in crate::orchestration::shell) fn submit_if_absent(
    store: &crate::store::Store,
    instance: crate::orchestration::core::Instance,
    proposal: &crate::orchestration::core::RevisionProposal,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    match &instance.pending_proposal {
        None => super::super::durability::persistence::apply_and_commit(
            store,
            &instance,
            crate::orchestration::core::Action::SubmitProposal {
                proposal: proposal.clone(),
            },
        ),
        Some(pending) if pending.id == proposal.id => Ok(instance),
        Some(_) => Err(super::super::Error::InvalidProposal("a different proposal is already pending".to_string())),
    }
}

pub(in crate::orchestration::shell) async fn decide_approved_proposal(
    store: &crate::store::Store,
    instance: crate::orchestration::core::Instance,
    proposal: crate::orchestration::core::RevisionProposal,
    actor: &str,
    execution: super::super::ProductExecution<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let is_successor_validated = super::admission::load_validated_workflow(store, &proposal.successor).is_ok();
    let approval: crate::orchestration::core::ApprovalDecision =
        crate::orchestration::core::make_approval(&instance, &proposal, actor, true)?;
    debug_assert!(approval.approved);
    debug_assert_eq!(approval.proposal_id, proposal.id);
    let approved = super::super::durability::persistence::apply_and_commit(
        store,
        &instance,
        crate::orchestration::core::Action::DecideProposal {
            approval,
            successor_validated: is_successor_validated,
        },
    )?;
    if approved.status == crate::orchestration::core::Status::Ready {
        super::dispatch::execute_ready_round(store, approved, execution).await
    } else {
        Ok(approved)
    }
}

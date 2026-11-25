pub struct AppConfigApproval<'a> {
    pub instance_id: String,
    pub proposal: crate::orchestration::core::RevisionProposal,
    pub actor: String,
    pub app_config: &'a crate::config::AppConfig,
}

struct BackendApproval<'a> {
    instance_id: String,
    proposal: crate::orchestration::core::RevisionProposal,
    actor: String,
    execution: super::super::ExecutionBackend<'a>,
}

pub async fn approve_with_app_config(
    store: &crate::store::Store,
    request: AppConfigApproval<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    approve_with_backend(store, BackendApproval {
        instance_id: request.instance_id,
        proposal: request.proposal,
        actor: request.actor,
        execution: super::super::ExecutionBackend::AppConfig(request.app_config),
    })
    .await
}

pub async fn approve_with_execution(
    store: &crate::store::Store,
    request: super::super::ProductApproval<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    approve_with_backend(store, BackendApproval {
        instance_id: request.instance_id,
        proposal: request.proposal,
        actor: request.actor,
        execution: super::super::ExecutionBackend::Authority(request.execution),
    })
    .await
}

async fn approve_with_backend(
    store: &crate::store::Store,
    request: BackendApproval<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let instance_id = crate::orchestration::InstanceId::try_from_hex(request.instance_id)
        .map_err(super::super::durability::persistence::identity_error)?;
    let instance = super::super::durability::persistence::load_instance_by_id(store, &instance_id)?;
    debug_assert_eq!(instance.id, instance_id);
    debug_assert_eq!(request.proposal.instance_id, instance_id);
    let instance = submit_if_absent(store, instance, &request.proposal)?;
    decide_approved_proposal_backend(store, instance, request.proposal, &request.actor, request.execution).await
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
    decide_approved_proposal_backend(
        store,
        instance,
        proposal,
        actor,
        super::super::ExecutionBackend::Authority(execution),
    )
    .await
}

async fn decide_approved_proposal_backend(
    store: &crate::store::Store,
    instance: crate::orchestration::core::Instance,
    proposal: crate::orchestration::core::RevisionProposal,
    actor: &str,
    execution: super::super::ExecutionBackend<'_>,
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
        super::dispatch::execute_ready_round_backend(store, approved, execution).await
    } else {
        Ok(approved)
    }
}

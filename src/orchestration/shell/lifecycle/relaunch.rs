// r[impl lattice.agent_orchestration.retry_round]
pub async fn retry(
    store: &crate::store::Store,
    request: super::super::RetryRequest<'_>,
    agent_runtime: crate::agent_node::RuntimeOptions,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let authority_provider = crate::engine::effect::execution::UnavailableRunContextSource;
    retry_with_execution(store, request, super::super::ProductExecution {
        agent_runtime,
        authority_provider: &authority_provider,
    })
    .await
}

pub async fn retry_with_execution(
    store: &crate::store::Store,
    request: super::super::RetryRequest<'_>,
    execution: super::super::ProductExecution<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let instance = super::super::load_instance(store, request.instance_id)?;
    reject_pending_retry(store, &instance)?;
    let workflow = super::admission::load_validated_workflow(store, &instance.workflow_revision)?;
    let plan = super::admission::planned_charge(&instance.definition, &workflow)?;
    let failed = crate::orchestration::core::preflight_retry(&instance, request.actor, plan)?;
    let run_id = store.next_run_id()?;
    let intent = crate::orchestration::core::DispatchIntent {
        id: crate::orchestration::core::dispatch_identity(&instance, run_id)?,
        run_id,
        workflow_revision: instance.workflow_revision.clone(),
        status: crate::orchestration::core::DispatchStatus::Pending,
    };
    debug_assert_ne!(intent.run_id, failed.run_id);
    debug_assert_ne!(intent.id, failed.dispatch_id);
    let started = super::super::durability::persistence::apply_and_commit(
        store,
        &instance,
        crate::orchestration::core::Action::RetryRound {
            actor: request.actor.to_string(),
            intent: intent.clone(),
            planned_charge: plan,
        },
    )?;
    super::dispatch::execute_started_round(store, started, workflow, intent, execution).await
}

pub(super) fn reject_pending_retry(
    store: &crate::store::Store,
    instance: &crate::orchestration::core::Instance,
) -> Result<(), super::super::Error> {
    let Some(intent) = instance.pending_dispatch.as_ref() else {
        return Ok(());
    };
    let Some(record) = store.load_run(intent.run_id)? else {
        return Err(super::super::Error::PendingDispatchCannotRetry { run_id: intent.run_id });
    };
    super::super::durability::persistence::validate_run_owner(instance, intent, &record)?;
    Err(super::super::Error::ObservedRunRequiresRecovery { run_id: intent.run_id })
}

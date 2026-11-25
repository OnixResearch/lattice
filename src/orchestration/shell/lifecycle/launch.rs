// r[impl lattice.agent_orchestration.cli]
// r[impl lattice.agent_orchestration.rounds]
pub async fn start(
    store: &crate::store::Store,
    definition: crate::orchestration::core::Definition,
    actor: &str,
    agent_runtime: crate::agent_node::RuntimeOptions,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let authority_provider = crate::engine::effect::execution::UnavailableRunContextSource;
    start_with_execution(store, definition, actor, super::super::ProductExecution {
        agent_runtime,
        authority_provider: &authority_provider,
    })
    .await
}

pub async fn start_with_app_config(
    store: &crate::store::Store,
    definition: crate::orchestration::core::Definition,
    actor: &str,
    app_config: &crate::config::AppConfig,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    start_with_backend(store, definition, actor, super::super::ExecutionBackend::AppConfig(app_config)).await
}

pub async fn start_with_execution(
    store: &crate::store::Store,
    definition: crate::orchestration::core::Definition,
    actor: &str,
    execution: super::super::ProductExecution<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    start_with_backend(store, definition, actor, super::super::ExecutionBackend::Authority(execution)).await
}

async fn start_with_backend(
    store: &crate::store::Store,
    definition: crate::orchestration::core::Definition,
    actor: &str,
    execution: super::super::ExecutionBackend<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    crate::orchestration::core::authorize_definition_actor(&definition, actor)?;
    let report = super::super::validate_definition_in_store(store, &definition)?;
    if !report.valid() {
        return Err(super::super::Error::DefinitionInvalid(Box::new(report)));
    }
    store.save_orchestration_definition(&definition)?;
    let sequence = store.next_orchestration_sequence()?;
    let instance = crate::orchestration::core::new_instance(definition, sequence, None)?;
    store.create_orchestration(&instance, &[])?;
    super::dispatch::execute_ready_round_backend(store, instance, execution).await
}

pub async fn start_from_trigger_with_execution(
    store: &crate::store::Store,
    request: super::super::ProductTriggerStart<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let super::super::ProductTriggerStart {
        definition_id,
        trigger_id,
        trigger_kind,
        idempotency_key,
        execution,
    } = request;
    let definition_id = crate::orchestration::DefinitionId::try_from_text(definition_id)
        .map_err(super::super::durability::persistence::identity_error)?;
    let trigger_id = crate::orchestration::TriggerId::try_from_text(trigger_id)
        .map_err(super::super::durability::persistence::identity_error)?;
    let idempotency_key = crate::orchestration::TriggerIdempotencyKey::try_from_text(idempotency_key)
        .map_err(super::super::durability::persistence::identity_error)?;
    let definition = store
        .load_orchestration_definition(&definition_id)?
        .ok_or_else(|| super::super::Error::WorkflowNotFound(definition_id.to_string()))?;
    let report = super::super::validate_definition_in_store(store, &definition)?;
    if !report.valid() {
        return Err(super::super::Error::DefinitionInvalid(Box::new(report)));
    }
    let definition_revision = crate::orchestration::core::definition_revision(&definition)?;
    let accepted = store
        .list_orchestration_trigger_admissions()?
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    let sequence = store.next_orchestration_sequence()?;
    let context = crate::orchestration::core::TriggerContext { accepted };
    let admission = crate::orchestration::core::admit_trigger_start(crate::orchestration::core::TriggerStartRequest {
        definition: &definition,
        definition_revision: &definition_revision,
        trigger_id: &trigger_id,
        trigger_kind,
        idempotency_key: &idempotency_key,
        next_sequence: sequence,
        context: &context,
    })?;
    match admission {
        crate::orchestration::core::TriggerAdmission::Duplicate { instance_id } => {
            super::super::durability::persistence::load_instance_by_id(store, &instance_id)
        }
        crate::orchestration::core::TriggerAdmission::Accepted(plan) => {
            let instance =
                crate::orchestration::core::new_instance(definition, sequence, Some(plan.scoped_key.clone()))?;
            debug_assert_eq!(instance.id, plan.instance_id);
            match store.create_triggered_orchestration(&plan.scoped_key, &instance)? {
                crate::store::TriggerAdmissionWrite::Inserted => {
                    super::dispatch::execute_ready_round(store, instance, execution).await
                }
                crate::store::TriggerAdmissionWrite::Duplicate { instance_id } => {
                    super::super::durability::persistence::load_instance_by_id(store, &instance_id)
                }
            }
        }
    }
}

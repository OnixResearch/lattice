// r[impl lattice.agent_orchestration.unobserved_abandon]
pub fn abandon_pending(
    store: &crate::store::Store,
    request: super::super::AbandonPendingRequest<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let instance = super::super::load_instance(store, request.instance_id)?;
    let intent = unobserved_dispatch(store, &instance, request.instance_id)?;
    super::super::durability::persistence::apply_and_commit(
        store,
        &instance,
        crate::orchestration::core::Action::AbandonPendingDispatch {
            actor: request.actor.to_string(),
            dispatch_id: intent.id.clone(),
            run_id: intent.run_id,
        },
    )
}

fn unobserved_dispatch<'a>(
    store: &crate::store::Store,
    instance: &'a crate::orchestration::core::Instance,
    instance_id: &str,
) -> Result<&'a crate::orchestration::core::DispatchIntent, super::super::Error> {
    let intent = instance
        .pending_dispatch
        .as_ref()
        .ok_or_else(|| super::super::Error::NoPendingDispatch(instance_id.to_string()))?;
    if let Some(record) = store.load_run(intent.run_id)? {
        super::super::durability::persistence::validate_run_owner(instance, intent, &record)?;
        return Err(super::super::Error::ObservedRunRequiresRecovery { run_id: intent.run_id });
    }
    Ok(intent)
}

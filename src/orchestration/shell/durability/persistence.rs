pub(in crate::orchestration::shell) fn apply_and_commit(
    store: &crate::store::Store,
    instance: &crate::orchestration::core::Instance,
    action: crate::orchestration::core::Action,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let fact = crate::orchestration::core::Fact {
        id: crate::orchestration::core::fact_identity(&instance.id, instance.version, &action)?,
        action,
    };
    let decision = crate::orchestration::core::transition(instance, fact)?;
    if decision.duplicate {
        return Ok(decision.instance);
    }
    let event = decision.event.as_ref().ok_or_else(|| {
        super::super::Error::InvalidProposal("applied orchestration transition emitted no event".to_string())
    })?;
    store.commit_orchestration_transition(instance.version, &decision.instance, event)?;
    Ok(decision.instance)
}

// r[impl lattice.agent_orchestration.run_identity]
pub(in crate::orchestration::shell) fn validate_run_owner(
    instance: &crate::orchestration::core::Instance,
    intent: &crate::orchestration::core::DispatchIntent,
    record: &crate::store::RunRecord,
) -> Result<(), super::super::Error> {
    if record.run_id != intent.run_id {
        return Err(super::super::Error::RunIdentityMismatch { run_id: record.run_id });
    }
    if record.workflow_name != intent.workflow_revision.workflow_name {
        return Err(super::super::Error::RunIdentityMismatch { run_id: record.run_id });
    }
    let is_owner_match = matches!(
        &record.owner,
        crate::store::RunOwner::Orchestration {
            instance_id,
            dispatch_id,
        } if instance_id == &instance.id && dispatch_id == &intent.id
    );
    if !is_owner_match {
        return Err(super::super::Error::RunIdentityMismatch { run_id: record.run_id });
    }
    debug_assert_eq!(record.run_id, intent.run_id);
    debug_assert!(is_owner_match);
    Ok(())
}

pub fn load_instance(
    store: &crate::store::Store,
    instance_id: &str,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let instance_id = crate::orchestration::InstanceId::try_from_hex(instance_id).map_err(identity_error)?;
    load_instance_by_id(store, &instance_id)
}

pub(in crate::orchestration::shell) fn load_instance_by_id(
    store: &crate::store::Store,
    instance_id: &crate::orchestration::InstanceId,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    store
        .load_orchestration(instance_id)?
        .ok_or_else(|| super::super::Error::MissingInstance(instance_id.to_string()))
}

pub(in crate::orchestration::shell) fn identity_error(
    error: crate::orchestration::AdmissionError,
) -> super::super::Error {
    super::super::Error::Core(crate::orchestration::core::Diagnostic {
        code: crate::orchestration::core::DiagnosticCode::InvalidIdentity,
        message: error.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerContext {
    pub accepted: std::collections::BTreeMap<
        crate::orchestration::identity::TriggerScopedKey,
        crate::orchestration::identity::InstanceId,
    >,
}

#[derive(Debug, Clone, Copy)]
pub struct TriggerStartRequest<'a> {
    pub definition: &'a super::super::Definition,
    pub definition_revision: &'a crate::orchestration::identity::DefinitionRevision,
    pub trigger_id: &'a crate::orchestration::identity::TriggerId,
    pub trigger_kind: super::super::TriggerKind,
    pub idempotency_key: &'a crate::orchestration::identity::TriggerIdempotencyKey,
    pub next_sequence: u64,
    pub context: &'a TriggerContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerPlan {
    pub scoped_key: crate::orchestration::identity::TriggerScopedKey,
    pub instance_id: crate::orchestration::identity::InstanceId,
    pub definition_revision: crate::orchestration::identity::DefinitionRevision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerAdmission {
    Accepted(TriggerPlan),
    Duplicate {
        instance_id: crate::orchestration::identity::InstanceId,
    },
}

// r[impl lattice.agent_orchestration.triggers]
pub fn admit_trigger_start(request: TriggerStartRequest<'_>) -> Result<TriggerAdmission, super::super::Diagnostic> {
    if !request
        .definition
        .triggers
        .iter()
        .any(|binding| &binding.trigger_id == request.trigger_id && binding.kind == request.trigger_kind)
    {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::TriggerNotBound,
            format!("trigger '{}' is not bound to orchestration '{}'", request.trigger_id, request.definition.id),
        ));
    }
    debug_assert!(
        request
            .definition
            .triggers
            .iter()
            .any(|binding| &binding.trigger_id == request.trigger_id && binding.kind == request.trigger_kind)
    );
    let scoped_key =
        super::super::scoped_trigger_key(request.definition_revision, request.trigger_id, request.idempotency_key)?;
    debug_assert!(!scoped_key.as_str().is_empty());
    if let Some(instance_id) = request.context.accepted.get(&scoped_key) {
        return Ok(TriggerAdmission::Duplicate {
            instance_id: instance_id.clone(),
        });
    }
    let instance_id =
        crate::orchestration::identity::InstanceId::from_generated(super::super::model::canonical::hash_serializable(
            super::super::INSTANCE_HASH_LABEL,
            &(request.definition_revision, request.next_sequence, Some(scoped_key.as_str())),
        )?);
    Ok(TriggerAdmission::Accepted(TriggerPlan {
        scoped_key,
        instance_id,
        definition_revision: request.definition_revision.clone(),
    }))
}

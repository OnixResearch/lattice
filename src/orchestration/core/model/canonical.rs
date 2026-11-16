// r[impl lattice.agent_orchestration.model]
pub fn workflow_revision(
    workflow: &crate::engine::Workflow,
) -> Result<super::super::WorkflowRevisionRef, super::super::Diagnostic> {
    let portable_workflow = crate::workflow_format::workflow_to_portable(workflow);
    let content_hash = crate::orchestration::identity::WorkflowRevision::from_generated(hash_serializable(
        super::super::WORKFLOW_HASH_LABEL,
        &portable_workflow,
    )?);
    Ok(super::super::WorkflowRevisionRef {
        workflow_name: workflow.meta.name.clone(),
        content_hash,
    })
}

// r[impl lattice.agent_orchestration.model]
pub fn definition_revision(
    definition: &super::super::Definition,
) -> Result<crate::orchestration::identity::DefinitionRevision, super::super::Diagnostic> {
    Ok(crate::orchestration::identity::DefinitionRevision::from_generated(hash_serializable(
        super::super::DEFINITION_HASH_LABEL,
        definition,
    )?))
}

pub fn authority_policy_hash(
    policy: &super::super::AuthorityPolicy,
) -> Result<crate::orchestration::identity::AuthorityPolicyIdentity, super::super::Diagnostic> {
    Ok(crate::orchestration::identity::AuthorityPolicyIdentity::from_generated(hash_serializable(
        super::super::AUTHORITY_HASH_LABEL,
        policy,
    )?))
}

pub fn proposal_identity(
    proposal: &super::super::RevisionProposal,
) -> Result<crate::orchestration::identity::ProposalId, super::super::Diagnostic> {
    let mut unhashed = proposal.clone();
    unhashed.id.clear_for_hash();
    Ok(crate::orchestration::identity::ProposalId::from_generated(hash_serializable(
        super::super::PROPOSAL_HASH_LABEL,
        &unhashed,
    )?))
}

pub fn approval_identity(
    approval: &super::super::ApprovalDecision,
) -> Result<crate::orchestration::identity::ApprovalId, super::super::Diagnostic> {
    let mut unhashed = approval.clone();
    unhashed.id.clear_for_hash();
    Ok(crate::orchestration::identity::ApprovalId::from_generated(hash_serializable(
        super::super::APPROVAL_HASH_LABEL,
        &unhashed,
    )?))
}

pub fn fact_identity(
    instance_id: &crate::orchestration::identity::InstanceId,
    version: u64,
    action: &super::super::Action,
) -> Result<crate::orchestration::identity::FactId, super::super::Diagnostic> {
    Ok(crate::orchestration::identity::FactId::from_generated(hash_serializable(
        super::super::FACT_HASH_LABEL,
        &(instance_id, version, action),
    )?))
}

pub fn scoped_trigger_key(
    definition_revision: &crate::orchestration::identity::DefinitionRevision,
    trigger_id: &crate::orchestration::identity::TriggerId,
    idempotency_key: &crate::orchestration::identity::TriggerIdempotencyKey,
) -> Result<crate::orchestration::identity::TriggerScopedKey, super::super::Diagnostic> {
    Ok(crate::orchestration::identity::TriggerScopedKey::from_generated(hash_serializable(
        super::super::TRIGGER_HASH_LABEL,
        &(definition_revision, trigger_id, idempotency_key),
    )?))
}

pub(in crate::orchestration::core) fn hash_serializable<T: serde::Serialize + ?Sized>(
    label: &[u8],
    value: &T,
) -> Result<String, super::super::Diagnostic> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        super::failures::diagnostic(
            super::super::DiagnosticCode::SerializationFailure,
            format!("failed to serialize orchestration identity input: {error}"),
        )
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(super::super::ORCHESTRATION_HASH_DOMAIN);
    hasher.update(label);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

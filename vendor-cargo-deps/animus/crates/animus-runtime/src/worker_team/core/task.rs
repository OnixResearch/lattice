struct OptionalDraftParts {
    scope: super::super::WorkerScope,
    context_packet: super::super::WorkerContextPacket,
    allowed_capabilities: Vec<super::super::WorkerCapability>,
    budgets: super::super::WorkerBudgets,
    output_schema: super::super::WorkerOutputSchema,
    parent_trace: super::super::ParentTrace,
    authority: super::super::WorkerAuthority,
}

// r[impl agent_harness_runtime.claude_code_agents.task_contract]
/// Validates a scoped worker task draft.
///
/// # Errors
///
/// Returns [`DelegationFailure`](super::super::DelegationFailure) when the task
/// omits required fields, asks for overbroad authority, leaks context, or
/// declares unsafe capabilities.
pub fn validate(
    draft: super::super::WorkerTaskDraft,
) -> Result<super::super::WorkerTask, super::super::DelegationFailure> {
    let task_id = super::validate_label(super::super::TASK_ID_FIELD, draft.task_id.as_str())?;
    let objective = super::validate_text(super::super::OBJECTIVE_FIELD, draft.objective.as_str())?;
    let parts = validate_optional_draft_parts(draft)?;
    Ok(super::super::WorkerTask {
        task_id,
        objective,
        scope: parts.scope,
        context_packet: parts.context_packet,
        allowed_capabilities: parts.allowed_capabilities,
        budgets: parts.budgets,
        output_schema: parts.output_schema,
        parent_trace: parts.parent_trace,
        authority: parts.authority,
    })
}

fn validate_optional_draft_parts(
    draft: super::super::WorkerTaskDraft,
) -> Result<OptionalDraftParts, super::super::DelegationFailure> {
    let scope = validate_required(draft.scope, super::super::SCOPE_FIELD)?;
    let context_packet = validate_required(draft.context_packet, super::super::CONTEXT_PACKET_FIELD)?;
    let budgets = validate_required(draft.budgets, super::super::BUDGET_FIELD)?;
    let output_schema = validate_required(draft.output_schema, super::super::OUTPUT_SCHEMA_FIELD)?;
    let parent_trace = validate_required(draft.parent_trace, super::super::PARENT_TRACE_FIELD)?;
    let authority = validate_required(draft.authority, super::super::AUTHORITY_FIELD)?;
    assert!(!context_packet.items.is_empty(), "context packet items must be present");
    assert!(!output_schema.required_fields.is_empty(), "output schema required fields must be present");
    super::contract::validate(super::contract::Input {
        scope: &scope,
        context_packet: &context_packet,
        budgets,
        output_schema: &output_schema,
        parent_trace: &parent_trace,
        authority: &authority,
    })?;
    let allowed_capabilities = super::capability::validate_capabilities(
        authority.parent_policy_required,
        draft.allowed_capabilities.as_slice(),
    )?;
    Ok(OptionalDraftParts {
        scope,
        context_packet,
        allowed_capabilities,
        budgets,
        output_schema,
        parent_trace,
        authority,
    })
}

fn validate_required<T>(value: Option<T>, field: &'static str) -> Result<T, super::super::DelegationFailure> {
    value.ok_or_else(|| {
        super::super::DelegationFailure::new(
            super::super::FailureKind::MissingField,
            field,
            super::super::MISSING_FIELD_REASON,
        )
    })
}

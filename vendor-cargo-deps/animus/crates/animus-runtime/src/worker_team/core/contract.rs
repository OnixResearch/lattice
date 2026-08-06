#[derive(Clone, Copy)]
pub(super) struct Input<'a> {
    pub(super) scope: &'a super::super::WorkerScope,
    pub(super) context_packet: &'a super::super::WorkerContextPacket,
    pub(super) budgets: super::super::WorkerBudgets,
    pub(super) output_schema: &'a super::super::WorkerOutputSchema,
    pub(super) parent_trace: &'a super::super::ParentTrace,
    pub(super) authority: &'a super::super::WorkerAuthority,
}

pub(super) fn validate(input: Input<'_>) -> Result<(), super::super::DelegationFailure> {
    validate_scope(input.scope)?;
    super::context::validate(input.context_packet)?;
    validate_budgets(input.budgets)?;
    validate_output_schema(input.output_schema)?;
    validate_parent_trace(input.parent_trace)?;
    validate_authority(input.authority)
}

fn validate_scope(scope: &super::super::WorkerScope) -> Result<(), super::super::DelegationFailure> {
    super::validate_label(super::super::SCOPE_FIELD, scope.label.as_str())?;
    super::validate_text(super::super::SCOPE_FIELD, scope.summary.as_str())?;
    super::validate_non_empty_slice(super::super::SCOPE_TARGET_FIELD, scope.allowed_targets.as_slice())?;
    for target in &scope.allowed_targets {
        super::validate_label(super::super::SCOPE_TARGET_FIELD, target.as_str())?;
    }
    Ok(())
}

fn validate_budgets(budgets: super::super::WorkerBudgets) -> Result<(), super::super::DelegationFailure> {
    for value in [budgets.turns, budgets.context_items, budgets.tool_calls] {
        if value < super::super::MIN_BUDGET_VALUE {
            return Err(super::super::DelegationFailure::new(
                super::super::FailureKind::EmptyField,
                super::super::BUDGET_FIELD,
                super::super::EMPTY_FIELD_REASON,
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_output_schema(
    schema: &super::super::WorkerOutputSchema,
) -> Result<(), super::super::DelegationFailure> {
    let schema_name = super::validate_label(super::super::OUTPUT_SCHEMA_FIELD, schema.name.as_str())?;
    super::validate_non_empty_slice(super::super::OUTPUT_SCHEMA_REQUIRED_FIELD, schema.required_fields.as_slice())?;
    assert!(!schema_name.is_empty(), "schema name must be validated");
    assert!(!schema.required_fields.is_empty(), "schema required fields must be non-empty");
    let mut seen = std::collections::BTreeSet::new();
    for field in &schema.required_fields {
        let field_name = super::validate_label(super::super::OUTPUT_SCHEMA_REQUIRED_FIELD, field.as_str())?;
        if !seen.insert(field_name) {
            return Err(super::super::DelegationFailure::new(
                super::super::FailureKind::MalformedReport,
                super::super::OUTPUT_SCHEMA_REQUIRED_FIELD,
                "duplicate required output field",
            ));
        }
    }
    Ok(())
}

fn validate_parent_trace(trace: &super::super::ParentTrace) -> Result<(), super::super::DelegationFailure> {
    super::validate_label(super::super::PARENT_TRACE_FIELD, trace.parent_run_id.as_str())?;
    super::validate_label(super::super::PARENT_TRACE_FIELD, trace.turn_id.as_str())?;
    super::text::validate_hash(super::text::HashInput {
        field: super::super::POLICY_HASH_FIELD,
        value: trace.policy_hash.as_str(),
    })
}

fn validate_authority(authority: &super::super::WorkerAuthority) -> Result<(), super::super::DelegationFailure> {
    super::validate_label(super::super::AUTHORITY_FIELD, authority.grant_label.as_str())?;
    if authority.parent_policy_required {
        return Ok(());
    }
    Err(super::super::DelegationFailure::new(
        super::super::FailureKind::OverbroadAuthority,
        super::super::AUTHORITY_FIELD,
        super::super::OVERBROAD_REASON,
    ))
}

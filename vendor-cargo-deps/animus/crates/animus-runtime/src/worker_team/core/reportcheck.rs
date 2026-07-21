pub(super) fn validate_report_header(
    report: &super::super::Report,
    schema: &super::super::WorkerOutputSchema,
) -> Result<(), super::super::DelegationFailure> {
    super::validate_label(super::super::WORKER_REPORT_FIELD, report.worker_id.as_str())?;
    super::contract::validate_output_schema(schema)?;
    let schema_label = super::validate_label(
        super::super::OUTPUT_SCHEMA_FIELD,
        report.schema_name.as_str(),
    )?;
    if schema_label != schema.name {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::OutputSchemaMismatch,
            super::super::OUTPUT_SCHEMA_FIELD,
            super::super::MALFORMED_REPORT_REASON,
        ));
    }
    Ok(())
}

pub(super) fn validate_report_fields(
    report: &super::super::Report,
    schema: &super::super::WorkerOutputSchema,
) -> Result<(), super::super::DelegationFailure> {
    debug_assert!(!schema.required_fields.is_empty());
    debug_assert!(schema.required_fields.len() <= super::super::MAX_REPORT_FIELDS);
    if report.fields.len() > super::super::MAX_REPORT_FIELDS {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::FieldTooLong,
            super::super::WORKER_REPORT_FIELD_MAP,
            super::super::TOO_LONG_REASON,
        ));
    }
    validate_required_fields(report, schema)
}

fn validate_required_fields(
    report: &super::super::Report,
    schema: &super::super::WorkerOutputSchema,
) -> Result<(), super::super::DelegationFailure> {
    debug_assert!(!schema.required_fields.is_empty());
    debug_assert!(report.fields.len() <= super::super::MAX_REPORT_FIELDS);
    for required in &schema.required_fields {
        if report
            .fields
            .get(required)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(super::super::DelegationFailure::new(
                super::super::FailureKind::MalformedReport,
                super::super::WORKER_REPORT_FIELD_MAP,
                super::super::MALFORMED_REPORT_REASON,
            ));
        }
    }
    for (key, value) in &report.fields {
        super::validate_label(super::super::WORKER_REPORT_FIELD_MAP, key.as_str())?;
        super::validate_text(super::super::WORKER_REPORT_FIELD_MAP, value.as_str())?;
    }
    Ok(())
}

pub(super) fn validate_report_evidence(
    evidence: &[super::super::EvidenceReference],
) -> Result<(), super::super::DelegationFailure> {
    super::validate_non_empty_slice(super::super::WORKER_EVIDENCE_FIELD, evidence)?;
    for reference in evidence {
        super::validate_label(
            super::super::WORKER_EVIDENCE_FIELD,
            reference.evidence_id.as_str(),
        )?;
        super::validate_text(
            super::super::WORKER_EVIDENCE_FIELD,
            reference.summary.as_str(),
        )?;
    }
    Ok(())
}

pub(super) fn validate_report_claims(
    claims: &[super::super::Claim],
) -> Result<(), super::super::DelegationFailure> {
    for claim in claims {
        super::validate_label(super::super::WORKER_CLAIM_FIELD, claim.subject.as_str())?;
        super::validate_text(super::super::WORKER_CLAIM_FIELD, claim.value.as_str())?;
        super::validate_non_empty_slice(
            super::super::WORKER_CLAIM_FIELD,
            claim.evidence_ids.as_slice(),
        )?;
        for evidence_id in &claim.evidence_ids {
            super::validate_label(super::super::WORKER_CLAIM_FIELD, evidence_id.as_str())?;
        }
    }
    Ok(())
}

pub(super) fn validate_report_mutations(
    intents: &[super::super::MutationIntent],
) -> Result<(), super::super::DelegationFailure> {
    for intent in intents {
        super::validate_label(super::super::WORKER_MUTATION_FIELD, intent.target.as_str())?;
        super::validate_label(
            super::super::WORKER_MUTATION_FIELD,
            intent.mutation_kind.as_str(),
        )?;
        super::validate_label(
            super::super::WORKER_MUTATION_FIELD,
            intent.evidence_id.as_str(),
        )?;
    }
    Ok(())
}

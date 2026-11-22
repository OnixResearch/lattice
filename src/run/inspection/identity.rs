//! Pure, content-omitting projection of a run's recorded input identity.

const BLAKE3_IDENTITY_PREFIX: &str = "b3:";

// r[impl lattice.workflow_templates.run_inputs.inspection]
/// Inspect a run and, when it was bound to inputs, show their identity without any value.
///
/// The input record must belong to the same run and saved workflow as the run record;
/// otherwise the report is invalid with `InputsIdentityMismatch` and shows no inputs.
pub fn inspect_with_inputs(
    input: super::model::Input<'_>,
    inputs: Option<&crate::workflow_template::inputs::RunInputRecord>,
) -> super::model::Report {
    let record = input.record;
    let mut report = super::core::inspect(input);
    let Some(inputs) = inputs else {
        return report;
    };
    let is_bound_to_run =
        inputs.run_id == record.run_id && inputs.inputs.workflow.workflow_name == record.workflow_name;
    if is_bound_to_run {
        report.inputs = Some(safe_inputs(&inputs.inputs));
    } else {
        report.diagnostic_codes.push(super::model::DiagnosticCode::InputsIdentityMismatch);
        report.valid = false;
    }
    debug_assert_eq!(report.valid, report.diagnostic_codes.is_empty());
    debug_assert_eq!(report.inputs.is_some(), is_bound_to_run);
    report
}

/// Names, kinds, and BLAKE3 identities only.
///
/// `values` is the `params::Resolved` identity, which a holder of the values reproduces with
/// `params::resolve`. Per-value digests are not shown: most declared values come from small
/// domains, so a digest of one would not hide it, and the values are not secret by rule.
pub fn safe_inputs(inputs: &crate::workflow_template::inputs::InputIdentity) -> super::model::SafeInputs {
    super::model::SafeInputs {
        template: format!("{BLAKE3_IDENTITY_PREFIX}{}", inputs.template.as_str()),
        bound: format!("{BLAKE3_IDENTITY_PREFIX}{}", inputs.bound.as_str()),
        workflow_revision: format!("{BLAKE3_IDENTITY_PREFIX}{}", inputs.workflow.content_hash.as_str()),
        values: format!("{BLAKE3_IDENTITY_PREFIX}{}", inputs.values.as_str()),
        params: inputs.params.clone(),
    }
}

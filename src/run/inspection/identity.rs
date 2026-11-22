//! Pure, content-omitting projection of a run's recorded input identity.

const RUN_REPORT_PREFIX: &str = "b3:";
const AUTHORITY_REPORT_PREFIX: &str = "blake3:";

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

/// Names, kinds, and BLAKE3 identities only, with the `b3:` prefix `runs show` uses for every
/// identity it renders.
///
/// `values` is the `params::Resolved` identity, which a holder of the values reproduces with
/// `params::resolve`. Per-value digests are not shown: most declared values come from small
/// domains, so a digest of one would not hide it. The shown identities are unsalted, so they omit
/// the values without hiding them: anyone with the catalog can enumerate low-entropy inputs.
pub fn safe_inputs(inputs: &crate::workflow_template::inputs::InputIdentity) -> super::model::SafeInputs {
    render(inputs, RUN_REPORT_PREFIX)
}

/// The same identities with the `blake3:` prefix `authority inspect` uses for its
/// `workflow_revision`, so one hash renders one way within one report.
pub fn authority_inputs(inputs: &crate::workflow_template::inputs::InputIdentity) -> super::model::SafeInputs {
    render(inputs, AUTHORITY_REPORT_PREFIX)
}

fn render(inputs: &crate::workflow_template::inputs::InputIdentity, prefix: &str) -> super::model::SafeInputs {
    super::model::SafeInputs {
        template: format!("{prefix}{}", inputs.template.as_str()),
        bound: format!("{prefix}{}", inputs.bound.as_str()),
        workflow_revision: format!("{prefix}{}", inputs.workflow.content_hash.as_str()),
        values: format!("{prefix}{}", inputs.values.as_str()),
        params: inputs.params.clone(),
    }
}

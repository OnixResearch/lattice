#[derive(serde::Serialize)]
struct ArtifactSummary<'a> {
    schema: &'static str,
    trial_identity: &'a str,
    receipt_identity: &'a str,
    artifact_manifest_identity: &'a str,
    artifact_count: u32,
    artifact_bytes: u64,
    redaction: &'a str,
    status: crate::agent_evaluation::model::TrialStatus,
}

pub(super) fn render_plan(args: &super::PlanArgs, store: Option<&crate::store::Store>) -> Result<(), String> {
    let catalog = crate::agent_evaluation::format::load_catalog(&args.root)
        .map_err(|_| "evaluation catalog admission failed".to_string())?;
    let mut request = crate::agent_evaluation::format::load_plan_request(&args.request)
        .map_err(|_| "evaluation plan request admission failed".to_string())?;
    if let Some(durable_store) = store {
        if !request.prior_trials.is_empty() {
            return Err("resume request must not supply prior trial claims".to_string());
        }
        request.prior_trials = durable_priors(&catalog, durable_store)?;
    }
    let plan = crate::agent_evaluation::planning::plan_matrix(&catalog, &request)
        .map_err(|_| "evaluation planning failed".to_string())?;
    render_matrix(&plan, args.output)
}

pub(super) fn render_artifacts(
    store: &crate::store::Store,
    trial_identity: &str,
    output: super::OutputFormat,
) -> Result<(), String> {
    let record = store
        .load_evaluation_trial(trial_identity)
        .map_err(|_| "evaluation trial lookup failed".to_string())?
        .ok_or_else(|| "evaluation trial was not found".to_string())?;
    let receipt_identity = record
        .receipt_identity
        .as_deref()
        .ok_or_else(|| "evaluation trial has no terminal artifact manifest".to_string())?;
    let receipt = load_receipt(store, receipt_identity)?;
    if receipt.trial_identity != record.plan.trial_identity || receipt.status != record.status {
        return Err("evaluation artifact metadata does not match durable trial state".to_string());
    }
    let summary = ArtifactSummary {
        schema: "lattice.agent-evaluation-artifact-summary.v1",
        trial_identity: &receipt.trial_identity,
        receipt_identity: &receipt.identity,
        artifact_manifest_identity: &receipt.artifact_manifest_identity,
        artifact_count: receipt.artifact_count,
        artifact_bytes: receipt.artifact_bytes,
        redaction: &receipt.redaction,
        status: receipt.status,
    };
    match output {
        super::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&summary).map_err(|_| "artifact output failed")?)
        }
        super::OutputFormat::Text => println!(
            "trial={} status={:?} manifest={} artifacts={} bytes={} redaction={}",
            summary.trial_identity,
            summary.status,
            summary.artifact_manifest_identity,
            summary.artifact_count,
            summary.artifact_bytes,
            summary.redaction
        ),
    }
    debug_assert_eq!(summary.schema, "lattice.agent-evaluation-artifact-summary.v1");
    debug_assert!(!summary.receipt_identity.is_empty());
    Ok(())
}

pub(super) fn render_matrix(
    plan: &crate::agent_evaluation::planning::MatrixPlan,
    output: super::OutputFormat,
) -> Result<(), String> {
    match output {
        super::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(plan).map_err(|_| "plan output failed")?)
        }
        super::OutputFormat::Text => {
            println!("plan {} catalog={}", plan.identity, plan.catalog_identity);
            for trial in &plan.trials {
                println!(
                    "{} order={} case={} route={:?} live={} resume={:?}",
                    trial.trial_identity, trial.order, trial.case_id, trial.route, trial.live, trial.resume
                );
            }
        }
    }
    debug_assert!(!plan.identity.is_empty());
    debug_assert!(!plan.trials.is_empty());
    Ok(())
}

fn durable_priors(
    catalog: &crate::agent_evaluation::model::AdmittedCatalog,
    store: &crate::store::Store,
) -> Result<Vec<crate::agent_evaluation::planning::PriorTrial>, String> {
    let records = store
        .list_evaluation_trials(super::MAX_RECORD_LIMIT_COUNT)
        .map_err(|_| "durable evaluation state is unavailable".to_string())?;
    let record_count = records.len();
    let mut priors = Vec::with_capacity(record_count);
    for record in records {
        let case = catalog
            .cases
            .iter()
            .find(|case| case.definition.id == record.plan.case_id)
            .ok_or_else(|| "durable evaluation case is absent from the catalog".to_string())?;
        let receipt = record.receipt_identity.as_deref().map(|identity| load_receipt(store, identity)).transpose()?;
        if let Some(value) = &receipt {
            let has_receipt_drift = [
                value.trial_identity != record.plan.trial_identity,
                value.case_revision_identity != record.plan.case_revision_identity,
                value.status != record.status,
                record.receipt_identity.as_deref() != Some(value.identity.as_str()),
            ]
            .into_iter()
            .any(|drift| drift);
            if has_receipt_drift {
                return Err("durable evaluation receipt drifted".to_string());
            }
        }
        priors.push(prior_from_record(case, &record, receipt.as_ref())?);
    }
    let maximum_records =
        usize::try_from(super::MAX_RECORD_LIMIT_COUNT).map_err(|_| "evaluation record bound is invalid".to_string())?;
    debug_assert!(priors.len() <= maximum_records);
    debug_assert_eq!(priors.len(), record_count);
    Ok(priors)
}

fn prior_from_record(
    case: &crate::agent_evaluation::model::AdmittedCase,
    record: &crate::agent_evaluation::persistence::TrialRecord,
    receipt: Option<&crate::agent_evaluation::evidence::Receipt>,
) -> Result<crate::agent_evaluation::planning::PriorTrial, String> {
    debug_assert_eq!(case.definition.id, record.plan.case_id);
    debug_assert!(receipt.is_none_or(|value| value.trial_identity == record.plan.trial_identity));
    let gate_set_identity =
        crate::agent_evaluation::identity::hash_value("lattice.agent-evaluation.gate-set.v1", &case.definition.gates)
            .map_err(|_| "evaluation gate identity failed".to_string())?;
    let reference_set_identity = crate::agent_evaluation::identity::hash_value(
        "lattice.agent-evaluation.reference-set.v1",
        &case.definition.references,
    )
    .map_err(|_| "evaluation reference identity failed".to_string())?;
    let receipt_identity = receipt.map(|value| value.identity.clone());
    Ok(crate::agent_evaluation::planning::PriorTrial {
        trial_identity: record.plan.trial_identity.clone(),
        case_revision_identity: record.plan.case_revision_identity.clone(),
        subject_identity: record.plan.subject_identity.clone(),
        agent_profile_identity: record.plan.agent_profile_identity.clone(),
        environment_identity: record.plan.environment_identity.clone(),
        policy_identity: record.plan.policy_identity.clone(),
        gate_set_identity,
        rubric_identity: case.definition.rubric.as_ref().map(|rubric| rubric.identity.clone()),
        reference_set_identity,
        artifact_policy_identity: case.definition.artifacts.identity.clone(),
        status: record.status,
        receipt_identity: receipt_identity.clone(),
        expected_receipt_identity: receipt_identity,
    })
}

fn load_receipt(
    store: &crate::store::Store,
    receipt_identity: &str,
) -> Result<crate::agent_evaluation::evidence::Receipt, String> {
    let receipt = store
        .load_evaluation_receipt(receipt_identity)
        .map_err(|_| "evaluation receipt lookup failed".to_string())?
        .ok_or_else(|| "evaluation receipt was not found".to_string())?;
    crate::agent_evaluation::evidence::validate_receipt(&receipt)
        .map_err(|_| "evaluation receipt validation failed".to_string())?;
    Ok(receipt)
}

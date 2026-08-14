pub(super) fn run_trial(
    args: &super::RunArgs,
    app_config: &crate::config::AppConfig,
    store: &crate::store::Store,
) -> Result<(), String> {
    let catalog = crate::agent_evaluation::format::load_catalog(&args.root)
        .map_err(|_| "evaluation catalog admission failed".to_string())?;
    let plan_request = crate::agent_evaluation::format::load_plan_request(&args.request)
        .map_err(|_| "evaluation plan request admission failed".to_string())?;
    let plan = crate::agent_evaluation::planning::plan_matrix(&catalog, &plan_request)
        .map_err(|_| "evaluation planning failed".to_string())?;
    debug_assert_eq!(plan.catalog_identity, catalog.identity);
    debug_assert!(!plan.trials.is_empty());
    let trial = plan
        .trials
        .iter()
        .find(|trial| trial.trial_identity == args.trial)
        .ok_or_else(|| "selected evaluation trial is absent from the exact plan".to_string())?;
    if trial.live && !args.confirm_live {
        return super::planning::render_matrix(&plan, args.output);
    }
    if !trial.live && args.confirm_live {
        return Err("live confirmation is invalid for a local trial".to_string());
    }
    let live_authority = plan_request.live_authority.iter().find(|fact| fact.case_id == trial.case_id);
    let host = build_host(args, app_config)?;
    let services = crate::agent_evaluation::shell::Services {
        workspace: &host,
        agent: &host,
        workflow: &host,
        verifier: &host,
        judge: &host,
        deployment: &host,
        cleanup: &host,
        artifact: &host,
        clock: &host,
        store,
    };
    let receipt = crate::agent_evaluation::shell::run_trial(
        &crate::agent_evaluation::shell::RunRequest {
            catalog: &catalog,
            trial,
            policy_identity: &args.policy_identity,
            live_authority,
        },
        &services,
    )
    .map_err(|_| "evaluation execution did not produce a terminal receipt".to_string())?;
    crate::agent_evaluation::evidence::validate_receipt(&receipt)
        .map_err(|_| "evaluation receipt validation failed".to_string())?;
    render_receipt(&receipt, args.output)
}

fn build_host(
    args: &super::RunArgs,
    app_config: &crate::config::AppConfig,
) -> Result<crate::agent_evaluation::host::Driver<crate::agent_evaluation::host::ProductionDriverProcess>, String> {
    let program = args.driver.to_str().ok_or_else(|| "evaluation driver path is invalid".to_string())?.to_string();
    let current_directory = args
        .current_directory
        .to_str()
        .ok_or_else(|| "evaluation driver directory is invalid".to_string())?
        .to_string();
    if !args.driver.is_absolute() || !args.current_directory.is_absolute() {
        return Err("evaluation driver paths must be absolute".to_string());
    }
    debug_assert!(args.driver.is_absolute());
    debug_assert!(args.current_directory.is_absolute());
    let config = crate::agent_evaluation::host::DriverConfig {
        driver_identity: args.driver_identity.clone(),
        agent_profile_identity: args.agent_profile_identity.clone(),
        environment_identity: args.environment_identity.clone(),
        policy_identity: args.policy_identity.clone(),
        program,
        args: args.driver_args.clone(),
        current_directory,
        timeout_secs: args.timeout_secs,
        output_max_bytes: args.output_max_bytes,
    };
    let process_config = crate::process::core::RuntimeConfig {
        shell_program: app_config.shell.clone(),
        limits: app_config.process_limits.clone(),
    };
    crate::agent_evaluation::host::Driver::new_production(
        config,
        [args.started_epoch_ms, args.completed_epoch_ms],
        process_config,
    )
    .map_err(|_| "evaluation command driver admission failed".to_string())
}

fn render_receipt(
    receipt: &crate::agent_evaluation::evidence::Receipt,
    output: super::OutputFormat,
) -> Result<(), String> {
    match output {
        super::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(receipt).map_err(|_| "evaluation receipt output failed")?)
        }
        super::OutputFormat::Text => println!(
            "trial={} case={} status={:?} receipt={} artifacts={} bytes={} redaction={}",
            receipt.trial_identity,
            receipt.case_id,
            receipt.status,
            receipt.identity,
            receipt.artifact_count,
            receipt.artifact_bytes,
            receipt.redaction
        ),
    }
    debug_assert!(!receipt.identity.is_empty());
    debug_assert_eq!(receipt.redaction, "metadata_only");
    Ok(())
}

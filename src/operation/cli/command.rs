pub fn run(app_config: &crate::config::AppConfig, args: &CommandArgs) -> std::process::ExitCode {
    match run_inner(app_config, args) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("operation command failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run_inner(app_config: &crate::config::AppConfig, args: &CommandArgs) -> Result<(), super::Error> {
    match &args.command {
        Command::Catalog(source) => print_json(&catalog_report(&source.catalog, &source.instances)?),
        Command::Show(show) => render_show(show),
        Command::Methods(methods) => render_methods(methods),
        Command::Plan(plan_args) => print_json(&plan_from_args(plan_args)?.plan),
        Command::Run(run_args) => run_invocation(app_config, run_args),
        Command::Promote(promote_args) => promote_invocation(promote_args),
    }
}

fn render_show(args: &ShowArgs) -> Result<(), super::Error> {
    let report = catalog_report(&args.source.catalog, &args.source.instances)?;
    if let Some(operation_type) = report.types.iter().find(|operation_type| operation_type.id == args.id) {
        return print_json(operation_type);
    }
    if let Some(instance) = report.instances.iter().find(|instance| instance.id == args.id) {
        return print_json(instance);
    }
    Err(super::Error::Invalid("operation metadata id was not found".to_string()))
}

fn render_methods(args: &MethodsArgs) -> Result<(), super::Error> {
    let report = catalog_report(&args.source.catalog, &args.source.instances)?;
    let operation_type = report
        .types
        .iter()
        .find(|operation_type| operation_type.id == args.type_id)
        .ok_or_else(|| super::Error::Invalid("operation type id was not found".to_string()))?;
    print_json(&operation_type.methods)
}

fn plan_from_args(args: &PlanArgs) -> Result<AdmittedPlan, super::Error> {
    admit(PlanInput {
        catalog_path: &args.source.catalog,
        instances_path: &args.source.instances,
        instance_id: &args.instance,
        method: &args.method,
        arguments_json: &args.arguments_json,
        executable_path: &args.executable,
    })
}

fn run_invocation(app_config: &crate::config::AppConfig, args: &RunArgs) -> Result<(), super::Error> {
    let admitted = plan_from_args(&args.plan)?;
    if !args.confirm_read_only {
        return print_json(&serde_json::json!({
            "preview": admitted.plan,
            "execution_authority_granted": false,
            "required_flag": "--confirm-read-only"
        }));
    }
    std::fs::create_dir_all(&app_config.data_dir)
        .map_err(|error| super::Error::Io(format!("operation data directory creation failed: {error}")))?;
    let store = crate::store::Store::new(crate::config::store_path(&app_config.data_dir))
        .map_err(|error| super::Error::Store(error.to_string()))?;
    let credentials = super::shell::environment_provider_for_plan(&admitted.plan);
    let authority =
        super::shell::LocalReadOnlyAuthorityProvider::new(&admitted.catalog, &admitted.instance, &admitted.plan)?;
    let process = crate::process::core::RuntimeConfig::default();
    let result = super::shell::run_direct(super::shell::DirectRunRequest {
        store: &store,
        plan: &admitted.plan,
        authority_provider: &authority,
        process: &process,
        credentials: &credentials,
        clock: &super::shell::SystemClock,
        cancellation: None,
    })?;
    let is_completed = result.record.state == super::run_model::RunState::Completed;
    print_json(&result)?;
    if is_completed {
        Ok(())
    } else {
        Err(super::Error::Runtime("operation finished without a confirmed completed outcome".to_string()))
    }
}

fn promote_invocation(args: &PromoteArgs) -> Result<(), super::Error> {
    let admitted = plan_from_args(&args.plan)?;
    let result = super::promotion::create(&admitted.plan, &args.workflow_name, &args.destination, args.confirm_create)?;
    print_json(&result)
}

fn print_json(value: &impl Serialize) -> Result<(), super::Error> {
    let output = serde_json::to_string_pretty(value).map_err(|_| super::Error::Encoding)?;
    println!("{output}");
    Ok(())
}

fn repository_root_admitted(instance: &super::model::ValidatedInstance) -> bool {
    let Some(root) = instance
        .definition
        .target
        .get(super::runtime::REPOSITORY_ROOT_TARGET)
        .and_then(serde_json::Value::as_str)
    else {
        return false;
    };
    let path = std::path::Path::new(root);
    if !path.is_absolute() {
        return false;
    }
    std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
}

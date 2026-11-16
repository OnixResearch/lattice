const PROMPT_PLACEHOLDER: &str = "{prompt}";
const MODEL_PLACEHOLDER: &str = "{model}";
const SESSION_PLACEHOLDER: &str = "{session}";
const OUTPUT_MODE_PLACEHOLDER: &str = "{output_mode}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPlan {
    pub target: super::super::Target,
    pub adapter_label: &'static str,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub output_mode: super::super::OutputMode,
    pub timeout_secs: u64,
    pub max_output_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

// r[impl n8nclone.workflow_execution.agent_nodes.production_adapters.command]
// r[impl n8nclone.workflow_execution.runtime_registry.command_runtime]
// r[impl n8nclone.workflow_execution.agent_nodes.output_file_handoff.contract]
pub fn plan_command(
    config: &super::super::RuntimeOptions,
    request: &super::super::Request,
) -> Result<CommandPlan, super::super::Error> {
    let runtime = config_for_target(config, request.target)?;
    let program = program_for_target(runtime, request.target)?;
    let projection = config.structured_input_projection.for_target(request.target);
    let prompt = super::super::render_with_projection(request, projection)?;
    let args = command_args_for_request(runtime, request, &prompt);
    Ok(CommandPlan {
        target: request.target,
        adapter_label: super::super::target_label(request.target),
        program,
        args,
        cwd: request.cwd.clone(),
        output_mode: request.output_mode,
        timeout_secs: config.local_process.limits.command_agent_timeout_secs,
        max_output_bytes: config.max_output_bytes,
    })
}

fn config_for_target(
    config: &super::super::RuntimeOptions,
    target: super::super::Target,
) -> Result<&super::super::CommandOptions, super::super::Error> {
    let target_label = super::super::target_label(target);
    config.runtimes.get(target_label).ok_or_else(|| {
        super::super::Error::PreflightRejected(format!("missing {target_label} agent runtime configuration"))
    })
}

fn program_for_target(
    runtime: &super::super::CommandOptions,
    target: super::super::Target,
) -> Result<String, super::super::Error> {
    let Some(command) = non_empty_trimmed(&runtime.command) else {
        return Err(super::super::Error::PreflightRejected(format!(
            "missing {} agent runtime command",
            super::super::target_label(target)
        )));
    };
    Ok(command.to_string())
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

fn command_args_for_request(
    runtime: &super::super::CommandOptions,
    request: &super::super::Request,
    prompt: &str,
) -> Vec<String> {
    runtime.args.iter().filter_map(|arg| render_command_arg_template(arg, request, prompt)).collect()
}

fn render_command_arg_template(
    template: impl AsRef<str>,
    request: &super::super::Request,
    prompt: &str,
) -> Option<String> {
    let rendered = template
        .as_ref()
        .replace(PROMPT_PLACEHOLDER, prompt)
        .replace(MODEL_PLACEHOLDER, request.model_hint.as_deref().unwrap_or_default())
        .replace(SESSION_PLACEHOLDER, request.session.as_deref().unwrap_or_default())
        .replace(OUTPUT_MODE_PLACEHOLDER, request.output_mode.as_str());
    non_empty_trimmed(&rendered).map(str::to_string)
}

// r[impl lattice.agent_runtime.bounded_command_process]
pub(super) fn run_command(
    plan: &CommandPlan,
    config: &crate::process::core::RuntimeConfig,
    context: &super::super::ExecutionContext,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
) -> Result<CommandOutput, super::super::Error> {
    let mut request = super::host::ExecutionProcessRequest(process_request(plan));
    request.0.env.clone_from(&context.secret_env);
    let runner = super::super::ProductionProcessRunner::new(config.clone());
    let output = match cancellation {
        Some(signal) => runner.run_with_cancellation(&request.0, signal),
        None => super::super::ProcessRunner::run(&runner, &request.0),
    }
    .map_err(|error| super::super::Error::Adapter(context.redact(&error.to_string())))?;
    Ok(decode_process_output(output))
}

// r[impl n8nclone.workflow_execution.trait_boundaries.process_runner]
pub fn run_command_with_runner(
    plan: &CommandPlan,
    runner: &impl super::super::ProcessRunner,
) -> Result<CommandOutput, super::super::Error> {
    let request = process_request(plan);
    let output = runner.run(&request).map_err(|error| {
        super::super::Error::Adapter(format!(
            "failed to run {} agent command '{}': {error}",
            plan.adapter_label, plan.program
        ))
    })?;
    Ok(decode_process_output(output))
}

fn process_request(plan: &CommandPlan) -> super::super::ProcessRequest {
    super::super::ProcessRequest {
        program: plan.program.clone(),
        args: plan.args.clone(),
        cwd: plan.cwd.clone(),
        env: std::collections::BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: plan.timeout_secs,
        output_limit_bytes: plan.max_output_bytes,
    }
}

fn decode_process_output(output: super::super::ProcessOutput) -> CommandOutput {
    CommandOutput {
        status_code: output.status_code,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

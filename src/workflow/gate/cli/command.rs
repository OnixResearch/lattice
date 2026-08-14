use std::io::Write;

const DEFAULT_APPROVAL_LIST_LIMIT: u32 = 20;
const HEX_RADIX: u32 = 16;
const HEX_CHARS_PER_BYTE: usize = 2;

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    command: Action,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    /// List bounded approval metadata without workflow values or outputs.
    List {
        #[arg(long = "limit", default_value_t = DEFAULT_APPROVAL_LIST_LIMIT)]
        count: u32,
        #[arg(long, default_value = "text")]
        output: String,
    },
    /// Show one approval without workflow values or outputs.
    Show {
        request: String,
        #[arg(long, default_value = "text")]
        output: String,
    },
    /// Export a metadata-only approval receipt.
    Receipt {
        request: String,
        #[arg(long, default_value = "text")]
        output: String,
    },
    /// Record one approval decision without resuming execution.
    Decide {
        request: String,
        #[arg(long)]
        workflow_revision: String,
        #[arg(long)]
        actor: String,
        #[arg(long)]
        decision: String,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        observed_epoch_ms: i64,
        #[arg(long, default_value = "text")]
        output: String,
    },
}

#[derive(clap::Args, Debug)]
pub struct ResumeGroupArgs {
    #[command(subcommand)]
    command: ResumeAction,
}

#[derive(clap::Subcommand, Debug)]
enum ResumeAction {
    /// Resume only remaining nodes from one exact durable approval checkpoint.
    Resume {
        run_id: u64,
        #[arg(long)]
        request: String,
        #[arg(long)]
        workflow_revision: String,
        #[arg(long)]
        event_head: String,
        #[arg(long)]
        owner: String,
        #[arg(long = "value")]
        values: Vec<String>,
        #[arg(long = "credential")]
        credentials: Vec<String>,
        #[arg(long)]
        observed_epoch_ms: i64,
        #[arg(long, default_value = "text")]
        output: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

pub fn run_approvals(config: &crate::config::AppConfig, args: &CommandArgs) -> std::process::ExitCode {
    let store = match open_store(config) {
        Ok(store) => store,
        Err(error) => {
            return fail(FailureArgs {
                prefix: "Approval command failed",
                error: &error,
            });
        }
    };
    let rendered = match &args.command {
        Action::List { count, output } => {
            parse_output_format(output).and_then(|format| render_list(&store, *count, format))
        }
        Action::Show { request, output } => {
            parse_output_format(output).and_then(|format| render_show(&store, request, format))
        }
        Action::Receipt { request, output } => {
            parse_output_format(output).and_then(|format| render_receipt(&store, request, format))
        }
        Action::Decide {
            request,
            workflow_revision,
            actor,
            decision,
            reason,
            observed_epoch_ms,
            output,
        } => parse_output_format(output).and_then(|format| {
            render_decision(&store, DecisionArgs {
                request,
                workflow_revision,
                actor,
                decision,
                reason: reason.clone(),
                observed_epoch_ms: *observed_epoch_ms,
                output: format,
            })
        }),
    };
    const { assert!(DEFAULT_APPROVAL_LIST_LIMIT > 0) };
    debug_assert!(matches!(
        &args.command,
        Action::List { .. } | Action::Show { .. } | Action::Receipt { .. } | Action::Decide { .. }
    ));
    write_result("Approval command failed", rendered)
}

pub async fn run_resume(config: &crate::config::AppConfig, args: &ResumeGroupArgs) -> std::process::ExitCode {
    let ResumeAction::Resume {
        run_id,
        request,
        workflow_revision,
        event_head,
        owner,
        values,
        credentials,
        observed_epoch_ms,
        output,
    } = &args.command;
    let prepared = prepare_resume(
        config,
        ResumeArgs {
            run_id: *run_id,
            request,
            workflow_revision,
            event_head,
            owner,
            values,
            credentials,
            observed_epoch_ms: *observed_epoch_ms,
        },
        output,
    );
    let (store, command, output) = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            return fail(FailureArgs {
                prefix: "Workflow resume failed",
                error: &error,
            });
        }
    };
    debug_assert_eq!(command.request_id.as_str(), request);
    debug_assert_eq!(command.expected_workflow_revision.as_str(), workflow_revision);
    let execution = super::shell::resume_with_authority_provider(
        &store,
        &command,
        config.effective_agent_runtime(),
        &crate::engine::effect::execution::UnavailableRunContextSource,
    )
    .await;
    match execution {
        Ok(execution) => finish_resume(&execution, output),
        Err(error) => fail(FailureArgs {
            prefix: "Workflow resume failed",
            error: &error.to_string(),
        }),
    }
}

fn prepare_resume(
    config: &crate::config::AppConfig,
    args: ResumeArgs<'_>,
    output: &str,
) -> Result<(crate::store::Store, super::ResumeCommand, OutputFormat), String> {
    let output = parse_output_format(output).map_err(str::to_string)?;
    let store = open_store(config)?;
    let command = resume_command(&store, args).map_err(str::to_string)?;
    Ok((store, command, output))
}

fn finish_resume(execution: &super::shell::ResumeExecution, output: OutputFormat) -> std::process::ExitCode {
    let rendered = match output {
        OutputFormat::Text => {
            let mut text = crate::headless::render_run_summary_text(&execution.summary);
            text.push('\n');
            if let Some(next) = &execution.next_request {
                text.push_str(&format!("next approval request={next}\n"));
            }
            Ok(text)
        }
        OutputFormat::Json => serde_json::to_string(execution).map_err(|_| "workflow resume output failed"),
    };
    debug_assert!(execution.summary.completed <= execution.summary.node_count);
    debug_assert!(execution.summary.failures <= execution.summary.node_count);
    match write_result("Workflow resume failed", rendered) {
        code if code == std::process::ExitCode::SUCCESS => {
            match crate::headless::process_result_for_summary(&execution.summary) {
                crate::headless::HeadlessProcessResult::Success => std::process::ExitCode::SUCCESS,
                crate::headless::HeadlessProcessResult::Failure => std::process::ExitCode::FAILURE,
            }
        }
        code => code,
    }
}

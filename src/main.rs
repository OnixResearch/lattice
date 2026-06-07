use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use n8nclone::config;
use n8nclone::headless;
use n8nclone::headless::HeadlessProcessResult;
use n8nclone::n8n_json;
use n8nclone::scheduler;
use n8nclone::scheduler::ScheduleDaemonOutcome;
use n8nclone::scheduler::ScheduleSkipReason;
use n8nclone::store::Store;
use n8nclone::triggers::TriggerDefinition;
use n8nclone::triggers::TriggerRunRecord;
use n8nclone::tui::App;
use n8nclone::workflow_format;
use n8nclone::workflow_validation;
use tracing::info;

const DEFAULT_CONFIG_PATH: &str = "~/.n8nclone/config.ncl";
const LOG_FILE_PATH: &str = "/tmp/n8nclone.log";
const STORE_FILE_NAME: &str = "n8nclone.redb";
const DEFAULT_TRIGGER_HISTORY_LIMIT: usize = 20;

#[derive(Parser, Debug)]
#[command(name = "n8nclone", about = "TUI workflow automation — like n8n, in your terminal")]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Config file path (Nickel .ncl)
    #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,

    /// Initialize config directory with defaults
    #[arg(long)]
    init: bool,

    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    /// Run a saved workflow without starting the TUI
    Run(RunCommandArgs),
    /// Export a saved workflow to a typed Nickel file
    Export(ExportWorkflowArgs),
    /// Import a typed Nickel workflow file into the store
    Import(ImportWorkflowArgs),
    /// Import an n8n workflow JSON file into the store
    ImportN8n(ImportN8nWorkflowArgs),
    /// Export a saved workflow to n8n workflow JSON
    ExportN8n(ExportN8nWorkflowArgs),
    /// Run one scheduler daemon evaluation without starting the TUI
    SchedulerDaemon(SchedulerDaemonArgs),
    /// Inspect persisted triggers and trigger run history
    Triggers(TriggersCommandArgs),
}

#[derive(clap::Args, Debug)]
struct RunCommandArgs {
    /// Saved workflow name to run
    workflow: String,

    /// Output format for the run summary
    #[arg(long, value_enum, default_value_t = RunOutputFormat::Text)]
    output: RunOutputFormat,
}

#[derive(clap::Args, Debug)]
struct ExportWorkflowArgs {
    /// Saved workflow name to export
    workflow: String,

    /// Nickel file to write
    output: PathBuf,
}

#[derive(clap::Args, Debug)]
struct ImportWorkflowArgs {
    /// Nickel workflow file to import
    path: PathBuf,
}

#[derive(clap::Args, Debug)]
struct ImportN8nWorkflowArgs {
    /// n8n workflow JSON file to import
    path: PathBuf,

    /// Mapping report JSON file to write
    #[arg(long)]
    report: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct ExportN8nWorkflowArgs {
    /// Saved workflow name to export
    workflow: String,

    /// n8n workflow JSON file to write
    output: PathBuf,

    /// Mapping report JSON file to write
    #[arg(long)]
    report: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct SchedulerDaemonArgs {
    /// Explicit current time for deterministic daemon evaluation
    #[arg(long)]
    now_epoch_ms: Option<i64>,
}

#[derive(clap::Args, Debug)]
struct TriggersCommandArgs {
    #[command(subcommand)]
    command: TriggerCliCommand,
}

#[derive(Subcommand, Debug)]
enum TriggerCliCommand {
    /// List persisted trigger definitions
    List,
    /// List recent trigger run history
    History {
        /// Maximum trigger run records to print
        #[arg(long, default_value_t = DEFAULT_TRIGGER_HISTORY_LIMIT)]
        limit: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum RunOutputFormat {
    Text,
    Json,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run_main().await {
        Ok(exit_code) => exit_code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn run_main() -> Result<ExitCode, n8nclone::error::Error> {
    let args = Args::parse();
    setup_logging(args.verbose);

    let config_path = shellexpand(&args.config);

    if args.init {
        init_config(&config_path)?;
        println!("Initialized config at {}", config_path.display());
        return Ok(ExitCode::SUCCESS);
    }

    let app_config = config::load_app_config(&config_path);

    if let Err(e) = std::fs::create_dir_all(&app_config.data_dir) {
        tracing::warn!(error = %e, "failed to create data dir");
    }

    info!(config = ?config_path, data_dir = ?app_config.data_dir, "starting n8nclone");

    match args.command {
        Some(CliCommand::Run(run_args)) => Ok(run_headless_command(&app_config, &run_args).await),
        Some(CliCommand::Export(export_args)) => Ok(export_workflow_command(&app_config, &export_args)),
        Some(CliCommand::Import(import_args)) => Ok(import_workflow_command(&app_config, &import_args)),
        Some(CliCommand::ImportN8n(import_args)) => Ok(import_n8n_workflow_command(&app_config, &import_args)),
        Some(CliCommand::ExportN8n(export_args)) => Ok(export_n8n_workflow_command(&app_config, &export_args)),
        Some(CliCommand::SchedulerDaemon(scheduler_args)) => {
            run_scheduler_daemon_once(&app_config, &scheduler_args).await
        }
        Some(CliCommand::Triggers(trigger_args)) => Ok(run_triggers_command(&app_config, &trigger_args)),
        None => {
            let templates_dir = config_path.parent().unwrap_or_else(|| std::path::Path::new(".")).join("templates");
            let template_load = config::load_node_templates_with_diagnostics(&templates_dir);
            let mut app = App::new(app_config);
            app.set_node_templates(template_load.templates);
            for diagnostic in template_load.diagnostics {
                app.output.push_line(&format!(
                    "Template diagnostic {}: {}",
                    diagnostic.path.display(),
                    diagnostic.message
                ));
            }
            app.run_tui().await.map_err(|e| n8nclone::error::Error::Io { source: e })?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn setup_logging(verbose: bool) {
    let log_file = std::fs::File::create(LOG_FILE_PATH).ok();
    if let Some(file) = log_file {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(if verbose {
                tracing::Level::DEBUG.into()
            } else {
                tracing::Level::INFO.into()
            }))
            .with_writer(file)
            .init();
    }
}

async fn run_headless_command(app_config: &config::AppConfig, args: &RunCommandArgs) -> ExitCode {
    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Run failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let run_id = match store.next_run_id() {
        Ok(run_id) => run_id,
        Err(error) => {
            eprintln!("Run failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    match headless::run_saved_workflow_with_agent_runtime_config(
        &store,
        &args.workflow,
        run_id,
        app_config.agent_runtime.clone(),
    )
    .await
    {
        Ok(summary) => {
            match args.output {
                RunOutputFormat::Text => println!("{}", headless::render_run_summary_text(&summary)),
                RunOutputFormat::Json => match serde_json::to_string(&summary) {
                    Ok(json) => println!("{json}"),
                    Err(error) => {
                        eprintln!("Run failed: {error}");
                        return ExitCode::FAILURE;
                    }
                },
            }
            process_result_to_exit_code(headless::process_result_for_summary(&summary))
        }
        Err(error) => {
            print_headless_run_error(&error);
            ExitCode::FAILURE
        }
    }
}

fn print_headless_run_error(error: &headless::HeadlessRunError) {
    eprintln!("Run failed: {error}");
    if let headless::HeadlessRunError::Validation(diagnostics) = error {
        for diagnostic in diagnostics {
            eprintln!("  {}", workflow_validation::format_diagnostic(diagnostic));
        }
    }
}

// r[impl n8nclone.scheduling.daemon.run_due]
async fn run_scheduler_daemon_once(
    app_config: &config::AppConfig,
    args: &SchedulerDaemonArgs,
) -> Result<ExitCode, n8nclone::error::Error> {
    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Scheduler failed: {error}");
            return Ok(ExitCode::FAILURE);
        }
    };
    let schedules = match store.list_schedules() {
        Ok(schedules) => schedules,
        Err(error) => {
            eprintln!("Scheduler failed: {error}");
            return Ok(ExitCode::FAILURE);
        }
    };
    let now_epoch_ms = args.now_epoch_ms.unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
    let evaluation = scheduler::evaluate_due_runs(&schedules, now_epoch_ms, &std::collections::HashSet::new());

    for skipped in evaluation.skipped_runs {
        if skipped.reason == ScheduleSkipReason::MissedRunSkipped {
            update_schedule_next_run(&store, &skipped.schedule_id, skipped.next_run_epoch_ms);
        }
        let outcome = ScheduleDaemonOutcome::Skipped(skipped);
        println!("{}", scheduler::render_daemon_outcome(&outcome));
    }

    for plan in evaluation.due_runs {
        let run_id = match store.next_run_id() {
            Ok(run_id) => run_id,
            Err(error) => {
                let outcome = ScheduleDaemonOutcome::Error {
                    schedule_id: plan.schedule_id.clone(),
                    workflow_name: plan.workflow_name.clone(),
                    message: error.to_string(),
                    next_run_epoch_ms: plan.next_run_epoch_ms,
                };
                println!("{}", scheduler::render_daemon_outcome(&outcome));
                continue;
            }
        };
        let outcome = match headless::run_saved_workflow_with_agent_runtime_config(
            &store,
            &plan.workflow_name,
            run_id,
            app_config.agent_runtime.clone(),
        )
        .await
        {
            Ok(summary) => ScheduleDaemonOutcome::Started {
                schedule_id: plan.schedule_id.clone(),
                workflow_name: plan.workflow_name.clone(),
                summary: Box::new(summary),
                next_run_epoch_ms: plan.next_run_epoch_ms,
            },
            Err(error) => ScheduleDaemonOutcome::Error {
                schedule_id: plan.schedule_id.clone(),
                workflow_name: plan.workflow_name.clone(),
                message: error.to_string(),
                next_run_epoch_ms: plan.next_run_epoch_ms,
            },
        };
        update_schedule_next_run(&store, &plan.schedule_id, plan.next_run_epoch_ms);
        println!("{}", scheduler::render_daemon_outcome(&outcome));
    }

    Ok(ExitCode::SUCCESS)
}

fn update_schedule_next_run(store: &Store, schedule_id: &str, next_run_epoch_ms: i64) {
    let Ok(Some(mut schedule)) = store.load_schedule(schedule_id) else {
        return;
    };
    schedule.next_run_epoch_ms = next_run_epoch_ms;
    if let Err(error) = store.save_schedule(&schedule) {
        eprintln!("Scheduler failed to update schedule {schedule_id}: {error}");
    }
}

// r[impl n8nclone.tui_ux.triggers.manage]
fn run_triggers_command(app_config: &config::AppConfig, args: &TriggersCommandArgs) -> ExitCode {
    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Triggers failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    match &args.command {
        TriggerCliCommand::List => match store.list_triggers() {
            Ok(triggers) => {
                print!("{}", render_trigger_list(&triggers));
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("Triggers failed: {error}");
                ExitCode::FAILURE
            }
        },
        TriggerCliCommand::History { limit } => match store.list_trigger_runs(*limit) {
            Ok(records) => {
                print!("{}", render_trigger_history(&records));
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("Triggers failed: {error}");
                ExitCode::FAILURE
            }
        },
    }
}

fn render_trigger_list(triggers: &[TriggerDefinition]) -> String {
    if triggers.is_empty() {
        return "No triggers configured\n".to_string();
    }
    let mut output = String::new();
    for trigger in triggers {
        output.push_str(&format!(
            "{} | {} | {:?} | {}\n",
            trigger.id,
            trigger.workflow_name,
            trigger.kind,
            if trigger.enabled { "enabled" } else { "disabled" },
        ));
    }
    output
}

fn render_trigger_history(records: &[TriggerRunRecord]) -> String {
    if records.is_empty() {
        return "No trigger runs recorded\n".to_string();
    }
    let mut output = String::new();
    for record in records {
        let run_id = record.run_id.map(|run_id| run_id.to_string()).unwrap_or_else(|| "-".to_string());
        output.push_str(&format!(
            "{} | {} | {} | {:?} | {}\n",
            record.trigger_id, record.workflow_name, run_id, record.status, record.message,
        ));
    }
    output
}

// r[impl n8nclone.workflow_format.nickel.export]
fn export_workflow_command(app_config: &config::AppConfig, args: &ExportWorkflowArgs) -> ExitCode {
    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Export failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    let workflow = match store.load_workflow(&args.workflow) {
        Ok(Some(workflow)) => workflow,
        Ok(None) => {
            eprintln!("Export failed: workflow not found: {}", args.workflow);
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("Export failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    if let Some(parent) = args.output.parent().filter(|parent| !parent.as_os_str().is_empty())
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        eprintln!("Export failed: {error}");
        return ExitCode::FAILURE;
    }

    let nickel = workflow_format::workflow_to_nickel(&workflow);
    match std::fs::write(&args.output, nickel) {
        Ok(()) => {
            println!("Exported '{}' to {}", workflow.meta.name, args.output.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Export failed: {error}");
            ExitCode::FAILURE
        }
    }
}

// r[impl n8nclone.workflow_format.nickel.import]
fn import_workflow_command(app_config: &config::AppConfig, args: &ImportWorkflowArgs) -> ExitCode {
    let workflow = match workflow_format::load_workflow_nickel_file(&args.path) {
        Ok(workflow) => workflow,
        Err(error) => {
            eprintln!("Import failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Import failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    match store.save_workflow(&workflow) {
        Ok(()) => {
            println!("Imported '{}'", workflow.meta.name);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Import failed: {error}");
            ExitCode::FAILURE
        }
    }
}

// r[impl n8nclone.workflow_format.n8n_json.import_cli]
fn import_n8n_workflow_command(app_config: &config::AppConfig, args: &ImportN8nWorkflowArgs) -> ExitCode {
    let input = match std::fs::read_to_string(&args.path) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("n8n import failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let result = n8n_json::import_n8n_json_str(&input);
    if let Some(report_path) = &args.report
        && let Err(error) = write_n8n_report(report_path, &result.report)
    {
        eprintln!("n8n import failed: {error}");
        return ExitCode::FAILURE;
    }
    let Some(workflow) = result.workflow else {
        eprintln!("n8n import failed with {} diagnostics", result.report.diagnostics.len());
        return ExitCode::FAILURE;
    };
    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("n8n import failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    match store.save_workflow(&workflow) {
        Ok(()) => {
            println!("Imported n8n workflow '{}'", workflow.meta.name);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("n8n import failed: {error}");
            ExitCode::FAILURE
        }
    }
}

// r[impl n8nclone.workflow_format.n8n_json.export]
fn export_n8n_workflow_command(app_config: &config::AppConfig, args: &ExportN8nWorkflowArgs) -> ExitCode {
    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("n8n export failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let workflow = match store.load_workflow(&args.workflow) {
        Ok(Some(workflow)) => workflow,
        Ok(None) => {
            eprintln!("n8n export failed: workflow not found: {}", args.workflow);
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("n8n export failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let result = n8n_json::export_workflow_to_n8n_json(&workflow);
    if let Some(report_path) = &args.report
        && let Err(error) = write_n8n_report(report_path, &result.report)
    {
        eprintln!("n8n export failed: {error}");
        return ExitCode::FAILURE;
    }
    let Some(json) = result.json else {
        eprintln!("n8n export failed with {} diagnostics", result.report.diagnostics.len());
        return ExitCode::FAILURE;
    };
    if let Err(error) = write_output_file(&args.output, n8n_json::n8n_json_to_pretty_string(&json)) {
        eprintln!("n8n export failed: {error}");
        return ExitCode::FAILURE;
    }
    println!("Exported '{}' to n8n JSON {}", workflow.meta.name, args.output.display());
    ExitCode::SUCCESS
}

fn write_n8n_report(path: &PathBuf, report: &n8n_json::N8nMappingReport) -> Result<(), String> {
    write_output_file(path, n8n_json::report_to_pretty_json(report))
}

fn write_output_file(path: &PathBuf, contents: String) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(path, contents).map_err(|error| error.to_string())
}

fn open_store(app_config: &config::AppConfig) -> Result<Store, String> {
    Store::new(app_config.data_dir.join(STORE_FILE_NAME)).map_err(|error| error.to_string())
}

fn process_result_to_exit_code(result: HeadlessProcessResult) -> ExitCode {
    match result {
        HeadlessProcessResult::Success => ExitCode::SUCCESS,
        HeadlessProcessResult::Failure => ExitCode::FAILURE,
    }
}

fn shellexpand(path: &str) -> PathBuf {
    if let Some(stripped_path) = path.strip_prefix("~/")
        && let Some(home) = home::home_dir()
    {
        return home.join(stripped_path);
    }
    PathBuf::from(path)
}

fn init_config(config_path: &PathBuf) -> Result<(), n8nclone::error::Error> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| n8nclone::error::Error::Io { source: e })?;
    }

    std::fs::write(config_path, config::default_config_ncl()).map_err(|e| n8nclone::error::Error::Io { source: e })?;

    let templates_dir = config_path.parent().unwrap_or(&PathBuf::from(".")).join("templates");
    std::fs::create_dir_all(&templates_dir).map_err(|e| n8nclone::error::Error::Io { source: e })?;
    for template_file in config::DEFAULT_NODE_TEMPLATE_FILES {
        std::fs::write(templates_dir.join(template_file.file_name), template_file.contents)
            .map_err(|e| n8nclone::error::Error::Io { source: e })?;
    }

    Ok(())
}

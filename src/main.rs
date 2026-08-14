use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use lattice::config;
use lattice::credentials::CredentialProviderCapabilities;
use lattice::credentials::CredentialProviderMetadata;
use lattice::credentials::CredentialProviderRegistry;
use lattice::credentials::EnvironmentCredentialProvider;
use lattice::headless;
use lattice::headless::HeadlessProcessResult;
use lattice::n8n_json;
use lattice::orchestration;
use lattice::scheduler;
use lattice::scheduler::ScheduleDaemonOutcome;
use lattice::scheduler::ScheduleSkipReason;
use lattice::store::Store;
use lattice::triggers::TriggerDefinition;
use lattice::triggers::TriggerRunRecord;
use lattice::tui::App;
use lattice::webhooks;
use lattice::workflow_format;
use lattice::workflow_validation;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tracing::info;

const LOG_FILE_PATH: &str = "/tmp/lattice.log";
const DEFAULT_TRIGGER_HISTORY_LIMIT: usize = 20;
const ENV_CREDENTIAL_PROVIDER_ID: &str = "env";
const ENV_CREDENTIAL_PROVIDER_NAME: &str = "Environment";
const WEBHOOK_MAX_IN_FLIGHT_CONNECTIONS: usize = 32;
const DEFAULT_SCHEDULE_OCCURRENCE_LIMIT: u64 = 20;
const MAX_SCHEDULE_OCCURRENCE_LIMIT: u64 = 100;

#[path = "doctor/core.rs"]
mod doctor;

// r[impl lattice.project_identity.canonical]
#[derive(Parser, Debug)]
#[command(name = "lattice", about = "Graph-native workflow automation in your terminal")]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Config file path (Nickel .ncl)
    #[arg(short, long)]
    config: Option<String>,

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
    /// Inspect persisted run evidence without emitting arbitrary content
    Runs(RunsCommandArgs),
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
    /// Inspect and explicitly close durable scheduled occurrences
    SchedulerOccurrences(SchedulerOccurrencesCommandArgs),
    /// Inspect persisted triggers and trigger run history
    Triggers(TriggersCommandArgs),
    /// Listen for persisted webhook trigger HTTP requests
    WebhookListen(WebhookListenArgs),
    /// Diagnose local operator configuration and readiness
    Doctor(DoctorCommandArgs),
    /// Manage durable multi-run agent orchestrations
    Orchestration(OrchestrationCommandArgs),
    /// Manage durable projected role-protocol sessions
    Protocol(lattice::protocol_cli::CommandArgs),
    /// Inspect and decide durable workflow approvals
    Approvals(lattice::workflow::gate::cli::CommandArgs),
    /// Resume a durably suspended workflow
    Workflow(lattice::workflow::gate::cli::ResumeGroupArgs),
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
struct RunsCommandArgs {
    #[command(subcommand)]
    command: RunsCliCommand,
}

#[derive(Subcommand, Debug)]
enum RunsCliCommand {
    /// Show one persisted run through a content-omitting evidence report
    Show {
        run_id: u64,
        #[arg(long, value_enum, default_value_t = RunOutputFormat::Text)]
        output: RunOutputFormat,
    },
    /// Explicitly repair an existing redb store after an unclean writer exit
    Repair,
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
struct SchedulerOccurrencesCommandArgs {
    #[command(subcommand)]
    command: SchedulerOccurrenceCliCommand,
}

#[derive(Subcommand, Debug)]
enum SchedulerOccurrenceCliCommand {
    /// List bounded durable occurrence state without workflow content
    List {
        #[arg(long, default_value_t = DEFAULT_SCHEDULE_OCCURRENCE_LIMIT)]
        limit: u64,
        #[arg(long, value_enum, default_value_t = RunOutputFormat::Text)]
        output: RunOutputFormat,
    },
    /// Show one durable occurrence by its BLAKE3 identity
    Show {
        identity: String,
        #[arg(long, value_enum, default_value_t = RunOutputFormat::Text)]
        output: RunOutputFormat,
    },
    /// Reconcile one pending occurrence from exact owned terminal run evidence
    Recover {
        identity: String,
        #[arg(long, value_enum, default_value_t = RunOutputFormat::Text)]
        output: RunOutputFormat,
    },
    /// Explicitly perform the one bounded retry despite duplicate-effect risk
    Retry {
        identity: String,
        #[arg(long)]
        acknowledge_duplicate_risk: bool,
        #[arg(long, value_enum, default_value_t = RunOutputFormat::Text)]
        output: RunOutputFormat,
    },
    /// Close one pending occurrence without inventing its unknown outcome
    Abandon {
        identity: String,
        #[arg(long)]
        acknowledge_unknown_outcome: bool,
        #[arg(long, value_enum, default_value_t = RunOutputFormat::Text)]
        output: RunOutputFormat,
    },
}

#[derive(clap::Args, Debug)]
struct TriggersCommandArgs {
    #[command(subcommand)]
    command: TriggerCliCommand,
}

#[derive(clap::Args, Debug)]
struct WebhookListenArgs {
    /// Bind address for the webhook HTTP listener
    #[arg(long, default_value = webhooks::DEFAULT_BIND_ADDR)]
    bind: String,

    /// Validate listener inputs and exit without binding a socket
    #[arg(long)]
    dry_run: bool,

    /// Maximum accepted request body size in bytes
    #[arg(long, default_value_t = webhooks::DEFAULT_MAX_BODY_BYTES)]
    max_body_bytes: usize,
}

#[derive(clap::Args, Debug)]
struct OrchestrationCommandArgs {
    #[command(subcommand)]
    command: OrchestrationCliCommand,
}

#[derive(Subcommand, Debug)]
enum OrchestrationCliCommand {
    /// Validate a typed Nickel orchestration definition
    Validate {
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Start a validated orchestration and execute its first round
    Start {
        path: PathBuf,
        #[arg(long)]
        actor: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Inspect one persisted orchestration instance
    Status {
        instance: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Pause new orchestration dispatch
    Pause {
        instance: String,
        #[arg(long)]
        actor: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Resume a paused orchestration
    Resume {
        instance: String,
        #[arg(long)]
        actor: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Request orchestration cancellation
    Cancel {
        instance: String,
        #[arg(long)]
        actor: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Approve a hash-bound Nickel proposal and execute its successor round
    Approve {
        instance: String,
        proposal: PathBuf,
        #[arg(long)]
        actor: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Reject a pending proposal
    Reject {
        instance: String,
        proposal_id: String,
        #[arg(long)]
        actor: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Replay persisted orchestration events without executing effects
    Replay {
        instance: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Reconcile a pending dispatch against persisted run history
    Recover {
        instance: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Retry an exact observed terminal failure under immutable policy and budget bounds
    Retry {
        instance: String,
        #[arg(long)]
        actor: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
    /// Fail a cancelled, locally unobserved dispatch without retrying its effect
    Abandon {
        instance: String,
        #[arg(long)]
        actor: String,
        #[arg(long, value_enum, default_value_t = OrchestrationOutputFormat::Text)]
        output: OrchestrationOutputFormat,
    },
}

#[derive(clap::Args, Debug)]
struct DoctorCommandArgs {
    /// Output format for the doctor report
    #[arg(long, value_enum, default_value_t = DoctorOutputFormat::Text)]
    output: DoctorOutputFormat,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum DoctorOutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum OrchestrationOutputFormat {
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

async fn run_main() -> Result<ExitCode, lattice::error::Error> {
    let args = Args::parse();
    setup_logging(args.verbose);

    let config_path = resolve_config_path(args.config.as_deref(), args.init);

    if args.init {
        init_config(&config_path)?;
        println!("Initialized config at {}", config_path.display());
        return Ok(ExitCode::SUCCESS);
    }

    if let Some(CliCommand::Doctor(doctor_args)) = &args.command {
        return Ok(run_doctor_command(&config_path, doctor_args));
    }

    let app_config = config::load_app_config(&config_path);

    if let Some(CliCommand::Runs(runs_args)) = &args.command {
        return Ok(run_runs_command(&app_config, runs_args));
    }

    if let Err(e) = std::fs::create_dir_all(&app_config.data_dir) {
        tracing::warn!(error = %e, "failed to create data dir");
    }

    info!(config = ?config_path, data_dir = ?app_config.data_dir, "starting lattice");

    match args.command {
        Some(CliCommand::Run(run_args)) => Ok(run_headless_command(&app_config, &run_args).await),
        Some(CliCommand::Runs(_)) => unreachable!("runs command returns before data-directory side effects"),
        Some(CliCommand::Export(export_args)) => Ok(export_workflow_command(&app_config, &export_args)),
        Some(CliCommand::Import(import_args)) => Ok(import_workflow_command(&app_config, &import_args)),
        Some(CliCommand::ImportN8n(import_args)) => Ok(import_n8n_workflow_command(&app_config, &import_args)),
        Some(CliCommand::ExportN8n(export_args)) => Ok(export_n8n_workflow_command(&app_config, &export_args)),
        Some(CliCommand::SchedulerDaemon(scheduler_args)) => {
            run_scheduler_daemon_once(&app_config, &scheduler_args).await
        }
        Some(CliCommand::SchedulerOccurrences(occurrence_args)) => {
            Ok(run_scheduler_occurrences_command(&app_config, &occurrence_args).await)
        }
        Some(CliCommand::Triggers(trigger_args)) => Ok(run_triggers_command(&app_config, &trigger_args)),
        Some(CliCommand::WebhookListen(webhook_args)) => {
            Ok(run_webhook_listener_command(&app_config, &webhook_args).await)
        }
        Some(CliCommand::Doctor(_)) => unreachable!("doctor command returns before app config side effects"),
        Some(CliCommand::Orchestration(orchestration_args)) => {
            Ok(run_orchestration_command(&app_config, &orchestration_args).await)
        }
        Some(CliCommand::Protocol(protocol_args)) => Ok(run_protocol_command(&app_config, &protocol_args)),
        Some(CliCommand::Approvals(approval_args)) => {
            Ok(lattice::workflow::gate::cli::run_approvals(&app_config, &approval_args))
        }
        Some(CliCommand::Workflow(workflow_args)) => {
            Ok(lattice::workflow::gate::cli::run_resume(&app_config, &workflow_args).await)
        }
        None => {
            let config_root = config_path.parent().unwrap_or_else(|| std::path::Path::new("."));
            let templates_dir = config_root.join("templates");
            let workflow_templates_dir = config_root.join("workflow-templates");
            let template_load = config::load_node_templates_with_diagnostics(&templates_dir);
            let workflow_template_catalog =
                lattice::workflow_template::load_catalog(&workflow_templates_dir).map_err(|error| {
                    lattice::error::Error::Config {
                        message: error.to_string(),
                    }
                })?;
            let mut app = App::new(app_config);
            app.set_node_templates(template_load.templates);
            app.set_workflow_templates(workflow_template_catalog.bundles);
            for diagnostic in template_load.diagnostics {
                app.output.push_line(&format!(
                    "Template diagnostic {}: {}",
                    diagnostic.path.display(),
                    diagnostic.message
                ));
            }
            for diagnostic in workflow_template_catalog.diagnostics {
                app.output.push_line(&format!(
                    "Workflow-template diagnostic {} {}: {}",
                    diagnostic.bundle,
                    diagnostic.code.as_str(),
                    diagnostic.message,
                ));
            }
            app.run_tui().await.map_err(|e| lattice::error::Error::Io { source: e })?;
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

// r[impl n8nclone.operator_diagnostics.doctor_command]
fn run_doctor_command(config_path: &Path, args: &DoctorCommandArgs) -> ExitCode {
    let facts = collect_doctor_facts(config_path);
    let report = doctor::evaluate(&facts);
    let rendered = match args.output {
        DoctorOutputFormat::Text => doctor::render_text(&report),
        DoctorOutputFormat::Json => doctor::render_json(&report),
    };
    if let Err(error) = write_stdout_and_flush(&rendered) {
        eprintln!("Doctor failed: {error}");
        return ExitCode::FAILURE;
    }
    if doctor::has_fatal(&report) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn load_app_config_checked_for_doctor(config_path: &Path) -> Result<config::AppConfig, String> {
    if !config_path.exists() {
        return Err("config file not found".to_string());
    }
    let value =
        lattice::nickel::evaluate_file(config_path).map_err(|error| format!("Nickel config failed: {error}"))?;
    serde_json::from_value(value).map_err(|_| "Nickel config schema decode failed".to_string())
}

// r[impl n8nclone.operator_diagnostics.doctor_checks]
fn collect_doctor_facts(config_path: &Path) -> doctor::Facts {
    let mut checks = Vec::new();
    let app_config = match load_app_config_checked_for_doctor(config_path) {
        Ok(app_config) => {
            checks.push(doctor::Fact::healthy(
                doctor::CHECK_CONFIG,
                format!("config loaded from {}", config_path.display()),
            ));
            app_config
        }
        Err(error) => {
            checks.push(
                doctor::Fact::fatal(doctor::CHECK_CONFIG, error)
                    .with_remediation("run `lattice --init --config <path>` or fix the Nickel config"),
            );
            return doctor::Facts::new(checks);
        }
    };

    let store_path = config::store_path(&app_config.data_dir);
    if !store_path.exists() {
        checks.push(
            doctor::Fact::fatal(doctor::CHECK_STORE, format!("store database not found: {}", store_path.display()))
                .with_remediation("import or run a workflow to initialize the configured store"),
        );
        return doctor::Facts::new(checks);
    }

    let store = match Store::new(&store_path) {
        Ok(store) => {
            checks
                .push(doctor::Fact::healthy(doctor::CHECK_STORE, format!("store opened at {}", store_path.display())));
            store
        }
        Err(error) => {
            checks.push(
                doctor::Fact::fatal(doctor::CHECK_STORE, format!("store open failed: {error}"))
                    .with_remediation("check store path permissions and redb file integrity"),
            );
            return doctor::Facts::new(checks);
        }
    };

    collect_workflow_doctor_facts(&store, &mut checks);
    let triggers = collect_trigger_doctor_facts(&store, &mut checks);
    collect_credential_doctor_facts(&triggers, &mut checks);
    collect_webhook_listener_doctor_facts(&triggers, &mut checks);
    doctor::Facts::new(checks)
}

// r[impl n8nclone.operator_diagnostics.doctor_checks.config_store]
fn collect_workflow_doctor_facts(store: &Store, checks: &mut Vec<doctor::Fact>) {
    let workflow_names = match store.list_workflows() {
        Ok(workflow_names) => workflow_names,
        Err(error) => {
            checks.push(
                doctor::Fact::fatal(doctor::CHECK_WORKFLOWS, format!("workflow listing failed: {error}"))
                    .with_remediation("check store integrity"),
            );
            return;
        }
    };
    let mut fatal_diagnostic_count = 0usize;
    for workflow_name in &workflow_names {
        match store.load_workflow(workflow_name) {
            Ok(Some(workflow)) => {
                let diagnostics = workflow_validation::validate_workflow(&workflow);
                if workflow_validation::has_fatal_diagnostics(&diagnostics) {
                    fatal_diagnostic_count += diagnostics.len();
                }
            }
            Ok(None) => {
                checks.push(
                    doctor::Fact::fatal(
                        doctor::CHECK_WORKFLOWS,
                        format!("workflow disappeared while checking: {workflow_name}"),
                    )
                    .with_remediation("rerun doctor or inspect the store"),
                );
            }
            Err(error) => {
                checks.push(
                    doctor::Fact::fatal(
                        doctor::CHECK_WORKFLOWS,
                        format!("workflow load failed for {workflow_name}: {error}"),
                    )
                    .with_remediation("inspect or re-import the workflow"),
                );
            }
        }
    }
    if fatal_diagnostic_count == 0 {
        checks.push(doctor::Fact::healthy(
            doctor::CHECK_WORKFLOWS,
            format!("{} workflows validated", workflow_names.len()),
        ));
    } else {
        checks.push(
            doctor::Fact::fatal(
                doctor::CHECK_WORKFLOWS,
                format!("workflow validation found {fatal_diagnostic_count} fatal diagnostics"),
            )
            .with_remediation("run workflow validation or re-import invalid workflows"),
        );
    }
}

fn collect_trigger_doctor_facts(store: &Store, checks: &mut Vec<doctor::Fact>) -> Vec<TriggerDefinition> {
    match store.list_triggers() {
        Ok(triggers) => triggers,
        Err(error) => {
            checks.push(
                doctor::Fact::fatal(doctor::CHECK_WEBHOOK_LISTENER, format!("trigger listing failed: {error}"))
                    .with_remediation("check store trigger table integrity"),
            );
            Vec::new()
        }
    }
}

// r[impl n8nclone.operator_diagnostics.doctor_checks.credentials]
fn collect_credential_doctor_facts(triggers: &[TriggerDefinition], checks: &mut Vec<doctor::Fact>) {
    let secret_refs = webhooks::secret_refs(triggers);
    let missing_env_count = secret_refs
        .iter()
        .filter(|secret_ref| secret_ref.provider == ENV_CREDENTIAL_PROVIDER_ID)
        .filter(|secret_ref| std::env::var_os(&secret_ref.key).is_none())
        .count();
    let unsupported_provider_count =
        secret_refs.iter().filter(|secret_ref| secret_ref.provider != ENV_CREDENTIAL_PROVIDER_ID).count();

    if secret_refs.is_empty() {
        checks
            .push(doctor::Fact::healthy(doctor::CHECK_CREDENTIALS, "no credential-backed webhook triggers configured"));
    } else if missing_env_count == 0 && unsupported_provider_count == 0 {
        checks.push(doctor::Fact::healthy(
            doctor::CHECK_CREDENTIALS,
            format!("{} credential references are reachable", secret_refs.len()),
        ));
    } else {
        checks.push(
            doctor::Fact::fatal(
                doctor::CHECK_CREDENTIALS,
                format!(
                    "credential readiness failed: {missing_env_count} missing env refs, {unsupported_provider_count} unsupported providers"
                ),
            )
            .with_remediation("set required environment variables or configure a supported provider"),
        );
    }
}

// r[impl n8nclone.operator_diagnostics.doctor_checks.webhook_listener]
fn collect_webhook_listener_doctor_facts(triggers: &[TriggerDefinition], checks: &mut Vec<doctor::Fact>) {
    let enabled_webhook_count = webhooks::enabled_trigger_count(triggers);
    if enabled_webhook_count == 0 {
        checks.push(doctor::Fact::healthy(
            doctor::CHECK_WEBHOOK_LISTENER,
            "no enabled webhook triggers require listener startup",
        ));
    } else {
        checks.push(doctor::Fact::healthy(
            doctor::CHECK_WEBHOOK_LISTENER,
            format!("{enabled_webhook_count} enabled webhook triggers are ready for listener dry-run"),
        ));
    }
}

// r[impl lattice.agent_orchestration.cli]
async fn run_orchestration_command(app_config: &config::AppConfig, args: &OrchestrationCommandArgs) -> ExitCode {
    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => return orchestration_failure(&error),
    };
    match &args.command {
        OrchestrationCliCommand::Validate { path, output } => {
            let definition = match orchestration::load_definition_nickel_file(path) {
                Ok(definition) => definition,
                Err(error) => return orchestration_failure(&error.to_string()),
            };
            match orchestration::validate_definition_in_store(&store, &definition) {
                Ok(report) => {
                    let valid = report.valid();
                    if render_orchestration_validation(&report, *output).is_err() || !valid {
                        ExitCode::FAILURE
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(error) => orchestration_failure(&error.to_string()),
            }
        }
        OrchestrationCliCommand::Start { path, actor, output } => {
            let definition = match orchestration::load_definition_nickel_file(path) {
                Ok(definition) => definition,
                Err(error) => return orchestration_failure(&error.to_string()),
            };
            match orchestration::start(&store, definition, actor, app_config.effective_agent_runtime()).await {
                Ok(instance) => render_orchestration_instance(&instance, *output),
                Err(error) => orchestration_failure(&error.to_string()),
            }
        }
        OrchestrationCliCommand::Status { instance, output } => match orchestration::load_instance(&store, instance) {
            Ok(instance) => render_orchestration_instance(&instance, *output),
            Err(error) => orchestration_failure(&error.to_string()),
        },
        OrchestrationCliCommand::Pause {
            instance,
            actor,
            output,
        } => match orchestration::pause(&store, instance, actor) {
            Ok(instance) => render_orchestration_instance(&instance, *output),
            Err(error) => orchestration_failure(&error.to_string()),
        },
        OrchestrationCliCommand::Resume {
            instance,
            actor,
            output,
        } => match orchestration::resume(&store, instance, actor) {
            Ok(instance) => render_orchestration_instance(&instance, *output),
            Err(error) => orchestration_failure(&error.to_string()),
        },
        OrchestrationCliCommand::Cancel {
            instance,
            actor,
            output,
        } => match orchestration::cancel(&store, instance, actor) {
            Ok(instance) => render_orchestration_instance(&instance, *output),
            Err(error) => orchestration_failure(&error.to_string()),
        },
        OrchestrationCliCommand::Approve {
            instance,
            proposal,
            actor,
            output,
        } => {
            let proposal = match orchestration::load_proposal_nickel_file(proposal) {
                Ok(proposal) => proposal,
                Err(error) => return orchestration_failure(&error.to_string()),
            };
            match orchestration::approve(&store, instance, proposal, actor, app_config.effective_agent_runtime()).await
            {
                Ok(instance) => render_orchestration_instance(&instance, *output),
                Err(error) => orchestration_failure(&error.to_string()),
            }
        }
        OrchestrationCliCommand::Reject {
            instance,
            proposal_id,
            actor,
            output,
        } => match orchestration::reject(&store, instance, proposal_id, actor) {
            Ok(instance) => render_orchestration_instance(&instance, *output),
            Err(error) => orchestration_failure(&error.to_string()),
        },
        OrchestrationCliCommand::Replay { instance, output } => match orchestration::replay(&store, instance) {
            Ok(report) => match output {
                OrchestrationOutputFormat::Text => {
                    let rendered = format!("replay valid\n{}\n", orchestration::render_status_text(&report.instance));
                    render_orchestration_output(&rendered)
                }
                OrchestrationOutputFormat::Json => render_orchestration_json(&report),
            },
            Err(error) => orchestration_failure(&error.to_string()),
        },
        OrchestrationCliCommand::Recover { instance, output } => {
            match orchestration::recover_pending(&store, instance) {
                Ok(outcome) => match output {
                    OrchestrationOutputFormat::Text => render_orchestration_output(&format!("{outcome:?}\n")),
                    OrchestrationOutputFormat::Json => render_orchestration_json(&outcome),
                },
                Err(error) => orchestration_failure(&error.to_string()),
            }
        }
        OrchestrationCliCommand::Retry {
            instance,
            actor,
            output,
        } => match orchestration::retry(
            &store,
            orchestration::RetryRequest {
                instance_id: instance,
                actor,
            },
            app_config.effective_agent_runtime(),
        )
        .await
        {
            Ok(instance) => render_orchestration_instance(&instance, *output),
            Err(error) => orchestration_failure(&error.to_string()),
        },
        OrchestrationCliCommand::Abandon {
            instance,
            actor,
            output,
        } => match orchestration::abandon_pending(&store, orchestration::AbandonPendingRequest {
            instance_id: instance,
            actor,
        }) {
            Ok(instance) => render_orchestration_instance(&instance, *output),
            Err(error) => orchestration_failure(&error.to_string()),
        },
    }
}

fn render_orchestration_validation(
    report: &orchestration::DefinitionValidationReport,
    output: OrchestrationOutputFormat,
) -> Result<(), std::io::Error> {
    let rendered = match output {
        OrchestrationOutputFormat::Json => serde_json::to_string_pretty(report)
            .unwrap_or_else(|_| "{\"valid\":false,\"error\":\"render failed\"}".to_string()),
        OrchestrationOutputFormat::Text => format!(
            "orchestration definition {}: {} orchestration diagnostics, {} workflow diagnostics\n",
            if report.valid() { "valid" } else { "invalid" },
            report.orchestration_diagnostics.len(),
            report.workflow_diagnostics.len()
        ),
    };
    write_stdout_and_flush(&rendered)
}

fn render_orchestration_instance(
    instance: &orchestration::OrchestrationInstance,
    output: OrchestrationOutputFormat,
) -> ExitCode {
    match output {
        OrchestrationOutputFormat::Text => {
            render_orchestration_output(&format!("{}\n", orchestration::render_status_text(instance)))
        }
        OrchestrationOutputFormat::Json => render_orchestration_json(instance),
    }
}

fn render_orchestration_json(value: &impl serde::Serialize) -> ExitCode {
    match serde_json::to_string_pretty(value) {
        Ok(rendered) => render_orchestration_output(&format!("{rendered}\n")),
        Err(error) => orchestration_failure(&format!("JSON render failed: {error}")),
    }
}

fn render_orchestration_output(output: &str) -> ExitCode {
    match write_stdout_and_flush(output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => orchestration_failure(&format!("output failed: {error}")),
    }
}

fn orchestration_failure(message: &str) -> ExitCode {
    eprintln!("Orchestration failed: {message}");
    ExitCode::FAILURE
}

fn run_runs_command(app_config: &config::AppConfig, args: &RunsCommandArgs) -> ExitCode {
    match &args.command {
        RunsCliCommand::Show { run_id, output } => run_show_command(app_config, *run_id, *output),
        RunsCliCommand::Repair => run_repair_command(app_config),
    }
}

// r[impl lattice.workflow_execution.safe_run_inspection]
fn run_show_command(app_config: &config::AppConfig, run_id: u64, output: RunOutputFormat) -> ExitCode {
    let store_path = config::store_path(&app_config.data_dir);
    let result = match lattice::run::shell::inspect_path(&store_path, lattice::engine::RunId(run_id)) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("Run inspection failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let rendered = match output {
        RunOutputFormat::Text => lattice::run::inspection::render_text(&result),
        RunOutputFormat::Json => match lattice::run::inspection::render_json(&result) {
            Ok(rendered) => rendered,
            Err(error) => {
                eprintln!("Run inspection failed: JSON render failed: {error}");
                return ExitCode::FAILURE;
            }
        },
    };
    if let Err(error) = write_stdout_and_flush(&rendered) {
        eprintln!("Run inspection failed: output failed: {error}");
        return ExitCode::FAILURE;
    }
    if result.valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

// r[impl lattice.workflow_execution.run_store_recovery]
fn run_repair_command(app_config: &config::AppConfig) -> ExitCode {
    let store_path = config::store_path(&app_config.data_dir);
    let outcome = match lattice::run::shell::repair_path(&store_path) {
        Ok(outcome) => outcome,
        Err(error) => {
            eprintln!("Run store repair failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let receipt = match outcome {
        lattice::run::shell::RepairOutcome::Repaired => "Run evidence store recovery completed.\n",
        lattice::run::shell::RepairOutcome::AlreadyClean => "Run evidence store is already clean.\n",
    };
    if let Err(error) = write_stdout_and_flush(receipt) {
        eprintln!("Run store repair failed: output failed: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
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
        app_config.effective_agent_runtime(),
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
) -> Result<ExitCode, lattice::error::Error> {
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
        let resolution = match scheduler::shell::claim_due_occurrence(&store, &plan) {
            Ok(resolution) => resolution,
            Err(scheduler::shell::Error::MissingWorkflow(error)) => {
                update_schedule_next_run(&store, &plan.schedule_id, plan.next_run_epoch_ms);
                let outcome = schedule_error_outcome(&plan, error);
                println!("{}", scheduler::render_daemon_outcome(&outcome));
                continue;
            }
            Err(error) => {
                let outcome = schedule_error_outcome(&plan, error.to_string());
                println!("{}", scheduler::render_daemon_outcome(&outcome));
                continue;
            }
        };
        let occurrence = match resolution {
            scheduler::ScheduleOccurrenceClaimResolution::Inserted(occurrence) => occurrence,
            scheduler::ScheduleOccurrenceClaimResolution::Existing(occurrence) => {
                let outcome = ScheduleDaemonOutcome::Skipped(scheduler::SkippedScheduleRun {
                    schedule_id: plan.schedule_id.clone(),
                    workflow_name: plan.workflow_name.clone(),
                    reason: ScheduleSkipReason::OccurrenceAlreadyClaimed,
                    next_run_epoch_ms: occurrence.next_run_epoch_ms,
                });
                println!("{}", scheduler::render_daemon_outcome(&outcome));
                continue;
            }
            scheduler::ScheduleOccurrenceClaimResolution::StaleSchedule {
                current_next_run_epoch_ms,
            } => {
                let outcome = schedule_error_outcome(
                    &plan,
                    format!("schedule occurrence is stale at {current_next_run_epoch_ms}"),
                );
                println!("{}", scheduler::render_daemon_outcome(&outcome));
                continue;
            }
            scheduler::ScheduleOccurrenceClaimResolution::MissingSchedule => {
                let outcome = schedule_error_outcome(&plan, "schedule disappeared before occurrence claim".to_string());
                println!("{}", scheduler::render_daemon_outcome(&outcome));
                continue;
            }
        };
        let outcome =
            match scheduler::shell::execute_occurrence(&store, &occurrence, app_config.effective_agent_runtime()).await
            {
                Ok(summary) => ScheduleDaemonOutcome::Started {
                    schedule_id: plan.schedule_id.clone(),
                    workflow_name: plan.workflow_name.clone(),
                    summary: Box::new(summary),
                    next_run_epoch_ms: plan.next_run_epoch_ms,
                },
                Err(error) => schedule_error_outcome(&plan, error.to_string()),
            };
        println!("{}", scheduler::render_daemon_outcome(&outcome));
    }

    Ok(ExitCode::SUCCESS)
}

fn schedule_error_outcome(plan: &scheduler::DueRunPlan, message: String) -> ScheduleDaemonOutcome {
    ScheduleDaemonOutcome::Error {
        schedule_id: plan.schedule_id.clone(),
        workflow_name: plan.workflow_name.clone(),
        message,
        next_run_epoch_ms: plan.next_run_epoch_ms,
    }
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

// r[impl lattice.scheduling.scheduler_occurrence_admission]
async fn run_scheduler_occurrences_command(
    app_config: &config::AppConfig,
    args: &SchedulerOccurrencesCommandArgs,
) -> ExitCode {
    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => return scheduler_occurrence_failure(&error.to_string()),
    };
    match &args.command {
        SchedulerOccurrenceCliCommand::List { limit, output } => {
            if *limit == 0 || *limit > MAX_SCHEDULE_OCCURRENCE_LIMIT {
                return scheduler_occurrence_failure("occurrence list limit is outside the supported bound");
            }
            match store.list_schedule_occurrences(*limit) {
                Ok(occurrences) => render_schedule_occurrence_list(&occurrences, *output),
                Err(error) => scheduler_occurrence_failure(&error.to_string()),
            }
        }
        SchedulerOccurrenceCliCommand::Show { identity, output } => match store.load_schedule_occurrence(identity) {
            Ok(Some(occurrence)) => render_schedule_occurrence_output(&occurrence, *output),
            Ok(None) => scheduler_occurrence_failure("schedule occurrence does not exist"),
            Err(error) => scheduler_occurrence_failure(&error.to_string()),
        },
        SchedulerOccurrenceCliCommand::Recover { identity, output } => {
            match scheduler::shell::recover_occurrence(&store, identity) {
                Ok(outcome) => render_schedule_occurrence_recovery(&outcome, *output),
                Err(error) => scheduler_occurrence_failure(&error.to_string()),
            }
        }
        SchedulerOccurrenceCliCommand::Retry {
            identity,
            acknowledge_duplicate_risk,
            output,
        } => match scheduler::shell::retry_occurrence(
            &store,
            identity,
            *acknowledge_duplicate_risk,
            app_config.effective_agent_runtime(),
        )
        .await
        {
            Ok(occurrence) => render_schedule_occurrence_output(&occurrence, *output),
            Err(error) => scheduler_occurrence_failure(&error.to_string()),
        },
        SchedulerOccurrenceCliCommand::Abandon {
            identity,
            acknowledge_unknown_outcome,
            output,
        } => match scheduler::shell::abandon_occurrence(&store, identity, *acknowledge_unknown_outcome) {
            Ok(occurrence) => render_schedule_occurrence_output(&occurrence, *output),
            Err(error) => scheduler_occurrence_failure(&error.to_string()),
        },
    }
}

fn render_schedule_occurrence_list(occurrences: &[scheduler::ScheduleOccurrence], output: RunOutputFormat) -> ExitCode {
    match output {
        RunOutputFormat::Text => {
            if occurrences.is_empty() {
                println!("No schedule occurrences recorded");
            } else {
                for occurrence in occurrences {
                    println!("{}", scheduler::render_schedule_occurrence(occurrence));
                }
            }
            ExitCode::SUCCESS
        }
        RunOutputFormat::Json => match serde_json::to_string_pretty(occurrences) {
            Ok(rendered) => {
                println!("{rendered}");
                ExitCode::SUCCESS
            }
            Err(error) => scheduler_occurrence_failure(&error.to_string()),
        },
    }
}

fn render_schedule_occurrence_output(occurrence: &scheduler::ScheduleOccurrence, output: RunOutputFormat) -> ExitCode {
    match output {
        RunOutputFormat::Text => {
            println!("{}", scheduler::render_schedule_occurrence(occurrence));
            ExitCode::SUCCESS
        }
        RunOutputFormat::Json => match serde_json::to_string_pretty(occurrence) {
            Ok(rendered) => {
                println!("{rendered}");
                ExitCode::SUCCESS
            }
            Err(error) => scheduler_occurrence_failure(&error.to_string()),
        },
    }
}

fn render_schedule_occurrence_recovery(
    outcome: &scheduler::shell::ScheduleOccurrenceRecoveryOutcome,
    output: RunOutputFormat,
) -> ExitCode {
    match output {
        RunOutputFormat::Text => {
            let (label, occurrence) = match outcome {
                scheduler::shell::ScheduleOccurrenceRecoveryOutcome::AlreadyTerminal { occurrence } => {
                    ("already_terminal", occurrence.as_ref())
                }
                scheduler::shell::ScheduleOccurrenceRecoveryOutcome::PendingUnobserved { occurrence } => {
                    ("pending_unobserved", occurrence.as_ref())
                }
                scheduler::shell::ScheduleOccurrenceRecoveryOutcome::Reconciled { occurrence } => {
                    ("reconciled", occurrence.as_ref())
                }
            };
            println!("{label} {}", scheduler::render_schedule_occurrence(occurrence));
            ExitCode::SUCCESS
        }
        RunOutputFormat::Json => match serde_json::to_string_pretty(outcome) {
            Ok(rendered) => {
                println!("{rendered}");
                ExitCode::SUCCESS
            }
            Err(error) => scheduler_occurrence_failure(&error.to_string()),
        },
    }
}

fn scheduler_occurrence_failure(message: &str) -> ExitCode {
    eprintln!("Scheduler occurrences failed: {message}");
    ExitCode::FAILURE
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
        let claim_identity = record.claim_identity.as_deref().unwrap_or("-");
        output.push_str(&format!(
            "{} | {} | {} | {:?} | claim={} | {}\n",
            record.trigger_id, record.workflow_name, run_id, record.status, claim_identity, record.message,
        ));
    }
    output
}

// r[impl n8nclone.scheduling.webhook_listener.process]
// r[impl n8nclone.tui_ux.webhook_listener.cli]
async fn run_webhook_listener_command(app_config: &config::AppConfig, args: &WebhookListenArgs) -> ExitCode {
    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Webhook listener failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let triggers = match store.list_triggers() {
        Ok(triggers) => triggers,
        Err(error) => {
            eprintln!("Webhook listener failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let provider = match webhook_credential_provider(&triggers) {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("Webhook listener failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    if args.dry_run {
        return run_webhook_listener_dry_run(&store, &triggers, &provider, args);
    }

    let listener = match TcpListener::bind(&args.bind).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Webhook listener failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let bind_addr = match listener.local_addr() {
        Ok(addr) => addr.to_string(),
        Err(error) => {
            eprintln!("Webhook listener failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let status = webhooks::render_status(&webhooks::Status::listening(
        &bind_addr,
        count_to_u64(triggers.len()),
        webhooks::enabled_trigger_count(&triggers),
    ));
    if let Err(error) = write_stdout_and_flush(&status) {
        eprintln!("Webhook listener failed: {error}");
        return ExitCode::FAILURE;
    }

    let limits = webhooks::RequestLimits {
        max_body_bytes: args.max_body_bytes,
    };
    let store = std::sync::Arc::new(store);
    let triggers = std::sync::Arc::new(triggers);
    let provider = std::sync::Arc::new(provider);
    let mut handlers = tokio::task::JoinSet::new();
    loop {
        while let Some(result) = handlers.try_join_next() {
            observe_webhook_handler(result);
        }
        while handlers.len() >= WEBHOOK_MAX_IN_FLIGHT_CONNECTIONS {
            join_webhook_handler(&mut handlers).await;
        }
        let (mut stream, _peer_addr) = match listener.accept().await {
            Ok(connection) => connection,
            Err(error) => {
                handlers.shutdown().await;
                eprintln!("Webhook listener failed: {error}");
                return ExitCode::FAILURE;
            }
        };
        let store = std::sync::Arc::clone(&store);
        let triggers = std::sync::Arc::clone(&triggers);
        let provider = std::sync::Arc::clone(&provider);
        handlers.spawn(async move {
            let response = handle_webhook_connection(&mut stream, &store, &triggers, &provider, limits).await;
            if let Err(error) = stream.write_all(&webhooks::render(&response)).await {
                tracing::warn!(error = %error, "failed to write webhook listener response");
            }
        });
    }
}

async fn join_webhook_handler(handlers: &mut tokio::task::JoinSet<()>) {
    if let Some(result) = handlers.join_next().await {
        observe_webhook_handler(result);
    }
}

fn observe_webhook_handler(result: Result<(), tokio::task::JoinError>) {
    if let Err(error) = result {
        tracing::warn!(error = %error, "webhook listener handler failed");
    }
}

fn run_webhook_listener_dry_run(
    store: &Store,
    triggers: &[TriggerDefinition],
    provider: &CredentialProviderRegistry,
    args: &WebhookListenArgs,
) -> ExitCode {
    let workflow_names = match store.list_workflows() {
        Ok(workflows) => workflows.into_iter().collect::<HashSet<_>>(),
        Err(error) => {
            eprintln!("Webhook listener failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let report = webhooks::dry_run_report(&args.bind, triggers, &workflow_names, provider);
    if let Err(error) = write_stdout_and_flush(&webhooks::render_dry_run_report(&report)) {
        eprintln!("Webhook listener failed: {error}");
        return ExitCode::FAILURE;
    }
    if report.diagnostics.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

async fn handle_webhook_connection(
    stream: &mut TcpStream,
    store: &Store,
    triggers: &[TriggerDefinition],
    provider: &CredentialProviderRegistry,
    limits: webhooks::RequestLimits,
) -> webhooks::Reply {
    let request = match webhooks::read_raw(stream, limits).await {
        Ok(request) => request,
        Err(webhooks::ReadError::Diagnostic(diagnostic)) => {
            return webhooks::for_planning(&diagnostic);
        }
        Err(webhooks::ReadError::Io(error)) => {
            tracing::warn!(error = %error, "failed to read webhook listener request");
            return webhooks::rejected_for_code(webhooks::RejectionCode::InvalidRequest);
        }
    };
    let accepted_epoch_ms = chrono::Utc::now().timestamp_millis();
    let context = webhooks::admission_context(&[], accepted_epoch_ms);

    match webhooks::decide_request(&request, triggers, provider, &context, limits) {
        webhooks::Decision::Admitted(plan) => execute_webhook_plan(store, triggers, plan, accepted_epoch_ms).await,
        webhooks::Decision::Rejected(rejection) => {
            if let Some(record) = webhooks::run_record_for_rejection(&rejection)
                && let Err(error) = store.save_trigger_run(&record)
            {
                tracing::warn!(error = %error, "failed to save rejected webhook trigger run");
            }
            webhooks::rejected(&rejection)
        }
    }
}

async fn execute_webhook_plan(
    store: &Store,
    triggers: &[TriggerDefinition],
    plan: lattice::triggers::TriggerRunPlan,
    accepted_epoch_ms: i64,
) -> webhooks::Reply {
    let Some(trigger) = triggers.iter().find(|trigger| trigger.id == plan.trigger_id) else {
        return webhooks::rejected_for_code(webhooks::RejectionCode::InvalidRequest);
    };
    let claim_plan = match lattice::triggers::plan_webhook_claim(trigger, &plan, accepted_epoch_ms) {
        Ok(claim_plan) => claim_plan,
        Err(error) => {
            tracing::warn!(trigger_id = %plan.trigger_id, error = ?error, "webhook claim planning failed");
            return webhooks::rejected_for_code(webhooks::RejectionCode::InvalidRequest);
        }
    };
    let resolution = match &claim_plan {
        lattice::triggers::WebhookClaimPlan::NotRequired => None,
        lattice::triggers::WebhookClaimPlan::Required(request) => match store.claim_webhook(request) {
            Ok(resolution) => Some(resolution),
            Err(error) => {
                tracing::warn!(trigger_id = %plan.trigger_id, error = %error, "webhook claim transaction failed");
                return webhooks::execution_failed(&plan.trigger_id);
            }
        },
    };
    let decision = match lattice::triggers::decide_webhook_admission(&claim_plan, resolution) {
        Ok(decision) => decision,
        Err(error) => {
            tracing::warn!(trigger_id = %plan.trigger_id, error = ?error, "webhook claim decision failed");
            return webhooks::execution_failed(&plan.trigger_id);
        }
    };
    match decision {
        lattice::triggers::WebhookAdmissionDecision::ExecuteUnclaimed => {
            match headless::run_admitted_trigger_plan(store, &plan).await {
                Ok(summary) => webhooks::accepted(&plan.trigger_id, summary.run_id),
                Err(error) => {
                    tracing::warn!(trigger_id = %plan.trigger_id, error = %error, "webhook trigger execution failed");
                    webhooks::execution_failed(&plan.trigger_id)
                }
            }
        }
        lattice::triggers::WebhookAdmissionDecision::ExecuteClaimed { run_id } => {
            execute_claimed_webhook_plan(store, &plan, &claim_plan, run_id).await
        }
        lattice::triggers::WebhookAdmissionDecision::RejectDuplicate { .. } => {
            webhooks::rejected_for_code(webhooks::RejectionCode::DuplicateIdempotencyKey)
        }
    }
}

async fn execute_claimed_webhook_plan(
    store: &Store,
    plan: &lattice::triggers::TriggerRunPlan,
    claim_plan: &lattice::triggers::WebhookClaimPlan,
    run_id: lattice::engine::RunId,
) -> webhooks::Reply {
    let lattice::triggers::WebhookClaimPlan::Required(request) = claim_plan else {
        return webhooks::execution_failed(&plan.trigger_id);
    };
    match headless::run_claimed_trigger_plan(store, plan, run_id).await {
        Ok(summary) if summary.status == lattice::headless::HeadlessRunStatus::Succeeded => {
            webhooks::accepted(&plan.trigger_id, summary.run_id)
        }
        Ok(_) => {
            persist_failed_webhook_claim(store, request, run_id);
            webhooks::execution_failed(&plan.trigger_id)
        }
        Err(error) => {
            tracing::warn!(trigger_id = %plan.trigger_id, error = %error, "claimed webhook execution failed");
            persist_failed_webhook_claim(store, request, run_id);
            webhooks::execution_failed(&plan.trigger_id)
        }
    }
}

fn persist_failed_webhook_claim(
    store: &Store,
    request: &lattice::triggers::WebhookClaimRequest,
    run_id: lattice::engine::RunId,
) {
    if let Err(error) = store.save_trigger_run(&request.failed_record(run_id)) {
        tracing::warn!(trigger_id = %request.trigger_id, error = %error, "failed to persist webhook claim failure");
    }
}

fn webhook_credential_provider(triggers: &[TriggerDefinition]) -> Result<CredentialProviderRegistry, String> {
    let secret_refs = webhooks::secret_refs(triggers);
    let env_keys = secret_refs
        .iter()
        .filter(|secret_ref| secret_ref.provider == ENV_CREDENTIAL_PROVIDER_ID)
        .map(|secret_ref| secret_ref.key.clone())
        .collect::<Vec<_>>();
    let mut registry = CredentialProviderRegistry::new();
    registry
        .insert_provider(
            CredentialProviderMetadata {
                id: ENV_CREDENTIAL_PROVIDER_ID.to_string(),
                display_name: ENV_CREDENTIAL_PROVIDER_NAME.to_string(),
                capabilities: CredentialProviderCapabilities::read_only_without_fields(),
            },
            Box::new(EnvironmentCredentialProvider::from_current_env(&env_keys)),
        )
        .map_err(|error| error.to_string())?;
    Ok(registry)
}

fn write_stdout_and_flush(output: &str) -> Result<(), std::io::Error> {
    print!("{output}");
    std::io::stdout().flush()
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

    let diagnostics = workflow_validation::validate_workflow(&workflow);
    if workflow_validation::has_fatal_diagnostics(&diagnostics) {
        eprintln!("Import failed: workflow validation rejected the input");
        for diagnostic in workflow_validation::fatal_diagnostics(&diagnostics) {
            eprintln!("  {}", workflow_validation::format_diagnostic(&diagnostic));
        }
        return ExitCode::FAILURE;
    }

    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Import failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    match save_workflow_with_import_triggers(&store, &workflow) {
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

fn save_workflow_with_import_triggers(store: &Store, workflow: &lattice::engine::Workflow) -> Result<(), String> {
    store.save_workflow(workflow).map_err(|error| error.to_string())?;
    for trigger in &workflow.triggers {
        store.save_trigger(trigger).map_err(|error| error.to_string())?;
    }
    Ok(())
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
    let timestamp = lattice::engine::workflow::WorkflowTimestamp::from_datetime(chrono::Utc::now());
    let result = n8n_json::import_n8n_json_str(&input, timestamp);
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

fn run_protocol_command(app_config: &config::AppConfig, args: &lattice::protocol_cli::CommandArgs) -> ExitCode {
    let store = match open_store(app_config) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Protocol command failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let result = lattice::protocol_cli::run(&store, args);
    if write_stdout_and_flush(&result.output).is_err() {
        return ExitCode::FAILURE;
    }
    if result.success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn open_store(app_config: &config::AppConfig) -> Result<Store, String> {
    Store::new(config::store_path(&app_config.data_dir)).map_err(|error| error.to_string())
}

fn process_result_to_exit_code(result: HeadlessProcessResult) -> ExitCode {
    match result {
        HeadlessProcessResult::Success => ExitCode::SUCCESS,
        HeadlessProcessResult::Failure => ExitCode::FAILURE,
    }
}

fn count_to_u64(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
}

fn shellexpand(path: &str) -> PathBuf {
    if let Some(stripped_path) = path.strip_prefix("~/")
        && let Some(home) = home::home_dir()
    {
        return home.join(stripped_path);
    }
    PathBuf::from(path)
}

fn resolve_config_path(requested: Option<&str>, initializes_config: bool) -> PathBuf {
    let uses_canonical_default = requested.is_none();
    let requested_path = shellexpand(requested.unwrap_or(config::identity::DEFAULT_CONFIG_PATH));
    let legacy_path = shellexpand(config::identity::LEGACY_CONFIG_PATH);
    config::identity::select_startup_path(config::identity::StartupPathFacts {
        has_canonical_config: requested_path.exists(),
        has_legacy_config: legacy_path.exists(),
        requested_path,
        legacy_path,
        uses_canonical_default,
        initializes_config,
    })
}

// r[impl lattice.config.init_no_clobber]
// r[impl lattice.workflow_templates.init]
fn init_config(config_path: &Path) -> Result<(), lattice::error::Error> {
    lattice::workflow_template::initialize(config_path)
        .map(|report| {
            debug_assert!(!report.created.is_empty());
        })
        .map_err(|error| lattice::error::Error::Config {
            message: error.to_string(),
        })
}

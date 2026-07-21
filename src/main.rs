use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::ErrorKind;
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

mod doctor {
    const FATAL_SORT_RANK: u8 = 0;
    const WARNING_SORT_RANK: u8 = 1;
    const HEALTHY_SORT_RANK: u8 = 2;
    const REDACTED_MARKER: &str = "[redacted]";

    pub const CHECK_CONFIG: &str = "config";
    pub const CHECK_STORE: &str = "store";
    pub const CHECK_WORKFLOWS: &str = "workflows";
    pub const CHECK_CREDENTIALS: &str = "credentials";
    pub const CHECK_WEBHOOK_LISTENER: &str = "webhook_listener";

    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Severity {
        Healthy,
        Warning,
        Fatal,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Status {
        Healthy,
        Warning,
        Fatal,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Fact {
        pub id: String,
        pub severity: Severity,
        pub message: String,
        pub remediation: Option<String>,
        pub redactions: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Facts {
        pub checks: Vec<Fact>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct Finding {
        pub id: String,
        pub severity: Severity,
        pub message: String,
        pub remediation: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct Report {
        pub status: Status,
        pub findings: Vec<Finding>,
    }

    impl Fact {
        pub fn healthy(id: impl Into<String>, message: impl Into<String>) -> Self {
            Self::new(id, Severity::Healthy, message)
        }

        pub fn fatal(id: impl Into<String>, message: impl Into<String>) -> Self {
            Self::new(id, Severity::Fatal, message)
        }

        pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
            self.remediation = Some(remediation.into());
            self
        }

        fn new(id: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
            Self {
                id: id.into(),
                severity,
                message: message.into(),
                remediation: None,
                redactions: Vec::new(),
            }
        }
    }

    impl Facts {
        pub fn new(checks: Vec<Fact>) -> Self {
            Self { checks }
        }
    }

    pub fn evaluate(facts: &Facts) -> Report {
        let mut findings = facts.checks.iter().map(finding_from_fact).collect::<Vec<_>>();
        findings.sort_by(|left, right| {
            severity_sort_rank(left.severity)
                .cmp(&severity_sort_rank(right.severity))
                .then_with(|| left.id.cmp(&right.id))
        });
        Report {
            status: report_status(&findings),
            findings,
        }
    }

    pub fn render_text(report: &Report) -> String {
        let mut output = format!("doctor status={}\n", status_label(report.status));
        for finding in &report.findings {
            output.push_str(&format!("{} {}: {}\n", severity_label(finding.severity), finding.id, finding.message));
            if let Some(remediation) = &finding.remediation {
                output.push_str(&format!("  hint: {remediation}\n"));
            }
        }
        output
    }

    pub fn render_json(report: &Report) -> String {
        match serde_json::to_string_pretty(report) {
            Ok(json) => json,
            Err(error) => format!(
                "{{\"status\":\"fatal\",\"findings\":[{{\"id\":\"doctor_output\",\"severity\":\"fatal\",\"message\":\"json render failed: {error}\",\"remediation\":null}}]}}"
            ),
        }
    }

    pub fn has_fatal(report: &Report) -> bool {
        report.status == Status::Fatal
    }

    fn finding_from_fact(fact: &Fact) -> Finding {
        Finding {
            id: fact.id.clone(),
            severity: fact.severity,
            message: sanitize_text(&fact.message, &fact.redactions),
            remediation: fact.remediation.as_ref().map(|remediation| sanitize_text(remediation, &fact.redactions)),
        }
    }

    fn report_status(findings: &[Finding]) -> Status {
        if findings.iter().any(|finding| finding.severity == Severity::Fatal) {
            Status::Fatal
        } else if findings.iter().any(|finding| finding.severity == Severity::Warning) {
            Status::Warning
        } else {
            Status::Healthy
        }
    }

    fn sanitize_text(input: &str, redactions: &[String]) -> String {
        let mut output = input.to_string();
        for redaction in redactions {
            if !redaction.is_empty() {
                output = output.replace(redaction, REDACTED_MARKER);
            }
        }
        output
    }

    fn severity_sort_rank(severity: Severity) -> u8 {
        match severity {
            Severity::Fatal => FATAL_SORT_RANK,
            Severity::Warning => WARNING_SORT_RANK,
            Severity::Healthy => HEALTHY_SORT_RANK,
        }
    }

    fn status_label(status: Status) -> &'static str {
        match status {
            Status::Healthy => "healthy",
            Status::Warning => "warning",
            Status::Fatal => "fatal",
        }
    }

    fn severity_label(severity: Severity) -> &'static str {
        match severity {
            Severity::Healthy => "healthy",
            Severity::Warning => "warning",
            Severity::Fatal => "fatal",
        }
    }
}

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
    /// Inspect persisted triggers and trigger run history
    Triggers(TriggersCommandArgs),
    /// Listen for persisted webhook trigger HTTP requests
    WebhookListen(WebhookListenArgs),
    /// Diagnose local operator configuration and readiness
    Doctor(DoctorCommandArgs),
    /// Manage durable multi-run agent orchestrations
    Orchestration(OrchestrationCommandArgs),
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
        Some(CliCommand::Triggers(trigger_args)) => Ok(run_triggers_command(&app_config, &trigger_args)),
        Some(CliCommand::WebhookListen(webhook_args)) => {
            Ok(run_webhook_listener_command(&app_config, &webhook_args).await)
        }
        Some(CliCommand::Doctor(_)) => unreachable!("doctor command returns before app config side effects"),
        Some(CliCommand::Orchestration(orchestration_args)) => {
            Ok(run_orchestration_command(&app_config, &orchestration_args).await)
        }
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
        return Err(format!("config file not found: {}", config_path.display()));
    }
    let mut program = nickel_lang_core::program::Program::<nickel_lang_core::eval::cache::CacheImpl>::new_from_file(
        config_path,
        std::io::sink(),
        nickel_lang_core::error::NullReporter {},
    )
    .map_err(|error| format!("failed to load {}: {error:?}", config_path.display()))?;
    let runtime = program
        .eval_full_for_export()
        .map_err(|error| format!("eval error in {}: {error:?}", config_path.display()))?;
    nickel_lang_core::serialize::validate(nickel_lang_core::serialize::ExportFormat::Json, &runtime)
        .map_err(|error| format!("validation error in {}: {error:?}", config_path.display()))?;
    let mut json = Vec::new();
    serde_json::to_writer(&mut json, &runtime).map_err(|error| format!("json serialize error: {error}"))?;
    let value = serde_json::from_slice(&json).map_err(|error| format!("json parse error: {error}"))?;
    serde_json::from_value(value).map_err(|error| format!("config JSON decode failed: {error}"))
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
            match orchestration::start(&store, definition, actor, app_config.agent_runtime.clone()).await {
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
            match orchestration::approve(&store, instance, proposal, actor, app_config.agent_runtime.clone()).await {
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
    loop {
        let (mut stream, _peer_addr) = match listener.accept().await {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("Webhook listener failed: {error}");
                return ExitCode::FAILURE;
            }
        };
        let response = handle_webhook_connection(&mut stream, &store, &triggers, &provider, limits).await;
        if let Err(error) = stream.write_all(&webhooks::render(&response)).await {
            tracing::warn!(error = %error, "failed to write webhook listener response");
        }
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
    let records = match store.list_trigger_runs(webhooks::DEFAULT_HISTORY_LOOKBACK_RECORDS) {
        Ok(records) => records,
        Err(error) => {
            tracing::warn!(error = %error, "failed to load webhook trigger history");
            Vec::new()
        }
    };
    let context = webhooks::admission_context(&records, chrono::Utc::now().timestamp_millis());

    match webhooks::decide_request(&request, triggers, provider, &context, limits) {
        webhooks::Decision::Admitted(plan) => match headless::run_admitted_trigger_plan(store, &plan).await {
            Ok(summary) => webhooks::accepted(&plan.trigger_id, summary.run_id),
            Err(error) => {
                tracing::warn!(trigger_id = %plan.trigger_id, error = %error, "webhook trigger execution failed");
                webhooks::execution_failed(&plan.trigger_id)
            }
        },
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
    Store::new(config::store_path(&app_config.data_dir)).map_err(|error| error.to_string())
}

fn process_result_to_exit_code(result: HeadlessProcessResult) -> ExitCode {
    match result {
        HeadlessProcessResult::Success => ExitCode::SUCCESS,
        HeadlessProcessResult::Failure => ExitCode::FAILURE,
    }
}

fn count_to_u64(count: usize) -> u64 {
    u64::try_from(count).map_or(u64::MAX, |value| value)
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
fn init_config(config_path: &Path) -> Result<(), lattice::error::Error> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| lattice::error::Error::Io { source })?;
    }

    let mut config_file = OpenOptions::new().write(true).create_new(true).open(config_path).map_err(|source| {
        if source.kind() == ErrorKind::AlreadyExists {
            lattice::error::Error::Config {
                message: format!("config already exists at {}; refusing to overwrite", config_path.display()),
            }
        } else {
            lattice::error::Error::Io { source }
        }
    })?;
    config_file
        .write_all(config::default_config_ncl().as_bytes())
        .map_err(|source| lattice::error::Error::Io { source })?;

    let templates_dir = config_path.parent().unwrap_or(Path::new(".")).join("templates");
    std::fs::create_dir_all(&templates_dir).map_err(|source| lattice::error::Error::Io { source })?;
    for template_file in config::DEFAULT_NODE_TEMPLATE_FILES {
        std::fs::write(templates_dir.join(template_file.file_name), template_file.contents)
            .map_err(|source| lattice::error::Error::Io { source })?;
    }

    debug_assert!(config_path.exists());
    debug_assert!(templates_dir.exists());
    Ok(())
}

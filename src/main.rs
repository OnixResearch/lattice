use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use n8nclone::config;
use n8nclone::headless;
use n8nclone::headless::HeadlessProcessResult;
use n8nclone::store::Store;
use n8nclone::tui::App;
use tracing::info;

const DEFAULT_CONFIG_PATH: &str = "~/.n8nclone/config.ncl";
const LOG_FILE_PATH: &str = "/tmp/n8nclone.log";
const STORE_FILE_NAME: &str = "n8nclone.redb";

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
}

#[derive(clap::Args, Debug)]
struct RunCommandArgs {
    /// Saved workflow name to run
    workflow: String,

    /// Output format for the run summary
    #[arg(long, value_enum, default_value_t = RunOutputFormat::Text)]
    output: RunOutputFormat,
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
        None => {
            let app = App::new(app_config);
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
    let store_path = app_config.data_dir.join(STORE_FILE_NAME);
    let store = match Store::new(&store_path) {
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

    match headless::run_saved_workflow(&store, &args.workflow, run_id).await {
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
            eprintln!("Run failed: {error}");
            ExitCode::FAILURE
        }
    }
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
    std::fs::write(templates_dir.join("grep.ncl"), config::example_template_ncl())
        .map_err(|e| n8nclone::error::Error::Io { source: e })?;

    Ok(())
}

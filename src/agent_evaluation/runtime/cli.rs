#![allow(
    unknown_lints,
    no_unwrap,
    reason = "clap derive owns bounded parser expansion; command handlers do not call unwrap"
)]

mod execution;
mod planning;

const DEFAULT_RECORD_LIMIT_COUNT: u32 = 20;
const MAX_RECORD_LIMIT_COUNT: u32 = 100;
const DEFAULT_DRIVER_TIMEOUT_SECS: u64 = 600;
const DEFAULT_DRIVER_OUTPUT_MAX_BYTES: u64 = 1_048_576;

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Validate and list an identity-bound Nickel evaluation catalog
    Catalog(CatalogArgs),
    /// Create one pure deterministic matrix plan
    Plan(PlanArgs),
    /// Replan with exact durable trial and receipt facts
    Resume(PlanArgs),
    /// Run one selected trial through a bounded command driver
    Run(Box<RunArgs>),
    /// List bounded durable trial metadata
    List {
        #[arg(long = "limit", default_value_t = DEFAULT_RECORD_LIMIT_COUNT)]
        maximum_records: u32,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
    },
    /// Inspect one durable trial without artifact content
    Status {
        trial: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
    },
    /// Inspect bounded artifact-manifest metadata for one trial
    Artifacts {
        trial: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
    },
    /// Request cancellation without deleting durable state
    Cancel {
        trial: String,
        #[arg(long)]
        observed_epoch_ms: u64,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
    },
}

#[derive(clap::Args, Debug)]
pub struct CatalogArgs {
    #[arg(long)]
    pub root: std::path::PathBuf,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub output: OutputFormat,
}

#[derive(clap::Args, Debug)]
pub struct PlanArgs {
    #[arg(long)]
    pub root: std::path::PathBuf,
    #[arg(long)]
    pub request: std::path::PathBuf,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub output: OutputFormat,
}

#[derive(clap::Args, Debug)]
pub struct RunArgs {
    #[arg(long)]
    pub root: std::path::PathBuf,
    #[arg(long)]
    pub request: std::path::PathBuf,
    #[arg(long)]
    pub trial: String,
    #[arg(long)]
    pub driver: std::path::PathBuf,
    #[arg(long = "driver-arg")]
    pub driver_args: Vec<String>,
    #[arg(long)]
    pub driver_identity: String,
    #[arg(long)]
    pub agent_profile_identity: String,
    #[arg(long)]
    pub environment_identity: String,
    #[arg(long)]
    pub policy_identity: String,
    #[arg(long)]
    pub current_directory: std::path::PathBuf,
    #[arg(long, default_value_t = DEFAULT_DRIVER_TIMEOUT_SECS)]
    pub timeout_secs: u64,
    #[arg(long, default_value_t = DEFAULT_DRIVER_OUTPUT_MAX_BYTES)]
    pub output_max_bytes: u64,
    #[arg(long)]
    pub started_epoch_ms: u64,
    #[arg(long)]
    pub completed_epoch_ms: u64,
    #[arg(long)]
    pub confirm_live: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub output: OutputFormat,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(serde::Serialize)]
struct CatalogSummary<'a> {
    schema: &'static str,
    identity: &'a str,
    cases: Vec<CaseSummary<'a>>,
}

#[derive(serde::Serialize)]
struct CaseSummary<'a> {
    id: &'a str,
    class: crate::agent_evaluation::model::CaseClass,
    revision_identity: &'a str,
    live: bool,
    maximum_trials: u32,
}

pub fn run(app_config: &crate::config::AppConfig, args: &CommandArgs) -> std::process::ExitCode {
    match run_inner(app_config, args) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("evaluation command failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run_inner(app_config: &crate::config::AppConfig, args: &CommandArgs) -> Result<(), String> {
    match &args.command {
        Command::Catalog(catalog_args) => render_catalog(&catalog_args.root, catalog_args.output),
        Command::Plan(plan_args) => planning::render_plan(plan_args, None),
        Command::Resume(plan_args) => {
            let store = open_store(app_config)?;
            planning::render_plan(plan_args, Some(&store))
        }
        Command::Run(run_args) => {
            let store = open_store(app_config)?;
            execution::run_trial(run_args, app_config, &store)
        }
        Command::List {
            maximum_records,
            output,
        } => {
            if *maximum_records == 0 || *maximum_records > MAX_RECORD_LIMIT_COUNT {
                return Err("trial list limit is outside its bound".to_string());
            }
            let store = open_store(app_config)?;
            let records = store
                .list_evaluation_trials(*maximum_records)
                .map_err(|_| "trial list is unavailable".to_string())?;
            render_records(&records, *output)
        }
        Command::Status { trial, output } => {
            let store = open_store(app_config)?;
            let record = store
                .load_evaluation_trial(trial)
                .map_err(|_| "trial status is unavailable".to_string())?
                .ok_or_else(|| "trial was not found".to_string())?;
            render_records(&[record], *output)
        }
        Command::Artifacts { trial, output } => {
            let store = open_store(app_config)?;
            planning::render_artifacts(&store, trial, *output)
        }
        Command::Cancel {
            trial,
            observed_epoch_ms,
            output,
        } => {
            let store = open_store(app_config)?;
            store
                .request_evaluation_cancel(trial, *observed_epoch_ms)
                .map_err(|_| "trial cancellation was rejected".to_string())?;
            let record = store
                .load_evaluation_trial(trial)
                .map_err(|_| "trial status is unavailable".to_string())?
                .ok_or_else(|| "trial was not found".to_string())?;
            render_records(&[record], *output)
        }
    }
}

fn render_catalog(root: &std::path::Path, output: OutputFormat) -> Result<(), String> {
    let catalog = super::format::load_catalog(root).map_err(|error| error.to_string())?;
    debug_assert!(catalog.cases.len() <= super::model::MAX_CASES);
    debug_assert!(!catalog.identity.is_empty());
    let summary = CatalogSummary {
        schema: super::model::CATALOG_SCHEMA,
        identity: &catalog.identity,
        cases: catalog
            .cases
            .iter()
            .map(|case| CaseSummary {
                id: &case.definition.id,
                class: case.definition.class,
                revision_identity: &case.revision_identity,
                live: case.definition.live.enabled,
                maximum_trials: case.definition.budget.trials,
            })
            .collect(),
    };
    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&summary).map_err(|_| "catalog output failed")?)
        }
        OutputFormat::Text => {
            println!("catalog {}", summary.identity);
            for case in summary.cases {
                println!(
                    "{} class={:?} revision={} live={} trials={}",
                    case.id, case.class, case.revision_identity, case.live, case.maximum_trials
                );
            }
        }
    }
    debug_assert_eq!(summary.schema, super::model::CATALOG_SCHEMA);
    debug_assert!(!summary.identity.is_empty());
    Ok(())
}

fn render_records(records: &[super::persistence::TrialRecord], output: OutputFormat) -> Result<(), String> {
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(records).map_err(|_| "trial output failed")?),
        OutputFormat::Text => {
            for record in records {
                println!(
                    "{} case={} status={:?} updated_epoch_ms={} cancel_requested={} receipt={}",
                    record.plan.trial_identity,
                    record.plan.case_id,
                    record.status,
                    record.updated_epoch_ms,
                    record.cancel_requested,
                    record.receipt_identity.as_deref().unwrap_or("none")
                );
            }
        }
    }
    Ok(())
}

fn open_store(app_config: &crate::config::AppConfig) -> Result<crate::store::Store, String> {
    crate::store::Store::new(crate::config::store_path(&app_config.data_dir))
        .map_err(|_| "evaluation store is unavailable".to_string())
}

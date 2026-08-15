#![allow(
    unknown_lints,
    reason = "Octet lint names are available only when the pinned policy driver is active"
)]

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    List {
        #[arg(long, default_value_t = super::MAX_QUERY_RESULTS)]
        maximum_result_count: usize,
    },
    Show {
        version: String,
    },
    Get {
        version: String,
        destination: std::path::PathBuf,
        #[arg(long)]
        allow_sensitive: bool,
    },
    Query {
        #[arg(long)]
        selector: std::path::PathBuf,
        #[arg(long)]
        allow_sensitive: bool,
    },
    GarbageCollect {
        #[arg(long)]
        now_epoch_seconds: i64,
        #[arg(long)]
        confirm_delete: bool,
    },
    OrphanPrune {
        #[arg(long)]
        catalog: std::path::PathBuf,
        #[arg(long)]
        instances: std::path::PathBuf,
        #[arg(long)]
        confirm_delete: bool,
    },
    Project {
        version: String,
        destination: std::path::PathBuf,
        #[arg(long)]
        confirm_create: bool,
    },
    Recover,
}

pub fn run(app_config: &crate::config::AppConfig, args: &CommandArgs) -> std::process::ExitCode {
    match run_inner(app_config, args) {
        Ok(value) => {
            println!("{value}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("operation data command failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run_inner(app_config: &crate::config::AppConfig, args: &CommandArgs) -> Result<String, String> {
    let store_path = crate::config::store_path(&app_config.data_dir);
    let payload_root = app_config.data_dir.join("operation-data");
    if !store_path.is_file() || !payload_root.is_dir() {
        return Err("operation data store is not initialized".to_string());
    }
    let plane = super::Plane::open(&store_path, &payload_root).map_err(|error| error.to_string())?;
    execute(&plane, &args.command)
}

#[allow(
    function_length,
    assertion_density,
    reason = "one exhaustive closed command dispatcher keeps every preview and confirmation branch visible"
)]
fn execute(plane: &super::Plane, command: &Command) -> Result<String, String> {
    match command {
        Command::List { maximum_result_count } => {
            let mut records = plane.list_metadata().map_err(|error| error.to_string())?;
            records.truncate((*maximum_result_count).min(super::MAX_QUERY_RESULTS));
            metadata_json(&records)
        }
        Command::Show { version } => {
            let record = required_record(plane, version)?;
            metadata_json(&record)
        }
        Command::Get {
            version,
            destination,
            allow_sensitive,
        } => {
            let record = required_record(plane, version)?;
            if record.sensitivity == super::SensitivityClass::Sensitive && !allow_sensitive {
                return Err("sensitive operation data needs explicit authority".to_string());
            }
            plane.export_payload_create_only(&record, destination).map_err(|error| error.to_string())?;
            metadata_json(&record)
        }
        Command::Query {
            selector,
            allow_sensitive,
        } => {
            let metadata =
                std::fs::symlink_metadata(selector).map_err(|_| "selector source is unavailable".to_string())?;
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() > u64::try_from(super::MAX_METADATA_BYTES).map_err(|_| "selector bound is invalid")?
            {
                return Err("selector source must be a bounded regular Nickel file".to_string());
            }
            let value = crate::config::load_nickel_as_json(selector)
                .map_err(|_| "selector Nickel evaluation failed".to_string())?;
            let query: super::Query = serde_json::from_value(value)
                .map_err(|_| "selector does not match the closed Rust query schema".to_string())?;
            let results = plane
                .query(&query, super::QueryAuthority {
                    allow_sensitive: *allow_sensitive,
                })
                .map_err(|error| error.to_string())?;
            metadata_json(&results)
        }
        Command::GarbageCollect {
            now_epoch_seconds,
            confirm_delete,
        } => {
            let records = plane.list_metadata().map_err(|error| error.to_string())?;
            let plan = super::retention_plan(&records, &super::RetentionFacts {
                now_epoch_seconds: *now_epoch_seconds,
                terminal_run_ids: Vec::new(),
            })
            .map_err(|error| error.to_string())?;
            if *confirm_delete {
                let receipt = plane.execute_deletion(&plan).map_err(|error| error.to_string())?;
                metadata_json(&receipt)
            } else {
                metadata_json(&plan)
            }
        }
        Command::OrphanPrune {
            catalog,
            instances,
            confirm_delete,
        } => {
            let catalog = crate::operation::format::load_catalog(catalog).map_err(|error| error.to_string())?;
            let instances =
                crate::operation::format::load_instances(instances, &catalog).map_err(|error| error.to_string())?;
            let types = catalog.types.keys().cloned().collect::<std::collections::BTreeSet<_>>();
            let instances = instances.keys().cloned().collect::<std::collections::BTreeSet<_>>();
            let records = plane.list_metadata().map_err(|error| error.to_string())?;
            let plan = super::orphan_prune_plan(&records, &types, &instances).map_err(|error| error.to_string())?;
            if *confirm_delete {
                let receipt = plane.execute_deletion(&plan).map_err(|error| error.to_string())?;
                metadata_json(&receipt)
            } else {
                metadata_json(&plan)
            }
        }
        Command::Project {
            version,
            destination,
            confirm_create,
        } => {
            let record = required_record(plane, version)?;
            let projection = super::project_recorded_only(&record).map_err(|error| error.to_string())?;
            if *confirm_create {
                super::publish_projection_create_only(destination, &projection).map_err(|error| error.to_string())?;
            }
            metadata_json(&projection)
        }
        Command::Recover => {
            let outcomes = plane.recover_publications().map_err(|error| error.to_string())?;
            metadata_json(&outcomes)
        }
    }
}

fn required_record(plane: &super::Plane, version: &str) -> Result<super::RecordMetadata, String> {
    if !crate::operation::identity::validate(version) {
        return Err("version identity is invalid".to_string());
    }
    plane
        .load_metadata(version)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "operation data version was not found".to_string())
}

fn metadata_json<T: serde::Serialize + ?Sized>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|_| "operation data metadata encoding failed".to_string())
}

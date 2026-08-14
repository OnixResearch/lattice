//! Guarded one-agent workflow rounds and strict progress admission.

pub mod authority;
pub mod core;
pub mod evidence;
pub mod shell;
pub mod status;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Kind {
    Cairn,
    Octet,
}

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Discover a bounded deterministic active Cairn change set.
    CairnDiscover {
        #[arg(long)]
        root: std::path::PathBuf,
        #[arg(long)]
        requested: Option<String>,
    },
    /// Run one bounded no-disabled Octet probe and restore exact policy bytes.
    OctetProbe {
        #[arg(long)]
        root: std::path::PathBuf,
        #[arg(long)]
        artifact_dir: std::path::PathBuf,
        #[arg(long)]
        timeout_secs: u64,
    },
    /// Decide strict progress from the merge node's bounded JSON input.
    Decide {
        #[arg(long, value_enum)]
        kind: Kind,
        #[arg(long)]
        minimum_delta: u64,
    },
}

pub fn run_command(args: &CommandArgs) -> Result<std::process::ExitCode, shell::Error> {
    match &args.command {
        Command::CairnDiscover { root, requested } => {
            let snapshot = shell::discover_cairn(root, requested.as_deref())?;
            print_json(&snapshot)?;
            Ok(std::process::ExitCode::SUCCESS)
        }
        Command::OctetProbe {
            root,
            artifact_dir,
            timeout_secs,
        } => {
            let snapshot = shell::probe_octet(root, artifact_dir, *timeout_secs)?;
            print_json(&snapshot)?;
            Ok(std::process::ExitCode::SUCCESS)
        }
        Command::Decide { kind, minimum_delta } => {
            let input = shell::read_stdin_bounded()?;
            let decision = shell::decide_from_merged_json(*kind, *minimum_delta, &input)?;
            print_json(&decision)?;
            match decision {
                core::ProgressDecision::Blocked { .. } => Ok(std::process::ExitCode::FAILURE),
                core::ProgressDecision::Improved { .. } | core::ProgressDecision::Complete { .. } => {
                    Ok(std::process::ExitCode::SUCCESS)
                }
            }
        }
    }
}

fn print_json(value: &impl serde::Serialize) -> Result<(), shell::Error> {
    let text = serde_json::to_string(value).map_err(|_| shell::Error::MalformedObservation)?;
    println!("{text}");
    Ok(())
}

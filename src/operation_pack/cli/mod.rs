#![allow(
    unknown_lints,
    assertion_density,
    reason = "CLI boundaries report typed operator errors instead of terminating with assertions"
)]
#![allow(
    path_segment_repetition,
    reason = "public CLI names retain explicit pack terminology for stable help and API output"
)]

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Inspect exact pack metadata without loading or executing content.
    Inspect(PackPath),
    /// Preview or create an exact pack lock.
    Lock(LockArgs),
    /// Install exact pack bytes from an admitted lock.
    Install(InstallArgs),
    /// Preview or run one exact read-only packed operation and its analyzers.
    Run(Box<RunArgs>),
    /// Disclose one trusted passive skill body after explicit selection.
    SkillShow(SkillShowArgs),
    /// Install one trusted passive skill through a create-only adapter.
    SkillInstall(SkillInstallArgs),
    /// Remove one exact installed skill.
    SkillRemove(SkillRemoveArgs),
    /// Search one explicitly selected local registry.
    RegistrySearch(RegistrySearchArgs),
    /// Inspect one exact registry entry without resolving its content locator.
    RegistryInfo(RegistryInfoArgs),
    /// Create an inert exact restore plan without install, update, or network effects.
    RegistryRestore(RegistryRestoreArgs),
    /// Create a metadata-only forward channel promotion.
    RegistryPromote(RegistryPromoteArgs),
    /// Check one pack integration without ordinary effects.
    Doctor(DoctorArgs),
}

#[derive(clap::Args, Debug)]
pub struct PackPath {
    #[arg(long)]
    pub pack: std::path::PathBuf,
}

#[derive(clap::Args, Debug)]
pub struct TrustArgs {
    /// Explicitly trust this exact local pack identity for this command.
    #[arg(long)]
    pub confirm_trust_local: bool,
    /// Allowed SPDX license identifiers. Repeat for multiple licenses.
    #[arg(long)]
    pub allow_license: Vec<String>,
    /// Optional pre-collected admission facts Nickel for dependency or evidence profiles.
    #[arg(long)]
    pub admission_facts: Option<std::path::PathBuf>,
}

#[derive(clap::Args, Debug)]
pub struct LockArgs {
    #[command(flatten)]
    pub source: PackPath,
    #[command(flatten)]
    pub trust: TrustArgs,
    #[arg(long)]
    pub destination: std::path::PathBuf,
    #[arg(long)]
    pub confirm_create: bool,
}

#[derive(clap::Args, Debug)]
pub struct InstallArgs {
    #[command(flatten)]
    pub source: PackPath,
    #[command(flatten)]
    pub trust: TrustArgs,
    #[arg(long)]
    pub lock: std::path::PathBuf,
    #[arg(long)]
    pub destination: std::path::PathBuf,
    #[arg(long)]
    pub confirm_create: bool,
}

#[derive(clap::Args, Debug)]
pub struct RunArgs {
    #[command(flatten)]
    pub source: PackPath,
    #[command(flatten)]
    pub trust: TrustArgs,
    #[arg(long)]
    pub lock: std::path::PathBuf,
    #[arg(long)]
    pub instances: std::path::PathBuf,
    #[arg(long)]
    pub instance: String,
    #[arg(long)]
    pub method: String,
    #[arg(long, default_value = "{}")]
    pub arguments_json: String,
    #[arg(long, default_value_t = 1)]
    pub sequence: u64,
    #[arg(long)]
    pub confirm_read_only: bool,
}

#[derive(clap::Args, Debug)]
pub struct SkillShowArgs {
    #[command(flatten)]
    pub source: PackPath,
    #[command(flatten)]
    pub trust: TrustArgs,
    #[arg(long)]
    pub lock: std::path::PathBuf,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub include_references: bool,
}

#[derive(clap::Args, Debug)]
pub struct SkillInstallArgs {
    #[command(flatten)]
    pub show: SkillShowArgs,
    #[arg(long)]
    pub host: String,
    #[arg(long)]
    pub destination_root: std::path::PathBuf,
    #[arg(long)]
    pub confirm_create: bool,
}

#[derive(clap::Args, Debug)]
pub struct SkillRemoveArgs {
    #[arg(long)]
    pub destination_root: std::path::PathBuf,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub skill_identity: String,
    #[arg(long)]
    pub confirm_remove: bool,
}

#[derive(clap::Args, Debug)]
pub struct RegistrySearchArgs {
    #[arg(long)]
    pub catalog: std::path::PathBuf,
    #[arg(long, default_value = "")]
    pub query: String,
    #[arg(long)]
    pub observed_epoch_seconds: i64,
    #[arg(long, default_value_t = 86_400)]
    pub maximum_age_seconds: u64,
}

#[derive(clap::Args, Debug)]
pub struct RegistryInfoArgs {
    #[arg(long)]
    pub catalog: std::path::PathBuf,
    #[arg(long)]
    pub pack_identity: String,
}

#[derive(clap::Args, Debug)]
pub struct RegistryRestoreArgs {
    #[arg(long)]
    pub catalog: std::path::PathBuf,
    #[arg(long)]
    pub pack_identity: String,
    #[arg(long)]
    pub destination: std::path::PathBuf,
}

#[derive(clap::Args, Debug)]
pub struct RegistryPromoteArgs {
    #[arg(long)]
    pub catalog: std::path::PathBuf,
    #[arg(long)]
    pub pack_identity: String,
    #[arg(long, value_enum)]
    pub target: ChannelArg,
    #[arg(long)]
    pub destination: std::path::PathBuf,
    #[arg(long)]
    pub confirm_create: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ChannelArg {
    Beta,
    Rc,
    Stable,
}

#[derive(clap::Args, Debug)]
pub struct DoctorArgs {
    #[command(flatten)]
    pub source: PackPath,
    #[command(flatten)]
    pub trust: TrustArgs,
    #[arg(long)]
    pub lock: Option<std::path::PathBuf>,
    #[arg(long)]
    pub cache_present: bool,
    #[arg(long)]
    pub registry_configured: bool,
    #[arg(long)]
    pub registry_available: bool,
    /// Run the declared no-effect probe with a cleared environment and no credentials.
    #[arg(long)]
    pub run_probe: bool,
}

pub fn run(app_config: &crate::config::AppConfig, args: &CommandArgs) -> std::process::ExitCode {
    match run_inner(app_config, args) {
        Ok(success) => {
            if success {
                std::process::ExitCode::SUCCESS
            } else {
                std::process::ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("operation pack command failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run_inner(app_config: &crate::config::AppConfig, args: &CommandArgs) -> Result<bool, super::Error> {
    match &args.command {
        Command::Inspect(source) => {
            let candidate = super::shell::load(&source.pack)?;
            print_json(&candidate)?;
            Ok(true)
        }
        Command::Lock(args) => lock_command(args),
        Command::Install(args) => install_command(args),
        Command::Run(args) => run_command(app_config, args),
        Command::SkillShow(args) => skill_show_command(args),
        Command::SkillInstall(args) => skill_install_command(args),
        Command::SkillRemove(args) => skill_remove_command(args),
        Command::RegistrySearch(args) => registry_search_command(args),
        Command::RegistryInfo(args) => registry_info_command(args),
        Command::RegistryRestore(args) => registry_restore_command(args),
        Command::RegistryPromote(args) => registry_promote_command(args),
        Command::Doctor(args) => doctor_command(app_config, args),
    }
}

include!("commands.rs");

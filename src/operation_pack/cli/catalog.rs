//! `lattice pack catalog-*` commands for git-backed runbook catalogs.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "CLI boundaries report typed operator errors instead of terminating with assertions"
)]

#[derive(clap::Args, Debug)]
pub struct FetchArgs {
    /// Catalog repository locator. Only `file://`, `https://` and `ssh://` are accepted.
    #[arg(long)]
    pub repository: String,
    /// Exact full commit object name. Branches, tags and abbreviations are refused.
    #[arg(long)]
    pub commit: String,
    /// Pack directory inside the repository tree, for example `packs/seed-runbooks`.
    #[arg(long)]
    pub subdirectory: Option<String>,
    /// Refuse publication unless the fetched pack has exactly this `b3:` identity.
    #[arg(long)]
    pub pack_identity: Option<String>,
    /// Hard budget for repository bytes written while fetching; every file git writes is capped
    /// at the remaining budget and the total is checked after each fetch.
    #[arg(long, default_value_t = super::super::catalog::transfer::DEFAULT_TRANSFER_LIMIT_BYTES)]
    pub maximum_transfer_bytes: u64,
    /// New absolute directory that receives the verified pack tree.
    #[arg(long)]
    pub destination: std::path::PathBuf,
    /// Contact the repository. Without this flag the command validates its inputs only.
    #[arg(long)]
    pub confirm_fetch: bool,
}

#[derive(clap::Args, Debug)]
pub struct InstallArgs {
    #[command(flatten)]
    pub source: super::PackPath,
    #[command(flatten)]
    pub trust: super::TrustArgs,
    #[arg(long)]
    pub lock: std::path::PathBuf,
    /// Absolute local `workflow-templates/` catalog directory next to `config.ncl`.
    #[arg(long)]
    pub catalog: std::path::PathBuf,
    #[arg(long)]
    pub confirm_create: bool,
}

#[derive(clap::Args, Debug)]
pub struct ListArgs {
    /// Absolute local `workflow-templates/` catalog directory next to `config.ncl`.
    #[arg(long)]
    pub catalog: std::path::PathBuf,
}

pub(super) fn fetch_command(
    app_config: &crate::config::AppConfig,
    args: &FetchArgs,
) -> Result<bool, super::super::Error> {
    super::super::catalog::source::validate_repository_locator(&args.repository)?;
    super::super::catalog::source::validate_commit(&args.commit)?;
    super::super::catalog::source::validate_subdirectory(args.subdirectory.as_deref())?;
    super::super::catalog::transfer::check_budget(args.maximum_transfer_bytes)?;
    if !args.confirm_fetch {
        super::print_json(&serde_json::json!({
            "commit": args.commit,
            "subdirectory": args.subdirectory,
            "fetched": false,
            "network_accessed": false,
            "required_flag": "--confirm-fetch"
        }))?;
        return Ok(true);
    }
    let receipt = super::super::shell::catalog::fetch::run(&super::super::shell::catalog::fetch::Request {
        repository: args.repository.clone(),
        commit: args.commit.clone(),
        subdirectory: args.subdirectory.clone(),
        expected_pack_identity: args.pack_identity.clone(),
        destination: args.destination.clone(),
        transfer_limit_bytes: args.maximum_transfer_bytes,
        process: app_config.effective_agent_runtime().local_process,
    })?;
    super::print_json(&receipt)?;
    Ok(true)
}

pub(super) fn install_command(args: &InstallArgs) -> Result<bool, super::super::Error> {
    let (manifest, drifted) = super::super::shell::catalog::drifted_bundles(&args.source.pack)?;
    if !drifted.is_empty() {
        super::print_json(&super::super::shell::catalog::refusal_report(&manifest, &drifted)?)?;
        return Ok(false);
    }
    let admitted = super::admitted_pack(&args.source.pack, &args.lock, &args.trust)?;
    let report = super::super::shell::catalog::install(&super::super::shell::catalog::InstallRequest {
        pack_root: &args.source.pack,
        admitted: &admitted,
        catalog: &args.catalog,
        confirm_create: args.confirm_create,
    })?;
    let is_complete = report.is_complete();
    super::print_json(&report)?;
    Ok(is_complete)
}

pub(super) fn list_command(args: &ListArgs) -> Result<bool, super::super::Error> {
    let report = super::super::shell::catalog::list(&args.catalog)?;
    let is_clean = report.diagnostics.is_empty();
    super::print_json(&report)?;
    Ok(is_clean)
}

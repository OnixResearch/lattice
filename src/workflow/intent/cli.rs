use std::io::Read as _;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Identify this exact admitted compiler executable without granting effects.
    CompilerIdentity,
    /// Compile a host-normalized intent to an inert canonical dependency plan.
    Compile { request: std::path::PathBuf },
    /// Compare proposed data; no proposal is executed or promoted.
    Compare {
        frozen: std::path::PathBuf,
        proposed: std::path::PathBuf,
    },
}

/// No configuration, store, credentials or execution adapters are opened here.
pub fn run(args: &Args) -> Result<(), String> {
    let value = match &args.command {
        Command::CompilerIdentity => serde_json::json!({
            "compiler_revision": super::COMPILER_REVISION,
            "identity": compiler_identity()?,
        }),
        Command::Compile { request } => {
            let plan = super::compile(load(request)?).map_err(|error| error.to_string())?;
            serde_json::to_value(plan).map_err(|_| "plan encoding failed")?
        }
        Command::Compare { frozen, proposed } => {
            let base = super::compile(load(frozen)?).map_err(|error| error.to_string())?;
            let comparison = super::compare_proposal(&base, load(proposed)?).map_err(|error| error.to_string())?;
            serde_json::to_value(comparison).map_err(|_| "comparison encoding failed")?
        }
    };
    serde_json::to_writer_pretty(std::io::stdout().lock(), &value).map_err(|_| "output unavailable".to_string())
}

fn load(path: &std::path::Path) -> Result<super::CompileRequest, String> {
    let file = std::fs::File::open(path).map_err(|_| "intent input unavailable")?;
    let mut bytes = Vec::new();
    let read_limit_bytes = super::MAX_INPUT_BYTES
        .checked_add(1)
        .and_then(|limit| u64::try_from(limit).ok())
        .ok_or("intent input byte bound unavailable")?;
    file.take(read_limit_bytes).read_to_end(&mut bytes).map_err(|_| "intent input unavailable")?;
    if bytes.len() > super::MAX_INPUT_BYTES {
        return Err("intent input exceeds byte bound".into());
    }
    let request: super::CompileRequest =
        serde_json::from_slice(&bytes).map_err(|_| "invalid typed workflow intent".to_string())?;
    if request.facts.compiler_artifact_identity != compiler_identity()? {
        return Err("compiler artifact differs from frozen input".into());
    }
    Ok(request)
}

fn compiler_identity() -> Result<String, String> {
    let executable = std::env::current_exe().map_err(|_| "compiler executable unavailable")?;
    crate::operation::identity::executor_file(&executable)
        .map_err(|_| "compiler artifact unavailable; select the bounded release binary".to_string())
}

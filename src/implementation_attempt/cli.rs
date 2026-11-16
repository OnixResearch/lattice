//! Operator-owned one-attempt shell over the existing isolated OMP worker.
mod artifacts;
mod cancellation;
mod execution;
mod export;
mod recovery;
#[cfg(test)]
mod tests;

pub const REQUEST_SCHEMA: &str = "lattice.implementation-attempt-run.v2";
const MAX_REQUEST_BYTES: usize = 2_097_152;
const MAX_OUTPUT_BYTES: usize = 16_777_216;

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,
}
#[derive(clap::Subcommand, Debug)]
pub enum Command {
    Run {
        #[arg(long)]
        request: std::path::PathBuf,
        #[arg(long)]
        state: std::path::PathBuf,
    },
    Recover {
        #[arg(long)]
        request: std::path::PathBuf,
        #[arg(long)]
        state: std::path::PathBuf,
    },
    /// Normalize canonical identities without executing or granting authority.
    Prepare {
        #[arg(long)]
        request: std::path::PathBuf,
        #[arg(long)]
        output: std::path::PathBuf,
    },
    /// Request cancellation, without claiming observed process termination.
    Cancel {
        #[arg(long)]
        request: std::path::PathBuf,
        #[arg(long)]
        state: std::path::PathBuf,
    },
    /// Export a retained native candidate without executing a worker.
    ExportCandidate {
        #[arg(long)]
        request: std::path::PathBuf,
        #[arg(long)]
        state: std::path::PathBuf,
        #[arg(long)]
        output_parent: std::path::PathBuf,
    },
}

/// Exported typed configuration, not worker-provided completion assertions.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub envelope: super::ImplementationAttemptEnvelope,
    pub admitted_envelope: String,
    pub expires_at_ms: u64,
    pub workspace_root: std::path::PathBuf,
    pub read_paths: Vec<String>,
    pub write_paths: Vec<String>,
    pub program: String,
    pub sandbox_program: String,
    pub omp: crate::candidate_worker::omp::Profile,
    pub output_limit_bytes: usize,
    #[serde(default)]
    pub allow_unchanged: bool,
}

impl Request {
    pub fn worker_profile_identity(&self) -> Result<String, String> {
        let bytes = serde_json::to_vec(&(
            &self.program,
            &self.sandbox_program,
            &self.omp,
            &self.read_paths,
            &self.write_paths,
            self.output_limit_bytes,
            self.allow_unchanged,
        ))
        .map_err(|error| error.to_string())?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }
    pub fn validate(&self, now_ms: u64) -> Result<(), String> {
        self.envelope.validate().map_err(|error| error.to_string())?;
        self.validate_binding()?;
        self.validate_execution(now_ms)
    }

    fn validate_binding(&self) -> Result<(), String> {
        if self.schema != REQUEST_SCHEMA
            || self.admitted_envelope != self.envelope.identity()
            || self.worker_profile_identity()? != self.envelope.worker_profile
        {
            return Err("implementation request is stale, crossed or unsupported".into());
        }
        Ok(())
    }

    fn validate_execution(&self, now_ms: u64) -> Result<(), String> {
        let invalid = "implementation request is stale, crossed or unsupported";
        if self.expires_at_ms <= now_ms || self.output_limit_bytes == 0 || self.output_limit_bytes > MAX_OUTPUT_BYTES {
            return Err(invalid.into());
        }
        self.validate_paths().map_err(|_| invalid.to_string())?;
        if self.omp.revision != crate::candidate_worker::omp::OMP_REVISION
            || self.omp.model_broker.expires_at_ms <= now_ms
            || self.omp.model_broker.expires_at_ms > self.expires_at_ms
        {
            return Err(invalid.into());
        }
        crate::candidate_worker::broker::admission::validate(&self.omp.model_broker)?;
        Ok(())
    }

    fn validate_paths(&self) -> Result<(), String> {
        if !self.workspace_root.is_absolute()
            || !std::path::Path::new(&self.program).is_absolute()
            || !std::path::Path::new(&self.sandbox_program).is_absolute()
        {
            return Err("worker paths must be absolute".into());
        }
        Ok(())
    }
}

fn now_ms() -> Result<u64, String> {
    let observed_ms = crate::operation::shell::Clock::observe_epoch_millis(&crate::operation::shell::SystemClock);
    u64::try_from(observed_ms).map_err(|error| error.to_string())
}

// r[impl lattice.implementation_attempt.consumer]
pub async fn run_command(args: &CommandArgs) -> Result<(), String> {
    match &args.command {
        Command::Cancel { request, state } => cancel(request, state),
        Command::Prepare { request, output } => prepare(request, output),
        Command::Run { request, state } => execution::run(request, state, true).await,
        Command::Recover { request, state } => execution::run(request, state, false).await,
        Command::ExportCandidate {
            request,
            state,
            output_parent,
        } => export::run(request, state, output_parent),
    }
}

fn cancel(request_path: &std::path::Path, state: &std::path::Path) -> Result<(), String> {
    let (request, bytes) = artifacts::load_request(request_path)?;
    request.envelope.validate().map_err(|error| error.to_string())?;
    let directory = artifacts::directory(state, &request);
    let identity = blake3::hash(&bytes).to_hex().to_string();
    if artifacts::read_bounded(&directory.join("intent.json"), identity.len())? != identity.as_bytes() {
        return Err("cancel request differs from persisted execution".into());
    }
    artifacts::write_once(&directory.join("cancel.request"), identity.as_bytes())?;
    println!("{}", serde_json::json!({"cancellation_requested":true,"termination_observed":false}));
    Ok(())
}

fn normalized_request(request_path: &std::path::Path) -> Result<(Request, Vec<u8>), String> {
    let (mut request, _) = artifacts::load_request(request_path)?;
    let baseline = crate::candidate_capture::workspace(&request.workspace_root).map_err(|error| error.to_string())?;
    request.omp.base_identity = crate::candidate::snapshot_identity(&baseline).map_err(|error| error.to_string())?;
    request.envelope.worker_profile = request.worker_profile_identity()?;
    request.admitted_envelope = request.envelope.identity();
    request.validate(now_ms()?)?;
    let bytes = serde_json::to_vec_pretty(&request).map_err(|error| error.to_string())?;
    Ok((request, bytes))
}

fn prepare(request_path: &std::path::Path, output: &std::path::Path) -> Result<(), String> {
    let (request, bytes) = normalized_request(request_path)?;
    if !artifacts::write_once(output, &bytes)? {
        return Err("prepared request output already exists".into());
    }
    println!(
        "{}",
        serde_json::json!({"request":output,"identity":request.admitted_envelope,
        "worker_request":blake3::hash(&bytes).to_hex().to_string(),
        "worker_profile":request.envelope.worker_profile,"admitted":false})
    );
    Ok(())
}

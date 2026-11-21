//! Operator-owned one-attempt shell over the existing isolated OMP worker.
mod artifacts;
mod binding;
mod cancellation;
mod execution;
mod export;
mod fencing;
mod inspect;
mod recovery;
mod refusal;
#[cfg(test)]
mod tests;

// r[impl lattice.execution_binding.profile_facts.scenario.unsupported]
// r[impl lattice.execution_binding.persistence.scenario.changed_binding]
pub const REQUEST_SCHEMA: &str = "lattice.implementation-attempt-run.v3";
/// Refused before any effect; such requests need explicit readmission as v3.
pub const LEGACY_REQUEST_SCHEMA: &str = "lattice.implementation-attempt-run.v2";
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
    /// Print separate profile and instance facts without executing anything.
    Inspect {
        #[arg(long)]
        request: std::path::PathBuf,
        #[arg(long)]
        state: std::path::PathBuf,
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
    /// Compute the worker profile requires; part of the profile revision.
    pub worker_compute: lattice_core::execution_binding::request::ComputeRequirement,
    /// Per-attempt expected compute allocation and model routes.
    pub execution_binding: lattice_core::execution_binding::request::Declaration,
    /// Optional declared definition, task families and tools; part of the
    /// profile revision only when present.
    #[serde(
        default = "lattice_core::execution_binding::capability::absent",
        skip_serializing_if = "Option::is_none"
    )]
    pub worker_capability: Option<lattice_core::execution_binding::capability::Declaration>,
    /// The coordinator's declared workspace network. Only `false` is admitted:
    /// the OMP worker never receives a network device, only its brokered model
    /// route. Absent means the same; it is not part of the profile revision.
    #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
    pub network: Option<bool>,
    /// The coordinator's dispatch-plan identity (64 lowercase hex), bound by the
    /// exact request bytes and echoed in every observation when present.
    #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
    pub dispatch_plan: Option<String>,
}

/// An absent optional coordinator field.
const fn absent<T>() -> Option<T> {
    None
}

impl Request {
    /// The profile revision: `lattice-worker-profile-revision-v2` over
    /// profile-level configuration only, spelled as bare lowercase hex. The
    /// base snapshot, task prompt, broker expiry and workspace root are
    /// per-attempt values bound by the envelope and the exact request bytes. A
    /// capability declaration is framed in canonical form only when present;
    /// a malformed declaration is refused here, before any effect.
    // r[impl lattice.worker_inventory.capability.scenario.undeclared]
    // r[impl lattice.worker_inventory.capability.scenario.declared]
    // r[impl lattice.implementation_attempt.consumer.scenario.profile_revision]
    pub fn worker_profile_identity(&self) -> Result<String, String> {
        let capability = match &self.worker_capability {
            None => None,
            Some(declared) => {
                let canonical = declared
                    .canonical()
                    .map_err(|defect| format!("worker capability declaration is unsupported: {defect:?}"))?;
                Some(serde_json::to_string(&canonical).map_err(|error| error.to_string())?)
            }
        };
        let compute = serde_json::to_string(&self.worker_compute).map_err(|error| error.to_string())?;
        let broker = &self.omp.model_broker;
        let credential = broker.credential.as_ref();
        let read_paths: Vec<&str> = self.read_paths.iter().map(String::as_str).collect();
        let write_paths: Vec<&str> = self.write_paths.iter().map(String::as_str).collect();
        let non_utf8 = "worker runtime path is not UTF-8";
        let relay_path = broker.relay.path.to_str().ok_or(non_utf8)?;
        let shell_path = broker.shell.path.to_str().ok_or(non_utf8)?;
        crate::content_identity::worker_profile_revision(crate::content_identity::WorkerProfileRevisionInput {
            program: &self.program,
            sandbox_program: &self.sandbox_program,
            harness_revision: &self.omp.revision,
            harness_executable_blake3: &self.omp.executable_blake3,
            harness_model: &self.omp.model,
            broker_schema: &broker.schema,
            broker_endpoint: &broker.endpoint,
            broker_model: &broker.model,
            credential_provider: credential.map(|reference| reference.provider.as_str()),
            credential_key: credential.map(|reference| reference.key.as_str()),
            credential_field: credential.and_then(|reference| reference.field.as_deref()),
            relay: crate::content_identity::RuntimeArtifactInput {
                path: relay_path,
                blake3: &broker.relay.blake3,
            },
            shell: crate::content_identity::RuntimeArtifactInput {
                path: shell_path,
                blake3: &broker.shell.blake3,
            },
            sandbox_blake3: &broker.sandbox_blake3,
            max_requests: u64::from(broker.max_requests),
            read_paths: &read_paths,
            write_paths: &write_paths,
            output_limit_bytes: u64::try_from(self.output_limit_bytes).map_err(|error| error.to_string())?,
            allow_unchanged: self.allow_unchanged,
            compute: &compute,
            capability: capability.as_deref(),
        })
        .map_err(|error| error.to_string())
    }

    /// Whether the envelope names this request's current-framing revision.
    /// Every attempt retained before `lattice-worker-profile-revision-v2` fails.
    fn is_current_profile_revision(&self) -> Result<bool, String> {
        Ok(self.worker_profile_identity()? == self.envelope.worker_profile)
    }

    pub fn validate(&self, now_ms: u64) -> Result<(), String> {
        self.envelope.validate().map_err(|error| error.to_string())?;
        self.validate_binding()?;
        self.validate_execution(now_ms)
    }

    fn validate_binding(&self) -> Result<(), String> {
        if self.schema != REQUEST_SCHEMA || self.admitted_envelope != self.envelope.identity() {
            return Err("implementation request is stale, crossed or unsupported".into());
        }
        if !self.is_current_profile_revision()? {
            return Err(refusal::Reason::ProfileRevisionSuperseded.message().into());
        }
        refusal::validate_coordinator(self)?;
        binding::validate_request(self)
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
// r[impl lattice.execution_binding.port_boundary.scenario.unavailable]
// r[impl lattice.serving_binding.port_boundary.scenario.unavailable]
pub async fn run_command(args: &CommandArgs) -> Result<(), String> {
    let ports = Ports {
        compute: &mut crate::execution_binding::UnavailableComputePort,
        serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
    };
    run_command_with_ports(args, ports).await
}

/// Embedder-supplied read-only ports of one run.
pub struct Ports<'a> {
    pub compute: &'a mut dyn crate::execution_binding::ComputeAllocationPort,
    pub serving: Box<dyn crate::execution_binding::serving::OwnerPort>,
}

/// Run with embedder-supplied compute and serving-owner ports. `inspect`
/// reads only the serving port's latest snapshots; it never admits a request.
pub async fn run_command_with_ports(args: &CommandArgs, mut ports: Ports<'_>) -> Result<(), String> {
    match &args.command {
        Command::Cancel { request, state } => cancellation::request(request, state),
        Command::Prepare { request, output } => prepare(request, output),
        Command::Run { request, state } => execution::run(request, state, Some(ports)).await,
        Command::Recover { request, state } => execution::run(request, state, None).await,
        Command::Inspect { request, state } => inspect::run(request, state, ports.serving.as_mut()),
        Command::ExportCandidate {
            request,
            state,
            output_parent,
        } => export::run(request, state, output_parent),
    }
}

fn normalized_request(request_path: &std::path::Path) -> Result<(Request, Vec<u8>), String> {
    let (mut request, _) = artifacts::load_request(request_path)?;
    let baseline = crate::candidate_capture::workspace(&request.workspace_root).map_err(|error| error.to_string())?;
    request.omp.base_identity = crate::candidate::snapshot_identity(&baseline).map_err(|error| error.to_string())?;
    request.envelope.worker_profile = request.worker_profile_identity()?;
    request.admitted_envelope = request.envelope.identity();
    binding::normalize_primary_route(&mut request)?;
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

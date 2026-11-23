//! Imperative shell adapters for launching an admitted candidate worker process.

static UNIT_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Port for executing an admitted candidate worker process.
pub trait SandboxLauncher: Send + Sync {
    fn execute(
        &self,
        process: &crate::agent_node::ProcessRequest,
        config: &crate::process::core::RuntimeConfig,
        cancellation: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::process::shell::Observation, crate::agent_node::runtime::host::ProcessError>;
}

/// Default direct-child launcher; works on hosts without systemd.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Direct;

impl SandboxLauncher for Direct {
    fn execute(
        &self,
        process: &crate::agent_node::ProcessRequest,
        config: &crate::process::core::RuntimeConfig,
        cancellation: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::process::shell::Observation, crate::agent_node::runtime::host::ProcessError> {
        crate::process::shell::run_bounded_clear_environment_observed_with_cancellation(
            process,
            config,
            Some(cancellation),
        )
    }
}

/// Transient systemd service launcher allowing hardened dispatchers to keep
/// `ProtectKernelTunables=yes` and `AF_UNIX`-only address families while
/// launching `bwrap` with byte-identical arguments inside its own unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Systemd {
    config: super::systemd::Config,
}

impl Systemd {
    pub fn new(config: super::systemd::Config) -> Result<Self, &'static str> {
        super::systemd::validate_config(&config)?;
        Ok(Self { config })
    }

    pub fn config(&self) -> &super::systemd::Config {
        &self.config
    }
}

impl SandboxLauncher for Systemd {
    fn execute(
        &self,
        process: &crate::agent_node::ProcessRequest,
        config: &crate::process::core::RuntimeConfig,
        cancellation: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::process::shell::Observation, crate::agent_node::runtime::host::ProcessError> {
        let systemd_run = canonical_executable(ExecutableSpec {
            configured: &self.config.systemd_run_program,
            diagnostic: "configured systemd-run executable is unavailable",
        })?;
        let systemctl = canonical_executable(ExecutableSpec {
            configured: &self.config.systemctl_program,
            diagnostic: "configured systemctl executable is unavailable",
        })?;
        let token = next_unit_token(process);
        let plan = super::systemd::plan_launch(super::systemd::PlanInput {
            config: &self.config,
            process,
            teardown_grace_millis: config.limits.teardown_grace_millis,
            unit_token: &token,
        })
        .map_err(crate::agent_node::runtime::host::ProcessError::InvalidRequest)?;
        if cancellation.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(cancelled_before_spawn());
        }
        let wrapped = crate::agent_node::ProcessRequest {
            program: systemd_run,
            args: plan.launch_args,
            cwd: process.cwd.clone(),
            env: plan.client_env.clone(),
            stdin: process.stdin.clone(),
            timeout_secs: process.timeout_secs,
            output_limit_bytes: process.output_limit_bytes,
        };
        let teardown = super::watchdog::TeardownContext {
            systemctl,
            stop_args: plan.stop_args,
            client_env: plan.client_env,
            cwd: process.cwd.clone(),
            runtime_config: config,
        };
        super::watchdog::run_and_reconcile(&wrapped, cancellation, &teardown)
    }
}

struct ExecutableSpec<'a> {
    configured: &'a str,
    diagnostic: &'static str,
}

fn canonical_executable(spec: ExecutableSpec<'_>) -> Result<String, crate::agent_node::runtime::host::ProcessError> {
    let path = std::fs::canonicalize(spec.configured)
        .map_err(|_| crate::agent_node::runtime::host::ProcessError::InvalidRequest(spec.diagnostic))?;
    if !path.is_file() {
        return Err(crate::agent_node::runtime::host::ProcessError::InvalidRequest(spec.diagnostic));
    }
    debug_assert!(path.is_absolute());
    path.into_os_string()
        .into_string()
        .map_err(|_| crate::agent_node::runtime::host::ProcessError::InvalidRequest(spec.diagnostic))
}

fn next_unit_token(process: &crate::agent_node::ProcessRequest) -> String {
    let sequence = UNIT_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let pid = std::process::id();
    let mut hasher = blake3::Hasher::new();
    hasher.update(&pid.to_le_bytes());
    hasher.update(&sequence.to_le_bytes());
    hasher.update(process.program.as_bytes());
    for arg in &process.args {
        hasher.update(&[0]);
        hasher.update(arg.as_bytes());
    }
    super::systemd::token_from_hash(hasher.finalize())
}

fn cancelled_before_spawn() -> crate::process::shell::Observation {
    crate::process::shell::Observation {
        output: crate::agent_node::ProcessOutput {
            status_code: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
        },
        stop: Some(crate::agent_node::runtime::host::ProcessError::Cancelled),
    }
}

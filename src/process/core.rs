//! Pure admission policy for bounded local processes.

const MINIMUM_POSITIVE_VALUE: usize = 1;
const BYTES_PER_KIBIBYTE: usize = 1_024;
const KIBIBYTES_PER_MEBIBYTE: usize = 1_024;
const DEFAULT_STREAM_LIMIT_MEBIBYTES: usize = 64;
const DEFAULT_STREAM_MAX_BYTES: usize = DEFAULT_STREAM_LIMIT_MEBIBYTES
    .saturating_mul(KIBIBYTES_PER_MEBIBYTE)
    .saturating_mul(BYTES_PER_KIBIBYTE);
const DEFAULT_COMMAND_AGENT_TIMEOUT_SECS: u64 = 600;
const DEFAULT_POLL_INTERVAL_MILLIS: u64 = 10;
const DEFAULT_TEARDOWN_GRACE_MILLIS: u64 = 1_000;
const DEFAULT_SHELL_PROGRAM: &str = "sh";
const ENVIRONMENT_ASSIGNMENT_SEPARATOR: char = '=';
const NUL_CHARACTER: char = '\0';

pub(super) const INVALID_PROGRAM_DIAGNOSTIC: &str = "local process program must resolve to an absolute executable";
pub(super) const INVALID_CURRENT_DIR_DIAGNOSTIC: &str =
    "local process current directory must resolve to an absolute directory";
pub(super) const INVALID_LIMIT_DIAGNOSTIC: &str = "local process limits and deadline must be positive and supported";
pub(super) const EXECUTION_FAILURE_DIAGNOSTIC: &str = "bounded local process execution failed";
const INVALID_ARGUMENT_DIAGNOSTIC: &str = "local process arguments must not contain null bytes";
const INVALID_ENVIRONMENT_DIAGNOSTIC: &str =
    "local process environment entries must be supported, non-empty, and unique";
const INVALID_STDIN_LIMIT_DIAGNOSTIC: &str = "local process stdin exceeds the configured bound";
const INVALID_OUTCOME_POLICY_DIAGNOSTIC: &str = "local process accepted-exit policy must be finite and unique";

fn default_stdin_max_bytes() -> usize {
    DEFAULT_STREAM_MAX_BYTES
}

fn default_stdout_max_bytes() -> usize {
    DEFAULT_STREAM_MAX_BYTES
}

fn default_stderr_max_bytes() -> usize {
    DEFAULT_STREAM_MAX_BYTES
}

fn default_command_agent_timeout_secs() -> u64 {
    DEFAULT_COMMAND_AGENT_TIMEOUT_SECS
}

fn default_poll_interval_millis() -> u64 {
    DEFAULT_POLL_INTERVAL_MILLIS
}

fn default_teardown_grace_millis() -> u64 {
    DEFAULT_TEARDOWN_GRACE_MILLIS
}

/// Typed, reviewable bounds for every production local-process path.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LimitConfig {
    #[serde(default = "default_stdin_max_bytes")]
    pub stdin_max_bytes: usize,
    #[serde(default = "default_stdout_max_bytes")]
    pub stdout_max_bytes: usize,
    #[serde(default = "default_stderr_max_bytes")]
    pub stderr_max_bytes: usize,
    #[serde(default = "default_command_agent_timeout_secs")]
    pub command_agent_timeout_secs: u64,
    #[serde(default = "default_poll_interval_millis")]
    pub poll_interval_millis: u64,
    #[serde(default = "default_teardown_grace_millis")]
    pub teardown_grace_millis: u64,
}

impl Default for LimitConfig {
    fn default() -> Self {
        Self {
            stdin_max_bytes: default_stdin_max_bytes(),
            stdout_max_bytes: default_stdout_max_bytes(),
            stderr_max_bytes: default_stderr_max_bytes(),
            command_agent_timeout_secs: default_command_agent_timeout_secs(),
            poll_interval_millis: default_poll_interval_millis(),
            teardown_grace_millis: default_teardown_grace_millis(),
        }
    }
}

/// Runtime-only settings derived from typed application config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub shell_program: String,
    pub limits: LimitConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            shell_program: DEFAULT_SHELL_PROGRAM.to_owned(),
            limits: LimitConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionInput {
    pub program: std::path::PathBuf,
    pub args: Vec<String>,
    pub current_dir: std::path::PathBuf,
    pub environment_mode: bounded_exec::EnvironmentMode,
    pub environment: Vec<(String, String)>,
    pub stdin: Vec<u8>,
    pub timeout: std::time::Duration,
    pub stdout_max_bytes: usize,
    pub stderr_max_bytes: usize,
    pub stdin_max_bytes: usize,
    pub poll_interval: std::time::Duration,
    pub teardown_grace: std::time::Duration,
    pub accepted_exit_codes: Vec<i32>,
    pub reject_stdout_truncation: bool,
    pub reject_stderr_truncation: bool,
    pub termination_scope: bounded_exec::TerminationScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedRequest {
    pub program: std::path::PathBuf,
    pub args: Vec<String>,
    pub current_dir: std::path::PathBuf,
    pub environment_mode: bounded_exec::EnvironmentMode,
    pub environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    pub stdin: Vec<u8>,
    pub timeout: std::time::Duration,
    pub stdout_max_bytes: usize,
    pub stderr_max_bytes: usize,
    pub stdin_max_bytes: usize,
    pub poll_interval: std::time::Duration,
    pub teardown_grace: std::time::Duration,
    pub outcome_policy: bounded_exec::OutcomePolicy,
    pub termination_scope: bounded_exec::TerminationScope,
}

/// Pure admission core. No filesystem, process, environment, or clock access.
// r[impl lattice.workflow_execution.bounded_process_policy]
pub fn admit(input: AdmissionInput) -> Result<AdmittedRequest, crate::agent_node::ProcessError> {
    if !input.program.is_absolute() {
        return Err(crate::agent_node::ProcessError::InvalidRequest(INVALID_PROGRAM_DIAGNOSTIC));
    }
    if !input.current_dir.is_absolute() {
        return Err(crate::agent_node::ProcessError::InvalidRequest(INVALID_CURRENT_DIR_DIAGNOSTIC));
    }
    if limits_are_invalid(&input) {
        return Err(crate::agent_node::ProcessError::InvalidRequest(INVALID_LIMIT_DIAGNOSTIC));
    }
    if input.stdin.len() > input.stdin_max_bytes {
        return Err(crate::agent_node::ProcessError::InvalidRequest(INVALID_STDIN_LIMIT_DIAGNOSTIC));
    }
    if input.args.iter().any(|argument| argument.contains(NUL_CHARACTER)) {
        return Err(crate::agent_node::ProcessError::InvalidRequest(INVALID_ARGUMENT_DIAGNOSTIC));
    }
    debug_assert!(input.program.is_absolute());
    debug_assert!(input.current_dir.is_absolute());

    let outcome_policy = bounded_exec::OutcomePolicy::new(
        input.accepted_exit_codes,
        input.reject_stdout_truncation,
        input.reject_stderr_truncation,
    )
    .map_err(|_| crate::agent_node::ProcessError::InvalidRequest(INVALID_OUTCOME_POLICY_DIAGNOSTIC))?;
    let environment = admit_environment(input.environment)?;

    Ok(AdmittedRequest {
        program: input.program,
        args: input.args,
        current_dir: input.current_dir,
        environment_mode: input.environment_mode,
        environment,
        stdin: input.stdin,
        timeout: input.timeout,
        stdout_max_bytes: input.stdout_max_bytes,
        stderr_max_bytes: input.stderr_max_bytes,
        stdin_max_bytes: input.stdin_max_bytes,
        poll_interval: input.poll_interval,
        teardown_grace: input.teardown_grace,
        outcome_policy,
        termination_scope: input.termination_scope,
    })
}

fn limits_are_invalid(input: &AdmissionInput) -> bool {
    if input.timeout.is_zero() {
        return true;
    }
    if input.stdout_max_bytes < MINIMUM_POSITIVE_VALUE {
        return true;
    }
    if input.stderr_max_bytes < MINIMUM_POSITIVE_VALUE {
        return true;
    }
    if input.stdin_max_bytes < MINIMUM_POSITIVE_VALUE {
        return true;
    }
    if input.poll_interval.is_zero() || input.poll_interval > input.timeout {
        return true;
    }
    input.teardown_grace.is_zero()
}

fn admit_environment(
    entries: Vec<(String, String)>,
) -> Result<Vec<(std::ffi::OsString, std::ffi::OsString)>, crate::agent_node::ProcessError> {
    let mut keys = std::collections::BTreeSet::new();
    let mut environment = Vec::with_capacity(entries.len());
    for (key, value) in entries {
        if environment_entry_is_invalid((&key, &value), &mut keys) {
            return Err(crate::agent_node::ProcessError::InvalidRequest(INVALID_ENVIRONMENT_DIAGNOSTIC));
        }
        environment.push((std::ffi::OsString::from(key), std::ffi::OsString::from(value)));
    }
    Ok(environment)
}

fn environment_entry_is_invalid(entry: (&str, &str), keys: &mut std::collections::BTreeSet<String>) -> bool {
    let (key, value) = entry;
    if key.is_empty() {
        return true;
    }
    if key.contains(ENVIRONMENT_ASSIGNMENT_SEPARATOR) || key.contains(NUL_CHARACTER) {
        return true;
    }
    if value.contains(NUL_CHARACTER) {
        return true;
    }
    !keys.insert(key.to_owned())
}

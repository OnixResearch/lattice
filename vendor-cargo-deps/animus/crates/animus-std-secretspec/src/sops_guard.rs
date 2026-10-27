//! Read-only bounded guard for the SOPS executable selected by SecretSpec.

use bounded_exec::{
    CommandSpec, Disposition, EnvironmentMode, ExecutionLimits, Input, OutcomePolicy, RunRequest,
    TerminationScope,
};
use std::ffi::OsString;
use std::path::PathBuf;

/// Absolute real SOPS executable environment key.
pub const SOPS_EXECUTABLE_ENV: &str = "ANIMUS_SOPS_EXECUTABLE";
/// Inner SOPS deadline environment key.
pub const SOPS_TIMEOUT_ENV: &str = "ANIMUS_SOPS_TIMEOUT_MS";
/// Inner SOPS output bound environment key.
pub const SOPS_OUTPUT_MAX_ENV: &str = "ANIMUS_SOPS_OUTPUT_MAX_BYTES";
/// Inner SOPS stderr bound environment key.
pub const SOPS_STDERR_MAX_ENV: &str = "ANIMUS_SOPS_STDERR_MAX_BYTES";
/// Inner SOPS poll interval environment key.
pub const SOPS_POLL_INTERVAL_ENV: &str = "ANIMUS_SOPS_POLL_INTERVAL_MS";
/// Inner SOPS teardown bound environment key.
pub const SOPS_TEARDOWN_TIMEOUT_ENV: &str = "ANIMUS_SOPS_TEARDOWN_TIMEOUT_MS";

const NULL_STDIN_BOUND_BYTES: usize = 1;
const SUCCESS_EXIT_CODE: i32 = 0;

/// Safe marker printed when the configured SOPS executable is absent.
pub const MARKER_EXECUTABLE_MISSING: &str = "ANIMUS_SOPS_EXECUTABLE_MISSING";
/// Safe marker printed when SOPS output exceeds its bound.
pub const MARKER_OUTPUT_LIMIT: &str = "ANIMUS_SOPS_OUTPUT_LIMIT";
/// Safe marker printed when SOPS output is not valid JSON.
pub const MARKER_MALFORMED_OUTPUT: &str = "ANIMUS_SOPS_MALFORMED_OUTPUT";
/// Safe marker printed for nonzero SOPS execution.
pub const MARKER_PROCESS_FAILED: &str = "ANIMUS_SOPS_PROCESS_FAILED";
/// Safe marker printed for a SOPS deadline.
pub const MARKER_TIMEOUT: &str = "ANIMUS_SOPS_TIMEOUT";
/// Safe marker printed for a rejected argument shape.
pub const MARKER_ARGUMENTS_REJECTED: &str = "ANIMUS_SOPS_ARGUMENTS_REJECTED";

/// Guard failure with a safe marker and process exit code.
pub struct GuardFailure {
    marker: &'static str,
    exit_code: i32,
}

impl GuardFailure {
    /// Return the safe stderr marker.
    #[must_use]
    pub const fn marker(&self) -> &'static str {
        self.marker
    }

    /// Return the stable nonzero exit code.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        self.exit_code
    }
}

/// Run one exact read-only SOPS decrypt command with bounded output.
///
/// # Errors
///
/// Returns a safe marker without child stderr or private paths.
pub fn run_sops_guard(args: Vec<OsString>) -> Result<Vec<u8>, GuardFailure> {
    validate_args(&args)?;
    let program = required_absolute_path(SOPS_EXECUTABLE_ENV, MARKER_EXECUTABLE_MISSING)?;
    let current_dir = std::env::current_dir().map_err(|_| failure(MARKER_PROCESS_FAILED))?;
    if !current_dir.is_absolute() {
        return Err(failure(MARKER_PROCESS_FAILED));
    }
    let limits = ExecutionLimits {
        timeout_ms: required_u64(SOPS_TIMEOUT_ENV)?,
        stdin_max_bytes: NULL_STDIN_BOUND_BYTES,
        stdout_max_bytes: required_usize(SOPS_OUTPUT_MAX_ENV)?,
        stderr_max_bytes: required_usize(SOPS_STDERR_MAX_ENV)?,
        poll_interval_ms: required_u64(SOPS_POLL_INTERVAL_ENV)?,
        teardown_timeout_ms: required_u64(SOPS_TEARDOWN_TIMEOUT_ENV)?,
    };
    limits
        .validate()
        .map_err(|_| failure(MARKER_ARGUMENTS_REJECTED))?;
    let outcome_policy = OutcomePolicy::new(vec![SUCCESS_EXIT_CODE], true, true)
        .map_err(|_| failure(MARKER_PROCESS_FAILED))?;
    let output = bounded_exec::run(RunRequest {
        command: CommandSpec {
            program,
            args,
            current_dir,
            environment_mode: EnvironmentMode::Inherit,
            environment: Vec::new(),
            input: Input::Null,
        },
        limits,
        termination_scope: TerminationScope::Child,
        outcome_policy,
    })
    .map_err(|error| match error {
        bounded_exec::RunError::Io {
            operation: bounded_exec::Operation::Spawn,
            ..
        } => failure(MARKER_EXECUTABLE_MISSING),
        _ => failure(MARKER_PROCESS_FAILED),
    })?;
    match output.disposition {
        Disposition::Succeeded => {}
        Disposition::TimedOut | Disposition::Cancelled => return Err(failure(MARKER_TIMEOUT)),
        Disposition::OutputLimitExceeded(_) => return Err(failure(MARKER_OUTPUT_LIMIT)),
        Disposition::ExitFailed => return Err(failure(MARKER_PROCESS_FAILED)),
    }
    serde_json::from_slice::<serde_json::Value>(&output.stdout.bytes)
        .map_err(|_| failure(MARKER_MALFORMED_OUTPUT))?;
    Ok(output.stdout.bytes)
}

fn validate_args(args: &[OsString]) -> Result<(), GuardFailure> {
    const SHORT_ARG_COUNT: usize = 4;
    const LONG_ARG_COUNT: usize = 6;
    let valid = match args {
        [decrypt, output_flag, output_type, path] => {
            args.len() == SHORT_ARG_COUNT
                && decrypt == "decrypt"
                && output_flag == "--output-type"
                && output_type == "json"
                && !path.is_empty()
        }
        [
            decrypt,
            input_flag,
            input_type,
            output_flag,
            output_type,
            path,
        ] => {
            args.len() == LONG_ARG_COUNT
                && decrypt == "decrypt"
                && input_flag == "--input-type"
                && matches!(input_type.to_str(), Some("json" | "yaml"))
                && output_flag == "--output-type"
                && output_type == "json"
                && !path.is_empty()
        }
        _ => false,
    };
    valid
        .then_some(())
        .ok_or_else(|| failure(MARKER_ARGUMENTS_REJECTED))
}

fn required_absolute_path(key: &str, marker: &'static str) -> Result<PathBuf, GuardFailure> {
    let value = std::env::var_os(key).ok_or_else(|| failure(marker))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(failure(marker));
    }
    Ok(path)
}

fn required_u64(key: &str) -> Result<u64, GuardFailure> {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| failure(MARKER_ARGUMENTS_REJECTED))
}

fn required_usize(key: &str) -> Result<usize, GuardFailure> {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| failure(MARKER_ARGUMENTS_REJECTED))
}

const fn failure(marker: &'static str) -> GuardFailure {
    const FAILURE_EXIT_CODE: i32 = 74;
    GuardFailure {
        marker,
        exit_code: FAILURE_EXIT_CODE,
    }
}

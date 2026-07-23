//! Imperative shell for resolving and executing admitted local processes.

const SUCCESS_EXIT_CODE: i32 = 0;
const OUTPUT_OVERFLOW_SENTINEL_BYTES: usize = 1;
const MINIMUM_PATH_COMPONENTS: usize = 1;
const PATH_ENVIRONMENT_KEY: &str = "PATH";
const CURRENT_DIRECTORY: &str = ".";
const NUL_CHARACTER: char = '\0';

// r[impl lattice.workflow_execution.bounded_process_mechanics]
// r[impl lattice.credentials.bounded_command_provider]
pub fn run_bounded(
    request: &crate::agent_node::ProcessRequest,
    config: &super::core::RuntimeConfig,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
) -> Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError> {
    let ambient_current_dir = std::path::Path::new(CURRENT_DIRECTORY)
        .canonicalize()
        .map_err(|_| crate::agent_node::ProcessError::Spawn(super::core::EXECUTION_FAILURE_DIAGNOSTIC.to_owned()))?;
    let current_dir = resolve_current_dir(request.cwd.as_deref().map(std::path::Path::new), &ambient_current_dir)?;
    if request.program.contains(NUL_CHARACTER) {
        return Err(crate::agent_node::ProcessError::InvalidRequest(super::core::INVALID_PROGRAM_DIAGNOSTIC));
    }
    let program = resolve_program(&request.program, &current_dir, &request.env)?;
    debug_assert!(current_dir.is_absolute());
    debug_assert!(program.is_absolute());
    let output_limit_bytes = request.output_limit_bytes;
    let admitted = super::core::admit(super::core::AdmissionInput {
        program,
        args: request.args.clone(),
        current_dir,
        environment_mode: bounded_exec::EnvironmentMode::Inherit,
        environment: request.env.iter().map(|(key, value)| (key.clone(), value.clone())).collect(),
        stdin: request.stdin.clone(),
        timeout: std::time::Duration::from_secs(request.timeout_secs),
        stdout_max_bytes: config.limits.stdout_max_bytes.min(output_limit_bytes),
        stderr_max_bytes: config.limits.stderr_max_bytes.min(output_limit_bytes),
        stdin_max_bytes: config.limits.stdin_max_bytes,
        poll_interval: std::time::Duration::from_millis(config.limits.poll_interval_millis),
        teardown_grace: std::time::Duration::from_millis(config.limits.teardown_grace_millis),
        accepted_exit_codes: vec![SUCCESS_EXIT_CODE],
        reject_stdout_truncation: true,
        reject_stderr_truncation: true,
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
    })?;
    execute_admitted(admitted, cancellation)
}

fn resolve_current_dir(
    requested: Option<&std::path::Path>,
    ambient: &std::path::Path,
) -> Result<std::path::PathBuf, crate::agent_node::ProcessError> {
    let resolved = match requested {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => ambient.join(path),
        None => ambient.to_path_buf(),
    };
    match std::fs::metadata(&resolved) {
        Ok(metadata) if metadata.is_dir() => Ok(resolved),
        Ok(_) | Err(_) => {
            Err(crate::agent_node::ProcessError::InvalidRequest(super::core::INVALID_CURRENT_DIR_DIAGNOSTIC))
        }
    }
}

fn resolve_program(
    configured: &str,
    current_dir: &std::path::Path,
    environment: &std::collections::BTreeMap<String, String>,
) -> Result<std::path::PathBuf, crate::agent_node::ProcessError> {
    debug_assert!(current_dir.is_absolute());
    debug_assert!(!configured.contains(NUL_CHARACTER));
    let requested = std::path::Path::new(configured);
    if requested.is_absolute() {
        return executable_candidate(requested.to_path_buf());
    }
    if requested.components().count() > MINIMUM_PATH_COMPONENTS {
        return executable_candidate(current_dir.join(requested));
    }

    let resolved = match environment.get(PATH_ENVIRONMENT_KEY) {
        Some(configured_path) => {
            let absolute_path = absolute_search_path(configured_path, current_dir)?;
            which::which_in(configured, Some(absolute_path), current_dir)
        }
        None => which::which(configured),
    };
    resolved.map_err(|_| crate::agent_node::ProcessError::InvalidRequest(super::core::INVALID_PROGRAM_DIAGNOSTIC))
}

fn absolute_search_path(
    configured_path: &str,
    current_dir: &std::path::Path,
) -> Result<std::ffi::OsString, crate::agent_node::ProcessError> {
    debug_assert!(current_dir.is_absolute());
    std::env::join_paths(std::env::split_paths(&std::ffi::OsString::from(configured_path)).map(|path_entry| {
        if path_entry.is_absolute() {
            return path_entry;
        }
        current_dir.join(path_entry)
    }))
    .map_err(|_| crate::agent_node::ProcessError::InvalidRequest(super::core::INVALID_PROGRAM_DIAGNOSTIC))
}

fn executable_candidate(candidate: std::path::PathBuf) -> Result<std::path::PathBuf, crate::agent_node::ProcessError> {
    if candidate.is_file() {
        return Ok(candidate);
    }
    Err(crate::agent_node::ProcessError::InvalidRequest(super::core::INVALID_PROGRAM_DIAGNOSTIC))
}

fn execute_admitted(
    admitted: super::core::AdmittedRequest,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
) -> Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError> {
    debug_assert!(admitted.program.is_absolute());
    debug_assert!(admitted.current_dir.is_absolute());
    debug_assert!(admitted.stdout_max_bytes > 0);
    debug_assert!(admitted.stderr_max_bytes > 0);
    let input = if admitted.stdin.is_empty() {
        bounded_exec::Input::Null
    } else {
        bounded_exec::Input::Bytes(admitted.stdin)
    };
    let bounded_request = bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: admitted.program,
            args: admitted.args.into_iter().map(std::ffi::OsString::from).collect(),
            current_dir: admitted.current_dir,
            environment_mode: admitted.environment_mode,
            environment: admitted.environment,
            input,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms: duration_millis(admitted.timeout)?,
            stdin_max_bytes: admitted.stdin_max_bytes,
            stdout_max_bytes: admitted.stdout_max_bytes,
            stderr_max_bytes: admitted.stderr_max_bytes,
            poll_interval_ms: duration_millis(admitted.poll_interval)?,
            teardown_timeout_ms: duration_millis(admitted.teardown_grace)?,
        },
        outcome_policy: admitted.outcome_policy,
        termination_scope: admitted.termination_scope,
    };
    let execution_output = match cancellation {
        Some(signal) => bounded_exec::run_with_cancellation(bounded_request, signal),
        None => bounded_exec::run(bounded_request),
    }
    .map_err(map_execution_error)?;
    classify_output(execution_output, OutputLimits {
        stdout_max_bytes: admitted.stdout_max_bytes,
        stderr_max_bytes: admitted.stderr_max_bytes,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OutputLimits {
    stdout_max_bytes: usize,
    stderr_max_bytes: usize,
}

fn classify_output(
    execution_output: bounded_exec::ExecutionOutput,
    limits: OutputLimits,
) -> Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError> {
    debug_assert!(limits.stdout_max_bytes > 0);
    debug_assert!(limits.stderr_max_bytes > 0);
    let stdout_facts = CaptureFacts {
        truncated: execution_output.stdout.truncated,
        observed_bytes: execution_output.stdout.observed_bytes,
        limit_bytes: limits.stdout_max_bytes,
    };
    let stderr_facts = CaptureFacts {
        truncated: execution_output.stderr.truncated,
        observed_bytes: execution_output.stderr.observed_bytes,
        limit_bytes: limits.stderr_max_bytes,
    };
    if stdout_facts.truncated || stderr_facts.truncated {
        return Err(truncation_error(stdout_facts, stderr_facts));
    }
    match execution_output.disposition {
        bounded_exec::Disposition::TimedOut => Err(crate::agent_node::ProcessError::Timeout),
        bounded_exec::Disposition::Cancelled => Err(crate::agent_node::ProcessError::Cancelled),
        bounded_exec::Disposition::OutputLimitExceeded(_) => Err(truncation_error(stdout_facts, stderr_facts)),
        bounded_exec::Disposition::Succeeded | bounded_exec::Disposition::ExitFailed => {
            Ok(crate::agent_node::ProcessOutput {
                status_code: execution_output.exit_code,
                stdout: execution_output.stdout.bytes,
                stderr: execution_output.stderr.bytes,
            })
        }
    }
}

fn duration_millis(duration: std::time::Duration) -> Result<u64, crate::agent_node::ProcessError> {
    u64::try_from(duration.as_millis())
        .map_err(|_| crate::agent_node::ProcessError::InvalidRequest(super::core::INVALID_LIMIT_DIAGNOSTIC))
}

fn map_execution_error(_error: bounded_exec::RunError) -> crate::agent_node::ProcessError {
    crate::agent_node::ProcessError::Spawn(super::core::EXECUTION_FAILURE_DIAGNOSTIC.to_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CaptureFacts {
    truncated: bool,
    observed_bytes: usize,
    limit_bytes: usize,
}

fn truncation_error(stdout: CaptureFacts, stderr: CaptureFacts) -> crate::agent_node::ProcessError {
    let (limit_bytes, observed_bytes) = match (stdout.truncated, stderr.truncated) {
        (true, true) => (stdout.limit_bytes.min(stderr.limit_bytes), stdout.observed_bytes.max(stderr.observed_bytes)),
        (true, false) => (stdout.limit_bytes, stdout.observed_bytes),
        (false, true) => (stderr.limit_bytes, stderr.observed_bytes),
        (false, false) => {
            (stdout.limit_bytes.min(stderr.limit_bytes), stdout.observed_bytes.max(stderr.observed_bytes))
        }
    };
    crate::agent_node::ProcessError::OutputLimitExceeded {
        limit: limit_bytes,
        actual: observed_bytes.max(limit_bytes.saturating_add(OUTPUT_OVERFLOW_SENTINEL_BYTES)),
    }
}

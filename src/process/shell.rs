//! Imperative shell for resolving and executing admitted local processes.

#[cfg(test)]
mod failure_tests;
mod output;

const SUCCESS_EXIT_CODE: i32 = 0;
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
    run_bounded_with_environment(request, config, cancellation, bounded_exec::EnvironmentMode::Inherit)
}

pub fn run_bounded_clear_environment(
    request: &crate::agent_node::ProcessRequest,
    config: &super::core::RuntimeConfig,
) -> Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError> {
    run_bounded_clear_environment_with_cancellation(request, config, None)
}

pub fn run_bounded_clear_environment_with_cancellation(
    request: &crate::agent_node::ProcessRequest,
    config: &super::core::RuntimeConfig,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
) -> Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError> {
    run_bounded_with_environment(request, config, cancellation, bounded_exec::EnvironmentMode::Clear)
}

/// Bounded bytes survive observed timeout/cancellation/output-limit stops.
/// Infrastructure RunError still cannot expose bytes the producer discarded.
#[derive(Debug)]
pub struct Observation {
    pub output: crate::agent_node::ProcessOutput,
    pub stop: Option<crate::agent_node::ProcessError>,
}

pub fn run_bounded_clear_environment_observed_with_cancellation(
    request: &crate::agent_node::ProcessRequest,
    config: &super::core::RuntimeConfig,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
) -> Result<Observation, crate::agent_node::ProcessError> {
    run_observed_with_environment(request, config, cancellation, bounded_exec::EnvironmentMode::Clear)
}

fn run_bounded_with_environment(
    request: &crate::agent_node::ProcessRequest,
    config: &super::core::RuntimeConfig,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
    environment_mode: bounded_exec::EnvironmentMode,
) -> Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError> {
    let observation = run_observed_with_environment(request, config, cancellation, environment_mode)?;
    match observation.stop {
        Some(error) => Err(error),
        None => Ok(observation.output),
    }
}

fn run_observed_with_environment(
    request: &crate::agent_node::ProcessRequest,
    config: &super::core::RuntimeConfig,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
    environment_mode: bounded_exec::EnvironmentMode,
) -> Result<Observation, crate::agent_node::ProcessError> {
    let ambient_current_dir = std::path::Path::new(CURRENT_DIRECTORY)
        .canonicalize()
        .map_err(|_| crate::agent_node::ProcessError::InvalidRequest(super::core::INVALID_CURRENT_DIR_DIAGNOSTIC))?;
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
        environment_mode,
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
) -> Result<Observation, crate::agent_node::ProcessError> {
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
    Ok(output::classify(execution_output, output::Limits {
        stdout_max_bytes: admitted.stdout_max_bytes,
        stderr_max_bytes: admitted.stderr_max_bytes,
    }))
}

fn duration_millis(duration: std::time::Duration) -> Result<u64, crate::agent_node::ProcessError> {
    u64::try_from(duration.as_millis())
        .map_err(|_| crate::agent_node::ProcessError::InvalidRequest(super::core::INVALID_LIMIT_DIAGNOSTIC))
}

// r[impl lattice.candidate_worker.process_observation]
// r[impl lattice.candidate_worker.process_observation.scenario.cleanup]
pub(crate) fn map_execution_error(error: bounded_exec::RunError) -> crate::agent_node::ProcessError {
    let (phase, termination) = match error {
        bounded_exec::RunError::InvalidLimits(_)
        | bounded_exec::RunError::EmptyProgram
        | bounded_exec::RunError::ProgramNotAbsolute
        | bounded_exec::RunError::EmptyCurrentDirectory
        | bounded_exec::RunError::CurrentDirectoryNotAbsolute
        | bounded_exec::RunError::DuplicateEnvironmentName
        | bounded_exec::RunError::InputLimitExceeded => {
            return crate::agent_node::ProcessError::InvalidRequest("bounded execution request rejected");
        }
        bounded_exec::RunError::Io {
            operation: bounded_exec::Operation::Spawn,
            ..
        } => return crate::agent_node::ProcessError::Spawn(super::core::EXECUTION_FAILURE_DIAGNOSTIC.to_owned()),
        // Wait is returned only after the pinned runner successfully reaps the child.
        bounded_exec::RunError::Io {
            operation: bounded_exec::Operation::Wait,
            ..
        } => (crate::agent_node::ProcessFailurePhase::Execution, crate::agent_node::ProcessTermination::Observed),
        bounded_exec::RunError::Io {
            operation: bounded_exec::Operation::Kill | bounded_exec::Operation::Reap,
            ..
        }
        | bounded_exec::RunError::ChildTeardownTimedOut
        | bounded_exec::RunError::ProcessIdentifierOverflow => {
            (crate::agent_node::ProcessFailurePhase::Teardown, crate::agent_node::ProcessTermination::Unknown)
        }
        // Missing pipes and stream failures follow successful child teardown.
        bounded_exec::RunError::MissingPipe(_)
        | bounded_exec::RunError::CaptureCountOverflow(_)
        | bounded_exec::RunError::WorkerDisconnected(_)
        | bounded_exec::RunError::WorkerPanicked(_)
        | bounded_exec::RunError::Io { .. } => {
            (crate::agent_node::ProcessFailurePhase::Capture, crate::agent_node::ProcessTermination::Observed)
        }
        bounded_exec::RunError::WorkerTeardownTimedOut(_) => {
            (crate::agent_node::ProcessFailurePhase::Teardown, crate::agent_node::ProcessTermination::Observed)
        }
    };
    debug_assert!(
        termination != crate::agent_node::ProcessTermination::Unknown
            || phase == crate::agent_node::ProcessFailurePhase::Teardown
    );
    debug_assert!(
        phase != crate::agent_node::ProcessFailurePhase::Execution
            || termination == crate::agent_node::ProcessTermination::Observed
    );
    crate::agent_node::ProcessError::Execution { phase, termination }
}

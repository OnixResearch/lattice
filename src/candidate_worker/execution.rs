/// The blocker of a worker the host stopped at its deadline.
pub(super) const DEADLINE_BLOCKER: &str = "worker exceeded its admitted deadline; termination observed";
/// The blocker of a worker the host stopped at its output limit.
pub(super) const OUTPUT_LIMIT_BLOCKER: &str = "worker exceeded its admitted output limit; termination observed";
/// The prefix of a non-zero exit's blocker, followed by `{status:?}`.
pub(super) const EXIT_BLOCKER_PREFIX: &str = "worker exited with status ";

pub(super) fn run(
    request: &super::Request,
    config: &crate::process::core::RuntimeConfig,
    cancellation: &std::sync::atomic::AtomicBool,
    expected_base: Option<&str>,
    execute: impl FnOnce(
        &crate::agent_node::ProcessRequest,
        &crate::process::core::RuntimeConfig,
        &std::sync::atomic::AtomicBool,
    ) -> Result<crate::process::shell::Observation, crate::agent_node::runtime::host::ProcessError>,
) -> Result<super::Outcome, super::Error> {
    if let Err(outcome) = admit(request) {
        return Ok(outcome);
    }
    let workspace = Workspace::capture(&request.boundary.root)?;
    if !workspace.matches_base(expected_base)? {
        return Ok(blocked("workspace base differs from the admitted source snapshot"));
    }
    let root = request.boundary.root.as_path();
    let modes = crate::candidate_capture::snapshot_modes(root, &workspace.base)?;
    let process = match process_request(request, workspace.cwd) {
        Ok(process) => process,
        Err(blocker) => return Ok(super::Outcome::Blocked { blocker }),
    };
    match observed_output(root, execute(&process, config, cancellation)) {
        Ok(output) => super::completion::finish(request, root, workspace.base, modes, output),
        Err(outcome) => Ok(outcome),
    }
}

struct Workspace {
    cwd: String,
    base: std::collections::BTreeMap<String, Vec<u8>>,
}

impl Workspace {
    fn capture(root: &std::path::Path) -> Result<Self, super::Error> {
        let cwd = root.to_str().ok_or(super::Error::NonUtf8Root)?.to_string();
        let base = crate::candidate_capture::workspace(root)?;
        Ok(Self { cwd, base })
    }

    fn matches_base(&self, expected: Option<&str>) -> Result<bool, super::Error> {
        match expected {
            Some(expected) => Ok(crate::candidate::snapshot_identity(&self.base)? == expected),
            None => Ok(true),
        }
    }
}

fn process_request(request: &super::Request, cwd: String) -> Result<crate::agent_node::ProcessRequest, String> {
    let (program, args) = if request.require_isolation {
        super::sandbox::command(request)?
    } else {
        (request.program.clone(), request.args.clone())
    };
    Ok(crate::agent_node::ProcessRequest {
        program,
        args,
        cwd: Some(cwd),
        env: std::collections::BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: request.timeout_secs,
        output_limit_bytes: request.output_limit_bytes,
    })
}

// r[impl lattice.candidate_worker.process_observation.scenario.interrupted]
fn interrupted(
    root: &std::path::Path,
    failure: super::ProcessFailure,
    output: crate::agent_node::ProcessOutput,
) -> super::Outcome {
    // A best-effort snapshot is explicitly non-atomic when termination is unknown.
    // Never derive a usable candidate or silently erase the workspace on this path.
    super::Outcome::Interrupted {
        failure,
        observed_files: crate::candidate_capture::workspace(root).ok(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

fn admit(request: &super::Request) -> Result<(), super::Outcome> {
    if request.timeout_secs == 0 || request.output_limit_bytes == 0 {
        return Err(blocked("worker deadline and output limit must be positive"));
    }
    if request.boundary.credentials {
        return Err(blocked("candidate worker does not admit credential-bearing workspaces"));
    }
    admit_boundary(&request.boundary)?;
    admit_bindings(request)?;
    if !request.boundary.root.is_dir() {
        return Err(blocked("admitted workspace root is unavailable"));
    }
    Ok(())
}

fn admit_boundary(boundary: &crate::candidate_capture::WorkspaceBoundary) -> Result<(), super::Outcome> {
    let readmitted = crate::candidate_capture::admit_workspace_boundary(
        &boundary.root,
        &boundary.read_paths,
        &boundary.write_paths,
        boundary.network,
        boundary.credentials,
    );
    if readmitted.as_ref().ok() != Some(boundary) {
        return Err(blocked("workspace boundary is not canonical or no longer admitted"));
    }
    Ok(())
}

fn admit_bindings(request: &super::Request) -> Result<(), super::Outcome> {
    if [
        &request.work_id,
        &request.attempt_id,
        &request.fence,
        &request.workspace_id,
    ]
    .iter()
    .any(|id| id.is_empty() || id.len() > crate::candidate::MAX_IDENTIFIER_BYTES || id.chars().any(char::is_control))
    {
        return Err(blocked("worker attempt bindings are invalid"));
    }
    Ok(())
}

fn observed_output(
    root: &std::path::Path,
    observation: Result<crate::process::shell::Observation, crate::agent_node::runtime::host::ProcessError>,
) -> Result<crate::agent_node::ProcessOutput, super::Outcome> {
    let (output, stop) = match observation {
        Ok(observation) => (observation.output, observation.stop),
        Err(error) => (
            crate::agent_node::ProcessOutput {
                status_code: None,
                stdout: Vec::new(),
                stderr: Vec::new(),
            },
            Some(error),
        ),
    };
    match stop {
        Some(error) => Err(stopped_output(root, output, error)),
        None => successful_output(output),
    }
}

fn stopped_output(
    root: &std::path::Path,
    output: crate::agent_node::ProcessOutput,
    error: crate::agent_node::runtime::host::ProcessError,
) -> super::Outcome {
    match error {
        crate::agent_node::runtime::host::ProcessError::Cancelled => super::Outcome::Cancelled {
            stdout: output.stdout,
            stderr: output.stderr,
        },
        crate::agent_node::runtime::host::ProcessError::Timeout => failed(output, DEADLINE_BLOCKER),
        crate::agent_node::runtime::host::ProcessError::OutputLimitExceeded { .. } => {
            failed(output, OUTPUT_LIMIT_BLOCKER)
        }
        crate::agent_node::runtime::host::ProcessError::Execution { phase, termination } => {
            interrupted(root, super::ProcessFailure { phase, termination }, output)
        }
        crate::agent_node::runtime::host::ProcessError::InvalidRequest(_) => {
            blocked("worker request rejected before spawn")
        }
        crate::agent_node::runtime::host::ProcessError::Spawn(_) => {
            blocked("worker spawn failed; process did not start")
        }
    }
}

fn successful_output(
    output: crate::agent_node::ProcessOutput,
) -> Result<crate::agent_node::ProcessOutput, super::Outcome> {
    if output.status_code != Some(0) {
        let blocker = format!("{EXIT_BLOCKER_PREFIX}{:?}", output.status_code);
        return Err(failed(output, &blocker));
    }
    Ok(output)
}

fn blocked(blocker: &str) -> super::Outcome {
    super::Outcome::Blocked {
        blocker: blocker.to_string(),
    }
}

fn failed(output: crate::agent_node::ProcessOutput, blocker: &str) -> super::Outcome {
    super::Outcome::Failed {
        blocker: blocker.to_string(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

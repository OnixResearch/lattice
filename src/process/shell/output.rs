const OVERFLOW_SENTINEL_BYTES: usize = 1;

pub(super) struct Limits {
    pub(super) stdout_max_bytes: usize,
    pub(super) stderr_max_bytes: usize,
}

pub(super) fn classify(execution: bounded_exec::ExecutionOutput, limits: Limits) -> super::Observation {
    debug_assert!(limits.stdout_max_bytes > 0);
    debug_assert!(limits.stderr_max_bytes > 0);
    let stdout = Facts {
        truncated: execution.stdout.truncated,
        observed_bytes: execution.stdout.observed_bytes,
        limit_bytes: limits.stdout_max_bytes,
    };
    let stderr = Facts {
        truncated: execution.stderr.truncated,
        observed_bytes: execution.stderr.observed_bytes,
        limit_bytes: limits.stderr_max_bytes,
    };
    let stop = match execution.disposition {
        bounded_exec::Disposition::TimedOut => Some(crate::agent_node::ProcessError::Timeout),
        bounded_exec::Disposition::Cancelled => Some(crate::agent_node::ProcessError::Cancelled),
        _ if stdout.truncated || stderr.truncated => Some(truncation_error(stdout, stderr)),
        bounded_exec::Disposition::OutputLimitExceeded(_) => Some(truncation_error(stdout, stderr)),
        bounded_exec::Disposition::Succeeded | bounded_exec::Disposition::ExitFailed => None,
    };
    super::Observation {
        output: crate::agent_node::ProcessOutput {
            status_code: execution.exit_code,
            stdout: execution.stdout.bytes,
            stderr: execution.stderr.bytes,
        },
        stop,
    }
}

#[derive(Clone, Copy)]
struct Facts {
    truncated: bool,
    observed_bytes: usize,
    limit_bytes: usize,
}

fn truncation_error(stdout: Facts, stderr: Facts) -> crate::agent_node::ProcessError {
    let (limit_bytes, observed_bytes) = match (stdout.truncated, stderr.truncated) {
        (true, true) => (stdout.limit_bytes.min(stderr.limit_bytes), stdout.observed_bytes.max(stderr.observed_bytes)),
        (true, false) => (stdout.limit_bytes, stdout.observed_bytes),
        (false, true) => (stderr.limit_bytes, stderr.observed_bytes),
        (false, false) => {
            (stdout.limit_bytes.min(stderr.limit_bytes), stdout.observed_bytes.max(stderr.observed_bytes))
        }
    };
    crate::agent_node::ProcessError::OutputLimitExceeded {
        limit_bytes,
        actual_bytes: observed_bytes.max(limit_bytes.saturating_add(OVERFLOW_SENTINEL_BYTES)),
    }
}

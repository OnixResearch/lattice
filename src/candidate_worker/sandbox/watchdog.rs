//! Scoped cancellation/deadline watchdog and synchronous `systemctl stop`
//! reconciliation for transient systemd worker units.

const SYSTEMCTL_NOT_LOADED_EXIT_CODE: i32 = 5;
const TEARDOWN_MIN_TIMEOUT_SECS: u64 = 5;
const TEARDOWN_MAX_TIMEOUT_SECS: u64 = 30;
const TEARDOWN_EXTRA_SECS: u64 = 5;
const TEARDOWN_OUTPUT_LIMIT_BYTES: usize = 65_536;
const WATCHDOG_MIN_POLL_MILLIS: u64 = 10;
const WATCHDOG_MAX_POLL_MILLIS: u64 = 50;
const WATCHDOG_DEADLINE_POLL_MARGIN: u64 = 2;
const WATCHDOG_CHANNEL_CAPACITY: usize = 1;
const MILLIS_PER_SEC: u64 = 1_000;
const ROUND_UP_OFFSET_MILLIS: u64 = 999;

pub(super) struct TeardownContext<'a> {
    pub(super) systemctl: String,
    pub(super) stop_args: Vec<String>,
    pub(super) client_env: std::collections::BTreeMap<String, String>,
    pub(super) cwd: Option<String>,
    pub(super) runtime_config: &'a crate::process::core::RuntimeConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Outcome {
    NotTriggered,
    Cancelled { is_stopped: bool },
    Deadline { is_stopped: bool },
}

pub(super) fn run_and_reconcile(
    wrapped: &crate::agent_node::ProcessRequest,
    cancellation: &std::sync::atomic::AtomicBool,
    teardown: &TeardownContext<'_>,
) -> Result<crate::process::shell::Observation, crate::agent_node::runtime::host::ProcessError> {
    let (done_tx, done_rx) = std::sync::mpsc::sync_channel::<()>(WATCHDOG_CHANNEL_CAPACITY);
    let (result, outcome) = std::thread::scope(|scope| {
        let handle = scope.spawn(move || watch_unit(wrapped.timeout_secs, cancellation, teardown, done_rx));
        let result = crate::process::shell::run_bounded_clear_environment_observed_with_cancellation(
            wrapped,
            teardown.runtime_config,
            Some(cancellation),
        );
        drop(done_tx);
        let outcome = handle.join().unwrap_or(Outcome::NotTriggered);
        (result, outcome)
    });
    reconcile_unit(result, outcome, teardown)
}

fn watch_unit(
    timeout_secs: u64,
    cancellation: &std::sync::atomic::AtomicBool,
    teardown: &TeardownContext<'_>,
    done_rx: std::sync::mpsc::Receiver<()>,
) -> Outcome {
    let poll_millis = teardown
        .runtime_config
        .limits
        .poll_interval_millis
        .clamp(WATCHDOG_MIN_POLL_MILLIS, WATCHDOG_MAX_POLL_MILLIS);
    debug_assert!(poll_millis >= WATCHDOG_MIN_POLL_MILLIS);
    debug_assert!(poll_millis <= WATCHDOG_MAX_POLL_MILLIS);
    let poll = std::time::Duration::from_millis(poll_millis);
    let total_millis = timeout_secs
        .saturating_mul(MILLIS_PER_SEC)
        .saturating_add(poll_millis.saturating_mul(WATCHDOG_DEADLINE_POLL_MARGIN));
    let ticks = total_millis.checked_div(poll_millis).unwrap_or(0);
    for _tick in 0..ticks {
        if cancellation.load(std::sync::atomic::Ordering::Acquire) {
            return Outcome::Cancelled {
                is_stopped: stop_unit(teardown),
            };
        }
        match done_rx.recv_timeout(poll) {
            Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return Outcome::NotTriggered,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
    if cancellation.load(std::sync::atomic::Ordering::Acquire) {
        return Outcome::Cancelled {
            is_stopped: stop_unit(teardown),
        };
    }
    Outcome::Deadline {
        is_stopped: stop_unit(teardown),
    }
}

fn reconcile_unit(
    result: Result<crate::process::shell::Observation, crate::agent_node::runtime::host::ProcessError>,
    outcome: Outcome,
    teardown: &TeardownContext<'_>,
) -> Result<crate::process::shell::Observation, crate::agent_node::runtime::host::ProcessError> {
    match result {
        Ok(observation) => Ok(reconcile_observation(observation, outcome, teardown)),
        Err(error @ crate::agent_node::runtime::host::ProcessError::Spawn(_))
        | Err(error @ crate::agent_node::runtime::host::ProcessError::InvalidRequest(_)) => Err(error),
        Err(error) => {
            if matches!(outcome, Outcome::NotTriggered) {
                let _ = stop_unit(teardown);
            }
            Err(error)
        }
    }
}

fn reconcile_observation(
    mut observation: crate::process::shell::Observation,
    outcome: Outcome,
    teardown: &TeardownContext<'_>,
) -> crate::process::shell::Observation {
    let (is_stopped, fallback) = match outcome {
        Outcome::NotTriggered => {
            let is_stopped = observation.stop.is_none() || stop_unit(teardown);
            (is_stopped, None)
        }
        Outcome::Cancelled { is_stopped } => {
            (is_stopped, Some(crate::agent_node::runtime::host::ProcessError::Cancelled))
        }
        Outcome::Deadline { is_stopped } => (is_stopped, Some(crate::agent_node::runtime::host::ProcessError::Timeout)),
    };
    if !is_stopped {
        observation.stop = Some(unknown_teardown_error());
    } else if observation.stop.is_none() {
        observation.stop = fallback;
    }
    observation
}

fn unknown_teardown_error() -> crate::agent_node::runtime::host::ProcessError {
    crate::agent_node::runtime::host::ProcessError::Execution {
        phase: crate::agent_node::ProcessFailurePhase::Teardown,
        termination: crate::agent_node::ProcessTermination::Unknown,
    }
}

fn stop_unit(teardown: &TeardownContext<'_>) -> bool {
    debug_assert!(!teardown.systemctl.is_empty());
    debug_assert!(!teardown.stop_args.is_empty());
    let grace_secs =
        teardown.runtime_config.limits.teardown_grace_millis.saturating_add(ROUND_UP_OFFSET_MILLIS) / MILLIS_PER_SEC;
    let timeout_secs = grace_secs
        .saturating_add(TEARDOWN_EXTRA_SECS)
        .clamp(TEARDOWN_MIN_TIMEOUT_SECS, TEARDOWN_MAX_TIMEOUT_SECS);
    let request = crate::agent_node::ProcessRequest {
        program: teardown.systemctl.clone(),
        args: teardown.stop_args.clone(),
        cwd: teardown.cwd.clone(),
        env: teardown.client_env.clone(),
        stdin: Vec::new(),
        timeout_secs,
        output_limit_bytes: TEARDOWN_OUTPUT_LIMIT_BYTES,
    };
    let observation = crate::process::shell::run_bounded_clear_environment_observed_with_cancellation(
        &request,
        teardown.runtime_config,
        None,
    );
    match observation {
        Ok(observed) if observed.stop.is_none() => {
            matches!(observed.output.status_code, Some(0) | Some(SYSTEMCTL_NOT_LOADED_EXIT_CODE))
        }
        Ok(_) | Err(_) => false,
    }
}

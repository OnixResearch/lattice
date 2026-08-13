//! Shared test helpers — workflow builders, temp stores, assertion utilities.
#![allow(dead_code)]

pub mod authority;

use std::collections::HashMap;

use lattice::engine::value::ContentHash;
use lattice::engine::value::Value;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::PortId;
use lattice::engine::workflow::Workflow;
use lattice::store::Store;
use tempfile::TempDir;

const SHELL_STDOUT_OUTPUT_INDEX: usize = 0;
const SHELL_STDERR_OUTPUT_INDEX: usize = 1;
const SHELL_EXIT_OUTPUT_INDEX: usize = 2;

#[cfg(unix)]
pub const FORCED_DEATH_MARKER_CONTENT: &str = "forced-death";
#[cfg(unix)]
pub const PROCESS_GATE_READY_CONTENT: &str = "ready";
#[cfg(unix)]
const PROCESS_GATE_POLL_INTERVAL_SECONDS: &str = "0.01";
const POST_EFFECT_AT_MOST_ONE_ATTEMPT_COUNT: usize = 1;

pub struct ChildGuard {
    child: Option<std::process::Child>,
}

impl ChildGuard {
    pub fn new(child: std::process::Child) -> Self {
        Self { child: Some(child) }
    }

    pub fn child_mut(&mut self) -> &mut std::process::Child {
        self.child.as_mut().expect("guarded child remains owned")
    }

    pub fn wait_with_output(mut self) -> std::process::Output {
        self.child
            .take()
            .expect("guarded child remains owned")
            .wait_with_output()
            .expect("guarded child output remains observable")
    }

    pub fn wait_for_exit(mut self, attempts: usize, poll_delay: std::time::Duration) -> std::process::ExitStatus {
        assert!(attempts > 0, "child wait must be bounded by at least one attempt");
        for _ in 0..attempts {
            let status = self
                .child
                .as_mut()
                .expect("guarded child remains owned")
                .try_wait()
                .expect("guarded child status remains observable");
            if let Some(status) = status {
                self.child = None;
                return status;
            }
            std::thread::sleep(poll_delay);
        }
        let mut child = self.child.take().expect("guarded child remains owned");
        let _ = child.kill();
        let _ = child.wait();
        panic!("child did not exit within the bounded wait");
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn wait_for_path(path: &std::path::Path, attempts: usize, poll_delay: std::time::Duration) {
    assert!(attempts > 0, "path wait must be bounded by at least one attempt");
    for _ in 0..attempts {
        if path.exists() {
            return;
        }
        std::thread::sleep(poll_delay);
    }
    panic!("path did not appear within the bounded wait: {}", path.display());
}

#[cfg(unix)]
fn quote_for_posix_shell(value: &str) -> String {
    assert!(!value.contains('\0'), "shell fixture value must not contain NUL");
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(unix)]
fn quote_path_for_posix_shell(path: &std::path::Path) -> String {
    assert!(path.is_absolute(), "fault-injection fixture paths must be absolute");
    quote_for_posix_shell(path.to_str().expect("fault-injection fixture path must be UTF-8"))
}

/// Build a deterministic Unix shell command that kills its Lattice parent after
/// recording a durable checkpoint.
#[cfg(unix)]
pub fn forced_parent_death_command(marker_path: &std::path::Path) -> String {
    let marker = quote_path_for_posix_shell(marker_path);
    format!("printf %s '{}' > {marker}; kill -KILL \"$PPID\"; exit", FORCED_DEATH_MARKER_CONTENT)
}

/// Kill the first Lattice parent at the workflow boundary, then emit one
/// recovery effect when the same workflow is started by a later process.
#[cfg(unix)]
pub fn forced_parent_death_once_then_write_command(
    marker_path: &std::path::Path,
    recovery_effect_path: &std::path::Path,
    recovery_effect: &str,
) -> String {
    assert!(!recovery_effect.is_empty(), "recovery effect must be observable");
    let marker = quote_path_for_posix_shell(marker_path);
    let recovery_effect_path = quote_path_for_posix_shell(recovery_effect_path);
    let recovery_effect = quote_for_posix_shell(recovery_effect);
    format!(
        "if [ ! -e {marker} ]; then printf %s '{}' > {marker}; kill -KILL \"$PPID\"; exit; fi; printf %s {recovery_effect} > {recovery_effect_path}",
        FORCED_DEATH_MARKER_CONTENT,
    )
}

/// Hold a workflow effect behind an observable file gate so another process
/// can contend while the first Lattice process still owns its state store.
#[cfg(unix)]
pub fn gated_write_command(
    ready_path: &std::path::Path,
    release_path: &std::path::Path,
    effect_path: &std::path::Path,
    effect: &str,
) -> String {
    assert!(!effect.is_empty(), "gated effect must be observable");
    let ready = quote_path_for_posix_shell(ready_path);
    let release = quote_path_for_posix_shell(release_path);
    let effect_path = quote_path_for_posix_shell(effect_path);
    let effect = quote_for_posix_shell(effect);
    format!(
        "printf %s '{}' > {ready}; while [ ! -e {release} ]; do sleep {PROCESS_GATE_POLL_INTERVAL_SECONDS}; done; printf %s {effect} > {effect_path}",
        PROCESS_GATE_READY_CONTENT,
    )
}

/// Append one identified effect for concurrent execution-count assertions.
#[cfg(unix)]
pub fn append_effect_command(effect_path: &std::path::Path, effect: &str) -> String {
    assert!(!effect.is_empty(), "appended effect must be observable");
    let effect_path = quote_path_for_posix_shell(effect_path);
    let effect = quote_for_posix_shell(effect);
    format!("printf '%s\\n' {effect} >> {effect_path}")
}

/// Record one effect attempt on every invocation and kill the first Lattice
/// parent only after the attempt and checkpoint are observable.
#[cfg(unix)]
pub fn post_effect_parent_death_once_then_repeat_command(
    checkpoint_path: &std::path::Path,
    attempts_path: &std::path::Path,
    effect: &str,
) -> String {
    assert!(!effect.is_empty(), "post-effect attempt must be observable");
    let checkpoint = quote_path_for_posix_shell(checkpoint_path);
    let attempts = quote_path_for_posix_shell(attempts_path);
    let effect = quote_for_posix_shell(effect);
    format!(
        "printf '%s\\n' {effect} >> {attempts}; sync; if [ ! -e {checkpoint} ]; then printf %s '{}' > {checkpoint}; sync; kill -KILL \"$PPID\"; exit; fi",
        FORCED_DEATH_MARKER_CONTENT,
    )
}

#[cfg(unix)]
pub fn identified_write_command(path: &std::path::Path, effect: &str) -> String {
    assert!(!effect.is_empty(), "identified write effect must be observable");
    let path = quote_path_for_posix_shell(path);
    let effect = quote_for_posix_shell(effect);
    format!("printf %s {effect} > {path}")
}

#[cfg(unix)]
pub fn timeout_once_then_succeed_command(
    marker_path: &std::path::Path,
    failure_delay_seconds: u64,
    failure_token: &str,
    success_token: &str,
) -> String {
    assert!(failure_delay_seconds > 0, "failure delay must be positive");
    assert!(!failure_token.is_empty(), "failure token must be observable");
    assert!(!success_token.is_empty(), "success token must be observable");
    let marker = quote_path_for_posix_shell(marker_path);
    let failure_token = quote_for_posix_shell(failure_token);
    let success_token = quote_for_posix_shell(success_token);
    format!(
        "if [ ! -e {marker} ]; then printf %s {failure_token} > {marker}; sleep {failure_delay_seconds}; fi; printf %s {success_token}"
    )
}

#[cfg(unix)]
pub fn ready_then_delayed_effect_command(
    ready_path: &std::path::Path,
    effect_path: &std::path::Path,
    delay_seconds: u64,
    effect: &str,
) -> String {
    assert!(delay_seconds > 0, "delayed effect requires a positive delay");
    assert!(!effect.is_empty(), "delayed effect must be observable");
    let ready = quote_path_for_posix_shell(ready_path);
    let effect_path = quote_path_for_posix_shell(effect_path);
    let effect = quote_for_posix_shell(effect);
    format!(
        "printf %s '{}' > {ready}; sleep {delay_seconds}; printf %s {effect} > {effect_path}",
        PROCESS_GATE_READY_CONTENT,
    )
}

#[cfg(unix)]
pub fn copy_crash_store_snapshot(source: &std::path::Path, destination: &std::path::Path) -> u64 {
    assert_ne!(source, destination, "crash snapshot must not replace the original store");
    let copied = std::fs::copy(source, destination).expect("crash store snapshot copies");
    assert!(copied > 0, "crash store snapshot must not be empty");
    copied
}

#[cfg(unix)]
pub fn identified_effect_attempt_count(path: &std::path::Path, expected_effect: &str) -> usize {
    assert!(!expected_effect.is_empty(), "expected effect must be identified");
    let contents = std::fs::read_to_string(path).expect("effect-attempt file reads");
    let attempts = contents.lines().collect::<Vec<_>>();
    assert!(!attempts.is_empty(), "at least one effect attempt must be observable");
    assert!(
        attempts.iter().all(|attempt| *attempt == expected_effect),
        "every effect attempt must match the reviewed identity"
    );
    attempts.len()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostEffectIngress {
    Scheduler,
    Webhook,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostEffectHttpOutcome {
    NotApplicable,
    FirstIncompleteThenConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostEffectCrashProfile {
    pub ingress: PostEffectIngress,
    pub pre_restart_run_count: usize,
    pub schedule_remains_due: Option<bool>,
    pub final_attempt_count: usize,
    pub final_run_count: usize,
    pub final_started_history_count: usize,
    pub final_rejected_history_count: usize,
    pub http_outcome: PostEffectHttpOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostEffectCrashObservation {
    pub ingress: PostEffectIngress,
    pub checkpoint_observed: bool,
    pub pre_restart_run_count: usize,
    pub schedule_remains_due: Option<bool>,
    pub final_attempt_count: usize,
    pub final_run_count: usize,
    pub final_started_history_count: usize,
    pub final_rejected_history_count: usize,
    pub http_outcome: PostEffectHttpOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostEffectCrashError {
    MissingCheckpoint,
    UnsupportedOutcome,
    PrematureTerminalRecord,
    PreRestartRunCountMismatch,
    ScheduleStateMismatch,
    AttemptCountMismatch,
    FinalRunCountMismatch,
    TriggerHistoryMismatch,
    HttpOutcomeMismatch,
}

pub const SCHEDULER_POST_EFFECT_PROFILE: PostEffectCrashProfile = PostEffectCrashProfile {
    ingress: PostEffectIngress::Scheduler,
    pre_restart_run_count: 0,
    schedule_remains_due: Some(false),
    final_attempt_count: POST_EFFECT_AT_MOST_ONE_ATTEMPT_COUNT,
    final_run_count: 0,
    final_started_history_count: 0,
    final_rejected_history_count: 0,
    http_outcome: PostEffectHttpOutcome::NotApplicable,
};

pub const WEBHOOK_POST_EFFECT_CLAIM_PROFILE: PostEffectCrashProfile = PostEffectCrashProfile {
    ingress: PostEffectIngress::Webhook,
    pre_restart_run_count: 0,
    schedule_remains_due: None,
    final_attempt_count: POST_EFFECT_AT_MOST_ONE_ATTEMPT_COUNT,
    final_run_count: 0,
    final_started_history_count: 1,
    final_rejected_history_count: 0,
    http_outcome: PostEffectHttpOutcome::FirstIncompleteThenConflict,
};

pub fn classify_post_effect_crash(
    profile: PostEffectCrashProfile,
    observation: PostEffectCrashObservation,
) -> Result<(), PostEffectCrashError> {
    if !observation.checkpoint_observed {
        return Err(PostEffectCrashError::MissingCheckpoint);
    }
    if observation.ingress != profile.ingress {
        return Err(PostEffectCrashError::UnsupportedOutcome);
    }
    if observation.pre_restart_run_count > 0 && profile.pre_restart_run_count == 0 {
        return Err(PostEffectCrashError::PrematureTerminalRecord);
    }
    if observation.pre_restart_run_count != profile.pre_restart_run_count {
        return Err(PostEffectCrashError::PreRestartRunCountMismatch);
    }
    if observation.schedule_remains_due != profile.schedule_remains_due {
        return Err(PostEffectCrashError::ScheduleStateMismatch);
    }
    if observation.final_attempt_count != profile.final_attempt_count {
        return Err(PostEffectCrashError::AttemptCountMismatch);
    }
    if observation.final_run_count != profile.final_run_count {
        return Err(PostEffectCrashError::FinalRunCountMismatch);
    }
    if observation.final_started_history_count != profile.final_started_history_count
        || observation.final_rejected_history_count != profile.final_rejected_history_count
    {
        return Err(PostEffectCrashError::TriggerHistoryMismatch);
    }
    if observation.http_outcome != profile.http_outcome {
        return Err(PostEffectCrashError::HttpOutcomeMismatch);
    }
    Ok(())
}

/// Build a simple linear two-node workflow: echo → grep.
/// Returns (workflow, node_a_id, node_b_id).
pub fn two_node_pipeline() -> (Workflow, lattice::engine::workflow::NodeId, lattice::engine::workflow::NodeId) {
    let mut wf = Workflow::new("test-pipeline", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let a = wf.add_shell_node("echo-hello", "echo hello world");
    let b = wf.add_shell_node("grep-hello", "grep hello");

    // Wire a's stdout (output port 0) → b's stdin (input port 0).
    let a_stdout = stdout_port_id(&wf, a);
    let b_stdin = wf.graph.node(b).unwrap().input_ports[0].id;
    wf.graph.add_edge(a_stdout, b_stdin).unwrap();

    (wf, a, b)
}

/// Build a diamond-shaped workflow:
///
///       A
///      / \
///     B   C
///      \ /
///       D
pub fn diamond_workflow() -> (
    Workflow,
    lattice::engine::workflow::NodeId,
    lattice::engine::workflow::NodeId,
    lattice::engine::workflow::NodeId,
    lattice::engine::workflow::NodeId,
) {
    let mut wf = Workflow::new("diamond", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let a = wf.add_shell_node("source", "echo diamond");
    let b = wf.add_shell_node("left", "cat");
    let c = wf.add_shell_node("right", "cat");
    let d = wf.add_shell_node("sink", "cat");

    // A → B
    let a_out = stdout_port_id(&wf, a);
    let b_in = wf.graph.node(b).unwrap().input_ports[0].id;
    wf.graph.add_edge(a_out, b_in).unwrap();

    // A → C  (use stderr port from A since stdout is taken; or add another node)
    // Actually, we need separate edges. Let's use A's stderr → C.
    let a_stderr = stderr_port_id(&wf, a);
    let c_in = wf.graph.node(c).unwrap().input_ports[0].id;
    wf.graph.add_edge(a_stderr, c_in).unwrap();

    // B → D
    let b_out = stdout_port_id(&wf, b);
    let d_in = wf.graph.node(d).unwrap().input_ports[0].id;
    wf.graph.add_edge(b_out, d_in).unwrap();

    (wf, a, b, c, d)
}

/// Build a single standalone node (no edges).
pub fn standalone_node(label: &str, command: &str) -> (Workflow, lattice::engine::workflow::NodeId) {
    let mut wf = Workflow::new("standalone", lattice::engine::workflow::WorkflowTimestamp::epoch());
    let id = wf.add_shell_node(label, command);
    (wf, id)
}

/// Create a Store backed by a temp directory. Returns (Store, TempDir).
/// Keep TempDir alive for the duration of the test — dropping it deletes the db.
pub fn temp_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let db_path = dir.path().join("test.redb");
    let store = Store::new(&db_path).expect("failed to create store");
    (store, dir)
}

pub fn stdout_port_id(workflow: &Workflow, node_id: NodeId) -> PortId {
    workflow.graph.node(node_id).expect("node exists").output_ports[SHELL_STDOUT_OUTPUT_INDEX].id
}

pub fn stderr_port_id(workflow: &Workflow, node_id: NodeId) -> PortId {
    workflow.graph.node(node_id).expect("node exists").output_ports[SHELL_STDERR_OUTPUT_INDEX].id
}

pub fn exit_port_id(workflow: &Workflow, node_id: NodeId) -> PortId {
    workflow.graph.node(node_id).expect("node exists").output_ports[SHELL_EXIT_OUTPUT_INDEX].id
}

/// Build a fake cache entry: maps an input hash to a set of real port→value outputs.
pub fn fake_cache_entry(
    workflow: &Workflow,
    node_id: NodeId,
    input_hash: ContentHash,
    stdout_val: &str,
) -> (ContentHash, HashMap<PortId, Value>) {
    let mut outputs = HashMap::new();
    outputs.insert(stdout_port_id(workflow, node_id), Value::from_string(stdout_val.into()));
    outputs.insert(stderr_port_id(workflow, node_id), Value::from_string(String::new()));
    outputs.insert(exit_port_id(workflow, node_id), Value::from_exit_code(0));
    (input_hash, outputs)
}

//! A scripted fake OMP executable, an unsandboxed catalog-check executor,
//! the pinned catalog and operator profiles.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

pub(super) type Hook = Box<dyn FnMut() + Send>;

/// What one launch of the fake OMP does before it prints a completed
/// terminal stream.
#[derive(Debug, Clone, Copy)]
pub(in super::super) enum Behaviour {
    /// Write this text to `src/lib.txt`.
    Write(&'static str),
    /// Exit with this status before any terminal event.
    Exit(i32),
    /// Kill itself with `SIGKILL`.
    Killed,
    /// Run until it is stopped.
    Spin,
}

/// The operator profile and catalog a run is given.
#[derive(Debug, Clone, Copy)]
pub(in super::super) enum Profile {
    /// Grants the worker and check tuples of the request's revision.
    Granting,
    /// No configuration, so no profile is selected.
    Unconfigured,
    /// Grants the worker tuple only.
    WorkerOnly,
    /// Grants both tuples of another worker profile revision.
    OtherRevision,
    /// A granting profile that has expired.
    Expired,
    /// A granting profile, but no `--repair-checks` catalog.
    NoCatalog,
}

/// The default check: passes exactly when `src/lib.txt` reads `fixed`.
pub(in super::super) const CHECK: &str = "read -r source < src/lib.txt; test \"$source\" = fixed";
pub(super) const TERMINAL: [&str; 5] = [
    r#"{"type":"session","version":3,"id":"fake-session","cwd":"/workspace"}"#,
    r#"{"type":"agent_start"}"#,
    r#"{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"done"}}"#,
    r#"{"type":"message_end","message":{"role":"assistant","stopReason":"stop"}}"#,
    r#"{"type":"agent_end","messages":[{"role":"assistant","stopReason":"stop"}]}"#,
];

/// Runs the scripted fake OMP through the OMP launcher's argv, process,
/// capture and JSONL observation path, without the sandbox or broker.
pub(super) struct Scripted {
    pub(super) launches: Arc<AtomicU32>,
    pub(super) after: Arc<Mutex<Option<Hook>>>,
    /// Runs once, at the start of the first repair launch.
    pub(super) before_repair: Arc<Mutex<Option<Hook>>>,
    pub(super) is_lost: Arc<AtomicBool>,
}

impl crate::implementation_attempt::cli::repair::operator::Launch for Scripted {
    fn launch(
        &mut self,
        worker: &crate::candidate_worker::Request,
        profile: &crate::candidate_worker::omp::Profile,
        cancellation: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::candidate_worker::omp::RunResult, String> {
        let launch = self.launches.fetch_add(1, Ordering::SeqCst);
        if launch > 0
            && let Some(mut before) = self.before_repair.lock().expect("hook").take()
        {
            before();
        }
        let result = crate::candidate_worker::omp::run_scripted(worker, profile, cancellation);
        assert!(!self.is_lost.load(Ordering::SeqCst), "controller lost after the OMP step");
        if let Some(mut after) = self.after.lock().expect("hook").take() {
            after();
        }
        result.map_err(|error| error.to_string())
    }
}

/// Runs a catalog check's pinned process through the candidate worker's
/// generic route without the sandbox.
pub(super) struct Unsandboxed {
    pub(super) runs: Arc<AtomicU32>,
    /// Runs once, after the first check.
    pub(super) after: Arc<Mutex<Option<Hook>>>,
}

impl crate::implementation_attempt::cli::repair::operator::checks::Execute for Unsandboxed {
    fn execute(
        &mut self,
        check: &crate::candidate_worker::Request,
        cancellation: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::candidate_worker::Outcome, String> {
        assert!(check.require_isolation && check.broker.is_none() && !check.boundary.network);
        assert!(check.boundary.write_paths.is_empty(), "a check writes nothing");
        self.runs.fetch_add(1, Ordering::SeqCst);
        let mut process = check.clone();
        process.require_isolation = false;
        process.sandbox_program = None;
        let config = crate::process::core::RuntimeConfig::default();
        let outcome = crate::candidate_worker::run(&process, &config, cancellation).map_err(|error| error.to_string());
        if let Some(mut after) = self.after.lock().expect("hook").take() {
            after();
        }
        outcome
    }
}

/// A shell inside the Nix store, the pinned program of the catalog check.
fn store_shell() -> crate::candidate_worker::broker::Executable {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let candidates = std::env::split_paths(&path)
        .map(|directory| directory.join("sh"))
        .chain([std::path::PathBuf::from("/bin/sh")]);
    let shell = candidates
        .filter_map(|candidate| std::fs::canonicalize(candidate).ok())
        .find(|candidate| candidate.starts_with("/nix/store") && candidate.is_file())
        .expect("a shell inside the Nix store");
    let blake3 = blake3::hash(&std::fs::read(&shell).expect("shell bytes")).to_hex().to_string();
    crate::candidate_worker::broker::Executable { path: shell, blake3 }
}

/// The operator catalog of a `unit` check running `check`, and with
/// `second` a `unit-second` check running it, in the pinned Nix store shell.
pub(super) fn catalog(check: &str, second: Option<&str>) -> serde_json::Value {
    let shell = store_shell();
    let mut checks = vec![serde_json::json!(
        {"name": "unit", "program": shell, "args": ["-c", check], "failure_class": "test"}
    )];
    if let Some(second) = second {
        checks.push(serde_json::json!(
            {"name": "unit-second", "program": shell, "args": ["-c", second], "failure_class": "test"}
        ));
    }
    serde_json::json!({
        "schema": crate::implementation_attempt::repair::checks::catalog::SCHEMA,
        "sandbox": shell,
        "checks": checks,
    })
}

/// The policy naming `catalog`'s executor and checks.
pub(super) fn policy(
    catalog: &serde_json::Value,
    max_iterations: u32,
    max_elapsed_ms: u64,
) -> crate::implementation_attempt::repair::Policy {
    let bytes = serde_json::to_vec(catalog).expect("catalog bytes");
    let parsed = crate::implementation_attempt::repair::checks::catalog::parse(&bytes).expect("catalog");
    let steps = u64::from(max_iterations) + 1;
    crate::implementation_attempt::repair::Policy {
        schema: crate::implementation_attempt::repair::POLICY_SCHEMA.into(),
        failure_classes: vec!["test".into()],
        max_iterations,
        max_elapsed_ms,
        max_model_requests: steps,
        max_tool_invocations: steps * 2,
        step_tool_invocations: 1,
        check_executor: parsed.executor(),
        checks: parsed
            .checks
            .iter()
            .map(|entry| crate::implementation_attempt::repair::Check {
                name: entry.name.clone(),
                configuration: entry.configuration(),
            })
            .collect(),
    }
}

/// Nickel text of a standalone profile for `worker_profile` granting `grants`.
fn profile_text(
    worker_profile: &str,
    grants: &[crate::engine::effect::standalone::core::Grant],
    expiry: u64,
) -> String {
    let required = crate::implementation_attempt::repair::step::operator::requirements(worker_profile);
    let grants: Vec<String> = grants
        .iter()
        .map(|grant| {
            format!(
                "{{ node_id = {}, operation = \"{}\", resource = \"{}\", ability = \"{}\" }}",
                grant.node_id, grant.operation, grant.resource, grant.ability
            )
        })
        .collect();
    format!(
        "{{ schema = \"{}\", workflow_revision = \"{}\", machine_identity = \"{}\", policy_ref = \"{}\", \
         expires_at_unix_ms = {expiry}, grants = [{}], roots = {{}} }}\n",
        required.schema,
        required.workflow_revision,
        required.machine_identity,
        required.policy_ref,
        grants.join(", "),
    )
}

/// Write `profile` for a request of `worker_profile` at `path`.
pub(super) fn write_profile(path: &std::path::Path, worker_profile: &str, profile: Profile) {
    let observed = crate::operation::shell::Clock::observe_epoch_millis(&crate::operation::shell::SystemClock);
    let now = u64::try_from(observed).expect("clock");
    let worker = crate::implementation_attempt::repair::step::operator::worker_grant();
    let check = crate::implementation_attempt::repair::step::operator::check_grant();
    let other = "e".repeat(64);
    let (revision, grants, expiry) = match profile {
        Profile::Granting | Profile::NoCatalog | Profile::Unconfigured => {
            (worker_profile, vec![worker, check], now + 3_600_000)
        }
        Profile::WorkerOnly => (worker_profile, vec![worker], now + 3_600_000),
        Profile::OtherRevision => (other.as_str(), vec![worker, check], now + 3_600_000),
        Profile::Expired => (worker_profile, vec![worker, check], 1),
    };
    std::fs::write(path, profile_text(revision, &grants, expiry)).expect("profile");
}

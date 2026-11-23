//! Candidate-producing command worker over the existing bounded process route.
//!
//! The worker is an imperative shell. It captures the admitted workspace before
//! and after a clear-environment bounded process and derives the candidate
//! receipt from the actual file maps. When isolation is required, a configured
//! bubblewrap executable constructs a private root with only admitted workspace
//! paths, system read-only bases, a private network unless admitted, and no host
//! credentials. It does not provide publication authority.

use std::sync::atomic::AtomicBool;

use thiserror::Error;

use crate::candidate::CandidateDisposition;
use crate::candidate::CandidateReceipt;
use crate::candidate::CandidateReceiptError;
use crate::candidate_capture::WorkspaceBoundary;
use crate::process::core::RuntimeConfig;
use crate::store::Store;
use crate::store::StoreError;

pub mod omp;

pub mod broker;
mod compatibility;
mod completion;
mod execution;
mod result;
pub mod sandbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProcessFailure {
    pub phase: crate::agent_node::ProcessFailurePhase,
    pub termination: crate::agent_node::ProcessTermination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub work_id: String,
    pub attempt_id: String,
    pub fence: String,
    pub workspace_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub boundary: WorkspaceBoundary,
    pub require_isolation: bool,
    /// Reviewed bubblewrap executable used when isolation is required.
    pub sandbox_program: Option<String>,
    /// Attempt-local host broker grant; never shares the host network.
    pub broker: Option<broker::Mount>,
    pub timeout_secs: u64,
    pub output_limit_bytes: usize,
}

#[derive(Debug)]
pub enum Outcome {
    Produced {
        receipt: Box<CandidateReceipt>,
        changed_files: std::collections::BTreeMap<String, Vec<u8>>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    Blocked {
        blocker: String,
    },
    Failed {
        blocker: String,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    /// A typed no-change observation; generic publication remains failed.
    Unchanged {
        observation: Box<crate::candidate::UnchangedObservation>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    Cancelled {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    /// No usable candidate receipt: files are evidence only, never retry authority.
    Interrupted {
        failure: ProcessFailure,
        observed_files: Option<std::collections::BTreeMap<String, Vec<u8>>>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    Unknown {
        blocker: String,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
}

/// Run one worker with the default direct-child launcher and derive a candidate
/// receipt from the actual workspace.
pub fn run(request: &Request, config: &RuntimeConfig, cancellation: &AtomicBool) -> Result<Outcome, Error> {
    run_with_launcher(request, config, cancellation, &sandbox::launcher::Direct)
}

/// Run one worker through an explicit [`sandbox::launcher::SandboxLauncher`]
/// and derive a candidate receipt from the actual workspace.
pub fn run_with_launcher(
    request: &Request,
    config: &RuntimeConfig,
    cancellation: &AtomicBool,
    launcher: &dyn sandbox::launcher::SandboxLauncher,
) -> Result<Outcome, Error> {
    execution::run(request, config, cancellation, None, |process, config, cancellation| {
        launcher.execute(process, config, cancellation)
    })
}

/// Run one worker with the default direct-child launcher and durably publish a
/// produced receipt.
pub fn run_and_publish(
    request: &Request,
    config: &RuntimeConfig,
    cancellation: &AtomicBool,
    store: &Store,
) -> Result<Outcome, Error> {
    let compatibility = compatibility::observe(request);
    let outcome = run(request, config, cancellation)?;
    Ok(publish_outcome(request, store, outcome, compatibility))
}

/// Run one worker through an explicit [`sandbox::launcher::SandboxLauncher`]
/// and durably publish a produced receipt.
pub fn run_and_publish_with_launcher(
    request: &Request,
    config: &RuntimeConfig,
    cancellation: &AtomicBool,
    store: &Store,
    launcher: &dyn sandbox::launcher::SandboxLauncher,
) -> Result<Outcome, Error> {
    let compatibility = compatibility::observe(request);
    let outcome = run_with_launcher(request, config, cancellation, launcher)?;
    Ok(publish_outcome(request, store, outcome, compatibility))
}

pub(crate) fn publish_outcome(
    request: &Request,
    store: &Store,
    outcome: Outcome,
    compatibility: Option<Compatibility>,
) -> Outcome {
    result::publication::save(request, store, outcome, compatibility)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recovery {
    Published(Box<AttemptResult>),
    Incomplete { blocker: String },
    Missing,
}

/// Reconcile the durable result and receipt without redispatching work.
pub fn reconcile_attempt(store: &Store, attempt_id: &str) -> Result<Recovery, Error> {
    result::reconciliation::load(store, attempt_id)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AttemptResult {
    pub schema: String,
    pub disposition: CandidateDisposition,
    pub receipt: Option<CandidateReceipt>,
    pub published_artifacts: Vec<String>,
    pub stdout_blake3: String,
    pub stderr_blake3: String,
    pub blocker: Option<String>,
    #[serde(default = "result::absent_compatibility", skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<Compatibility>,
    #[serde(default = "result::absent_process_failure", skip_serializing_if = "Option::is_none")]
    pub process_failure: Option<ProcessFailure>,
    /// Best-effort file identities, not a candidate or proof of quiescence.
    #[serde(
        default = "result::empty_observed_artifacts",
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    pub observed_artifacts: std::collections::BTreeMap<String, String>,
}

pub const CANDIDATE_ATTEMPT_RESULT_SCHEMA: &str = "lattice.candidate-attempt-result.v1";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Compatibility {
    pub schema: String,
    pub worker_program_identity: String,
    pub environment_identity: String,
    pub workspace_provider_identity: String,
    pub result_schema: String,
}

pub const CANDIDATE_WORKER_COMPATIBILITY_SCHEMA: &str = "lattice.candidate-worker-compatibility.v1";
pub const CANDIDATE_WORKER_ENVIRONMENT: &str = "lattice.candidate-environment.clear.v1";

#[derive(Debug, Error)]
pub enum Error {
    #[error("workspace capture failed: {0}")]
    Capture(#[from] crate::candidate_capture::Error),
    #[error("candidate receipt derivation failed: {0}")]
    Receipt(#[from] CandidateReceiptError),
    #[error("workspace root is not valid UTF-8")]
    NonUtf8Root,
    #[error("candidate attempt result is invalid: {0}")]
    Result(String),
    #[error("candidate store operation failed: {0}")]
    Store(#[from] StoreError),
}

#[cfg(test)]
mod tests;

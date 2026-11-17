//! Imperative thin adapter. Uses the producer CLI/iroh boundary, never places a
//! tile or invokes a render component locally. The journal precedes Submit.
use std::io::Read as _;

use anyhow::Context;

const MAX_RECORDS: u64 = 512;
const MAX_BINDING_BYTES: u64 = 262_144;
const MAX_RECORD_BYTES: usize = 1024 * 1024;

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    clap::ValueEnum,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "kebab-case")]
pub enum Fault {
    #[default]
    None,
    AfterIntent,
    LostSubmitAck,
    AfterObservation,
    BeforePublication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Advance,
    Observe,
    Cancel,
    Supersede,
}

/// Issued from actual Executor context, never predicted by a workflow author.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostExecution {
    pub run_id: u64,
    pub authority_run_identity: String,
    pub node_id: u64,
    pub workflow_revision: String,
    pub invocation_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchPermit {
    pub binding: molten_batch_core::Id,
    pub owner: HostExecution,
    pub observer: HostExecution,
    pub method: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub binding: molten_batch_core::Id,
    pub request: crate::render::model::Request,
    pub sequence: u64,
    pub dispatch_possible: bool,
    pub acknowledged: bool,
    pub cancellation_requested: bool,
    pub superseded: bool,
    pub outcome: crate::render::model::Outcome,
    pub snapshot: Option<molten_batch_core::ProgressSnapshot>,
    pub accepted: Option<crate::render::model::AcceptedManifest>,
    pub measured_adapter_wall_millis: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub binding: molten_batch_core::Id,
    pub execution: String,
    pub attempt: String,
    pub budget_identity: String,
    pub outcome: crate::render::model::Outcome,
    pub recovery: String,
    pub accepted_manifest: Option<molten_batch_core::Id>,
    pub host_execution: Option<HostExecution>,
    pub observed_host_execution: Option<HostExecution>,
    pub measured_adapter_wall_millis: u64,
    pub remote_usage: Option<serde_json::Value>,
    pub remote_stop_observed: bool,
}

pub trait Batch {
    fn put(
        &mut self,
        kind: molten_batch_core::ContentKind,
        file: &std::path::Path,
    ) -> anyhow::Result<molten_batch_core::Id>;
    fn submit(&mut self, key: &str, file: &std::path::Path) -> anyhow::Result<molten_batch_core::Id>;
    fn snapshot(&mut self, run: molten_batch_core::Id) -> anyhow::Result<molten_batch_core::ProgressSnapshot>;
    fn get(&mut self, id: molten_batch_core::Id, file: &std::path::Path) -> anyhow::Result<()>;
    fn cancel(&mut self, run: molten_batch_core::Id) -> anyhow::Result<()>;
}

/// No shell interpolation or environment inheritance. Actual producer CLI owns
/// keys, authentication, transport, admission, worker leases and cancellation.
pub struct Producer<'a> {
    pub request: &'a crate::render::model::Request,
    pub permit: &'a DispatchPermit,
    pub process: crate::process::core::RuntimeConfig,
}

mod artifacts;
mod context;
mod dispatch;
mod execution;
mod journal;
mod producer;
mod publication;

fn now_millis() -> anyhow::Result<u64> {
    Ok(u64::try_from(crate::operation::shell::Clock::observe_epoch_millis(
        &crate::operation::shell::SystemClock,
    ))?)
}

pub fn read_bounded(path: &std::path::Path, bound_bytes: u64) -> anyhow::Result<Vec<u8>> {
    let bound = usize::try_from(bound_bytes)?;
    let read_limit_bytes = bound_bytes.checked_add(1).context("file read bound overflow")?;
    anyhow::ensure!(!std::fs::symlink_metadata(path)?.file_type().is_symlink(), "symlink input rejected");
    let file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    anyhow::ensure!(metadata.is_file() && metadata.len() <= bound_bytes, "file bound exceeded");
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len())?);
    file.take(read_limit_bytes).read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() <= bound, "file grew past bound");
    Ok(bytes)
}

pub fn load_request(
    file: &std::path::Path,
    expected: molten_batch_core::Id,
) -> anyhow::Result<crate::render::model::Request> {
    let request: crate::render::model::Request = serde_json::from_slice(&read_bounded(file, MAX_BINDING_BYTES)?)?;
    request.prepare()?;
    anyhow::ensure!(request.id()? == expected, "planned render binding changed");
    Ok(request)
}

fn check_current(request: &crate::render::model::Request) -> anyhow::Result<()> {
    let authority: crate::render::model::Authority =
        serde_json::from_slice(&read_bounded(std::path::Path::new(&request.authority_file), 4096)?)?;
    request.check_authority(&authority)
}

/// Private service-owned directory. Cap-relative I/O confines journal members;
/// callers must admit/control the root and its ancestry, not an untrusted workspace.
struct Journal {
    dir: cap_std::fs::Dir,
    root: std::path::PathBuf,
    _lock: std::fs::File,
    record: Record,
    is_restored: bool,
}

/// One bounded owner operation. Reopening after any possible Submit only observes
/// the precomputed exact run. No error branch resets dispatch_possible.
pub fn step(
    request: &crate::render::model::Request,
    batch: &mut dyn Batch,
    action: Action,
    fault: Fault,
) -> anyhow::Result<Receipt> {
    execution::step(request, batch, action, fault)
}

/// Read-only final acceptance check used by the ordinary operation classifier.
pub fn verify_committed(request: &crate::render::model::Request, receipt: &Receipt) -> anyhow::Result<()> {
    artifacts::verify_committed(request, receipt)
}

/// Bind only the actual context allocated by the in-process Executor.
pub(crate) fn bind_host_execution(
    request: &crate::render::model::Request,
    execution: &HostExecution,
    method: &str,
) -> anyhow::Result<(std::path::PathBuf, molten_batch_core::Id)> {
    context::bind_host_execution(request, execution, method)
}

pub fn load_permit(
    request: &crate::render::model::Request,
    file: &std::path::Path,
    expected: molten_batch_core::Id,
) -> anyhow::Result<DispatchPermit> {
    let required = std::path::PathBuf::from(&request.journal_root)
        .join(request.id()?.to_string())
        .join(format!("permit-{expected}.json"));
    anyhow::ensure!(file == required, "dispatch context is not in the owned journal");
    let permit: DispatchPermit = serde_json::from_slice(&read_bounded(file, 16 * 1024)?)?;
    anyhow::ensure!(
        molten_orchestrator_codec::identity("lattice.render-dispatch.v1", &permit)? == expected,
        "dispatch context identity mismatch"
    );
    context::validate_permit(request, &permit)?;
    Ok(permit)
}

pub fn step_bound(
    request: &crate::render::model::Request,
    batch: &mut dyn Batch,
    permit: &DispatchPermit,
    fault: Fault,
) -> anyhow::Result<Receipt> {
    context::validate_permit(request, permit)?;
    let action = match permit.method.as_str() {
        "advance" => Action::Advance,
        "observe" => Action::Observe,
        "cancel" => Action::Cancel,
        "supersede" => Action::Supersede,
        _ => anyhow::bail!("unsupported dispatch method"),
    };
    let mut receipt = step(request, batch, action, fault)?;
    receipt.observed_host_execution = Some(permit.observer.clone());
    Ok(receipt)
}

pub(super) fn observe(
    request: &crate::render::model::Request,
    batch: &mut dyn Batch,
    permit: &DispatchPermit,
    fault: Fault,
) -> anyhow::Result<Receipt> {
    let started = crate::operation::shell::SystemClock.observe_monotonic_instant();
    let mut receipt = step_bound(request, batch, permit, fault)?;
    // Pending is not completion; reconnect to this same run and journal.
    while matches!(
        receipt.outcome,
        crate::render::model::Outcome::Pending | crate::render::model::Outcome::CancellationRequested
    ) && started.elapsed().as_secs() < 590
    {
        if now_millis()? >= request.budget.deadline_epoch_millis {
            return step_bound(request, batch, permit, Fault::None);
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
        receipt = step_bound(request, batch, permit, Fault::None)?;
    }
    Ok(receipt)
}

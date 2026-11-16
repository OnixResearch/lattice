//! Imperative orchestration shell over the pure decision core.

use std::fmt;

use serde::Serialize;

use crate::headless;
use crate::orchestration::core;
use crate::orchestration::core::ReplayReport;
use crate::orchestration::core::RevisionProposal;
use crate::store::StoreError;
use crate::workflow_validation::WorkflowDiagnostic;

mod durability;
mod lifecycle;
#[cfg(test)]
mod tests;

pub use durability::persistence::load_instance;
pub use durability::recovery::recover_pending;
pub use durability::recovery::replay;
#[cfg(test)]
use durability::settlement::FinishRequest;
#[cfg(test)]
use durability::settlement::accounting_charge;
#[cfg(test)]
use durability::settlement::finish_action;
#[cfg(test)]
use durability::telemetry::provider_metadata;
#[cfg(test)]
use durability::telemetry::provider_reports;
#[cfg(test)]
use durability::telemetry::usage_cost;
#[cfg(test)]
use durability::telemetry::usage_tokens;
pub use lifecycle::admission::validate_definition_in_store;
pub use lifecycle::control::abandon_pending;
pub use lifecycle::launch::start;
pub use lifecycle::launch::start_from_trigger_with_execution;
pub use lifecycle::launch::start_with_execution;
pub use lifecycle::relaunch::retry;
pub use lifecycle::relaunch::retry_with_execution;
pub use lifecycle::review::approve_with_execution;

const ROUND_CHARGE: u64 = 1;

pub struct ProductExecution<'a> {
    pub agent_runtime: crate::agent_node::RuntimeOptions,
    pub authority_provider: &'a dyn crate::engine::effect::execution::RunContextProvider,
}

pub struct ProductApproval<'a> {
    pub instance_id: String,
    pub proposal: RevisionProposal,
    pub actor: String,
    pub execution: ProductExecution<'a>,
}

pub struct ProductTriggerStart<'a> {
    pub definition_id: String,
    pub trigger_id: String,
    pub trigger_kind: crate::orchestration::core::TriggerKind,
    pub idempotency_key: String,
    pub execution: ProductExecution<'a>,
}
const AGENT_INVOCATION_CHARGE: u64 = 1;
const UNOBSERVED_ROUND_FAILURE_SUMMARY: &str =
    "round execution failed before exact terminal run evidence was persisted";

#[derive(Debug, Clone, Serialize)]
pub struct DefinitionValidationReport {
    pub definition_revision: Option<String>,
    pub workflow_revision: Option<core::WorkflowRevisionRef>,
    pub orchestration_diagnostics: Vec<crate::orchestration::core::Diagnostic>,
    pub workflow_diagnostics: Vec<WorkflowDiagnostic>,
}

impl DefinitionValidationReport {
    pub fn valid(&self) -> bool {
        self.orchestration_diagnostics.is_empty()
            && !crate::workflow_validation::has_fatal_diagnostics(&self.workflow_diagnostics)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RecoveryOutcome {
    NothingPending,
    PendingUnobserved { run_id: crate::engine::RunId },
    Reconciled(Box<crate::orchestration::core::Instance>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbandonPendingRequest<'a> {
    pub instance_id: &'a str,
    pub actor: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryRequest<'a> {
    pub instance_id: &'a str,
    pub actor: &'a str,
}

#[derive(Debug)]
pub enum Error {
    Store(StoreError),
    Core(crate::orchestration::core::Diagnostic),
    DefinitionInvalid(Box<DefinitionValidationReport>),
    WorkflowNotFound(String),
    Headless(headless::HeadlessRunError),
    MissingInstance(String),
    MissingProposal(String),
    InvalidProposal(String),
    NoPendingDispatch(String),
    ObservedRunRequiresRecovery { run_id: crate::engine::RunId },
    PendingDispatchCannotRetry { run_id: crate::engine::RunId },
    RunIdentityMismatch { run_id: crate::engine::RunId },
    CancellationObservationUnknown { run_id: crate::engine::RunId },
    ReplayInvalid(Box<ReplayReport>),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "{error}"),
            Self::Core(error) => write!(formatter, "orchestration rejected: {}", error.message),
            Self::DefinitionInvalid(report) => write!(
                formatter,
                "orchestration definition is invalid: {} orchestration diagnostics, {} workflow diagnostics",
                report.orchestration_diagnostics.len(),
                report.workflow_diagnostics.len()
            ),
            Self::WorkflowNotFound(name) => write!(formatter, "workflow not found: {name}"),
            Self::Headless(error) => write!(formatter, "orchestration round failed: {error}"),
            Self::MissingInstance(id) => write!(formatter, "orchestration instance not found: {id}"),
            Self::MissingProposal(id) => write!(formatter, "orchestration proposal not found: {id}"),
            Self::InvalidProposal(message) => write!(formatter, "orchestration proposal is invalid: {message}"),
            Self::NoPendingDispatch(id) => write!(formatter, "orchestration instance has no pending dispatch: {id}"),
            Self::ObservedRunRequiresRecovery { run_id } => {
                write!(formatter, "run {run_id} is already persisted; use orchestration recover before another action")
            }
            Self::PendingDispatchCannotRetry { run_id } => write!(
                formatter,
                "run {run_id} remains pending and locally unobserved; use orchestration recover or authorized abandonment, not retry"
            ),
            Self::RunIdentityMismatch { run_id } => {
                write!(formatter, "run {run_id} is not owned by the pending orchestration dispatch")
            }
            Self::CancellationObservationUnknown { run_id } => {
                write!(
                    formatter,
                    "owned cancellation observation for run {run_id} is unknown; reconcile the pending receipt before settlement"
                )
            }
            Self::ReplayInvalid(report) => {
                write!(formatter, "orchestration replay is invalid: {} diagnostics", report.diagnostics.len())
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<StoreError> for Error {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<crate::orchestration::core::Diagnostic> for Error {
    fn from(error: crate::orchestration::core::Diagnostic) -> Self {
        Self::Core(error)
    }
}

impl From<headless::HeadlessRunError> for Error {
    fn from(error: headless::HeadlessRunError) -> Self {
        Self::Headless(error)
    }
}
// r[impl lattice.agent_orchestration.controls]
pub fn pause(
    store: &crate::store::Store,
    instance_id: &str,
    actor: &str,
) -> Result<crate::orchestration::core::Instance, Error> {
    let instance = load_instance(store, instance_id)?;
    durability::persistence::apply_and_commit(store, &instance, crate::orchestration::core::Action::PauseRequested {
        actor: actor.to_string(),
    })
}
// r[impl lattice.agent_orchestration.controls]
pub fn resume(
    store: &crate::store::Store,
    instance_id: &str,
    actor: &str,
) -> Result<crate::orchestration::core::Instance, Error> {
    let instance = load_instance(store, instance_id)?;
    durability::persistence::apply_and_commit(store, &instance, crate::orchestration::core::Action::ResumeRequested {
        actor: actor.to_string(),
    })
}
// r[impl lattice.agent_orchestration.controls]
pub fn cancel(
    store: &crate::store::Store,
    instance_id: &str,
    actor: &str,
) -> Result<crate::orchestration::core::Instance, Error> {
    let instance = load_instance(store, instance_id)?;
    durability::persistence::apply_and_commit(store, &instance, crate::orchestration::core::Action::CancelRequested {
        actor: actor.to_string(),
    })
}
// r[impl lattice.agent_orchestration.triggers]
pub async fn start_from_trigger(
    store: &crate::store::Store,
    definition_id: impl Into<String>,
    trigger_id: impl AsRef<str>,
    trigger_kind: crate::orchestration::core::TriggerKind,
    idempotency_key: &str,
    agent_runtime: crate::agent_node::RuntimeOptions,
) -> Result<crate::orchestration::core::Instance, Error> {
    let authority_provider = crate::engine::effect::execution::UnavailableRunContextSource;
    start_from_trigger_with_execution(store, ProductTriggerStart {
        definition_id: definition_id.into(),
        trigger_id: trigger_id.as_ref().to_owned(),
        trigger_kind,
        idempotency_key: idempotency_key.to_owned(),
        execution: ProductExecution {
            agent_runtime,
            authority_provider: &authority_provider,
        },
    })
    .await
}
// r[impl lattice.agent_orchestration.authority]
pub async fn approve(
    store: &crate::store::Store,
    instance_id: &str,
    proposal: crate::orchestration::core::RevisionProposal,
    actor: &str,
    agent_runtime: crate::agent_node::RuntimeOptions,
) -> Result<crate::orchestration::core::Instance, Error> {
    let authority_provider = crate::engine::effect::execution::UnavailableRunContextSource;
    let execution = ProductExecution {
        agent_runtime,
        authority_provider: &authority_provider,
    };
    let instance = lifecycle::review::submit_if_absent(store, load_instance(store, instance_id)?, &proposal)?;
    lifecycle::review::decide_approved_proposal(store, instance, proposal, actor, execution).await
}
// r[impl lattice.agent_orchestration.authority]
pub fn reject(
    store: &crate::store::Store,
    instance_id: &str,
    proposal_id: &str,
    actor: &str,
) -> Result<crate::orchestration::core::Instance, Error> {
    let instance = load_instance(store, instance_id)?;
    let proposal_id =
        crate::orchestration::ProposalId::try_from_hex(proposal_id).map_err(durability::persistence::identity_error)?;
    durability::persistence::apply_and_commit(store, &instance, crate::orchestration::core::Action::RejectProposal {
        actor: actor.to_string(),
        proposal_id,
    })
}

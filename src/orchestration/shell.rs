//! Imperative orchestration shell over the pure decision core.

use std::collections::BTreeMap;
use std::fmt;

use serde::Serialize;

use crate::agent_node::AgentRuntimeConfig;
use crate::engine::NodeConfig;
use crate::engine::RunId;
use crate::engine::Workflow;
use crate::engine::runner;
use crate::headless;
use crate::headless::HeadlessRunStatus;
use crate::orchestration::core;
use crate::orchestration::core::ApprovalDecision;
use crate::orchestration::core::BudgetCharge;
use crate::orchestration::core::BudgetChargePlan;
use crate::orchestration::core::DispatchIntent;
use crate::orchestration::core::DispatchStatus;
use crate::orchestration::core::OrchestrationAction;
use crate::orchestration::core::OrchestrationDefinition;
use crate::orchestration::core::OrchestrationDiagnostic;
use crate::orchestration::core::OrchestrationDiagnosticCode;
use crate::orchestration::core::OrchestrationFact;
use crate::orchestration::core::OrchestrationInstance;
use crate::orchestration::core::OrchestrationStatus;
use crate::orchestration::core::OrchestrationTriggerAdmission;
use crate::orchestration::core::OrchestrationTriggerContext;
use crate::orchestration::core::OrchestrationTriggerKind;
use crate::orchestration::core::ReplayReport;
use crate::orchestration::core::RevisionProposal;
use crate::store::RunRecord;
use crate::store::Store;
use crate::store::StoreError;
use crate::store::TriggerAdmissionWrite;
use crate::workflow_validation;
use crate::workflow_validation::WorkflowDiagnostic;

const ROUND_CHARGE: u64 = 1;
const AGENT_INVOCATION_CHARGE: u64 = 1;

#[derive(Debug, Clone, Serialize)]
pub struct DefinitionValidationReport {
    pub definition_revision: Option<String>,
    pub workflow_revision: Option<core::WorkflowRevisionRef>,
    pub orchestration_diagnostics: Vec<OrchestrationDiagnostic>,
    pub workflow_diagnostics: Vec<WorkflowDiagnostic>,
}

impl DefinitionValidationReport {
    pub fn valid(&self) -> bool {
        self.orchestration_diagnostics.is_empty()
            && !workflow_validation::has_fatal_diagnostics(&self.workflow_diagnostics)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RecoveryOutcome {
    NothingPending,
    PendingUnobserved { run_id: RunId },
    Reconciled(Box<OrchestrationInstance>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbandonPendingRequest<'a> {
    pub instance_id: &'a str,
    pub actor: &'a str,
}

#[derive(Debug)]
pub enum OrchestrationShellError {
    Store(StoreError),
    Core(OrchestrationDiagnostic),
    DefinitionInvalid(Box<DefinitionValidationReport>),
    WorkflowNotFound(String),
    Headless(headless::HeadlessRunError),
    MissingInstance(String),
    MissingProposal(String),
    InvalidProposal(String),
    NoPendingDispatch(String),
    ObservedRunRequiresRecovery { run_id: RunId },
    ReplayInvalid(Box<ReplayReport>),
}

impl fmt::Display for OrchestrationShellError {
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
                write!(formatter, "run {run_id} is already persisted; use orchestration recover instead of abandon")
            }
            Self::ReplayInvalid(report) => {
                write!(formatter, "orchestration replay is invalid: {} diagnostics", report.diagnostics.len())
            }
        }
    }
}

impl std::error::Error for OrchestrationShellError {}

impl From<StoreError> for OrchestrationShellError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<OrchestrationDiagnostic> for OrchestrationShellError {
    fn from(error: OrchestrationDiagnostic) -> Self {
        Self::Core(error)
    }
}

impl From<headless::HeadlessRunError> for OrchestrationShellError {
    fn from(error: headless::HeadlessRunError) -> Self {
        Self::Headless(error)
    }
}

// r[impl lattice.agent_orchestration.model]
// r[impl lattice.agent_orchestration.rounds]
pub fn validate_definition_in_store(
    store: &Store,
    definition: &OrchestrationDefinition,
) -> Result<DefinitionValidationReport, OrchestrationShellError> {
    let workflow = store.load_workflow(&definition.workflow.workflow_name)?;
    Ok(validation_report(definition, workflow.as_ref()))
}

fn validation_report(definition: &OrchestrationDefinition, workflow: Option<&Workflow>) -> DefinitionValidationReport {
    let workflow_revision = workflow.and_then(|workflow| core::workflow_revision(workflow).ok());
    let orchestration_diagnostics = core::validate_definition(definition, workflow_revision.as_ref());
    let workflow_diagnostics = workflow.map_or_else(Vec::new, workflow_validation::validate_workflow);
    DefinitionValidationReport {
        definition_revision: core::definition_revision(definition).ok(),
        workflow_revision,
        orchestration_diagnostics,
        workflow_diagnostics,
    }
}

// r[impl lattice.agent_orchestration.cli]
// r[impl lattice.agent_orchestration.rounds]
pub async fn start(
    store: &Store,
    definition: OrchestrationDefinition,
    actor: &str,
    agent_runtime: AgentRuntimeConfig,
) -> Result<OrchestrationInstance, OrchestrationShellError> {
    core::authorize_definition_actor(&definition, actor)?;
    let report = validate_definition_in_store(store, &definition)?;
    if !report.valid() {
        return Err(OrchestrationShellError::DefinitionInvalid(Box::new(report)));
    }
    store.save_orchestration_definition(&definition)?;
    let sequence = store.next_orchestration_sequence()?;
    let instance = core::new_instance(definition, sequence, None)?;
    store.create_orchestration(&instance, &[])?;
    execute_ready_round(store, instance, agent_runtime).await
}

// r[impl lattice.agent_orchestration.triggers]
pub async fn start_from_trigger(
    store: &Store,
    definition_id: &str,
    trigger_id: &str,
    trigger_kind: OrchestrationTriggerKind,
    idempotency_key: &str,
    agent_runtime: AgentRuntimeConfig,
) -> Result<OrchestrationInstance, OrchestrationShellError> {
    let definition = store
        .load_orchestration_definition(definition_id)?
        .ok_or_else(|| OrchestrationShellError::WorkflowNotFound(definition_id.to_string()))?;
    let report = validate_definition_in_store(store, &definition)?;
    if !report.valid() {
        return Err(OrchestrationShellError::DefinitionInvalid(Box::new(report)));
    }
    let definition_revision = core::definition_revision(&definition)?;
    let accepted = store.list_orchestration_trigger_admissions()?.into_iter().collect::<BTreeMap<_, _>>();
    let sequence = store.next_orchestration_sequence()?;
    let context = OrchestrationTriggerContext { accepted };
    let admission = core::admit_trigger_start(core::TriggerStartRequest {
        definition: &definition,
        definition_revision: &definition_revision,
        trigger_id,
        trigger_kind,
        idempotency_key,
        next_sequence: sequence,
        context: &context,
    })?;
    match admission {
        OrchestrationTriggerAdmission::Duplicate { instance_id } => load_instance(store, &instance_id),
        OrchestrationTriggerAdmission::Accepted(plan) => {
            let instance = core::new_instance(definition, sequence, Some(plan.scoped_key.clone()))?;
            debug_assert_eq!(instance.id, plan.instance_id);
            match store.create_triggered_orchestration(&plan.scoped_key, &instance)? {
                TriggerAdmissionWrite::Inserted => execute_ready_round(store, instance, agent_runtime).await,
                TriggerAdmissionWrite::Duplicate { instance_id } => load_instance(store, &instance_id),
            }
        }
    }
}

// r[impl lattice.agent_orchestration.rounds]
async fn execute_ready_round(
    store: &Store,
    instance: OrchestrationInstance,
    agent_runtime: AgentRuntimeConfig,
) -> Result<OrchestrationInstance, OrchestrationShellError> {
    let workflow = load_validated_workflow(store, &instance.workflow_revision)?;
    let run_id = store.next_run_id()?;
    let intent = DispatchIntent {
        id: core::dispatch_identity(&instance, run_id)?,
        run_id,
        workflow_revision: instance.workflow_revision.clone(),
        status: DispatchStatus::Pending,
    };
    let plan = planned_charge(&instance.definition, &workflow)?;
    let start_action = OrchestrationAction::StartRound {
        intent: intent.clone(),
        planned_charge: plan,
    };
    let started = match apply_and_commit(store, &instance, start_action) {
        Ok(started) => started,
        Err(OrchestrationShellError::Core(error))
            if matches!(
                error.code,
                OrchestrationDiagnosticCode::BudgetExceeded | OrchestrationDiagnosticCode::MissingBudgetBound
            ) =>
        {
            return apply_and_commit(store, &instance, OrchestrationAction::BudgetExhausted {
                safe_reason: error.message,
            });
        }
        Err(error) => return Err(error),
    };
    let acknowledged = apply_and_commit(store, &started, OrchestrationAction::DispatchAcknowledged {
        dispatch_id: intent.id.clone(),
    })?;

    let result =
        headless::run_saved_workflow_with_agent_runtime_config(store, &workflow.meta.name, run_id, agent_runtime).await;
    let current = load_instance(store, &acknowledged.id)?;
    let (succeeded, summary) = match result {
        Ok(summary) => (summary.status == HeadlessRunStatus::Succeeded, safe_run_summary(&summary)),
        Err(error) => (false, format!("round execution failed: {error}")),
    };
    let observed_usage = observed_usage(&current, &intent.id).unwrap_or_default();
    apply_and_commit(store, &current, OrchestrationAction::RunFinished {
        dispatch_id: intent.id,
        run_id,
        succeeded,
        observed_usage,
        safe_summary: summary,
    })
}

fn load_validated_workflow(
    store: &Store,
    reference: &core::WorkflowRevisionRef,
) -> Result<Workflow, OrchestrationShellError> {
    let workflow = store
        .load_workflow(&reference.workflow_name)?
        .ok_or_else(|| OrchestrationShellError::WorkflowNotFound(reference.workflow_name.clone()))?;
    let actual = core::workflow_revision(&workflow)?;
    if actual != *reference {
        return Err(OrchestrationShellError::Core(OrchestrationDiagnostic {
            code: OrchestrationDiagnosticCode::WorkflowRevisionMismatch,
            message: "stored workflow changed after orchestration revision admission".to_string(),
        }));
    }
    let diagnostics = workflow_validation::validate_workflow(&workflow);
    if workflow_validation::has_fatal_diagnostics(&diagnostics) {
        return Err(OrchestrationShellError::DefinitionInvalid(Box::new(DefinitionValidationReport {
            definition_revision: None,
            workflow_revision: Some(actual),
            orchestration_diagnostics: Vec::new(),
            workflow_diagnostics: diagnostics,
        })));
    }
    let build = runner::build_runner_plan(&workflow);
    let Some(plan) = build.plan else {
        return Err(OrchestrationShellError::InvalidProposal(
            "successor workflow cannot produce a runner plan".to_string(),
        ));
    };
    if matches!(runner::admit_runner_plan(&plan), runner::RunnerAdmission::Rejected { .. }) {
        return Err(OrchestrationShellError::InvalidProposal(
            "successor workflow is not admitted by the Seaglass-backed runner".to_string(),
        ));
    }
    Ok(workflow)
}

fn planned_charge(
    definition: &OrchestrationDefinition,
    workflow: &Workflow,
) -> Result<BudgetChargePlan, OrchestrationShellError> {
    let agent_count = workflow.configs.values().filter(|config| matches!(config, NodeConfig::Agent(_))).count();
    let agent_invocations = u64::try_from(agent_count).map_err(|_| {
        OrchestrationShellError::Core(OrchestrationDiagnostic {
            code: OrchestrationDiagnosticCode::BudgetOverflow,
            message: "workflow agent invocation count does not fit the budget model".to_string(),
        })
    })?;
    let configured = definition.budgets.per_round_upper_bound;
    Ok(BudgetChargePlan {
        rounds: Some(ROUND_CHARGE),
        agent_invocations: Some(agent_invocations.saturating_mul(AGENT_INVOCATION_CHARGE)),
        tokens: Some(configured.tokens),
        monetary_microunits: Some(configured.monetary_microunits),
        duration_millis: Some(configured.duration_millis),
    })
}

fn observed_usage(instance: &OrchestrationInstance, dispatch_id: &str) -> Option<BudgetCharge> {
    instance.budgets.reservations.get(dispatch_id).map(|reservation| reservation.charge)
}

fn safe_run_summary(summary: &headless::HeadlessRunSummary) -> String {
    format!(
        "workflow {} run {} status {:?}: {} nodes, {} failures, {} cache hits",
        summary.workflow_name, summary.run_id, summary.status, summary.node_count, summary.failures, summary.cache_hits
    )
}

fn apply_and_commit(
    store: &Store,
    instance: &OrchestrationInstance,
    action: OrchestrationAction,
) -> Result<OrchestrationInstance, OrchestrationShellError> {
    let fact = OrchestrationFact {
        id: core::fact_identity(&instance.id, instance.version, &action)?,
        action,
    };
    let decision = core::transition(instance, fact)?;
    if decision.duplicate {
        return Ok(decision.instance);
    }
    let event = decision.event.as_ref().ok_or_else(|| {
        OrchestrationShellError::InvalidProposal("applied orchestration transition emitted no event".to_string())
    })?;
    store.commit_orchestration_transition(instance.version, &decision.instance, event)?;
    Ok(decision.instance)
}

// r[impl lattice.agent_orchestration.controls]
pub fn pause(store: &Store, instance_id: &str, actor: &str) -> Result<OrchestrationInstance, OrchestrationShellError> {
    let instance = load_instance(store, instance_id)?;
    apply_and_commit(store, &instance, OrchestrationAction::PauseRequested {
        actor: actor.to_string(),
    })
}

// r[impl lattice.agent_orchestration.controls]
pub fn resume(store: &Store, instance_id: &str, actor: &str) -> Result<OrchestrationInstance, OrchestrationShellError> {
    let instance = load_instance(store, instance_id)?;
    apply_and_commit(store, &instance, OrchestrationAction::ResumeRequested {
        actor: actor.to_string(),
    })
}

// r[impl lattice.agent_orchestration.controls]
pub fn cancel(store: &Store, instance_id: &str, actor: &str) -> Result<OrchestrationInstance, OrchestrationShellError> {
    let instance = load_instance(store, instance_id)?;
    apply_and_commit(store, &instance, OrchestrationAction::CancelRequested {
        actor: actor.to_string(),
    })
}

// r[impl lattice.agent_orchestration.authority]
pub async fn approve(
    store: &Store,
    instance_id: &str,
    proposal: RevisionProposal,
    actor: &str,
    agent_runtime: AgentRuntimeConfig,
) -> Result<OrchestrationInstance, OrchestrationShellError> {
    let mut instance = load_instance(store, instance_id)?;
    match &instance.pending_proposal {
        None => {
            instance = apply_and_commit(store, &instance, OrchestrationAction::SubmitProposal {
                proposal: proposal.clone(),
            })?;
        }
        Some(pending) if pending.id == proposal.id => {}
        Some(_) => {
            return Err(OrchestrationShellError::InvalidProposal(
                "a different proposal is already pending".to_string(),
            ));
        }
    }
    let successor_validated = load_validated_workflow(store, &proposal.successor).is_ok();
    let approval: ApprovalDecision = core::make_approval(&instance, &proposal, actor, true)?;
    let approved = apply_and_commit(store, &instance, OrchestrationAction::DecideProposal {
        approval,
        successor_validated,
    })?;
    if approved.status == OrchestrationStatus::Ready {
        execute_ready_round(store, approved, agent_runtime).await
    } else {
        Ok(approved)
    }
}

// r[impl lattice.agent_orchestration.authority]
pub fn reject(
    store: &Store,
    instance_id: &str,
    proposal_id: &str,
    actor: &str,
) -> Result<OrchestrationInstance, OrchestrationShellError> {
    let instance = load_instance(store, instance_id)?;
    apply_and_commit(store, &instance, OrchestrationAction::RejectProposal {
        actor: actor.to_string(),
        proposal_id: proposal_id.to_string(),
    })
}

// r[impl lattice.agent_orchestration.events]
pub fn replay(store: &Store, instance_id: &str) -> Result<ReplayReport, OrchestrationShellError> {
    let current = load_instance(store, instance_id)?;
    let initial =
        core::new_instance(current.definition.clone(), current.sequence, current.trigger_idempotency_key.clone())?;
    let events = store.load_orchestration_events(instance_id)?;
    let report = core::replay_events(&initial, &events, current.event_head.as_deref());
    if !report.valid || report.instance != current {
        return Err(OrchestrationShellError::ReplayInvalid(Box::new(report)));
    }
    Ok(report)
}

// r[impl lattice.agent_orchestration.durability]
pub fn recover_pending(store: &Store, instance_id: &str) -> Result<RecoveryOutcome, OrchestrationShellError> {
    let instance = load_instance(store, instance_id)?;
    let Some(intent) = instance.pending_dispatch.clone() else {
        return Ok(RecoveryOutcome::NothingPending);
    };
    let Some(record) = store.load_run(intent.run_id)? else {
        return Ok(RecoveryOutcome::PendingUnobserved { run_id: intent.run_id });
    };
    let reconciled = reconcile_run_record(store, &instance, &intent, &record)?;
    Ok(RecoveryOutcome::Reconciled(Box::new(reconciled)))
}

// r[impl lattice.agent_orchestration.unobserved_abandon]
pub fn abandon_pending(
    store: &Store,
    request: AbandonPendingRequest<'_>,
) -> Result<OrchestrationInstance, OrchestrationShellError> {
    let instance = load_instance(store, request.instance_id)?;
    let intent = instance
        .pending_dispatch
        .clone()
        .ok_or_else(|| OrchestrationShellError::NoPendingDispatch(request.instance_id.to_string()))?;
    if store.load_run(intent.run_id)?.is_some() {
        return Err(OrchestrationShellError::ObservedRunRequiresRecovery { run_id: intent.run_id });
    }
    apply_and_commit(store, &instance, OrchestrationAction::AbandonPendingDispatch {
        actor: request.actor.to_string(),
        dispatch_id: intent.id,
        run_id: intent.run_id,
    })
}

fn reconcile_run_record(
    store: &Store,
    instance: &OrchestrationInstance,
    intent: &DispatchIntent,
    record: &RunRecord,
) -> Result<OrchestrationInstance, OrchestrationShellError> {
    let observed_usage = observed_usage(instance, &intent.id).unwrap_or_default();
    apply_and_commit(store, instance, OrchestrationAction::RunFinished {
        dispatch_id: intent.id.clone(),
        run_id: intent.run_id,
        succeeded: record.failures == 0,
        observed_usage,
        safe_summary: format!(
            "recovered workflow {} run {} with {} failures",
            record.workflow_name, record.run_id, record.failures
        ),
    })
}

pub fn load_instance(store: &Store, instance_id: &str) -> Result<OrchestrationInstance, OrchestrationShellError> {
    store
        .load_orchestration(instance_id)?
        .ok_or_else(|| OrchestrationShellError::MissingInstance(instance_id.to_string()))
}

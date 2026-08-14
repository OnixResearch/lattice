const DEFAULT_REQUESTER_ID: &str = "lattice-runtime";
const TERMINAL_APPROVAL_FAILURE_COUNT: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Store(String),
    Admission(String),
    Choregraph(String),
    WorkflowMissing,
    WorkflowChanged,
    GateMissing,
    CheckpointInvalid,
    RunMissing,
    AuthorityUnavailable,
    ActiveRequestChanged,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(message) => write!(formatter, "approval store failed: {message}"),
            Self::Admission(message) => write!(formatter, "approval admission failed: {message}"),
            Self::Choregraph(message) => write!(formatter, "approval graph transition failed: {message}"),
            Self::WorkflowMissing => formatter.write_str("approval workflow is missing"),
            Self::WorkflowChanged => formatter.write_str("approval workflow revision changed"),
            Self::GateMissing => formatter.write_str("approval gate configuration is missing"),
            Self::CheckpointInvalid => formatter.write_str("approval checkpoint is invalid"),
            Self::RunMissing => formatter.write_str("approval run history is missing"),
            Self::AuthorityUnavailable => formatter.write_str("approval resume authority is unavailable"),
            Self::ActiveRequestChanged => formatter.write_str("active workflow approval request changed"),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::store::StoreError> for Error {
    fn from(error: crate::store::StoreError) -> Self {
        Self::Store(error.to_string())
    }
}

impl From<super::core::AdmissionError> for Error {
    fn from(error: super::core::AdmissionError) -> Self {
        Self::Admission(error.to_string())
    }
}

impl From<super::choregraph::AdapterError> for Error {
    fn from(error: super::choregraph::AdapterError) -> Self {
        Self::Choregraph(error.to_string())
    }
}

pub struct SuspensionCommitInput<'a> {
    pub store: &'a crate::store::Store,
    pub workflow: &'a crate::engine::workflow::Workflow,
    pub run_id: crate::engine::executor::RunId,
    pub owner: crate::store::RunOwner,
    pub started_epoch_millis: i64,
    pub observed_epoch_millis: i64,
    pub results: &'a [crate::engine::executor::StepResult],
    pub suspension: crate::engine::executor::ApprovalSuspensionPoint,
}

// r[impl lattice.workflow_approval.suspension]
// r[impl lattice.workflow_approval.suspension.persisted]
// r[impl lattice.workflow_approval.suspension.uncertain]
pub fn persist_suspension(
    input: SuspensionCommitInput<'_>,
) -> Result<(crate::headless::HeadlessRunSummary, super::ApprovalRequest), Error> {
    let completed_nodes = checkpoint_from_results(input.results)?;
    let graph_events = super::choregraph::events_before_suspension(&completed_nodes);
    let requester =
        super::ActorId::try_from_text(DEFAULT_REQUESTER_ID).map_err(|error| Error::Admission(error.to_string()))?;
    let request = super::core::build_request(super::core::RequestBuildInput {
        run_id: input.run_id,
        workflow: input.workflow,
        node_id: input.suspension.node_id,
        requester,
        observed_epoch_millis: input.observed_epoch_millis,
        gate_inputs: &input.suspension.inputs,
        completed_nodes,
        graph_events,
    })?;
    let mut request = request;
    super::choregraph::apply_suspend(input.workflow, &mut request)?;
    let request = super::core::record_suspension(&request)?;

    let explicit_failures = input
        .results
        .iter()
        .filter(|step| matches!(step.status, crate::engine::executor::NodeStatus::Failed { .. }))
        .count();
    let cache_hits = input
        .results
        .iter()
        .filter(|step| matches!(step.status, crate::engine::executor::NodeStatus::Completed { cached: true, .. }))
        .count();
    let completed = input
        .results
        .iter()
        .filter(|step| matches!(step.status, crate::engine::executor::NodeStatus::Completed { .. }))
        .count();
    let summary = crate::headless::HeadlessRunSummary {
        run_id: input.run_id,
        workflow_name: input.workflow.meta.name.clone(),
        status: crate::headless::HeadlessRunStatus::Suspended,
        node_count: input.workflow.graph.node_count(),
        steps_run: input.results.len(),
        completed,
        failures: explicit_failures,
        cache_hits,
    };
    let run = crate::store::RunRecord {
        run_id: input.run_id,
        owner: input.owner,
        workflow_name: input.workflow.meta.name.clone(),
        started_epoch_ms: input.started_epoch_millis,
        finished_epoch_ms: None,
        node_count: summary.node_count,
        cache_hits,
        failures: explicit_failures,
    };
    let details = crate::run::history::detail_record_from_results(input.run_id, input.workflow, input.results);
    let replay_events = crate::replay::replay_events_from_results(
        input.run_id,
        input.workflow,
        input.results,
        input.started_epoch_millis,
        None,
    );
    input.store.create_workflow_approval_suspension(&request, &run, &details, &replay_events)?;
    debug_assert_eq!(request.status, super::ApprovalRequestStatus::Suspended);
    debug_assert!(run.finished_epoch_ms.is_none());
    Ok((summary, request))
}

// r[impl lattice.workflow_approval.decision]
// r[impl lattice.workflow_approval.authority]
pub fn decide(store: &crate::store::Store, command: &super::DecisionCommand) -> Result<super::DecisionOutcome, Error> {
    let current = store.load_workflow_approval(&command.request_id)?.ok_or(Error::RunMissing)?;
    require_active_request(store, &current)?;
    let workflow = load_bound_revision(store, &current)?;
    let config = approval_config(&workflow, current.node_id)?;
    let plan = super::core::plan_decision(&current, config, command)?;
    if matches!(plan.outcome, super::DecisionOutcome::Idempotent { .. }) {
        return Ok(plan.outcome);
    }
    if super::core::terminal_outcome(&plan.request, config).is_some() {
        finish_terminal_request(store, &current, &plan.request, command.observed_epoch_millis)?;
    } else {
        store.transition_workflow_approval(current.version, &plan.request)?;
    }
    Ok(plan.outcome)
}

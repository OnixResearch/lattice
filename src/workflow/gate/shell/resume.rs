#[derive(Debug, Clone, serde::Serialize)]
pub struct ResumeExecution {
    pub outcome: super::ResumeOutcome,
    pub summary: crate::headless::HeadlessRunSummary,
    pub next_request: Option<super::ApprovalRequestId>,
}

// r[impl lattice.workflow_approval.resume]
// r[impl lattice.workflow_approval.resume.valid]
// r[impl lattice.workflow_approval.resume.denied]
// r[impl lattice.workflow_approval.terminal]
// r[impl lattice.workflow_approval.terminal.duplicate]
pub async fn resume_with_authority_provider(
    store: &crate::store::Store,
    command: &super::ResumeCommand,
    agent_runtime: crate::agent_node::AgentRuntimeConfig,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
) -> Result<ResumeExecution, Error> {
    let prepared = match admit_resume(store, command)? {
        ResumeAdmission::Idempotent(execution) => return Ok(execution),
        ResumeAdmission::Ready(prepared) => *prepared,
    };
    let executed = execute_remaining(store, command, agent_runtime, authority_provider, prepared).await?;
    persist_resume(store, command, executed)
}

struct PreparedResume {
    current: super::ApprovalRequest,
    workflow: crate::engine::workflow::Workflow,
    signalled: super::ApprovalRequest,
    claim: super::core::ResumeClaimPlan,
}

enum ResumeAdmission {
    Idempotent(ResumeExecution),
    Ready(Box<PreparedResume>),
}

struct ExecutedResume {
    prepared: PreparedResume,
    execution: crate::engine::executor::ApprovalRunOutcome,
    completed_nodes: Vec<super::PersistedNodeOutcome>,
    completed_request: super::ApprovalRequest,
}

fn admit_resume(store: &crate::store::Store, command: &super::ResumeCommand) -> Result<ResumeAdmission, Error> {
    let current = store.load_workflow_approval(&command.request_id)?.ok_or(Error::RunMissing)?;
    if current.status != super::ApprovalRequestStatus::Resumed {
        require_active_request(store, &current)?;
    }
    let workflow = load_bound_revision(store, &current)?;
    let config = approval_config(&workflow, current.node_id)?;
    let claim = match super::core::plan_resume_claim(&current, config, command) {
        Ok(claim) => claim,
        Err(error) if error.code == super::core::AdmissionCode::Expired => {
            let expired = super::core::expire_before_resume(&current, config, command.observed_epoch_millis)?;
            finish_terminal_request(store, &current, &expired, command.observed_epoch_millis)?;
            return Err(error.into());
        }
        Err(error) => return Err(error.into()),
    };
    if matches!(claim.outcome, super::ResumeOutcome::Idempotent { .. }) {
        let run = store.load_run(current.run_id)?.ok_or(Error::RunMissing)?;
        return Ok(ResumeAdmission::Idempotent(ResumeExecution {
            outcome: claim.outcome,
            summary: summary_from_run_record(&run),
            next_request: None,
        }));
    }
    store.transition_workflow_approval(current.version, &claim.request)?;
    let mut signalled = claim.request.clone();
    super::choregraph::apply_resume_signal(&workflow, &mut signalled)?;
    let signalled = super::core::record_resume_signal(&signalled, &command.owner)?;
    store.transition_workflow_approval(claim.request.version, &signalled)?;
    debug_assert_eq!(signalled.status, super::ApprovalRequestStatus::Resuming);
    debug_assert_eq!(signalled.active_resume_owner.as_ref(), Some(&command.owner));
    Ok(ResumeAdmission::Ready(Box::new(PreparedResume {
        current,
        workflow,
        signalled,
        claim,
    })))
}

async fn execute_remaining(
    store: &crate::store::Store,
    command: &super::ResumeCommand,
    agent_runtime: crate::agent_node::AgentRuntimeConfig,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
    prepared: PreparedResume,
) -> Result<ExecutedResume, Error> {
    let checkpoint = checkpoint_to_results(&prepared.current.completed_nodes)?;
    let gate_outputs = resume_outputs(&prepared.workflow, prepared.current.node_id, command)?;
    let authority = authority_provider
        .load(prepared.current.run_id, Some(store.clone()))
        .map_err(|_| Error::AuthorityUnavailable)
        .inspect_err(|_| mark_uncertain(store, &prepared.signalled, &command.owner))?;
    let workflow_catalog = std::sync::Arc::new(crate::engine::WorkflowCatalogRepository::from_repository(store)?);
    let subworkflow_shell = crate::engine::RepositorySubworkflowRunnerShell::product(workflow_catalog);
    let mut executor = crate::engine::executor::Executor::new_with_agent_runtime_config(agent_runtime)
        .with_subworkflow_runner_shell(Box::new(subworkflow_shell));
    let cache = std::collections::HashMap::new();
    let execution = executor
        .resume_authorized_after_approval(crate::engine::executor::ApprovalResumeInput {
            workflow: &prepared.workflow,
            cache: &cache,
            context: &authority,
            checkpoint: &checkpoint,
            resumed_gate: prepared.current.node_id,
            resumed_gate_outputs: &gate_outputs,
        })
        .await;
    let completed_nodes = checkpoint_from_results(&execution.results)
        .inspect_err(|_| mark_uncertain(store, &prepared.signalled, &command.owner))?;
    let prior_nodes = prepared
        .current
        .completed_nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<std::collections::BTreeSet<_>>();
    let new_outcomes = completed_nodes
        .iter()
        .filter(|node| !prior_nodes.contains(&node.node_id))
        .cloned()
        .collect::<Vec<_>>();
    let mut observed = prepared.signalled.clone();
    super::choregraph::apply_execution_outcomes(&prepared.workflow, &mut observed, &new_outcomes)
        .inspect_err(|_| mark_uncertain(store, &prepared.signalled, &command.owner))?;
    observed.completed_nodes = completed_nodes.clone();
    let completed_request = super::core::complete_resume(&observed, &command.owner, prepared.claim.plan.id.clone())?;
    crate::headless::persist_completed_outputs_to_cache(store, &execution.results);
    debug_assert!(completed_nodes.len() >= prepared.current.completed_nodes.len());
    debug_assert_eq!(completed_request.status, super::ApprovalRequestStatus::Resumed);
    Ok(ExecutedResume {
        prepared,
        execution,
        completed_nodes,
        completed_request,
    })
}

fn persist_resume(
    store: &crate::store::Store,
    command: &super::ResumeCommand,
    executed: ExecutedResume,
) -> Result<ResumeExecution, Error> {
    let ExecutedResume {
        prepared,
        execution,
        completed_nodes,
        completed_request,
    } = executed;
    let mut run = store.load_run(prepared.current.run_id)?.ok_or(Error::RunMissing)?;
    let details = crate::run::history::detail_record_from_results(
        prepared.current.run_id,
        &prepared.workflow,
        &execution.results,
    );
    let replay_finished_epoch_millis = execution.suspension.is_none().then_some(command.observed_epoch_millis);
    let replay_events = crate::replay::replay_events_from_results(
        prepared.current.run_id,
        &prepared.workflow,
        &execution.results,
        run.started_epoch_ms,
        replay_finished_epoch_millis,
    );
    if let Some(suspension) = execution.suspension.clone() {
        return persist_next_suspension(store, command, prepared, NextSuspensionPersistence {
            results: execution.results,
            completed_nodes,
            completed_request,
            suspension,
            run,
            details,
            replay_events,
        });
    }
    let summary = crate::headless::summarize_run(prepared.current.run_id, &prepared.workflow, &execution.results);
    run.finished_epoch_ms = Some(command.observed_epoch_millis);
    run.node_count = summary.node_count;
    run.cache_hits = summary.cache_hits;
    run.failures = summary.failures;
    store.finish_workflow_approval_run(
        prepared.signalled.version,
        &completed_request,
        &run,
        &details,
        &replay_events,
    )?;
    debug_assert!(run.finished_epoch_ms.is_some());
    debug_assert!(execution.suspension.is_none());
    Ok(ResumeExecution {
        outcome: prepared.claim.outcome,
        summary,
        next_request: None,
    })
}

struct NextSuspensionPersistence {
    results: Vec<crate::engine::executor::StepResult>,
    completed_nodes: Vec<super::PersistedNodeOutcome>,
    completed_request: super::ApprovalRequest,
    suspension: crate::engine::executor::ApprovalSuspensionPoint,
    run: crate::store::RunRecord,
    details: crate::store::RunDetailRecord,
    replay_events: Vec<crate::replay::ReplayEventRecord>,
}

fn persist_next_suspension(
    store: &crate::store::Store,
    command: &super::ResumeCommand,
    prepared: PreparedResume,
    input: NextSuspensionPersistence,
) -> Result<ResumeExecution, Error> {
    let graph_events = super::choregraph::events_before_suspension(&input.completed_nodes);
    let mut next = super::core::build_request(super::core::RequestBuildInput {
        run_id: prepared.current.run_id,
        workflow: &prepared.workflow,
        node_id: input.suspension.node_id,
        requester: super::ActorId::try_from_text(DEFAULT_REQUESTER_ID)
            .map_err(|error| Error::Admission(error.to_string()))?,
        observed_epoch_millis: command.observed_epoch_millis,
        gate_inputs: &input.suspension.inputs,
        completed_nodes: input.completed_nodes,
        graph_events,
    })?;
    super::choregraph::apply_suspend(&prepared.workflow, &mut next)?;
    let next = super::core::record_suspension(&next)?;
    let summary = suspended_summary(prepared.current.run_id, &prepared.workflow, &input.results);
    let mut run = input.run;
    run.finished_epoch_ms = None;
    run.node_count = summary.node_count;
    run.cache_hits = summary.cache_hits;
    run.failures = summary.failures;
    store.advance_workflow_approval_gate(crate::store::WorkflowApprovalAdvance {
        expected_version: prepared.signalled.version,
        completed_request: &input.completed_request,
        next_request: &next,
        run: &run,
        details: &input.details,
        replay_events: &input.replay_events,
    })?;
    debug_assert!(run.finished_epoch_ms.is_none());
    debug_assert_eq!(next.status, super::ApprovalRequestStatus::Suspended);
    Ok(ResumeExecution {
        outcome: prepared.claim.outcome,
        summary,
        next_request: Some(next.id),
    })
}

fn mark_uncertain(store: &crate::store::Store, signalled: &super::ApprovalRequest, owner: &super::ResumeOwnerId) {
    if let Ok(uncertain) = super::core::mark_resume_uncertain(signalled, owner) {
        let _result = store.transition_workflow_approval(signalled.version, &uncertain);
    }
}

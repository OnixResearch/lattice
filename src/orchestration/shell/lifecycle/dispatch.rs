// r[impl lattice.agent_orchestration.rounds]
pub(in crate::orchestration::shell) async fn execute_ready_round(
    store: &crate::store::Store,
    instance: crate::orchestration::core::Instance,
    execution: super::super::ProductExecution<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    execute_ready_round_backend(store, instance, super::super::ExecutionBackend::Authority(execution)).await
}

pub(in crate::orchestration::shell) async fn execute_ready_round_backend(
    store: &crate::store::Store,
    instance: crate::orchestration::core::Instance,
    execution: super::super::ExecutionBackend<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let workflow = super::admission::load_validated_workflow(store, &instance.workflow_revision)?;
    debug_assert_eq!(workflow.meta.name, instance.workflow_revision.workflow_name);
    let run_id = store.next_run_id()?;
    let intent = crate::orchestration::core::DispatchIntent {
        id: crate::orchestration::core::dispatch_identity(&instance, run_id)?,
        run_id,
        workflow_revision: instance.workflow_revision.clone(),
        status: crate::orchestration::core::DispatchStatus::Pending,
    };
    let plan = super::admission::planned_charge(&instance.definition, &workflow)?;
    debug_assert_eq!(intent.workflow_revision, instance.workflow_revision);
    let start_action = crate::orchestration::core::Action::StartRound {
        intent: intent.clone(),
        planned_charge: plan,
    };
    let started = match super::super::durability::persistence::apply_and_commit(store, &instance, start_action) {
        Ok(started) => started,
        Err(super::super::Error::Core(error))
            if matches!(
                error.code,
                crate::orchestration::core::DiagnosticCode::BudgetExceeded
                    | crate::orchestration::core::DiagnosticCode::MissingBudgetBound
            ) =>
        {
            return super::super::durability::persistence::apply_and_commit(
                store,
                &instance,
                crate::orchestration::core::Action::BudgetExhausted {
                    safe_reason: error.message,
                },
            );
        }
        Err(error) => return Err(error),
    };
    execute_started_round_backend(store, started, workflow, intent, execution).await
}

pub(in crate::orchestration::shell) async fn execute_started_round_backend(
    store: &crate::store::Store,
    started: crate::orchestration::core::Instance,
    workflow: crate::engine::Workflow,
    intent: crate::orchestration::core::DispatchIntent,
    execution: super::super::ExecutionBackend<'_>,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let run_id = intent.run_id;
    let acknowledged = acknowledge(store, &started, &intent)?;
    debug_assert_eq!(acknowledged.active_run_id, Some(run_id));
    debug_assert!(
        acknowledged
            .pending_dispatch
            .as_ref()
            .is_some_and(|dispatch| dispatch.status == crate::orchestration::core::DispatchStatus::Acknowledged)
    );

    let owner = crate::store::RunOwner::Orchestration {
        instance_id: acknowledged.id.clone(),
        dispatch_id: intent.id.clone(),
    };
    let result = run_round_backend(store, workflow, run_id, owner, execution).await;
    let current = super::super::durability::persistence::load_instance_by_id(store, &acknowledged.id)?;
    // Guarded summaries contain typed redacted facts only.
    let guarded_summary = crate::guarded::status::optional(store, run_id, &current.definition.id)?;
    let (status, is_retryable_observed_failure, summary) = match result {
        Ok(outcome) => {
            let is_succeeded = outcome.summary.status == crate::headless::HeadlessRunStatus::Succeeded;
            let is_retryable = !is_succeeded && !outcome.cancelled && exact_owned_failed_run(store, &current, &intent)?;
            (
                crate::orchestration::usage::RunOutcomeObservation {
                    succeeded: is_succeeded,
                    cancelled: Some(outcome.cancelled),
                },
                is_retryable,
                guarded_summary.unwrap_or_else(|| safe_run_summary(&outcome.summary)),
            )
        }
        Err(_) => (
            crate::orchestration::usage::RunOutcomeObservation {
                succeeded: false,
                cancelled: None,
            },
            false,
            guarded_summary.unwrap_or_else(|| super::super::UNOBSERVED_ROUND_FAILURE_SUMMARY.to_string()),
        ),
    };
    super::super::durability::persistence::apply_and_commit(
        store,
        &current,
        super::super::durability::settlement::finish_action(
            store,
            &current,
            super::super::durability::settlement::FinishRequest {
                dispatch_id: intent.id,
                run_id,
                status,
                is_retryable_observed_failure,
                safe_summary: summary,
            },
        )?,
    )
}

async fn run_round_backend(
    store: &crate::store::Store,
    workflow: crate::engine::Workflow,
    run_id: crate::engine::RunId,
    owner: crate::store::RunOwner,
    execution: super::super::ExecutionBackend<'_>,
) -> Result<crate::headless::SnapshotRunOutcome, crate::headless::HeadlessRunError> {
    debug_assert!(!workflow.meta.name.is_empty());
    debug_assert!(!matches!(owner, crate::store::RunOwner::Unscoped));
    let cancellation = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    match execution {
        super::super::ExecutionBackend::Authority(execution) => {
            crate::headless::run_workflow_snapshot_with_authority_provider_and_cancellation(
                store,
                crate::headless::SnapshotRunRequest {
                    workflow,
                    run_id,
                    owner,
                    agent_runtime: execution.agent_runtime,
                    cancellation,
                },
                execution.authority_provider,
            )
            .await
        }
        super::super::ExecutionBackend::AppConfig(app_config) => {
            crate::headless::run_workflow_snapshot_with_app_config(
                store,
                crate::headless::AppConfigSnapshotRun {
                    workflow,
                    run_id,
                    owner,
                    cancellation,
                },
                app_config,
            )
            .await
        }
    }
}

fn acknowledge(
    store: &crate::store::Store,
    started: &crate::orchestration::core::Instance,
    intent: &crate::orchestration::core::DispatchIntent,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    super::super::durability::persistence::apply_and_commit(
        store,
        started,
        crate::orchestration::core::Action::DispatchAcknowledged {
            dispatch_id: intent.id.clone(),
        },
    )
}

// r[impl lattice.agent_orchestration.observed_failure_retry]
pub(super) fn exact_owned_failed_run(
    store: &crate::store::Store,
    instance: &crate::orchestration::core::Instance,
    intent: &crate::orchestration::core::DispatchIntent,
) -> Result<bool, super::super::Error> {
    if instance.definition.retry.is_none() {
        return Ok(false);
    }
    let Some(record) = store.load_run(intent.run_id)? else {
        return Ok(false);
    };
    super::super::durability::persistence::validate_run_owner(instance, intent, &record)?;
    Ok(record.finished_epoch_ms.is_some() && record.failures > 0)
}

pub(super) fn safe_run_summary(summary: &crate::headless::HeadlessRunSummary) -> String {
    format!(
        "workflow {} run {} status {:?}: {} nodes, {} failures, {} cache hits",
        summary.workflow_name, summary.run_id, summary.status, summary.node_count, summary.failures, summary.cache_hits
    )
}

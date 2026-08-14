pub(crate) fn checkpoint_from_results(
    results: &[crate::engine::executor::StepResult],
) -> Result<Vec<super::PersistedNodeOutcome>, Error> {
    let mut checkpoint = Vec::with_capacity(results.len());
    for step in results {
        let (state, outputs, error) = match &step.status {
            crate::engine::executor::NodeStatus::Completed { outputs, .. } => {
                if outputs.values().any(|value| !value.sensitive_fragments.is_empty()) {
                    return Err(Error::CheckpointInvalid);
                }
                (
                    super::PersistedNodeState::Completed,
                    outputs.iter().map(|(key, value)| (*key, value.clone())).collect(),
                    None,
                )
            }
            crate::engine::executor::NodeStatus::Failed { .. } => (
                super::PersistedNodeState::Failed,
                std::collections::BTreeMap::new(),
                Some("workflow node failed before approval".to_string()),
            ),
            crate::engine::executor::NodeStatus::Skipped => {
                (super::PersistedNodeState::Cancelled, std::collections::BTreeMap::new(), None)
            }
            crate::engine::executor::NodeStatus::Pending | crate::engine::executor::NodeStatus::Running => {
                return Err(Error::CheckpointInvalid);
            }
        };
        checkpoint.push(super::PersistedNodeOutcome {
            node_id: step.node_id,
            state,
            input_hash: step.input_hash.to_hex(),
            outputs,
            error,
        });
    }
    debug_assert_eq!(checkpoint.len(), results.len());
    debug_assert!(
        checkpoint
            .iter()
            .all(|node| node.outputs.values().all(|value| value.sensitive_fragments.is_empty()))
    );
    Ok(checkpoint)
}

pub(crate) fn checkpoint_to_results(
    checkpoint: &[super::PersistedNodeOutcome],
) -> Result<Vec<crate::engine::executor::StepResult>, Error> {
    let mut results = Vec::with_capacity(checkpoint.len());
    for node in checkpoint {
        let hash = blake3::Hash::from_hex(&node.input_hash).map_err(|_| Error::CheckpointInvalid)?;
        let input_hash = crate::engine::value::ContentHash(*hash.as_bytes());
        let status = match node.state {
            super::PersistedNodeState::Completed => crate::engine::executor::NodeStatus::Completed {
                outputs: node.outputs.iter().map(|(key, value)| (*key, value.clone())).collect(),
                cached: false,
            },
            super::PersistedNodeState::Failed => crate::engine::executor::NodeStatus::Failed {
                error: node.error.clone().unwrap_or_else(|| "workflow node failed before approval".to_string()),
            },
            super::PersistedNodeState::Cancelled => crate::engine::executor::NodeStatus::Skipped,
        };
        results.push(crate::engine::executor::StepResult {
            node_id: node.node_id,
            status,
            input_hash,
        });
    }
    debug_assert_eq!(results.len(), checkpoint.len());
    debug_assert!(results.iter().all(|step| !matches!(
        step.status,
        crate::engine::executor::NodeStatus::Pending | crate::engine::executor::NodeStatus::Running
    )));
    Ok(results)
}

pub(crate) fn resume_outputs(
    workflow: &crate::engine::workflow::Workflow,
    node_id: crate::engine::workflow::NodeId,
    command: &super::ResumeCommand,
) -> Result<std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::value::Value>, Error> {
    const CREDENTIAL_OUTPUT_PREFIX: &str = "credential:";
    let node = workflow.graph.node(node_id).ok_or(Error::GateMissing)?;
    let values = command
        .values
        .iter()
        .map(|value| (value.name.as_str(), value.value.clone()))
        .collect::<std::collections::HashMap<_, _>>();
    let credentials = command
        .credentials
        .iter()
        .map(|binding| (binding.name.as_str(), binding.reference.identity()))
        .collect::<std::collections::HashMap<_, _>>();
    let mut outputs = std::collections::HashMap::with_capacity(node.output_ports.len());
    for port in &node.output_ports {
        if let Some(value) = values.get(port.label.as_str()) {
            outputs.insert(port.id, value.clone());
            continue;
        }
        let Some(name) = port.label.strip_prefix(CREDENTIAL_OUTPUT_PREFIX) else {
            return Err(Error::CheckpointInvalid);
        };
        let reference = credentials.get(name).ok_or(Error::CheckpointInvalid)?;
        outputs.insert(port.id, crate::engine::value::Value::from_string(reference.clone()));
    }
    debug_assert_eq!(outputs.len(), node.output_ports.len());
    debug_assert!(node.output_ports.iter().all(|port| outputs.contains_key(&port.id)));
    Ok(outputs)
}

fn suspended_summary(
    run_id: crate::engine::executor::RunId,
    workflow: &crate::engine::workflow::Workflow,
    results: &[crate::engine::executor::StepResult],
) -> crate::headless::HeadlessRunSummary {
    let completed = results
        .iter()
        .filter(|step| matches!(step.status, crate::engine::executor::NodeStatus::Completed { .. }))
        .count();
    let failures = results
        .iter()
        .filter(|step| matches!(step.status, crate::engine::executor::NodeStatus::Failed { .. }))
        .count();
    let cache_hits = results
        .iter()
        .filter(|step| matches!(step.status, crate::engine::executor::NodeStatus::Completed { cached: true, .. }))
        .count();
    debug_assert!(completed.saturating_add(failures) <= results.len());
    debug_assert!(cache_hits <= completed);
    crate::headless::HeadlessRunSummary {
        run_id,
        workflow_name: workflow.meta.name.clone(),
        status: crate::headless::HeadlessRunStatus::Suspended,
        node_count: workflow.graph.node_count(),
        steps_run: results.len(),
        completed,
        failures,
        cache_hits,
    }
}

fn summary_from_run_record(record: &crate::store::RunRecord) -> crate::headless::HeadlessRunSummary {
    let status = if record.finished_epoch_ms.is_none() {
        crate::headless::HeadlessRunStatus::Suspended
    } else if record.failures == 0 {
        crate::headless::HeadlessRunStatus::Succeeded
    } else {
        crate::headless::HeadlessRunStatus::Failed
    };
    crate::headless::HeadlessRunSummary {
        run_id: record.run_id,
        workflow_name: record.workflow_name.clone(),
        status,
        node_count: record.node_count,
        steps_run: record.node_count,
        completed: record.node_count.saturating_sub(record.failures),
        failures: record.failures,
        cache_hits: record.cache_hits,
    }
}

fn finish_terminal_request(
    store: &crate::store::Store,
    current: &super::ApprovalRequest,
    updated: &super::ApprovalRequest,
    observed_epoch_millis: i64,
) -> Result<(), Error> {
    let mut run = store.load_run(current.run_id)?.ok_or(Error::RunMissing)?;
    let details = store.load_run_details(current.run_id)?.ok_or(Error::RunMissing)?;
    let replay_events = store.load_run_replay_events(current.run_id)?.ok_or(Error::RunMissing)?;
    run.finished_epoch_ms = Some(observed_epoch_millis);
    run.failures = run.failures.max(TERMINAL_APPROVAL_FAILURE_COUNT);
    store.finish_workflow_approval_run(current.version, updated, &run, &details, &replay_events)?;
    Ok(())
}

fn require_active_request(store: &crate::store::Store, request: &super::ApprovalRequest) -> Result<(), Error> {
    let active = store.load_workflow_approval_for_run(request.run_id)?.ok_or(Error::ActiveRequestChanged)?;
    if active.id != request.id {
        return Err(Error::ActiveRequestChanged);
    }
    Ok(())
}

fn load_bound_revision(
    store: &crate::store::Store,
    request: &super::ApprovalRequest,
) -> Result<crate::engine::workflow::Workflow, Error> {
    let workflow = store.load_workflow(&request.workflow_name)?.ok_or(Error::WorkflowMissing)?;
    let revision = crate::orchestration::workflow_revision(&workflow).map_err(|_| Error::WorkflowChanged)?;
    if revision.content_hash != request.workflow_revision {
        return Err(Error::WorkflowChanged);
    }
    let config = approval_config(&workflow, request.node_id)?;
    super::core::validate_request_integrity(request, &workflow, config)?;
    super::choregraph::replay_request(&workflow, request)?;
    Ok(workflow)
}

fn approval_config(
    workflow: &crate::engine::workflow::Workflow,
    node_id: crate::engine::workflow::NodeId,
) -> Result<&super::ApprovalNodeConfig, Error> {
    let config = workflow.node_config(node_id).ok_or(Error::GateMissing)?;
    if let crate::engine::workflow::NodeConfig::Approval(config) = config {
        return Ok(config);
    }
    Err(Error::GateMissing)
}

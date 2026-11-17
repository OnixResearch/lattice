use anyhow::Context;

pub(super) fn accepted_task(
    request: &super::Request,
    snapshot: &molten_batch_core::ProgressSnapshot,
    task: &molten_batch_core::TaskSpec,
) -> anyhow::Result<()> {
    let molten_batch_core::ProgressTask::Accepted {
        output,
        result,
        fence,
        request: request_ref,
    } = &snapshot.tasks[&task.id]
    else {
        return Ok(());
    };
    validate_fence(fence, snapshot, task)?;
    let execution = execution_request(request, snapshot, task, fence)?;
    molten_batch_core::validate_request(&execution).map_err(|e| anyhow::anyhow!("{e}"))?;
    anyhow::ensure!(
        *request_ref == molten_orchestrator_codec::identity("execution-request.v1", &execution)?,
        "crossed execution request"
    );
    validate_result(*request_ref, *output, *result)
}

fn validate_fence(
    fence: &molten_batch_core::Fence,
    snapshot: &molten_batch_core::ProgressSnapshot,
    task: &molten_batch_core::TaskSpec,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        (fence.cluster, fence.generation, fence.run, fence.spec_ref, &fence.task)
            == (snapshot.cluster, snapshot.generation, snapshot.run, snapshot.spec_ref, &task.id),
        "crossed task fence"
    );
    anyhow::ensure!(
        fence.node != molten_batch_core::Id([0; 32]) && fence.epoch > 0 && fence.epoch <= task.max_attempts,
        "invalid task lease"
    );
    let assignment = molten_orchestrator_codec::identity(
        "assignment-id.v1",
        &(fence.cluster, fence.generation, fence.run, &fence.task, fence.node, fence.epoch, fence.spec_ref),
    )?;
    anyhow::ensure!(assignment == fence.assignment, "wrong assignment identity");
    Ok(())
}

fn execution_request(
    request: &super::Request,
    snapshot: &molten_batch_core::ProgressSnapshot,
    task: &molten_batch_core::TaskSpec,
    fence: &molten_batch_core::Fence,
) -> anyhow::Result<molten_batch_core::ExecutionRequest> {
    Ok(molten_batch_core::ExecutionRequest {
        fence: fence.clone(),
        policy_ref: request.route.policy_ref,
        component: task.component,
        inputs: execution_inputs(snapshot, task)?,
        resources: task.resources,
        fuel: task.fuel,
        timeout_ms: task.timeout_ms,
        max_output_bytes: task.max_output_bytes,
    })
}

fn execution_inputs(
    snapshot: &molten_batch_core::ProgressSnapshot,
    task: &molten_batch_core::TaskSpec,
) -> anyhow::Result<Vec<molten_batch_core::Id>> {
    let input_count =
        task.inputs.len().checked_add(task.dependencies.len()).context("execution input count overflow")?;
    anyhow::ensure!(input_count <= molten_batch_core::MAX_INPUTS, "execution input bound exceeded");
    let mut inputs = Vec::with_capacity(input_count);
    inputs.extend_from_slice(&task.inputs);
    for dependency in &task.dependencies {
        let molten_batch_core::ProgressTask::Accepted { output, .. } = &snapshot.tasks[dependency] else {
            anyhow::bail!("accepted assembly lacks accepted dependency")
        };
        inputs.push(*output);
    }
    Ok(inputs)
}

fn validate_result(
    request_ref: molten_batch_core::Id,
    output: molten_batch_core::Id,
    result: molten_batch_core::Id,
) -> anyhow::Result<()> {
    let accepted = molten_batch_core::ExecutionResult {
        request_ref,
        outcome: molten_batch_core::ExecutionOutcome::Succeeded { output },
        stopped: true,
    };
    anyhow::ensure!(
        result == molten_orchestrator_codec::identity("execution-result.v1", &accepted)?,
        "wrong accepted result identity"
    );
    Ok(())
}

pub(super) fn snapshot_binding(
    request: &super::Request,
    prepared: &molten_render_pilot::Prepared,
    snapshot: &molten_batch_core::ProgressSnapshot,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        snapshot.schema == molten_batch_core::PROGRESS_SCHEMA
            && snapshot.cluster == request.route.cluster
            && snapshot.generation == request.route.generation
            && snapshot.run == request.run_id()?
            && snapshot.spec_ref == molten_orchestrator_codec::identity("job-spec.v1", &prepared.job)?,
        "crossed producer observation"
    );
    anyhow::ensure!(
        snapshot.tasks.len() == prepared.job.tasks.len()
            && prepared.job.tasks.iter().all(|task| snapshot.tasks.contains_key(&task.id)),
        "missing or unexpected tasks"
    );
    Ok(())
}

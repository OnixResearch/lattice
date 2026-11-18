pub(super) fn host_observation(
    store: &crate::store::Store,
    instance: &crate::orchestration::core::Instance,
    request: &super::FinishRequest,
    reserved: crate::orchestration::core::BudgetCharge,
) -> Result<crate::orchestration::usage::CommandWorkerObservation, super::super::super::Error> {
    let record = store.load_run(request.run_id)?;
    if let Some(record) = &record {
        let intent = instance
            .pending_dispatch
            .as_ref()
            .ok_or_else(|| super::super::super::Error::NoPendingDispatch(instance.id.to_string()))?;
        super::super::persistence::validate_run_owner(instance, intent, record)?;
    }
    let interval = record.as_ref().and_then(|record| {
        let interval = crate::orchestration::usage::HostInterval {
            started_epoch_ms: record.started_epoch_ms,
            finished_epoch_ms: record.finished_epoch_ms?,
        };
        interval.duration_millis().map(|_| interval)
    });
    let reports = super::super::telemetry::provider_reports(store, instance, request.run_id)?;
    let charged = super::accounting_charge(reserved, interval.as_ref(), &reports, &instance.definition.budgets);
    // Provider reports cannot refund unobserved token consumption, and the
    // unqualified cost unit cannot replace admitted monetary units.
    debug_assert!(charged.tokens >= reserved.tokens);
    debug_assert_eq!(charged.monetary_microunits, reserved.monetary_microunits);
    Ok(crate::orchestration::usage::CommandWorkerObservation {
        schema: crate::orchestration::usage::USAGE_OBSERVATION_SCHEMA.to_string(),
        scope: crate::orchestration::usage::Scope::HostRun,
        workflow_revision: instance.workflow_revision.content_hash.to_string(),
        instance_id: instance.id.to_string(),
        dispatch_id: request.dispatch_id.to_string(),
        run_id: request.run_id.0,
        node_id: 0,
        invocation_id: request.dispatch_id.to_string(),
        // No command-worker or Kiln attempt identity is invented.
        worker_profile: String::new(),
        attempt_id: request.dispatch_id.to_string(),
        agent_invocations: None,
        model_requests: None,
        subagent_calls: None,
        tokens: None,
        cost_microunits: None,
        duration_millis: interval.as_ref().and_then(crate::orchestration::usage::HostInterval::duration_millis),
        provenance: crate::orchestration::usage::UsageProvenance::HostMeasured,
        complete: false,
        accounting: Some(crate::orchestration::usage::RunAccounting {
            schema: crate::orchestration::usage::RUN_USAGE_ACCOUNTING_SCHEMA.to_string(),
            reserved,
            charged,
            host_interval: interval,
            provider_reports: reports,
            unknown_tokens: true,
            unknown_cost: true,
            unknown_model_requests: true,
            unknown_subagent_calls: true,
            diagnostic: "conservative fallback; qualified worker/service/request bindings require #24; provider cost lacks currency/pricing identity".to_string(),
            outcome: Some(request.status),
        }),
        // Orchestration Agent nodes have no execution binding.
        execution_binding: None,
    })
}

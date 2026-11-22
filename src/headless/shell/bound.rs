//! Imperative shell for one product run of a saved workflow bound to admitted inputs.

impl crate::workflow_template::inputs::Admitted {
    // r[impl lattice.workflow_templates.run_inputs.record]
    /// Run the bound workflow with the configured runtime and authority.
    ///
    /// The run id is reserved together with the input identity, after every input check passed
    /// and before the first node runs. The bound workflow then runs as a snapshot under the
    /// saved workflow's name, and its run, detail, and replay records are written as for any run.
    pub async fn run_with_app_config(
        self,
        store: &crate::store::Store,
        app_config: &crate::config::AppConfig,
    ) -> Result<super::super::HeadlessRunSummary, super::super::HeadlessRunError> {
        let agent_runtime = app_config.effective_agent_runtime();
        let frontier_policy = app_config.frontier_capacity_policy().unwrap_or_else(|error| {
            tracing::error!(?error, "invalid frontier policy, preserving serial execution");
            crate::engine::frontier::serial_capacity_policy()
        });
        debug_assert!(frontier_policy.total_slots > 0);
        debug_assert!(!frontier_policy.lane_slots.is_empty());
        let authority_provider = app_config.effect_authority_provider();
        execute(store, self, authority_provider.as_ref(), || {
            super::configured_executor(agent_runtime, frontier_policy)
        })
        .await
    }

    /// Run the bound workflow with an explicit effect authority source.
    pub async fn run_with_authority_provider(
        self,
        store: &crate::store::Store,
        authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
    ) -> Result<super::super::HeadlessRunSummary, super::super::HeadlessRunError> {
        execute(store, self, authority_provider, crate::engine::executor::Executor::new).await
    }
}

async fn execute(
    store: &crate::store::Store,
    admitted: crate::workflow_template::inputs::Admitted,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
    executor_factory: impl FnOnce() -> crate::engine::executor::Executor,
) -> Result<super::super::HeadlessRunSummary, super::super::HeadlessRunError> {
    // r[impl lattice.workflow_templates.run_inputs.start]
    // A run with inputs is a start path: drifted bound content never reserves a run id. The
    // refusal is a fatal diagnostic like an authority-context failure; a dedicated category
    // would change `HeadlessRunError` or `WorkflowDiagnosticCode`, outside this change.
    crate::workflow_template::inputs::reverify_admitted(&admitted).map_err(|error| {
        super::super::HeadlessRunError::Validation(vec![super::authority_context_diagnostic(error.to_string())])
    })?;
    let (workflow, identity) = admitted.into_run_parts();
    let workflow_name = identity.workflow.workflow_name.clone();
    debug_assert_eq!(workflow.meta.name, workflow_name);
    let run_id = store.reserve_run_with_inputs(&identity)?;
    let outcome = super::run_workflow_snapshot_with_executor_factory(
        super::ProductRunRequest {
            store,
            workflow_name: &workflow_name,
            run_id,
            owner: crate::store::RunOwner::Unscoped,
            authority_provider,
        },
        workflow,
        executor_factory,
    )
    .await?;
    debug_assert_eq!(outcome.summary.run_id, run_id);
    Ok(outcome.summary)
}

#[cfg(test)]
mod tests;

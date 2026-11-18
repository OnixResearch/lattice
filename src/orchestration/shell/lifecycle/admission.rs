// r[impl lattice.agent_orchestration.model]
// r[impl lattice.agent_orchestration.rounds]
pub fn validate_definition_in_store(
    store: &crate::store::Store,
    definition: &crate::orchestration::core::Definition,
) -> Result<super::super::DefinitionValidationReport, super::super::Error> {
    let workflow = store.load_workflow(&definition.workflow.workflow_name)?;
    Ok(validation_report(definition, workflow.as_ref()))
}

pub(super) fn validation_report(
    definition: &crate::orchestration::core::Definition,
    workflow: Option<&crate::engine::Workflow>,
) -> super::super::DefinitionValidationReport {
    let workflow_revision = workflow.and_then(|workflow| crate::orchestration::core::workflow_revision(workflow).ok());
    let orchestration_diagnostics =
        crate::orchestration::core::validate_definition(definition, workflow_revision.as_ref());
    let workflow_diagnostics = workflow.map_or_else(Vec::new, crate::workflow_validation::validate_workflow);
    super::super::DefinitionValidationReport {
        definition_revision: crate::orchestration::core::definition_revision(definition)
            .ok()
            .map(|revision| revision.to_string()),
        workflow_revision,
        orchestration_diagnostics,
        workflow_diagnostics,
    }
}

pub(in crate::orchestration::shell) fn load_validated_workflow(
    store: &crate::store::Store,
    reference: &crate::orchestration::core::WorkflowRevisionRef,
) -> Result<crate::engine::Workflow, super::super::Error> {
    let workflow = store
        .load_workflow(&reference.workflow_name)?
        .ok_or_else(|| super::super::Error::WorkflowNotFound(reference.workflow_name.clone()))?;
    let actual = crate::orchestration::core::workflow_revision(&workflow)?;
    if actual != *reference {
        return Err(super::super::Error::Core(crate::orchestration::core::Diagnostic {
            code: crate::orchestration::core::DiagnosticCode::WorkflowRevisionMismatch,
            message: "stored workflow changed after orchestration revision admission".to_string(),
        }));
    }
    let diagnostics = crate::workflow_validation::validate_workflow(&workflow);
    if crate::workflow_validation::has_fatal_diagnostics(&diagnostics) {
        return Err(super::super::Error::DefinitionInvalid(Box::new(super::super::DefinitionValidationReport {
            definition_revision: None,
            workflow_revision: Some(actual),
            orchestration_diagnostics: Vec::new(),
            workflow_diagnostics: diagnostics,
        })));
    }
    let build = crate::engine::runner::build_runner_plan(&workflow);
    let Some(plan) = build.plan else {
        return Err(super::super::Error::InvalidProposal(
            "successor workflow cannot produce a runner plan".to_string(),
        ));
    };
    if matches!(
        crate::engine::runner::admit_runner_plan(&plan),
        crate::engine::runner::RunnerAdmission::Rejected { .. }
    ) {
        return Err(super::super::Error::InvalidProposal(
            "successor workflow is not admitted by the Seaglass-backed runner".to_string(),
        ));
    }
    debug_assert_eq!(actual, *reference);
    debug_assert!(!crate::workflow_validation::has_fatal_diagnostics(&diagnostics));
    Ok(workflow)
}

// r[impl lattice.agent_orchestration.usage.scenario.shell]
// r[impl lattice.graph_construction.runtime_controls.budget]
pub(in crate::orchestration::shell) fn planned_charge(
    definition: &crate::orchestration::core::Definition,
    workflow: &crate::engine::Workflow,
) -> Result<crate::orchestration::core::BudgetChargePlan, super::super::Error> {
    validate_accounted_routes(definition, workflow)?;
    let agent_invocations = workflow_agent_invocations(workflow)?;
    let configured = definition.budgets.per_round_upper_bound;
    Ok(crate::orchestration::core::BudgetChargePlan {
        rounds: Some(super::super::ROUND_CHARGE),
        agent_invocations: Some(agent_invocations.saturating_mul(super::super::AGENT_INVOCATION_CHARGE)),
        tokens: Some(configured.tokens),
        monetary_microunits: Some(configured.monetary_microunits),
        duration_millis: Some(configured.duration_millis),
    })
}

fn workflow_agent_invocations(workflow: &crate::engine::Workflow) -> Result<u64, super::super::Error> {
    let agent_count = workflow
        .configs
        .values()
        .filter(|config| matches!(config, crate::engine::NodeConfig::Agent(_)))
        .count();
    u64::try_from(agent_count).map_err(|_| {
        super::super::Error::Core(crate::orchestration::core::Diagnostic {
            code: crate::orchestration::core::DiagnosticCode::BudgetOverflow,
            message: "workflow agent invocation count does not fit the budget model".to_string(),
        })
    })
}

pub(super) fn validate_accounted_routes(
    definition: &crate::orchestration::core::Definition,
    workflow: &crate::engine::Workflow,
) -> Result<(), super::super::Error> {
    let enforced = definition.budgets.enforced;
    let is_worker_accounting_enforced = enforced.agent_invocations || enforced.tokens || enforced.monetary_microunits;
    // Arbitrary shells can invoke models behind any wrapper. Reject unqualified
    // routes, not command-name guesses, when worker/model accounting is required.
    if is_worker_accounting_enforced
        && workflow.configs.values().any(|config| matches!(config, crate::engine::NodeConfig::Shell { .. }))
    {
        return Err(super::super::Error::InvalidProposal(
            "shell routes require an admitted accounted worker profile when worker/model accounting is enforced; execution bindings #24 are unavailable".to_string(),
        ));
    }
    Ok(())
}

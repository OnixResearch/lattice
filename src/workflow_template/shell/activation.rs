pub fn application_facts(
    store: &crate::store::Store,
    bundle: &super::model::ValidatedBundle,
) -> Result<super::model::ApplicationFacts, Error> {
    let workflow = match store.load_workflow(&bundle.workflow.meta.name)? {
        None => super::model::ExistingContent::Missing,
        Some(workflow) => {
            let revision =
                crate::orchestration::workflow_revision(&workflow).map_err(|_| Error::ApplicationConflict)?;
            let identity = super::model::ContentIdentity::try_from_hex(revision.content_hash.as_str())
                .map_err(|_| Error::ApplicationConflict)?;
            if revision == bundle.workflow_revision {
                super::model::ExistingContent::Exact(identity)
            } else {
                super::model::ExistingContent::Conflicting(identity)
            }
        }
    };
    let orchestration = match &bundle.orchestration {
        None => None,
        Some(definition) => match store.load_orchestration_definition(&definition.id)? {
            None => Some(super::model::ExistingContent::Missing),
            Some(stored) => {
                let revision =
                    crate::orchestration::definition_revision(&stored).map_err(|_| Error::ApplicationConflict)?;
                let identity = super::model::ContentIdentity::try_from_hex(revision.as_str())
                    .map_err(|_| Error::ApplicationConflict)?;
                if Some(&revision) == bundle.orchestration_revision.as_ref() {
                    Some(super::model::ExistingContent::Exact(identity))
                } else {
                    Some(super::model::ExistingContent::Conflicting(identity))
                }
            }
        },
    };
    debug_assert!(!bundle.workflow_revision.content_hash.as_str().is_empty());
    debug_assert_eq!(bundle.orchestration.is_some(), bundle.orchestration_revision.is_some());
    Ok(super::model::ApplicationFacts {
        workflow,
        orchestration,
    })
}

// r[impl lattice.workflow_templates.activation]
pub fn save_draft(
    store: &crate::store::Store,
    bundle: &super::model::ValidatedBundle,
) -> Result<super::model::ApplicationPlan, Error> {
    let facts = application_facts(store, bundle)?;
    let plan = super::core::plan_application(bundle, &facts);
    match &plan {
        super::model::ApplicationPlan::Conflict(_) => Err(Error::ApplicationConflict),
        super::model::ApplicationPlan::ExactAlreadyPresent { .. } => Ok(plan),
        super::model::ApplicationPlan::CreateDraft { .. } => {
            store.save_workflow(&bundle.workflow)?;
            Ok(plan)
        }
        super::model::ApplicationPlan::Preview { .. } => Ok(plan),
    }
}

pub async fn start_orchestration_draft(
    store: &crate::store::Store,
    bundle: &super::model::ValidatedBundle,
    facts: &super::model::ActivationFacts,
    actor: &str,
    runtime: crate::agent_node::AgentRuntimeConfig,
) -> Result<crate::orchestration::OrchestrationInstance, Error> {
    let definition = admit_activation(store, bundle, facts, actor)?;
    crate::orchestration::start(store, definition, actor, runtime).await.map_err(Error::Orchestration)
}

pub async fn start_orchestration_draft_with_execution(
    store: &crate::store::Store,
    bundle: &super::model::ValidatedBundle,
    facts: &super::model::ActivationFacts,
    actor: &str,
    execution: crate::orchestration::ProductExecution<'_>,
) -> Result<crate::orchestration::OrchestrationInstance, Error> {
    let definition = admit_activation(store, bundle, facts, actor)?;
    crate::orchestration::start_with_execution(store, definition, actor, execution)
        .await
        .map_err(Error::Orchestration)
}

fn admit_activation(
    store: &crate::store::Store,
    bundle: &super::model::ValidatedBundle,
    facts: &super::model::ActivationFacts,
    actor: &str,
) -> Result<crate::orchestration::OrchestrationDefinition, Error> {
    let plan = super::core::plan_activation(bundle, facts);
    let super::model::ActivationPlan::StartOrchestration { .. } = plan else {
        let block = match plan {
            super::model::ActivationPlan::Blocked(block) => block,
            _ => super::model::ActivationBlock::AuthorityNotAdmitted,
        };
        return Err(Error::ActivationBlocked(block));
    };
    debug_assert!(facts.explicit_start_requested);
    debug_assert_eq!(facts.saved_workflow_revision.as_ref(), Some(&bundle.workflow_revision));
    let stored = application_facts(store, bundle)?;
    if !matches!(stored.workflow, super::model::ExistingContent::Exact(_))
        || matches!(stored.orchestration, Some(super::model::ExistingContent::Conflicting(_)))
    {
        return Err(Error::ApplicationConflict);
    }
    let definition = bundle
        .orchestration
        .clone()
        .ok_or(Error::ActivationBlocked(super::model::ActivationBlock::OrchestrationMissing))?;
    let receipt = super::core::activation_receipt(bundle, actor)
        .map_err(|_| Error::ActivationBlocked(super::model::ActivationBlock::AuthorityNotAdmitted))?;
    store.save_workflow_template_activation(&receipt)?;
    Ok(definition)
}

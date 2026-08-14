pub fn plan_application(
    bundle: &super::model::ValidatedBundle,
    facts: &super::model::ApplicationFacts,
) -> super::model::ApplicationPlan {
    debug_assert!(!bundle.manifest.id.is_empty());
    debug_assert!(!bundle.workflow_revision.content_hash.as_str().is_empty());
    let Ok(workflow_identity) =
        super::model::ContentIdentity::try_from_hex(bundle.workflow_revision.content_hash.as_str())
    else {
        return super::model::ApplicationPlan::Conflict(super::model::DiagnosticCode::IdentityMismatch);
    };
    let is_workflow_exact = content_is_exact(&facts.workflow, &workflow_identity);
    if content_is_conflicting(&facts.workflow, &workflow_identity) {
        return super::model::ApplicationPlan::Conflict(super::model::DiagnosticCode::IncompatibleContent);
    }
    let is_orchestration_exact = match (&bundle.orchestration_revision, &facts.orchestration) {
        (None, None | Some(super::model::ExistingContent::Missing)) => true,
        (None, Some(super::model::ExistingContent::Exact(_) | super::model::ExistingContent::Conflicting(_))) => false,
        (Some(revision), Some(existing)) => {
            let Ok(expected) = super::model::ContentIdentity::try_from_hex(revision.as_str()) else {
                return super::model::ApplicationPlan::Conflict(super::model::DiagnosticCode::IdentityMismatch);
            };
            if content_is_conflicting(existing, &expected) {
                return super::model::ApplicationPlan::Conflict(super::model::DiagnosticCode::IncompatibleContent);
            }
            content_is_exact(existing, &expected)
        }
        (Some(_), None) => false,
    };
    if is_workflow_exact && is_orchestration_exact {
        return super::model::ApplicationPlan::ExactAlreadyPresent {
            bundle: bundle.identity.clone(),
            workflow: bundle.workflow_revision.clone(),
            orchestration: bundle.orchestration_revision.clone(),
        };
    }
    super::model::ApplicationPlan::CreateDraft {
        bundle: bundle.identity.clone(),
        workflow: bundle.workflow_revision.clone(),
        orchestration: bundle.orchestration_revision.clone(),
    }
}

pub fn preview_plan(bundle: &super::model::ValidatedBundle) -> super::model::ApplicationPlan {
    super::model::ApplicationPlan::Preview {
        bundle: bundle.identity.clone(),
        workflow: bundle.workflow_revision.clone(),
    }
}

fn content_is_exact(existing: &super::model::ExistingContent, expected: &super::model::ContentIdentity) -> bool {
    matches!(existing, super::model::ExistingContent::Exact(actual) if actual == expected)
}

fn content_is_conflicting(existing: &super::model::ExistingContent, expected: &super::model::ContentIdentity) -> bool {
    match existing {
        super::model::ExistingContent::Missing => false,
        super::model::ExistingContent::Exact(actual) => actual != expected,
        super::model::ExistingContent::Conflicting(_) => true,
    }
}

// r[impl lattice.workflow_templates.activation]
pub fn plan_activation(
    bundle: &super::model::ValidatedBundle,
    facts: &super::model::ActivationFacts,
) -> super::model::ActivationPlan {
    debug_assert!(!bundle.manifest.id.is_empty());
    debug_assert!(!bundle.workflow_revision.content_hash.as_str().is_empty());
    if !facts.explicit_save_requested && !facts.explicit_start_requested {
        return super::model::ActivationPlan::PreviewOnly;
    }
    if facts.explicit_save_requested && !facts.explicit_start_requested {
        return super::model::ActivationPlan::SaveWorkflow {
            bundle: bundle.identity.clone(),
            workflow: bundle.workflow_revision.clone(),
        };
    }
    let Some(saved_workflow) = &facts.saved_workflow_revision else {
        return super::model::ActivationPlan::Blocked(super::model::ActivationBlock::WorkflowNotSaved);
    };
    if saved_workflow != &bundle.workflow_revision {
        return super::model::ActivationPlan::Blocked(super::model::ActivationBlock::WorkflowDrift);
    }
    let Some(expected_orchestration) = &bundle.orchestration_revision else {
        return super::model::ActivationPlan::Blocked(super::model::ActivationBlock::OrchestrationMissing);
    };
    if facts.current_orchestration_revision.as_ref() != Some(expected_orchestration) {
        return super::model::ActivationPlan::Blocked(super::model::ActivationBlock::OrchestrationDrift);
    }
    for (admitted, blocked) in [
        (facts.runtime_admitted, super::model::ActivationBlock::RuntimeNotAdmitted),
        (facts.effects_admitted, super::model::ActivationBlock::EffectsNotAdmitted),
        (facts.credentials_admitted, super::model::ActivationBlock::CredentialsNotAdmitted),
        (facts.triggers_admitted, super::model::ActivationBlock::TriggersNotAdmitted),
        (facts.positive_budgets_admitted, super::model::ActivationBlock::BudgetsNotAdmitted),
        (facts.actor_authority_admitted, super::model::ActivationBlock::AuthorityNotAdmitted),
    ] {
        if !admitted {
            return super::model::ActivationPlan::Blocked(blocked);
        }
    }
    super::model::ActivationPlan::StartOrchestration {
        bundle: bundle.identity.clone(),
        workflow: bundle.workflow_revision.clone(),
        orchestration: expected_orchestration.clone(),
    }
}

pub fn activation_receipt(
    bundle: &super::model::ValidatedBundle,
    actor: &str,
) -> Result<super::model::ActivationReceipt, super::model::Diagnostic> {
    if actor.trim().is_empty() || actor.len() > super::model::MAX_IDENTIFIER_BYTES {
        return Err(diagnostic(
            &bundle.manifest.id,
            super::model::DiagnosticCode::InvalidIdentifier,
            "activation actor identity is invalid",
        ));
    }
    let orchestration = bundle.orchestration.as_ref().ok_or_else(|| {
        diagnostic(
            &bundle.manifest.id,
            super::model::DiagnosticCode::InvalidOrchestration,
            "activation orchestration is missing",
        )
    })?;
    crate::orchestration::authorize_definition_actor(orchestration, actor).map_err(|_| {
        diagnostic(
            &bundle.manifest.id,
            super::model::DiagnosticCode::IncompatibleContent,
            "activation actor authority is denied",
        )
    })?;
    let orchestration_revision = bundle.orchestration_revision.clone().ok_or_else(|| {
        diagnostic(
            &bundle.manifest.id,
            super::model::DiagnosticCode::InvalidOrchestration,
            "activation orchestration identity is missing",
        )
    })?;
    let authority_policy = crate::orchestration::authority_policy_hash(&orchestration.authority).map_err(|_| {
        diagnostic(
            &bundle.manifest.id,
            super::model::DiagnosticCode::InvalidOrchestration,
            "activation authority policy identity failed",
        )
    })?;
    let payload = serde_json::to_vec(&(
        &bundle.identity,
        &bundle.workflow_revision,
        &orchestration_revision,
        &authority_policy,
        actor,
    ))
    .map_err(|_| {
        diagnostic(
            &bundle.manifest.id,
            super::model::DiagnosticCode::InvalidOrchestration,
            "activation receipt identity failed",
        )
    })?;
    debug_assert!(!payload.is_empty());
    debug_assert!(bundle.orchestration_revision.as_ref().is_some_and(|current| current == &orchestration_revision));
    Ok(super::model::ActivationReceipt {
        identity: super::model::ContentIdentity::from_bytes(&payload),
        bundle: bundle.identity.clone(),
        workflow: bundle.workflow_revision.clone(),
        orchestration: orchestration_revision,
        authority_policy,
        actor: actor.to_string(),
    })
}

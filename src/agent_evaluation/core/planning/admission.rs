pub(super) fn require_services(
    services: &std::collections::BTreeSet<crate::agent_evaluation::model::ServiceKind>,
    is_live: bool,
) -> Result<(), crate::agent_evaluation::Error> {
    debug_assert!(services.len() <= crate::agent_evaluation::model::SERVICE_KIND_COUNT);
    let required = [
        crate::agent_evaluation::model::ServiceKind::Workspace,
        crate::agent_evaluation::model::ServiceKind::Verifier,
        crate::agent_evaluation::model::ServiceKind::Artifact,
        crate::agent_evaluation::model::ServiceKind::Clock,
        crate::agent_evaluation::model::ServiceKind::Store,
    ];
    if required.iter().any(|service| !services.contains(service)) {
        return Err(crate::agent_evaluation::Error::ServiceUnavailable);
    }
    let is_missing_live_service = is_live
        && [
            crate::agent_evaluation::model::ServiceKind::Deployment,
            crate::agent_evaluation::model::ServiceKind::Cleanup,
        ]
        .iter()
        .any(|service| !services.contains(service));
    if is_missing_live_service {
        return Err(crate::agent_evaluation::Error::ServiceUnavailable);
    }
    Ok(())
}

pub(super) fn require_route_service(
    route: crate::agent_evaluation::model::RouteKind,
    services: &std::collections::BTreeSet<crate::agent_evaluation::model::ServiceKind>,
) -> Result<(), crate::agent_evaluation::Error> {
    let route_service = match route {
        crate::agent_evaluation::model::RouteKind::Agent => crate::agent_evaluation::model::ServiceKind::Agent,
        crate::agent_evaluation::model::RouteKind::Workflow => crate::agent_evaluation::model::ServiceKind::Workflow,
    };
    if !services.contains(&route_service) {
        return Err(crate::agent_evaluation::Error::ServiceUnavailable);
    }
    Ok(())
}

pub(super) fn validate_budget_availability(
    case: &crate::agent_evaluation::model::AdmittedCase,
    request: &super::PlanRequest,
) -> Result<(), crate::agent_evaluation::Error> {
    debug_assert!(case.definition.budget.trials > 0);
    debug_assert!(case.definition.budget.max_tokens > 0);
    let facts = request
        .budget_availability
        .iter()
        .filter(|fact| fact.case_id == case.definition.id)
        .collect::<Vec<_>>();
    if facts.len() != 1 {
        return Err(crate::agent_evaluation::Error::InvalidBudget);
    }
    let fact = facts[0];
    let required = case.definition.budget;
    let has_insufficient_trials = fact.trials < required.trials || fact.concurrency < required.concurrency;
    let has_insufficient_usage = fact.tokens < required.max_tokens || fact.duration_ms < required.max_duration_ms;
    let has_insufficient_deployments = fact.deployments < required.max_deployments;
    let has_insufficient_budget = [
        has_insufficient_trials,
        has_insufficient_usage,
        has_insufficient_deployments,
    ]
    .into_iter()
    .any(|value| value);
    if has_insufficient_budget {
        return Err(crate::agent_evaluation::Error::InvalidBudget);
    }
    debug_assert!(fact.trials >= required.trials);
    debug_assert!(fact.tokens >= required.max_tokens);
    Ok(())
}

pub(super) fn validate_live_authority(
    case: &crate::agent_evaluation::model::AdmittedCase,
    request: &super::PlanRequest,
) -> Result<(), crate::agent_evaluation::Error> {
    debug_assert!(case.definition.live.enabled);
    debug_assert!(case.definition.budget.max_deployments > 0);
    let facts = request.live_authority.iter().filter(|fact| fact.case_id == case.definition.id).collect::<Vec<_>>();
    if facts.len() != 1 {
        return Err(crate::agent_evaluation::Error::AuthorityDenied);
    }
    let fact = facts[0];
    let posture = &case.definition.live;
    let is_actor_mismatch = posture.operator_actor.as_deref() != Some(fact.actor.as_str());
    let is_target_mismatch = posture.target_namespace.as_deref() != Some(fact.target_namespace.as_str());
    let is_credential_mismatch = posture.credential_handles != fact.credential_handles;
    let is_invalid_deployment_budget =
        fact.max_deployments == 0 || fact.max_deployments > case.definition.budget.max_deployments;
    let is_unsafe_target = fact.target_namespace.contains("prod");
    let is_unsafe_class = matches!(
        case.definition.class,
        crate::agent_evaluation::model::CaseClass::Unit | crate::agent_evaluation::model::CaseClass::Skill
    );
    let has_invalid_authority = [
        is_actor_mismatch,
        is_target_mismatch,
        is_credential_mismatch,
        is_invalid_deployment_budget,
        is_unsafe_target,
        is_unsafe_class,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_authority {
        return Err(crate::agent_evaluation::Error::AuthorityDenied);
    }
    Ok(())
}

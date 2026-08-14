pub(super) fn validate_posture(
    case: &crate::agent_evaluation::model::CaseDefinition,
) -> Result<(), crate::agent_evaluation::Error> {
    debug_assert!(case.budget.max_deployments <= crate::agent_evaluation::model::MAX_DEPLOYMENTS);
    debug_assert!(!case.environment.target_class.is_empty());
    let is_live_class = case.class == crate::agent_evaluation::model::CaseClass::Live;
    let is_hard_class = case.class == crate::agent_evaluation::model::CaseClass::Hard;
    let is_invalid_class_posture =
        (is_live_class && !case.live.enabled) || (!is_live_class && !is_hard_class && case.live.enabled);
    let is_invalid_network_posture =
        case.environment.network == crate::agent_evaluation::model::NetworkPosture::Live && !case.live.enabled;
    if is_invalid_class_posture || is_invalid_network_posture {
        return Err(crate::agent_evaluation::Error::InvalidLivePosture);
    }
    if case.live.enabled {
        validate_enabled_posture(case)?;
    } else {
        validate_disabled_posture(case)?;
    }
    debug_assert!(!is_live_class || case.live.enabled);
    debug_assert!(case.live.enabled || case.live.credential_handles.is_empty());
    Ok(())
}

fn validate_enabled_posture(
    case: &crate::agent_evaluation::model::CaseDefinition,
) -> Result<(), crate::agent_evaluation::Error> {
    debug_assert!(case.live.enabled);
    debug_assert!(case.budget.max_deployments <= crate::agent_evaluation::model::MAX_DEPLOYMENTS);
    let is_missing_deployment_budget = case.budget.max_deployments == 0;
    let is_invalid_actor = case
        .live
        .operator_actor
        .as_deref()
        .is_none_or(|actor| !crate::agent_evaluation::identity::valid_public_label(actor));
    let is_invalid_target =
        case.live.target_namespace.as_deref().is_none_or(|target| {
            !crate::agent_evaluation::identity::valid_public_label(target) || target.contains("prod")
        });
    let is_invalid_credentials = case.live.credential_handles.is_empty()
        || case
            .live
            .credential_handles
            .iter()
            .any(|handle| !crate::agent_evaluation::identity::valid_public_label(handle));
    let has_invalid_live_setting = [
        is_missing_deployment_budget,
        is_invalid_actor,
        is_invalid_target,
        is_invalid_credentials,
        !case.live.cleanup_required,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_live_setting {
        return Err(crate::agent_evaluation::Error::InvalidLivePosture);
    }
    Ok(())
}

fn validate_disabled_posture(
    case: &crate::agent_evaluation::model::CaseDefinition,
) -> Result<(), crate::agent_evaluation::Error> {
    debug_assert!(!case.live.enabled);
    debug_assert!(case.budget.max_deployments <= crate::agent_evaluation::model::MAX_DEPLOYMENTS);
    let has_deployment_budget = case.budget.max_deployments != 0;
    let has_actor = case.live.operator_actor.is_some();
    let has_target = case.live.target_namespace.is_some();
    let has_credentials = !case.live.credential_handles.is_empty();
    let has_disabled_live_setting =
        [has_deployment_budget, has_actor, has_target, has_credentials].into_iter().any(|value| value);
    if has_disabled_live_setting {
        return Err(crate::agent_evaluation::Error::InvalidLivePosture);
    }
    Ok(())
}

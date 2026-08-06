/// Materializes a deterministic router execution plan.
///
/// # Errors
///
/// Returns [`super::ExecutionPlanError`] when selected roles, quorum, budget, or fallback
/// constraints cannot produce a safe deterministic execution plan.
// r[impl agent_harness_runtime.model_router.execution_plan]
pub fn materialize_execution_plan(
    selected: &crate::SelectedPlan,
    catalog: &[crate::CatalogEntry],
    budget: &crate::Budget,
) -> Result<super::RouterExecutionPlan, super::ExecutionPlanError> {
    materialize_execution_plan_inner(selected, catalog, budget, None)
}

/// Materializes a deterministic router execution plan with explicit fallback rotation.
///
/// # Errors
///
/// Returns [`super::ExecutionPlanError`] when selected roles, quorum, budget, rotation, or fallback
/// constraints fail.
pub fn materialize_execution_plan_with_rotation(
    selected: &crate::SelectedPlan,
    catalog: &[crate::CatalogEntry],
    budget: &crate::Budget,
    rotation: &crate::RouterRotationSnapshot,
) -> Result<super::RouterExecutionPlan, super::ExecutionPlanError> {
    materialize_execution_plan_inner(selected, catalog, budget, Some(rotation))
}

fn materialize_execution_plan_inner(
    selected: &crate::SelectedPlan,
    catalog: &[crate::CatalogEntry],
    budget: &crate::Budget,
    rotation: Option<&crate::RouterRotationSnapshot>,
) -> Result<super::RouterExecutionPlan, super::ExecutionPlanError> {
    let roles = super::super::plan::roles_for_family(selected.family());
    validate_plan_constraints(selected, roles.len())?;
    let (role_steps, route_labels) = build_role_steps(selected, catalog, budget, roles)?;
    let fallback_chain = super::super::plan::build_fallback_chain(selected, catalog, budget, &route_labels)?;
    let (fallback_chain, fallback_decision) = rotate_fallback_chain(fallback_chain, rotation)?;
    let mut rotation_decisions = selected.rotation_decisions().to_vec();
    if let Some(decision) = fallback_decision {
        rotation_decisions.push(decision);
    }

    Ok(super::super::record::assemble_execution_plan(
        selected,
        role_steps,
        fallback_chain,
        rotation_decisions,
    ))
}

fn validate_plan_constraints(
    selected: &crate::SelectedPlan,
    roles_len: usize,
) -> Result<(), super::ExecutionPlanError> {
    assert!(roles_len > 0, "role mapping must exist");
    assert!(selected.bounds().max_candidates() > crate::limit::ZERO_COUNT, "candidate cap must be positive");
    assert!(selected.bounds().max_parallelism() > crate::limit::ZERO_COUNT, "parallelism cap must be positive");

    if selected.bounds().min_success_quorum() == crate::limit::ZERO_COUNT {
        return Err(super::ExecutionPlanError::ZeroQuorum);
    }

    if selected.family() != crate::RouteFamily::Single
        && selected.bounds().max_repair_attempts() == crate::limit::ZERO_COUNT
    {
        return Err(super::ExecutionPlanError::UnboundedRepair);
    }

    if selected.selected_models().len() != roles_len {
        return Err(super::ExecutionPlanError::MissingRoleMapping);
    }

    Ok(())
}

fn rotate_fallback_chain(
    chain: super::FallbackChain,
    rotation: Option<&crate::RouterRotationSnapshot>,
) -> Result<(super::FallbackChain, Option<crate::RouterRotationDecision>), super::ExecutionPlanError> {
    let Some(snapshot) = rotation else {
        return Ok((chain, None));
    };
    if !snapshot.policy().rotates_role(crate::Role::Fallback) || chain.steps().is_empty() {
        return Ok((chain, None));
    }

    let labels = chain
        .steps()
        .iter()
        .map(|step| alloc::string::String::from(step.model_label()))
        .collect::<alloc::vec::Vec<_>>();
    let decision = crate::rotate_eligible_candidates(crate::Role::Fallback, &labels, snapshot)
        .map_err(|error| execution_error_from_rotation(&error))?;
    let rotated_chain = rotate_chain_to_label(&chain, decision.selected_model())?;
    Ok((rotated_chain, Some(decision)))
}

fn rotate_chain_to_label(
    chain: &super::FallbackChain,
    selected_label: &str,
) -> Result<super::FallbackChain, super::ExecutionPlanError> {
    let Some(start_index) = chain.steps().iter().position(|step| step.model_label() == selected_label) else {
        return Err(super::ExecutionPlanError::FallbackModelNotEligible);
    };
    let mut steps = alloc::vec::Vec::with_capacity(chain.steps().len());
    steps.extend(chain.steps()[start_index..].iter().cloned());
    steps.extend(chain.steps()[..start_index].iter().cloned());
    Ok(super::FallbackChain { steps })
}

fn execution_error_from_rotation(error: &crate::RouterRotationError) -> super::ExecutionPlanError {
    match error {
        crate::RouterRotationError::StaleCursor
        | crate::RouterRotationError::NoEligibleCandidates
        | crate::RouterRotationError::Contract(_)
        | crate::RouterRotationError::ReceiptLeak(_) => super::ExecutionPlanError::FallbackModelNotEligible,
    }
}

fn build_role_steps(
    selected: &crate::SelectedPlan,
    catalog: &[crate::CatalogEntry],
    budget: &crate::Budget,
    roles: &[crate::Role],
) -> Result<(alloc::vec::Vec<super::RoleStep>, alloc::vec::Vec<alloc::string::String>), super::ExecutionPlanError> {
    assert!(!roles.is_empty(), "role mapping must exist");
    assert!(selected.bounds().max_candidates() > crate::limit::ZERO_COUNT, "candidate cap must be positive");
    assert!(selected.bounds().max_parallelism() > crate::limit::ZERO_COUNT, "parallelism cap must be positive");
    let mut role_steps = alloc::vec::Vec::with_capacity(roles.len());
    let mut route_labels = alloc::vec::Vec::with_capacity(roles.len());
    for (index, role) in roles.iter().copied().enumerate() {
        let selected_label = match selected.selected_models().get(index) {
            Some(label) => label.as_str(),
            None => return Err(super::ExecutionPlanError::MissingRoleMapping),
        };
        let candidate_labels = super::super::plan::collect_role_candidates(super::super::plan::RoleCandidateRequest {
            selected,
            selected_label,
            role,
            catalog,
            budget,
        })?;
        let quorum = selected.bounds().min_success_quorum();
        let Ok(quorum_index) = usize::try_from(quorum) else {
            return Err(super::ExecutionPlanError::ImpossibleQuorum);
        };
        if quorum_index > candidate_labels.len() {
            return Err(super::ExecutionPlanError::ImpossibleQuorum);
        }
        route_labels.extend(candidate_labels.iter().cloned());
        role_steps.push(super::RoleStep {
            role,
            candidate_labels,
            parallelism_cap: selected.bounds().max_parallelism(),
            quorum,
        });
    }

    Ok((role_steps, route_labels))
}

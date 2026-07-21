const SINGLE_ROLES: &[crate::Role] = &[crate::Role::Primary];
const CONFIDENCE_ROLES: &[crate::Role] = &[crate::Role::Primary, crate::Role::Escalation];
const RATINGS_ROLES: &[crate::Role] = &[crate::Role::Primary, crate::Role::Rater];
const REMOM_ROLES: &[crate::Role] = &[crate::Role::Worker, crate::Role::Synthesizer];
const FUSION_ROLES: &[crate::Role] = &[crate::Role::Judge, crate::Role::Finalizer];
const WORKFLOW_ROLES: &[crate::Role] = &[crate::Role::Worker, crate::Role::Finalizer];

pub(super) fn roles_for_family(family: crate::RouteFamily) -> &'static [crate::Role] {
    match family {
        crate::RouteFamily::Single => SINGLE_ROLES,
        crate::RouteFamily::Confidence => CONFIDENCE_ROLES,
        crate::RouteFamily::Ratings => RATINGS_ROLES,
        crate::RouteFamily::Remom => REMOM_ROLES,
        crate::RouteFamily::Fusion => FUSION_ROLES,
        crate::RouteFamily::Workflow => WORKFLOW_ROLES,
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RoleCandidateRequest<'a> {
    pub(super) selected: &'a crate::SelectedPlan,
    pub(super) selected_label: &'a str,
    pub(super) role: crate::Role,
    pub(super) catalog: &'a [crate::CatalogEntry],
    pub(super) budget: &'a crate::Budget,
}

pub(super) fn collect_role_candidates(
    request: RoleCandidateRequest<'_>,
) -> Result<alloc::vec::Vec<alloc::string::String>, crate::ExecutionPlanError> {
    assert!(
        request.selected.bounds().max_candidates() > 0,
        "candidate cap must be positive"
    );
    assert!(
        request.budget.max_candidates() > 0,
        "budget cap must be positive"
    );
    let mut eligible_labels = alloc::vec::Vec::with_capacity(request.catalog.len());
    for entry in request.catalog {
        if entry.role() != request.role {
            continue;
        }
        if !entry_allowed(entry, request.budget, request.selected.privacy()) {
            continue;
        }
        if eligible_labels.iter().any(|label| *label == entry.label()) {
            continue;
        }
        eligible_labels.push(entry.label());
    }

    if !eligible_labels.contains(&request.selected_label) {
        return Err(crate::ExecutionPlanError::MissingRoleMapping);
    }

    let Ok(candidate_cap) = usize::try_from(request.selected.bounds().max_candidates()) else {
        return Err(crate::ExecutionPlanError::ImpossibleQuorum);
    };
    let mut candidate_labels = alloc::vec::Vec::with_capacity(candidate_cap);
    candidate_labels.push(alloc::string::String::from(request.selected_label));
    for label in eligible_labels {
        if candidate_labels.len() >= candidate_cap {
            break;
        }
        if label != request.selected_label {
            candidate_labels.push(alloc::string::String::from(label));
        }
    }

    Ok(candidate_labels)
}

pub(super) fn build_fallback_chain(
    selected: &crate::SelectedPlan,
    catalog: &[crate::CatalogEntry],
    budget: &crate::Budget,
    route_labels: &[alloc::string::String],
) -> Result<crate::FallbackChain, crate::ExecutionPlanError> {
    assert!(
        selected.bounds().max_candidates() > 0,
        "candidate cap must be positive"
    );
    assert!(budget.max_candidates() > 0, "budget cap must be positive");
    if selected.family() == crate::RouteFamily::Single
        || selected.bounds().fallback_policy() == crate::FallbackPolicy::Deny
    {
        return Ok(crate::FallbackChain {
            steps: alloc::vec::Vec::new(),
        });
    }

    if selected.fallback_eligibility() == crate::FallbackEligibility::Ineligible {
        return Err(crate::ExecutionPlanError::FallbackModelNotEligible);
    }

    let mut steps = alloc::vec::Vec::with_capacity(catalog.len());
    let mut fallback_labels = alloc::vec::Vec::with_capacity(catalog.len());
    let Ok(candidate_cap) = usize::try_from(selected.bounds().max_candidates()) else {
        return Err(crate::ExecutionPlanError::ImpossibleQuorum);
    };

    for entry in catalog {
        if entry.role() != crate::Role::Fallback {
            continue;
        }

        if !entry_allowed(entry, budget, selected.privacy()) {
            continue;
        }

        if slice_contains_str(route_labels, entry.label())
            || slice_contains_str(&fallback_labels, entry.label())
        {
            return Err(crate::ExecutionPlanError::CyclicFallback);
        }

        if steps.len() >= candidate_cap {
            continue;
        }

        fallback_labels.push(alloc::string::String::from(entry.label()));
        steps.push(crate::FallbackStep {
            role: crate::Role::Fallback,
            model_label: alloc::string::String::from(entry.label()),
            backend: alloc::string::String::from(entry.backend()),
            local: entry.is_local(),
        });
    }

    if steps.is_empty() {
        return Err(crate::ExecutionPlanError::FallbackModelNotEligible);
    }

    Ok(crate::FallbackChain { steps })
}

fn slice_contains_str(labels: &[alloc::string::String], target: &str) -> bool {
    labels.iter().any(|label| label.as_str() == target)
}

fn entry_allowed(
    entry: &crate::CatalogEntry,
    budget: &crate::Budget,
    privacy: crate::PrivacyBand,
) -> bool {
    budget.allows_backend(entry.backend())
        && (privacy == crate::PrivacyBand::RemoteAllowed || entry.is_local())
}

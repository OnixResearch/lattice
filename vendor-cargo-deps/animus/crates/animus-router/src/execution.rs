//! Pure router execution-plan materialization.

use alloc::string::String;
use alloc::vec::Vec;

const ZERO_COUNT: u32 = 0;

const SINGLE_ROLES: &[crate::Role] = &[crate::Role::Primary];
const CONFIDENCE_ROLES: &[crate::Role] = &[crate::Role::Primary, crate::Role::Escalation];
const RATINGS_ROLES: &[crate::Role] = &[crate::Role::Primary, crate::Role::Rater];
const REMOM_ROLES: &[crate::Role] = &[crate::Role::Worker, crate::Role::Synthesizer];
const FUSION_ROLES: &[crate::Role] = &[crate::Role::Judge, crate::Role::Finalizer];
const WORKFLOW_ROLES: &[crate::Role] = &[crate::Role::Worker, crate::Role::Finalizer];

/// Materialized router execution plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterExecutionPlan {
    recipe_id: String,
    family: crate::RouteFamily,
    selected_models: Vec<String>,
    output_contract: crate::OutputContract,
    bounds: crate::Bounds,
    privacy: crate::PrivacyBand,
    fallback_eligibility: crate::FallbackEligibility,
    role_steps: Vec<RoleStep>,
    remaining_synthesis_attempts: u32,
    remaining_repair_attempts: u32,
    fallback_chain: FallbackChain,
}

impl RouterExecutionPlan {
    /// Returns the selected recipe identifier.
    #[must_use]
    pub fn recipe_id(&self) -> &str {
        self.recipe_id.as_str()
    }

    /// Returns the route family.
    #[must_use]
    pub const fn family(&self) -> crate::RouteFamily {
        self.family
    }

    /// Returns selected safe model labels.
    #[must_use]
    pub fn selected_models(&self) -> &[String] {
        self.selected_models.as_slice()
    }

    /// Returns the final output contract.
    #[must_use]
    pub const fn output_contract(&self) -> crate::OutputContract {
        self.output_contract
    }

    /// Returns the validated execution bounds.
    #[must_use]
    pub const fn bounds(&self) -> &crate::Bounds {
        &self.bounds
    }

    /// Returns the projected privacy band used for step eligibility.
    #[must_use]
    pub const fn privacy(&self) -> crate::PrivacyBand {
        self.privacy
    }

    /// Returns the projected fallback eligibility band.
    #[must_use]
    pub const fn fallback_eligibility(&self) -> crate::FallbackEligibility {
        self.fallback_eligibility
    }

    /// Returns ordered role steps.
    #[must_use]
    pub fn role_steps(&self) -> &[RoleStep] {
        self.role_steps.as_slice()
    }

    /// Returns remaining synthesis attempts.
    #[must_use]
    pub const fn remaining_synthesis_attempts(&self) -> u32 {
        self.remaining_synthesis_attempts
    }

    /// Returns remaining repair attempts.
    #[must_use]
    pub const fn remaining_repair_attempts(&self) -> u32 {
        self.remaining_repair_attempts
    }

    /// Returns the ordered fallback chain.
    #[must_use]
    pub fn fallback_chain(&self) -> &FallbackChain {
        &self.fallback_chain
    }
}

/// One ordered role step in a materialized router execution plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleStep {
    role: crate::Role,
    candidate_labels: Vec<String>,
    parallelism_cap: u32,
    quorum: u32,
}

impl RoleStep {
    /// Returns the role for this step.
    #[must_use]
    pub const fn role(&self) -> crate::Role {
        self.role
    }

    /// Returns the ordered candidate labels for this step.
    #[must_use]
    pub fn candidate_labels(&self) -> &[String] {
        self.candidate_labels.as_slice()
    }

    /// Returns the parallelism cap for this step.
    #[must_use]
    pub const fn parallelism_cap(&self) -> u32 {
        self.parallelism_cap
    }

    /// Returns the quorum threshold for this step.
    #[must_use]
    pub const fn quorum(&self) -> u32 {
        self.quorum
    }
}

/// Ordered fallback chain materialized from the catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackChain {
    steps: Vec<FallbackStep>,
}

impl FallbackChain {
    /// Returns the ordered fallback steps.
    #[must_use]
    pub fn steps(&self) -> &[FallbackStep] {
        self.steps.as_slice()
    }
}

/// One fallback step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackStep {
    role: crate::Role,
    model_label: String,
    backend: String,
    local: bool,
}

impl FallbackStep {
    /// Returns the fallback role.
    #[must_use]
    pub const fn role(&self) -> crate::Role {
        self.role
    }

    /// Returns the fallback model label.
    #[must_use]
    pub fn model_label(&self) -> &str {
        self.model_label.as_str()
    }

    /// Returns the fallback backend label.
    #[must_use]
    pub fn backend(&self) -> &str {
        self.backend.as_str()
    }

    /// Returns whether the fallback model is local-only.
    #[must_use]
    pub const fn is_local(&self) -> bool {
        self.local
    }
}

/// Typed router execution-plan denial.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionPlanError {
    /// The quorum threshold was zero.
    ZeroQuorum,
    /// The quorum threshold exceeded the available candidate labels.
    ImpossibleQuorum,
    /// Collaborative repair budget was not available.
    UnboundedRepair,
    /// A recipe role could not be mapped to a selected model step.
    MissingRoleMapping,
    /// The fallback chain revisited a model already used in the route.
    CyclicFallback,
    /// No fallback model satisfied backend or privacy constraints.
    FallbackModelNotEligible,
}

/// Materializes a deterministic router execution plan.
// r[impl agent_harness_runtime.model_router.execution_plan]
pub fn materialize_execution_plan(
    selected: &crate::SelectedPlan,
    catalog: &[crate::CatalogEntry],
    budget: &crate::Budget,
) -> Result<RouterExecutionPlan, ExecutionPlanError> {
    let roles = roles_for_family(selected.family());

    if selected.bounds().min_success_quorum() == ZERO_COUNT {
        return Err(ExecutionPlanError::ZeroQuorum);
    }

    if selected.family() != crate::RouteFamily::Single
        && selected.bounds().max_repair_attempts() == ZERO_COUNT
    {
        return Err(ExecutionPlanError::UnboundedRepair);
    }

    if selected.selected_models().len() != roles.len() {
        return Err(ExecutionPlanError::MissingRoleMapping);
    }

    let mut role_steps = Vec::with_capacity(roles.len());
    let mut route_labels = Vec::new();
    for (index, role) in roles.iter().copied().enumerate() {
        let selected_label = match selected.selected_models().get(index) {
            Some(label) => label.as_str(),
            None => return Err(ExecutionPlanError::MissingRoleMapping),
        };
        let candidate_labels = collect_role_candidates(
            selected,
            selected_label,
            role,
            catalog,
            budget,
        )?;
        let quorum = selected.bounds().min_success_quorum();
        if quorum as usize > candidate_labels.len() {
            return Err(ExecutionPlanError::ImpossibleQuorum);
        }
        route_labels.extend(candidate_labels.iter().cloned());
        role_steps.push(RoleStep {
            role,
            candidate_labels,
            parallelism_cap: selected.bounds().max_parallelism(),
            quorum,
        });
    }

    let fallback_chain = build_fallback_chain(selected, catalog, budget, &route_labels)?;

    Ok(RouterExecutionPlan {
        recipe_id: String::from(selected.recipe_id()),
        family: selected.family(),
        selected_models: selected.selected_models().to_vec(),
        output_contract: selected.output_contract(),
        bounds: selected.bounds().clone(),
        privacy: selected.privacy(),
        fallback_eligibility: selected.fallback_eligibility(),
        role_steps,
        remaining_synthesis_attempts: if selected.family() == crate::RouteFamily::Single {
            ZERO_COUNT
        } else {
            selected.bounds().max_synthesis_attempts()
        },
        remaining_repair_attempts: if selected.family() == crate::RouteFamily::Single {
            ZERO_COUNT
        } else {
            selected.bounds().max_repair_attempts()
        },
        fallback_chain,
    })
}

fn roles_for_family(family: crate::RouteFamily) -> &'static [crate::Role] {
    match family {
        crate::RouteFamily::Single => SINGLE_ROLES,
        crate::RouteFamily::Confidence => CONFIDENCE_ROLES,
        crate::RouteFamily::Ratings => RATINGS_ROLES,
        crate::RouteFamily::Remom => REMOM_ROLES,
        crate::RouteFamily::Fusion => FUSION_ROLES,
        crate::RouteFamily::Workflow => WORKFLOW_ROLES,
    }
}

fn collect_role_candidates(
    selected: &crate::SelectedPlan,
    selected_label: &str,
    role: crate::Role,
    catalog: &[crate::CatalogEntry],
    budget: &crate::Budget,
) -> Result<Vec<String>, ExecutionPlanError> {
    let mut eligible_labels = Vec::new();
    for entry in catalog {
        if entry.role() != role {
            continue;
        }
        if !entry_allowed(entry, budget, selected.privacy()) {
            continue;
        }
        if eligible_labels.iter().any(|label| label == entry.label()) {
            continue;
        }
        eligible_labels.push(entry.label());
    }

    if !eligible_labels.iter().any(|label| *label == selected_label) {
        return Err(ExecutionPlanError::MissingRoleMapping);
    }

    let candidate_cap = selected.bounds().max_candidates() as usize;
    let mut candidate_labels = Vec::with_capacity(eligible_labels.len().min(candidate_cap));
    candidate_labels.push(String::from(selected_label));
    for label in eligible_labels {
        if candidate_labels.len() >= candidate_cap {
            break;
        }
        if label != selected_label {
            candidate_labels.push(String::from(label));
        }
    }

    Ok(candidate_labels)
}

fn build_fallback_chain(
    selected: &crate::SelectedPlan,
    catalog: &[crate::CatalogEntry],
    budget: &crate::Budget,
    route_labels: &[String],
) -> Result<FallbackChain, ExecutionPlanError> {
    if selected.family() == crate::RouteFamily::Single
        || selected.bounds().fallback_policy() == crate::FallbackPolicy::Deny
    {
        return Ok(FallbackChain { steps: Vec::new() });
    }

    if selected.fallback_eligibility() == crate::FallbackEligibility::Ineligible {
        return Err(ExecutionPlanError::FallbackModelNotEligible);
    }

    let mut steps = Vec::new();
    let mut fallback_labels = Vec::new();
    let candidate_cap = selected.bounds().max_candidates() as usize;

    for entry in catalog {
        if entry.role() != crate::Role::Fallback {
            continue;
        }

        if !entry_allowed(entry, budget, selected.privacy()) {
            continue;
        }

        if route_labels
            .iter()
            .any(|label| label.as_str() == entry.label())
            || fallback_labels.iter().any(|label| label.as_str() == entry.label())
        {
            return Err(ExecutionPlanError::CyclicFallback);
        }

        if steps.len() >= candidate_cap {
            continue;
        }

        fallback_labels.push(String::from(entry.label()));
        steps.push(FallbackStep {
            role: crate::Role::Fallback,
            model_label: String::from(entry.label()),
            backend: String::from(entry.backend()),
            local: entry.is_local(),
        });
    }

    if steps.is_empty() {
        return Err(ExecutionPlanError::FallbackModelNotEligible);
    }

    Ok(FallbackChain { steps })
}

fn entry_allowed(
    entry: &crate::CatalogEntry,
    budget: &crate::Budget,
    privacy: crate::PrivacyBand,
) -> bool {
    budget.allows_backend(entry.backend())
        && (privacy == crate::PrivacyBand::RemoteAllowed || entry.is_local())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn roles_match_route_families() {
        assert_eq!(roles_for_family(crate::RouteFamily::Single), SINGLE_ROLES);
        assert_eq!(roles_for_family(crate::RouteFamily::Confidence), CONFIDENCE_ROLES);
        assert_eq!(roles_for_family(crate::RouteFamily::Ratings), RATINGS_ROLES);
        assert_eq!(roles_for_family(crate::RouteFamily::Remom), REMOM_ROLES);
        assert_eq!(roles_for_family(crate::RouteFamily::Fusion), FUSION_ROLES);
        assert_eq!(roles_for_family(crate::RouteFamily::Workflow), WORKFLOW_ROLES);
    }
}

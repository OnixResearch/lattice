fn validate_catalog_entry(
    draft: &RoutingModelCatalogEntryDraft<'_>,
) -> Result<RoutingModelCatalogEntry, ModelRoutingError> {
    validate_item_count(
        "routing_policy_tags",
        draft.policy_tags.len(),
        MAX_POLICY_TAGS,
    )?;
    let mut policy_tags = Vec::with_capacity(draft.policy_tags.len());
    for tag in draft.policy_tags {
        policy_tags.push(normalize(POLICY_TAG_FIELD, tag)?);
    }
    Ok(RoutingModelCatalogEntry {
        role_label: normalize(ROLE_LABEL_FIELD, draft.role_label)?,
        model_label: normalize(MODEL_LABEL_FIELD, draft.model_label)?,
        capability: draft.capability,
        price_units: draft.price_units,
        availability: draft.availability,
        policy_tags,
    })
}

/// Borrowed current budget usage before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingUsageDraft {
    /// Current cost units.
    pub current_cost: u32,
    /// Soft budget units.
    pub soft_budget: u32,
    /// Hard budget units.
    pub hard_budget: u32,
}

/// Validated current usage/cost summary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingUsage {
    current_cost: u32,
    soft_budget: u32,
    hard_budget: u32,
}

impl RoutingUsage {
    /// Validates budget ordering.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRoutingError`] when hard budget is below soft budget.
    pub const fn validate(draft: RoutingUsageDraft) -> Result<Self, ModelRoutingError> {
        if draft.hard_budget < draft.soft_budget {
            return Err(ModelRoutingError::Policy(
                ModelRoutingPolicyIssue::UnorderedBudgets,
            ));
        }
        Ok(Self {
            current_cost: draft.current_cost,
            soft_budget: draft.soft_budget,
            hard_budget: draft.hard_budget,
        })
    }

    /// Returns current cost units.
    #[must_use]
    pub const fn current_cost_units(self) -> u32 {
        self.current_cost
    }

    /// Returns soft budget units.
    #[must_use]
    pub const fn soft_budget_units(self) -> u32 {
        self.soft_budget
    }

    /// Returns hard budget units.
    #[must_use]
    pub const fn hard_budget_units(self) -> u32 {
        self.hard_budget
    }
}

/// Borrowed model routing input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRoutingInput<'a> {
    /// Validated routing policy.
    pub policy: &'a ModelRoutingPolicy,
    /// Validated task signals.
    pub signals: &'a RoutingTaskSignals,
    /// Validated injected catalog projection.
    pub catalog: &'a RoutingModelCatalog,
    /// Validated usage/cost summary.
    pub usage: RoutingUsage,
}

/// Pure model routing decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRoutingDecision {
    status: RoutingDecisionStatus,
    selected_role_label: Option<String>,
    selected_model_label: Option<String>,
    score_bucket: RoutingScoreBucket,
    budget_status: RoutingBudgetStatus,
    candidate_count: usize,
    fallback_class: Option<RoutingFallbackClass>,
    policy_hash: crate::artifact::Hash,
    provider_request_sent: bool,
}

impl ModelRoutingDecision {
    /// Returns decision status.
    #[must_use]
    pub const fn status(&self) -> RoutingDecisionStatus {
        self.status
    }

    /// Returns selected role label, if any.
    #[must_use]
    pub fn selected_role_label(&self) -> Option<&str> {
        self.selected_role_label.as_deref()
    }

    /// Returns selected model label, if any.
    #[must_use]
    pub fn selected_model_label(&self) -> Option<&str> {
        self.selected_model_label.as_deref()
    }

    /// Returns score bucket.
    #[must_use]
    pub const fn score_bucket(&self) -> RoutingScoreBucket {
        self.score_bucket
    }

    /// Returns budget status.
    #[must_use]
    pub const fn budget_status(&self) -> RoutingBudgetStatus {
        self.budget_status
    }

    /// Returns number of catalog candidates observed.
    #[must_use]
    pub const fn candidate_count(&self) -> usize {
        self.candidate_count
    }

    /// Returns fallback or reason class.
    #[must_use]
    pub const fn fallback_class(&self) -> Option<RoutingFallbackClass> {
        self.fallback_class
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }

    /// Returns whether routing sent a provider request.
    #[must_use]
    pub const fn provider_request_sent(&self) -> bool {
        self.provider_request_sent
    }
}

/// Routes to a safe model role/model without provider side effects.
///
/// # Errors
///
/// Returns [`ModelRoutingError`] when routing policy references become inconsistent.
// r[impl agent_harness_runtime.cost_model_routing.decision]
// r[impl agent_harness_runtime.cost_model_routing.budget]
pub fn route_model(
    input: ModelRoutingInput<'_>,
) -> Result<ModelRoutingDecision, ModelRoutingError> {
    let score = complexity_score(input.signals, input.policy.weights());
    let score_bucket = bucket_for_score(score, input.policy.thresholds());
    let budget_status = budget_status(input.usage);
    let selected = select_target_role(input.policy, input.signals, score)?;
    let routed = apply_budget(
        input.policy,
        selected.role,
        selected.fallback,
        budget_status,
    );
    if let RoutedRole::Denied { fallback_class } = routed {
        return Ok(denied_decision(
            input,
            score_bucket,
            budget_status,
            fallback_class,
        ));
    }
    let RoutedRole::Selected {
        role,
        fallback_class,
    } = routed
    else {
        unreachable!("routed role must be selected or denied");
    };
    let selection = select_catalog_model(input.policy, input.catalog, role.role_label());
    match selection {
        CandidateSelection::Selected(model) => Ok(selected_decision(SelectedDecisionInput {
            routing: input,
            score_bucket,
            budget_status,
            role,
            model,
            fallback_class,
        })),
        CandidateSelection::Denied { fallback_class } => Ok(denied_decision(
            input,
            score_bucket,
            budget_status,
            fallback_class,
        )),
    }
}

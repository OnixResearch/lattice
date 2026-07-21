
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SelectedRole<'a> {
    role: &'a RoutingRole,
    fallback: RoutingFallbackClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RoutedRole<'a> {
    Selected {
        role: &'a RoutingRole,
        fallback_class: RoutingFallbackClass,
    },
    Denied {
        fallback_class: RoutingFallbackClass,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateSelection<'a> {
    Selected(&'a RoutingModelCatalogEntry),
    Denied {
        fallback_class: RoutingFallbackClass,
    },
}

fn complexity_score(signals: &RoutingTaskSignals, weights: RoutingComplexityWeights) -> u32 {
    u32::from(signals.token_bucket()) * u32::from(weights.token_bucket_weight())
        + u32::from(signals.file_bucket()) * u32::from(weights.file_bucket_weight())
        + u32::from(signals.tool_bucket()) * u32::from(weights.tool_bucket_weight())
        + u32::from(signals.risk_bucket()) * u32::from(weights.risk_bucket_weight())
}

fn bucket_for_score(score: u32, thresholds: RoutingThresholds) -> RoutingScoreBucket {
    if score <= u32::from(thresholds.simple_max_score()) {
        return RoutingScoreBucket::Simple;
    }
    if score <= u32::from(thresholds.medium_max_score()) {
        return RoutingScoreBucket::Medium;
    }
    RoutingScoreBucket::Complex
}

const fn budget_status(usage: RoutingUsage) -> RoutingBudgetStatus {
    if usage.current_cost_units() >= usage.hard_budget_units() {
        return RoutingBudgetStatus::HardLimit;
    }
    if usage.current_cost_units() >= usage.soft_budget_units() {
        return RoutingBudgetStatus::SoftPressure;
    }
    RoutingBudgetStatus::WithinBudget
}

fn select_target_role<'a>(
    policy: &'a ModelRoutingPolicy,
    signals: &RoutingTaskSignals,
    score: u32,
) -> Result<SelectedRole<'a>, ModelRoutingError> {
    if let Some(role_hint) = signals.user_hint_role() {
        if let Some(role) = role_by_label(policy, role_hint) {
            return Ok(SelectedRole {
                role,
                fallback: RoutingFallbackClass::ExplicitUserHint,
            });
        }
    }
    let role = role_for_score(policy, score).ok_or(ModelRoutingError::Policy(
        ModelRoutingPolicyIssue::EmptyRoles,
    ))?;
    Ok(SelectedRole {
        role,
        fallback: RoutingFallbackClass::ComplexityScore,
    })
}

fn role_for_score(policy: &ModelRoutingPolicy, score: u32) -> Option<&RoutingRole> {
    let mut selected = policy.roles().first()?;
    for role in policy.roles() {
        if score >= u32::from(role.minimum_score()) {
            selected = role;
        }
    }
    Some(selected)
}

fn role_by_label<'a>(policy: &'a ModelRoutingPolicy, label: &str) -> Option<&'a RoutingRole> {
    policy
        .roles()
        .iter()
        .find(|role| role.role_label() == label)
}

fn apply_budget<'a>(
    policy: &'a ModelRoutingPolicy,
    role: &'a RoutingRole,
    fallback: RoutingFallbackClass,
    status: RoutingBudgetStatus,
) -> RoutedRole<'a> {
    match (policy.budget_behavior(), status) {
        (RoutingBudgetBehavior::PreferCheaperThenDeny, RoutingBudgetStatus::WithinBudget) => {
            RoutedRole::Selected {
                role,
                fallback_class: fallback,
            }
        }
        (RoutingBudgetBehavior::PreferCheaperThenDeny, RoutingBudgetStatus::SoftPressure) => {
            downgraded_role(policy, role).map_or(
                RoutedRole::Selected {
                    role,
                    fallback_class: fallback,
                },
                |downgrade| RoutedRole::Selected {
                    role: downgrade,
                    fallback_class: RoutingFallbackClass::SoftBudgetPressure,
                },
            )
        }
        (RoutingBudgetBehavior::PreferCheaperThenDeny, RoutingBudgetStatus::HardLimit) => {
            downgraded_role(policy, role).map_or(
                RoutedRole::Denied {
                    fallback_class: RoutingFallbackClass::HardBudgetDenied,
                },
                |downgrade| RoutedRole::Selected {
                    role: downgrade,
                    fallback_class: RoutingFallbackClass::HardBudgetDowngrade,
                },
            )
        }
    }
}

fn downgraded_role<'a>(
    policy: &'a ModelRoutingPolicy,
    role: &'a RoutingRole,
) -> Option<&'a RoutingRole> {
    role.downgrade_role_label()
        .and_then(|label| role_by_label(policy, label))
}

fn select_catalog_model<'a>(
    policy: &ModelRoutingPolicy,
    catalog: &'a RoutingModelCatalog,
    role_label: &str,
) -> CandidateSelection<'a> {
    let mut role_seen = false;
    let mut unavailable_seen = false;
    let mut missing_price_seen = false;
    let mut candidates: Vec<&RoutingModelCatalogEntry> = Vec::new();
    for entry in catalog.entries() {
        if entry.role_label() != role_label {
            continue;
        }
        role_seen = true;
        if entry.availability() == RoutingModelAvailability::Unavailable {
            unavailable_seen = true;
            continue;
        }
        if entry.price_units().is_none()
            && policy.missing_price_behavior() == RoutingMissingPriceBehavior::Deny
        {
            missing_price_seen = true;
            continue;
        }
        candidates.push(entry);
    }
    if candidates.is_empty() {
        return CandidateSelection::Denied {
            fallback_class: denied_candidate_reason(
                role_seen,
                unavailable_seen,
                missing_price_seen,
            ),
        };
    }
    candidates.sort_by_key(|candidate| (price_sort_key(candidate), candidate.model_label()));
    CandidateSelection::Selected(candidates[ZERO_ROUTING_COUNT])
}

const fn denied_candidate_reason(
    role_seen: bool,
    unavailable_seen: bool,
    missing_price_seen: bool,
) -> RoutingFallbackClass {
    if missing_price_seen {
        return RoutingFallbackClass::MissingPrice;
    }
    if role_seen && unavailable_seen {
        return RoutingFallbackClass::UnavailableModel;
    }
    RoutingFallbackClass::NoEligibleModel
}

const fn price_sort_key(entry: &RoutingModelCatalogEntry) -> u32 {
    match entry.price_units() {
        Some(price) => price,
        None => UNKNOWN_PRICE_SORT_UNITS,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SelectedDecisionInput<'a> {
    routing: ModelRoutingInput<'a>,
    score_bucket: RoutingScoreBucket,
    budget_status: RoutingBudgetStatus,
    role: &'a RoutingRole,
    model: &'a RoutingModelCatalogEntry,
    fallback_class: RoutingFallbackClass,
}

fn selected_decision(input: SelectedDecisionInput<'_>) -> ModelRoutingDecision {
    ModelRoutingDecision {
        status: RoutingDecisionStatus::Selected,
        selected_role_label: Some(String::from(input.role.role_label())),
        selected_model_label: Some(String::from(input.model.model_label())),
        score_bucket: input.score_bucket,
        budget_status: input.budget_status,
        candidate_count: input.routing.catalog.entries().len(),
        fallback_class: Some(input.fallback_class),
        policy_hash: input.routing.policy.policy_hash().clone(),
        provider_request_sent: false,
    }
}

fn denied_decision(
    input: ModelRoutingInput<'_>,
    score_bucket: RoutingScoreBucket,
    budget_status: RoutingBudgetStatus,
    fallback_class: RoutingFallbackClass,
) -> ModelRoutingDecision {
    ModelRoutingDecision {
        status: RoutingDecisionStatus::Denied,
        selected_role_label: None,
        selected_model_label: None,
        score_bucket,
        budget_status,
        candidate_count: input.catalog.entries().len(),
        fallback_class: Some(fallback_class),
        policy_hash: input.policy.policy_hash().clone(),
        provider_request_sent: false,
    }
}

/// Borrowed routing receipt fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRoutingReceiptDraft<'a> {
    /// Pure routing decision.
    pub decision: &'a ModelRoutingDecision,
    /// Raw prompt text, which is forbidden.
    pub raw_prompt: Option<&'a str>,
    /// Provider payload, which is forbidden.
    pub provider_payload: Option<&'a str>,
    /// Credential-looking value, which is forbidden.
    pub credential: Option<&'a str>,
    /// Full pricing store dump, which is forbidden.
    pub pricing_store_dump: Option<&'a str>,
    /// Hidden reasoning text, which is forbidden.
    pub hidden_reasoning: Option<&'a str>,
}

/// Safe routing receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRoutingReceipt {
    decision: ModelRoutingDecision,
    receipt_hash: crate::artifact::Hash,
}

impl ModelRoutingReceipt {
    /// Validates a safe routing receipt and computes its BLAKE3 receipt hash.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRoutingError`] when forbidden receipt fields are present.
    // r[impl agent_harness_runtime.cost_model_routing.receipts]
    pub fn validate(draft: &ModelRoutingReceiptDraft<'_>) -> Result<Self, ModelRoutingError> {
        reject_receipt_leaks(draft)?;
        Ok(Self {
            decision: draft.decision.clone(),
            receipt_hash: routing_receipt_hash(draft.decision),
        })
    }

    /// Returns the pure routing decision.
    #[must_use]
    pub const fn decision(&self) -> &ModelRoutingDecision {
        &self.decision
    }

    /// Returns deterministic BLAKE3 receipt hash.
    #[must_use]
    pub const fn receipt_hash(&self) -> &crate::artifact::Hash {
        &self.receipt_hash
    }
}


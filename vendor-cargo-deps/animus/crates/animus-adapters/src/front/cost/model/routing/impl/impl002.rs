
/// Borrowed complexity weights before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingComplexityWeightsDraft {
    /// Weight applied to token complexity bucket.
    pub token: i16,
    /// Weight applied to file complexity bucket.
    pub file: i16,
    /// Weight applied to tool complexity bucket.
    pub tool: i16,
    /// Weight applied to risk complexity bucket.
    pub risk: i16,
}

/// Validated complexity weights.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingComplexityWeights {
    token: u16,
    file: u16,
    tool: u16,
    risk: u16,
}

impl RoutingComplexityWeights {
    /// Returns token bucket weight.
    #[must_use]
    pub const fn token_bucket_weight(self) -> u16 {
        self.token
    }

    /// Returns file bucket weight.
    #[must_use]
    pub const fn file_bucket_weight(self) -> u16 {
        self.file
    }

    /// Returns tool bucket weight.
    #[must_use]
    pub const fn tool_bucket_weight(self) -> u16 {
        self.tool
    }

    /// Returns risk bucket weight.
    #[must_use]
    pub const fn risk_bucket_weight(self) -> u16 {
        self.risk
    }
}

/// Borrowed routing role policy before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingRoleDraft<'a> {
    /// Safe role label.
    pub role_label: &'a str,
    /// Minimum score that can select the role by complexity.
    pub minimum_score: u16,
    /// Optional cheaper downgrade role label.
    pub downgrade_role_label: Option<&'a str>,
}

/// Validated routing role policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingRole {
    role_label: String,
    minimum_score: u16,
    downgrade_role_label: Option<String>,
}

impl RoutingRole {
    /// Returns the safe role label.
    #[must_use]
    pub fn role_label(&self) -> &str {
        self.role_label.as_str()
    }

    /// Returns the minimum complexity score for this role.
    #[must_use]
    pub const fn minimum_score(&self) -> u16 {
        self.minimum_score
    }

    /// Returns the optional downgrade role label.
    #[must_use]
    pub fn downgrade_role_label(&self) -> Option<&str> {
        self.downgrade_role_label.as_deref()
    }
}

/// Borrowed routing policy before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRoutingPolicyDraft<'a> {
    /// Non-zero schema version.
    pub schema_version: u16,
    /// Role policies.
    pub roles: &'a [RoutingRoleDraft<'a>],
    /// Complexity score thresholds.
    pub thresholds: RoutingThresholdsDraft,
    /// Complexity scoring weights.
    pub weights: RoutingComplexityWeightsDraft,
    /// Missing-price behavior.
    pub missing_price_behavior: Option<RoutingMissingPriceBehavior>,
    /// Budget behavior.
    pub budget_behavior: RoutingBudgetBehavior,
    /// Lowercase BLAKE3 policy hash.
    pub policy_hash: &'a str,
}

/// Validated model routing policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRoutingPolicy {
    schema_version: NonZeroU16,
    roles: Vec<RoutingRole>,
    thresholds: RoutingThresholds,
    weights: RoutingComplexityWeights,
    missing_price_behavior: RoutingMissingPriceBehavior,
    budget_behavior: RoutingBudgetBehavior,
    policy_hash: crate::artifact::Hash,
}

impl ModelRoutingPolicy {
    /// Validates routing policy contracts or checked fixtures.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRoutingError`] when roles, weights, thresholds, downgrade targets,
    /// missing-price behavior, or policy hash are invalid.
    // r[impl agent_harness_runtime.cost_model_routing.policy]
    pub fn validate(draft: &ModelRoutingPolicyDraft<'_>) -> Result<Self, ModelRoutingError> {
        let schema_version = NonZeroU16::new(draft.schema_version).ok_or(
            ModelRoutingError::Policy(ModelRoutingPolicyIssue::ZeroSchemaVersion),
        )?;
        validate_item_count("routing_roles", draft.roles.len(), MAX_ROUTING_ROLES)?;
        if draft.roles.is_empty() {
            return Err(ModelRoutingError::Policy(
                ModelRoutingPolicyIssue::EmptyRoles,
            ));
        }
        let thresholds = validate_thresholds(draft.thresholds)?;
        let weights = validate_weights(draft.weights)?;
        let missing_price_behavior =
            draft
                .missing_price_behavior
                .ok_or(ModelRoutingError::Policy(
                    ModelRoutingPolicyIssue::MissingPriceBehaviorUnspecified,
                ))?;
        let mut roles = validate_roles(draft.roles)?;
        roles.sort_by_key(|role| role.minimum_score);
        Ok(Self {
            schema_version,
            roles,
            thresholds,
            weights,
            missing_price_behavior,
            budget_behavior: draft.budget_behavior,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash)
                .map_err(ModelRoutingError::InvalidHash)?,
        })
    }

    /// Returns the schema version.
    #[must_use]
    pub const fn schema_version(&self) -> NonZeroU16 {
        self.schema_version
    }

    /// Returns role policies.
    #[must_use]
    pub fn roles(&self) -> &[RoutingRole] {
        self.roles.as_slice()
    }

    /// Returns score thresholds.
    #[must_use]
    pub const fn thresholds(&self) -> RoutingThresholds {
        self.thresholds
    }

    /// Returns complexity weights.
    #[must_use]
    pub const fn weights(&self) -> RoutingComplexityWeights {
        self.weights
    }

    /// Returns missing-price behavior.
    #[must_use]
    pub const fn missing_price_behavior(&self) -> RoutingMissingPriceBehavior {
        self.missing_price_behavior
    }

    /// Returns budget behavior.
    #[must_use]
    pub const fn budget_behavior(&self) -> RoutingBudgetBehavior {
        self.budget_behavior
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }
}

fn validate_thresholds(
    draft: RoutingThresholdsDraft,
) -> Result<RoutingThresholds, ModelRoutingError> {
    if draft.simple_max_score >= draft.medium_max_score {
        return Err(ModelRoutingError::Policy(
            ModelRoutingPolicyIssue::UnorderedThresholds,
        ));
    }
    Ok(RoutingThresholds {
        simple_max_score: draft.simple_max_score,
        medium_max_score: draft.medium_max_score,
    })
}

fn validate_weights(
    draft: RoutingComplexityWeightsDraft,
) -> Result<RoutingComplexityWeights, ModelRoutingError> {
    Ok(RoutingComplexityWeights {
        token: validate_weight("token", draft.token)?,
        file: validate_weight("file", draft.file)?,
        tool: validate_weight("tool", draft.tool)?,
        risk: validate_weight("risk", draft.risk)?,
    })
}

fn validate_weight(field: &'static str, value: i16) -> Result<u16, ModelRoutingError> {
    u16::try_from(value)
        .map_err(|_| ModelRoutingError::Policy(ModelRoutingPolicyIssue::NegativeWeight { field }))
}

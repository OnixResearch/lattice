
fn validate_roles(roles: &[RoutingRoleDraft<'_>]) -> Result<Vec<RoutingRole>, ModelRoutingError> {
    let mut validated = Vec::with_capacity(roles.len());
    for draft in roles {
        let role_label = normalize(ROLE_LABEL_FIELD, draft.role_label)?;
        if validated
            .iter()
            .any(|role: &RoutingRole| role.role_label == role_label)
        {
            return Err(ModelRoutingError::Policy(
                ModelRoutingPolicyIssue::DuplicateRole,
            ));
        }
        validated.push(RoutingRole {
            role_label,
            minimum_score: draft.minimum_score,
            downgrade_role_label: normalize_optional(ROLE_LABEL_FIELD, draft.downgrade_role_label)?,
        });
    }
    for role in &validated {
        if let Some(target) = role.downgrade_role_label() {
            if !validated
                .iter()
                .any(|candidate| candidate.role_label() == target)
            {
                return Err(ModelRoutingError::Policy(
                    ModelRoutingPolicyIssue::UnknownDowngradeTarget,
                ));
            }
        }
    }
    Ok(validated)
}

/// Borrowed task signals before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingTaskSignalsDraft<'a> {
    /// Bucketed token complexity signal.
    pub token_bucket: u16,
    /// Bucketed file/reference complexity signal.
    pub file_bucket: u16,
    /// Bucketed tool-use complexity signal.
    pub tool_bucket: u16,
    /// Bucketed risk signal.
    pub risk_bucket: u16,
    /// Optional explicit role hint supplied by the host/user policy layer.
    pub user_hint_role: Option<&'a str>,
}

/// Validated task signals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingTaskSignals {
    token_bucket: u16,
    file_bucket: u16,
    tool_bucket: u16,
    risk_bucket: u16,
    user_hint_role: Option<String>,
}

impl RoutingTaskSignals {
    /// Validates task routing signals.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRoutingError`] when the optional role hint label is unsafe.
    pub fn validate(draft: &RoutingTaskSignalsDraft<'_>) -> Result<Self, ModelRoutingError> {
        Ok(Self {
            token_bucket: draft.token_bucket,
            file_bucket: draft.file_bucket,
            tool_bucket: draft.tool_bucket,
            risk_bucket: draft.risk_bucket,
            user_hint_role: normalize_optional(ROLE_LABEL_FIELD, draft.user_hint_role)?,
        })
    }

    /// Returns token bucket.
    #[must_use]
    pub const fn token_bucket(&self) -> u16 {
        self.token_bucket
    }

    /// Returns file bucket.
    #[must_use]
    pub const fn file_bucket(&self) -> u16 {
        self.file_bucket
    }

    /// Returns tool bucket.
    #[must_use]
    pub const fn tool_bucket(&self) -> u16 {
        self.tool_bucket
    }

    /// Returns risk bucket.
    #[must_use]
    pub const fn risk_bucket(&self) -> u16 {
        self.risk_bucket
    }

    /// Returns optional explicit role hint.
    #[must_use]
    pub fn user_hint_role(&self) -> Option<&str> {
        self.user_hint_role.as_deref()
    }
}

/// Borrowed model catalog entry before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingModelCatalogEntryDraft<'a> {
    /// Safe role label.
    pub role_label: &'a str,
    /// Safe model label.
    pub model_label: &'a str,
    /// Capability class.
    pub capability: RoutingModelCapability,
    /// Optional price units for budget routing.
    pub price_units: Option<u32>,
    /// Availability class.
    pub availability: RoutingModelAvailability,
    /// Safe policy tags.
    pub policy_tags: &'a [&'a str],
}

/// Validated safe model catalog entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingModelCatalogEntry {
    role_label: String,
    model_label: String,
    capability: RoutingModelCapability,
    price_units: Option<u32>,
    availability: RoutingModelAvailability,
    policy_tags: Vec<String>,
}

impl RoutingModelCatalogEntry {
    /// Returns the safe role label.
    #[must_use]
    pub fn role_label(&self) -> &str {
        self.role_label.as_str()
    }

    /// Returns the safe model label.
    #[must_use]
    pub fn model_label(&self) -> &str {
        self.model_label.as_str()
    }

    /// Returns capability class.
    #[must_use]
    pub const fn capability(&self) -> RoutingModelCapability {
        self.capability
    }

    /// Returns optional price units.
    #[must_use]
    pub const fn price_units(&self) -> Option<u32> {
        self.price_units
    }

    /// Returns availability class.
    #[must_use]
    pub const fn availability(&self) -> RoutingModelAvailability {
        self.availability
    }

    /// Returns policy tags.
    #[must_use]
    pub fn policy_tags(&self) -> &[String] {
        self.policy_tags.as_slice()
    }
}

/// Validated injected model catalog projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingModelCatalog {
    entries: Vec<RoutingModelCatalogEntry>,
}

impl RoutingModelCatalog {
    /// Validates injected safe model catalog projections.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRoutingError`] when catalog or tag counts exceed bounds or
    /// labels fail safe boundary validation.
    // r[impl agent_harness_runtime.cost_model_routing.catalog]
    pub fn validate(
        drafts: &[RoutingModelCatalogEntryDraft<'_>],
    ) -> Result<Self, ModelRoutingError> {
        validate_item_count("routing_models", drafts.len(), MAX_ROUTING_MODELS)?;
        let mut entries = Vec::with_capacity(drafts.len());
        for draft in drafts {
            entries.push(validate_catalog_entry(draft)?);
        }
        Ok(Self { entries })
    }

    /// Returns safe model entries.
    #[must_use]
    pub fn entries(&self) -> &[RoutingModelCatalogEntry] {
        self.entries.as_slice()
    }
}

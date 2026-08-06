const MODEL_LABEL_FIELD: crate::FieldName = crate::FieldName::new("router_model_label");
const BACKEND_FIELD: crate::FieldName = crate::FieldName::new("router_backend");

/// Safe catalog projection used by recipe selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogEntry {
    label: alloc::string::String,
    backend: alloc::string::String,
    role: crate::Role,
    local: bool,
}

impl CatalogEntry {
    /// Builds a safe catalog projection.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when labels or roles are invalid.
    pub fn new(draft: CatalogEntryDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            label: crate::normalized_adapter_field(MODEL_LABEL_FIELD, draft.label)?,
            backend: crate::normalized_adapter_field(BACKEND_FIELD, draft.backend)?,
            role: crate::Role::parse(draft.role)?,
            local: draft.local,
        })
    }

    /// Returns safe model label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns safe backend label.
    #[must_use]
    pub fn backend(&self) -> &str {
        self.backend.as_str()
    }

    /// Returns catalog role.
    #[must_use]
    pub const fn role(&self) -> crate::Role {
        self.role
    }

    /// Returns whether this entry satisfies local-only privacy constraints.
    #[must_use]
    pub const fn is_local(&self) -> bool {
        self.local
    }
}

/// Borrowed catalog projection fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogEntryDraft<'a> {
    /// Safe model label.
    pub label: &'a str,
    /// Safe backend label.
    pub backend: &'a str,
    /// Role label.
    pub role: &'a str,
    /// Whether the model satisfies local-only privacy constraints.
    pub local: bool,
}

/// Budget and backend eligibility used by recipe selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Budget {
    max_candidates: u32,
    allowed_backends: alloc::vec::Vec<alloc::string::String>,
}

impl Budget {
    /// Builds router budget state.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when backend labels are invalid.
    pub fn new(max_candidates: u32, allowed_backends: &[&str]) -> Result<Self, crate::AdapterContractError> {
        let allowed_backends = allowed_backends
            .iter()
            .map(|backend| crate::normalized_adapter_field(BACKEND_FIELD, backend))
            .collect::<Result<alloc::vec::Vec<_>, _>>()?;
        Ok(Self {
            max_candidates,
            allowed_backends,
        })
    }

    pub(crate) fn allows_backend(&self, backend: &str) -> bool {
        self.allowed_backends.iter().any(|allowed| allowed == backend)
    }

    /// Returns max candidates.
    #[must_use]
    pub const fn max_candidates(&self) -> u32 {
        self.max_candidates
    }
}

/// Selected recipe and safe model labels produced before provider dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedPlan {
    pub(super) recipe_id: alloc::string::String,
    pub(super) family: crate::RouteFamily,
    pub(super) selected_models: alloc::vec::Vec<alloc::string::String>,
    pub(super) output_contract: crate::OutputContract,
    pub(super) bounds: crate::Bounds,
    pub(super) privacy: crate::PrivacyBand,
    pub(super) fallback_eligibility: crate::FallbackEligibility,
    pub(super) rotation_decisions: alloc::vec::Vec<crate::RouterRotationDecision>,
}

impl SelectedPlan {
    /// Returns selected recipe identifier.
    #[must_use]
    pub fn recipe_id(&self) -> &str {
        self.recipe_id.as_str()
    }

    /// Returns selected route family.
    #[must_use]
    pub const fn family(&self) -> crate::RouteFamily {
        self.family
    }

    /// Returns selected safe model labels.
    #[must_use]
    pub fn selected_models(&self) -> &[alloc::string::String] {
        self.selected_models.as_slice()
    }

    /// Returns final output contract.
    #[must_use]
    pub const fn output_contract(&self) -> crate::OutputContract {
        self.output_contract
    }

    /// Returns execution bounds.
    #[must_use]
    pub const fn bounds(&self) -> &crate::Bounds {
        &self.bounds
    }

    /// Returns the projected privacy band.
    #[must_use]
    pub const fn privacy(&self) -> crate::PrivacyBand {
        self.privacy
    }

    /// Returns the projected fallback eligibility.
    #[must_use]
    pub const fn fallback_eligibility(&self) -> crate::FallbackEligibility {
        self.fallback_eligibility
    }

    /// Returns safe rotation decisions produced during selection.
    #[must_use]
    pub fn rotation_decisions(&self) -> &[crate::RouterRotationDecision] {
        self.rotation_decisions.as_slice()
    }
}

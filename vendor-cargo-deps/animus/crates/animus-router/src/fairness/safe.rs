use alloc::string::ToString;

/// Safe rotation receipt for router fairness evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterRotationReceipt {
    recipe_id: alloc::string::String,
    route_family: crate::RouteFamily,
    role: crate::Role,
    policy: super::RouterRotationPolicy,
    cursor_class: super::RouterRotationCursorClass,
    selected_model: alloc::string::String,
    candidate_count: u32,
    catalog_hash: crate::SafeHash,
    next_cursor_hash: crate::SafeHash,
    redactions: alloc::vec::Vec<crate::Redaction>,
    identity_hash: alloc::string::String,
}

impl RouterRotationReceipt {
    /// Builds a safe rotation receipt and deterministic BLAKE3 identity.
    ///
    /// # Errors
    ///
    /// Returns [`super::RouterRotationError`] when labels, hashes, redactions, or leak checks fail.
    pub fn new(draft: RouterRotationReceiptDraft<'_>) -> Result<Self, super::RouterRotationError> {
        let recipe_id = receipt_label(super::RECIPE_ID_FIELD, draft.recipe_id)?;
        let route_family = crate::RouteFamily::parse(draft.route_family)?;
        let role = crate::Role::parse(draft.role)?;
        let policy = super::RouterRotationPolicy::parse(draft.policy)?;
        let cursor_class = super::RouterRotationCursorClass::parse(draft.cursor_class)?;
        let selected_model = receipt_label(super::MODEL_LABEL_FIELD, draft.selected_model)?;
        let catalog_hash = crate::SafeHash::new(draft.catalog_hash)?;
        let next_cursor_hash =
            parse_receipt_hash(super::NEXT_CURSOR_HASH_FIELD, draft.next_cursor_hash)?;
        let redactions = collect_redactions(draft.redactions)?;
        let candidate_count = draft.candidate_count.to_string();
        let identity_hash = crate::hash_parts(&[
            super::RECEIPT_HASH_SCHEMA,
            recipe_id.as_str(),
            route_family.as_str(),
            role.as_str(),
            policy.as_str(),
            cursor_class.as_str(),
            selected_model.as_str(),
            candidate_count.as_str(),
            catalog_hash.as_str(),
            next_cursor_hash.as_str(),
        ]);

        Ok(Self {
            recipe_id,
            route_family,
            role,
            policy,
            cursor_class,
            selected_model,
            candidate_count: draft.candidate_count,
            catalog_hash,
            next_cursor_hash,
            redactions,
            identity_hash,
        })
    }

    /// Builds a safe receipt from a rotation decision.
    ///
    /// # Errors
    ///
    /// Returns [`super::RouterRotationError`] when receipt metadata fails validation.
    pub fn from_decision(
        recipe_id: &str,
        route_family: crate::RouteFamily,
        decision: &super::RouterRotationDecision,
        redactions: &[&str],
    ) -> Result<Self, super::RouterRotationError> {
        Self::new(RouterRotationReceiptDraft {
            recipe_id,
            route_family: route_family.as_str(),
            role: decision.role().as_str(),
            policy: decision.policy().as_str(),
            cursor_class: decision.cursor_class().as_str(),
            selected_model: decision.selected_model(),
            candidate_count: decision.candidate_count(),
            catalog_hash: decision.catalog_hash(),
            next_cursor_hash: decision.next_cursor_hash(),
            redactions,
        })
    }

    /// Returns recipe identifier.
    #[must_use]
    pub fn recipe_id(&self) -> &str {
        self.recipe_id.as_str()
    }

    /// Returns route family.
    #[must_use]
    pub const fn route_family(&self) -> crate::RouteFamily {
        self.route_family
    }

    /// Returns role.
    #[must_use]
    pub const fn role(&self) -> crate::Role {
        self.role
    }

    /// Returns rotation policy.
    #[must_use]
    pub const fn policy(&self) -> super::RouterRotationPolicy {
        self.policy
    }

    /// Returns cursor class.
    #[must_use]
    pub const fn cursor_class(&self) -> super::RouterRotationCursorClass {
        self.cursor_class
    }

    /// Returns selected safe model label.
    #[must_use]
    pub fn selected_model(&self) -> &str {
        self.selected_model.as_str()
    }

    /// Returns eligible candidate count.
    #[must_use]
    pub const fn candidate_count(&self) -> u32 {
        self.candidate_count
    }

    /// Returns catalog hash.
    #[must_use]
    pub fn catalog_hash(&self) -> &str {
        self.catalog_hash.as_str()
    }

    /// Returns next-cursor hash.
    #[must_use]
    pub fn next_cursor_hash(&self) -> &str {
        self.next_cursor_hash.as_str()
    }

    /// Returns redaction markers.
    #[must_use]
    pub fn redactions(&self) -> &[crate::Redaction] {
        self.redactions.as_slice()
    }

    /// Returns deterministic BLAKE3 receipt identity.
    #[must_use]
    pub fn identity_hash(&self) -> &str {
        self.identity_hash.as_str()
    }
}

/// Borrowed fields used to build a rotation receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouterRotationReceiptDraft<'a> {
    /// Recipe identifier.
    pub recipe_id: &'a str,
    /// Route-family label.
    pub route_family: &'a str,
    /// Role label.
    pub role: &'a str,
    /// Rotation policy label.
    pub policy: &'a str,
    /// Cursor-class label.
    pub cursor_class: &'a str,
    /// Safe selected model label.
    pub selected_model: &'a str,
    /// Bounded eligible-candidate count.
    pub candidate_count: u32,
    /// BLAKE3 catalog-generation hash.
    pub catalog_hash: &'a str,
    /// BLAKE3 next-cursor hash.
    pub next_cursor_hash: &'a str,
    /// Redaction markers.
    pub redactions: &'a [&'a str],
}

fn receipt_label(
    field: crate::FieldName,
    value: &str,
) -> Result<alloc::string::String, super::RouterRotationError> {
    crate::validate_receipt_text(value).map_err(super::RouterRotationError::ReceiptLeak)?;
    crate::normalized_adapter_field(field, value).map_err(super::RouterRotationError::from)
}

fn parse_receipt_hash(
    _field: crate::FieldName,
    value: &str,
) -> Result<crate::SafeHash, super::RouterRotationError> {
    crate::validate_receipt_text(value).map_err(super::RouterRotationError::ReceiptLeak)?;
    crate::SafeHash::new(value).map_err(super::RouterRotationError::from)
}

fn collect_redactions(
    values: &[&str],
) -> Result<alloc::vec::Vec<crate::Redaction>, super::RouterRotationError> {
    if values.is_empty() {
        return Err(super::RouterRotationError::Contract(
            crate::AdapterContractError::EmptyCollection {
                field: super::EMPTY_REDACTIONS_FIELD,
            },
        ));
    }
    values
        .iter()
        .map(|value| crate::Redaction::parse(value).map_err(super::RouterRotationError::from))
        .collect()
}

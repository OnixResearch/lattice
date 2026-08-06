const ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("dataspace_assertion_id");
const REDACTIONS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("dataspace_assertion_redactions");

/// Bounded redacted assertion projected to an optional dataspace backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    stable_id: alloc::string::String,
    kind: super::Kind,
    status: super::Status,
    scope: crate::dataspace::Scope,
    redactions: alloc::vec::Vec<super::Redaction>,
    content_hash: Option<crate::effect::SafeHash>,
    item_count: u32,
    lease_expires_at: Option<u64>,
}

impl Entry {
    /// Validates a bounded redacted dataspace assertion.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when labels, kinds, statuses,
    /// redactions, or hashes are empty, malformed, or unsupported.
    pub fn new(draft: Draft<'_>) -> Result<Self, crate::AdapterContractError> {
        let stable_id = crate::validation::normalized_adapter_field(ID_FIELD, draft.stable_id)?;
        let kind = super::Kind::parse(draft.kind)?;
        let status = super::Status::parse(draft.status)?;
        let redactions = collect_redactions(draft.redactions)?;
        let content_hash = draft.content_hash.map(crate::effect::SafeHash::new).transpose()?;

        Ok(Self {
            stable_id,
            kind,
            status,
            scope: draft.scope,
            redactions,
            content_hash,
            item_count: draft.item_count,
            lease_expires_at: draft.lease_expires_at,
        })
    }

    /// Returns the stable assertion identity.
    #[must_use]
    pub fn stable_id(&self) -> &str {
        self.stable_id.as_str()
    }

    /// Returns the assertion kind.
    #[must_use]
    pub const fn kind(&self) -> super::Kind {
        self.kind
    }

    /// Returns the safe status class.
    #[must_use]
    pub const fn status(&self) -> super::Status {
        self.status
    }

    /// Returns the explicit assertion scope.
    #[must_use]
    pub const fn scope(&self) -> &crate::dataspace::Scope {
        &self.scope
    }

    /// Returns the redaction markers attached to this assertion.
    #[must_use]
    pub fn redactions(&self) -> &[super::Redaction] {
        self.redactions.as_slice()
    }

    /// Returns the optional BLAKE3 content hash.
    #[must_use]
    pub fn content_hash(&self) -> Option<&crate::effect::SafeHash> {
        self.content_hash.as_ref()
    }

    /// Returns the bounded safe item count.
    #[must_use]
    pub const fn item_count(&self) -> u32 {
        self.item_count
    }

    /// Returns the optional deterministic lease-expiry tick.
    #[must_use]
    pub const fn lease_expires_at(&self) -> Option<u64> {
        self.lease_expires_at
    }

    /// Returns whether the assertion is stale at the supplied deterministic tick.
    #[must_use]
    pub fn is_stale_at(&self, now: u64) -> bool {
        self.lease_expires_at.is_some_and(|expires_at| expires_at <= now)
    }

    #[cfg(test)]
    pub(in crate::dataspace) const fn set_status(&mut self, status: super::Status) {
        self.status = status;
    }
}

/// Borrowed input used to validate a bounded dataspace assertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Stable assertion identity.
    pub stable_id: &'a str,
    /// Assertion kind label.
    pub kind: &'a str,
    /// Assertion status label.
    pub status: &'a str,
    /// Explicit assertion scope.
    pub scope: crate::dataspace::Scope,
    /// Redaction marker labels proving raw material was omitted.
    pub redactions: &'a [&'a str],
    /// Optional BLAKE3 content hash for redacted source material.
    pub content_hash: Option<&'a str>,
    /// Bounded safe item count.
    pub item_count: u32,
    /// Optional deterministic lease-expiry tick.
    pub lease_expires_at: Option<u64>,
}

fn collect_redactions(values: &[&str]) -> Result<alloc::vec::Vec<super::Redaction>, crate::AdapterContractError> {
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: REDACTIONS_FIELD.as_str(),
        });
    }

    values.iter().map(|value| super::Redaction::parse(value)).collect()
}

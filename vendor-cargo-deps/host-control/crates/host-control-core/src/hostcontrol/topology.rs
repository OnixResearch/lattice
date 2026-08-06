/// Borrowed selector fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostSelectorDraft<'a> {
    /// Host identity.
    pub host_id: &'a str,
    /// Host resource identity.
    pub resource_id: &'a str,
    /// Resource kind label.
    pub kind: &'a str,
    /// Selector lifetime label.
    pub lifetime: &'a str,
    /// Topology revision observed with the selector.
    pub revision: u64,
}

/// Validated host selector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostSelector {
    host_id: String,
    resource_id: String,
    kind: HostResourceKind,
    lifetime: SelectorLifetime,
    revision: u64,
}

impl HostSelector {
    /// Validates a host selector.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when text fields or labels are invalid.
    pub fn validate(draft: HostSelectorDraft<'_>) -> Result<Self, HostControlError> {
        Ok(Self {
            host_id: required_text(HOST_ID_FIELD, draft.host_id)?,
            resource_id: required_text(RESOURCE_ID_FIELD, draft.resource_id)?,
            kind: HostResourceKind::parse(draft.kind)?,
            lifetime: SelectorLifetime::parse(draft.lifetime)?,
            revision: draft.revision,
        })
    }

    /// Returns host identity.
    #[must_use]
    pub fn host_id(&self) -> &str {
        self.host_id.as_str()
    }

    /// Returns resource identity.
    #[must_use]
    pub fn resource_id(&self) -> &str {
        self.resource_id.as_str()
    }

    /// Returns resource kind.
    #[must_use]
    pub const fn kind(&self) -> HostResourceKind {
        self.kind
    }

    /// Returns selector lifetime.
    #[must_use]
    pub const fn lifetime(&self) -> SelectorLifetime {
        self.lifetime
    }

    /// Returns observed topology revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
}
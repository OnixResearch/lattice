impl PromptAssemblyPolicy {
    /// Validates prompt assembly policy metadata.
    ///
    /// # Errors
    ///
    /// Returns [`PromptAssemblyError`] when section order, budget, or hash validation fails.
    pub fn validate(draft: &PromptAssemblyPolicyDraft<'_>) -> Result<Self, PromptAssemblyError> {
        if draft.section_order.is_empty() {
            return Err(PromptAssemblyError::EmptySectionOrder);
        }
        if draft.section_order.len() > MAX_PROMPT_SECTIONS {
            return Err(PromptAssemblyError::TooManyItems {
                field: SECTION_LABEL_FIELD.as_str(),
                max_items: MAX_PROMPT_SECTIONS,
                actual_items: draft.section_order.len(),
            });
        }
        let section_budget_units = NonZeroU32::new(draft.section_budget_units)
            .ok_or(PromptAssemblyError::ZeroSectionBudget)?;
        Ok(Self {
            mode: draft.mode,
            section_order: draft.section_order.to_vec(),
            section_budget_units,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash)
                .map_err(PromptAssemblyError::InvalidHash)?,
        })
    }

    /// Returns assembly mode.
    #[must_use]
    pub const fn mode(&self) -> PromptAssemblyMode {
        self.mode
    }

    /// Returns section order.
    #[must_use]
    pub fn section_order(&self) -> &[PromptSectionKind] {
        self.section_order.as_slice()
    }

    /// Returns maximum units per section.
    #[must_use]
    pub const fn section_budget_units(&self) -> NonZeroU32 {
        self.section_budget_units
    }

    /// Returns policy identity.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }
}

/// Prompt assembly ledger state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptAssemblyLedgerState {
    /// Source contributed fully to a model-visible section.
    Included,
    /// Source was skipped without consuming section budget.
    Skipped,
    /// Source was partially included and truncation was recorded.
    Truncated,
    /// Source was redacted by policy.
    Redacted,
    /// Source was denied by trust or admission policy.
    Denied,
    /// Source conflicted with another selected source.
    Conflicting,
    /// Source kind or discovery path was unsupported or unavailable.
    Unsupported,
}

/// Prompt assembly ledger reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptAssemblyLedgerReason {
    /// Source exceeded section budget.
    OverBudget,
    /// Source redaction class did not permit model-visible content.
    RedactedByPolicy,
    /// Source was denied by trust policy.
    UntrustedSource,
    /// Source was denied by admission policy.
    DeniedByAdmission,
    /// Required source or adapter was unavailable.
    RequiredSourceUnavailable,
    /// Source conflicts with an already selected source.
    SourceConflict,
    /// Host-context-only mode has no discovery adapter for this source.
    DiscoveryUnavailable,
    /// Source kind is unsupported.
    UnsupportedSourceKind,
}

/// Prompt assembly ledger entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptAssemblyLedgerEntry {
    source_kind: PromptSourceKind,
    source_label: String,
    trust_status: PromptSourceTrustStatus,
    admission_status: PromptSourceAdmissionStatus,
    section: PromptSectionKind,
    budget_class: PromptBudgetClass,
    requested_units: NonZeroU32,
    consumed_units: u32,
    body_hash: crate::artifact::Hash,
    state: PromptAssemblyLedgerState,
    reason: Option<PromptAssemblyLedgerReason>,
}

impl PromptAssemblyLedgerEntry {
    /// Returns the source kind.
    #[must_use]
    pub const fn source_kind(&self) -> PromptSourceKind {
        self.source_kind
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns trust status.
    #[must_use]
    pub const fn trust_status(&self) -> PromptSourceTrustStatus {
        self.trust_status
    }

    /// Returns admission status.
    #[must_use]
    pub const fn admission_status(&self) -> PromptSourceAdmissionStatus {
        self.admission_status
    }

    /// Returns prompt section.
    #[must_use]
    pub const fn section(&self) -> PromptSectionKind {
        self.section
    }

    /// Returns budget class.
    #[must_use]
    pub const fn budget_class(&self) -> PromptBudgetClass {
        self.budget_class
    }

    /// Returns requested units.
    #[must_use]
    pub const fn requested_units(&self) -> NonZeroU32 {
        self.requested_units
    }

    /// Returns consumed units.
    #[must_use]
    pub const fn consumed_units(&self) -> u32 {
        self.consumed_units
    }

    /// Returns source body identity.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }

    /// Returns ledger state.
    #[must_use]
    pub const fn state(&self) -> PromptAssemblyLedgerState {
        self.state
    }

    /// Returns ledger reason.
    #[must_use]
    pub const fn reason(&self) -> Option<PromptAssemblyLedgerReason> {
        self.reason
    }
}

/// Model-visible section metadata assembled from an admitted source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptAssemblySection {
    section: PromptSectionKind,
    source_label: String,
    body_hash: crate::artifact::Hash,
    consumed_units: u32,
    truncated: bool,
}

impl PromptAssemblySection {
    /// Returns prompt section.
    #[must_use]
    pub const fn section(&self) -> PromptSectionKind {
        self.section
    }

    /// Returns safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns BLAKE3 body identity.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }

    /// Returns consumed budget units.
    #[must_use]
    pub const fn consumed_units(&self) -> u32 {
        self.consumed_units
    }

    /// Returns true when content was truncated.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

/// Prompt assembly result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptAssembly {
    sections: Vec<PromptAssemblySection>,
    ledger: Vec<PromptAssemblyLedgerEntry>,
    policy_hash: crate::artifact::Hash,
}

impl PromptAssembly {
    /// Returns assembled model-visible section metadata.
    #[must_use]
    pub fn sections(&self) -> &[PromptAssemblySection] {
        self.sections.as_slice()
    }

    /// Returns safe assembly ledger entries.
    #[must_use]
    pub fn ledger(&self) -> &[PromptAssemblyLedgerEntry] {
        self.ledger.as_slice()
    }

    /// Returns policy identity.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }
}


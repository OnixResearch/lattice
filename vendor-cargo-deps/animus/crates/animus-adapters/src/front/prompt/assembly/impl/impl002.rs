impl PromptSource {
    /// Validates a host-supplied prompt source descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`PromptAssemblyError`] when text, hash, or budget invariants fail.
    pub fn validate(draft: &PromptSourceDraft<'_>) -> Result<Self, PromptAssemblyError> {
        let label = normalize(SOURCE_LABEL_FIELD, draft.label)?;
        let body_hash = crate::artifact::Hash::new(draft.body_hash)
            .map_err(PromptAssemblyError::InvalidHash)?;
        let requested_units = NonZeroU32::new(draft.requested_units).ok_or_else(|| {
            PromptAssemblyError::ZeroBudgetUnits {
                source_label: label.clone(),
            }
        })?;
        Ok(Self {
            kind: draft.kind,
            scope: draft.scope,
            trust_status: draft.trust_status,
            admission_status: draft.admission_status,
            requirement: draft.requirement,
            priority: draft.priority,
            budget_class: draft.budget_class,
            redaction_class: draft.redaction_class,
            label,
            section: draft.section,
            body_hash,
            requested_units,
        })
    }

    /// Returns the source kind.
    #[must_use]
    pub const fn kind(&self) -> PromptSourceKind {
        self.kind
    }

    /// Returns the source scope.
    #[must_use]
    pub const fn scope(&self) -> PromptResourceScope {
        self.scope
    }

    /// Returns the trust status.
    #[must_use]
    pub const fn trust_status(&self) -> PromptSourceTrustStatus {
        self.trust_status
    }

    /// Returns admission status.
    #[must_use]
    pub const fn admission_status(&self) -> PromptSourceAdmissionStatus {
        self.admission_status
    }

    /// Returns source requirement rule.
    #[must_use]
    pub const fn requirement(&self) -> PromptSourceRequirement {
        self.requirement
    }

    /// Returns source precedence priority.
    #[must_use]
    pub const fn priority(&self) -> PromptSourcePriority {
        self.priority
    }

    /// Returns budget class.
    #[must_use]
    pub const fn budget_class(&self) -> PromptBudgetClass {
        self.budget_class
    }

    /// Returns redaction class.
    #[must_use]
    pub const fn redaction_class(&self) -> PromptRedactionClass {
        self.redaction_class
    }

    /// Returns safe source label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns prompt section target.
    #[must_use]
    pub const fn section(&self) -> PromptSectionKind {
        self.section
    }

    /// Returns BLAKE3 source body identity.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }

    /// Returns requested budget units.
    #[must_use]
    pub const fn requested_units(&self) -> NonZeroU32 {
        self.requested_units
    }
}

/// Prompt assembly mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptAssemblyMode {
    /// Only already supplied host context can influence assembly.
    HostContextOnly,
    /// Host has supplied admitted sources and discovery adapters out of band.
    AdmittedSources,
}

/// Borrowed prompt assembly policy fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromptAssemblyPolicyDraft<'a> {
    /// Assembly mode.
    pub mode: PromptAssemblyMode,
    /// Section order from first to last.
    pub section_order: &'a [PromptSectionKind],
    /// Maximum units per section.
    pub section_budget_units: u32,
    /// Lowercase BLAKE3 policy identity.
    pub policy_hash: &'a str,
}

/// Validated prompt assembly policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptAssemblyPolicy {
    mode: PromptAssemblyMode,
    section_order: Vec<PromptSectionKind>,
    section_budget_units: NonZeroU32,
    policy_hash: crate::artifact::Hash,
}


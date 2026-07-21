/// Borrowed memory descriptor fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryDescriptorDraft<'a> {
    /// Stable descriptor identity.
    pub id: &'a str,
    /// Descriptor kind.
    pub kind: MemoryDescriptorKind,
    /// Safe source label supplied by the host.
    pub source_label: &'a str,
    /// Trust scope; missing scope is rejected.
    pub trust_scope: Option<MemoryTrustScope>,
    /// Prompt precedence.
    pub precedence: MemoryPrecedence,
    /// Freshness state.
    pub freshness: MemoryFreshness,
    /// Inclusion policy; missing policy is rejected.
    pub inclusion_policy: Option<MemoryInclusionPolicy>,
    /// Host admission state.
    pub admission_status: MemoryAdmissionStatus,
    /// Redaction class.
    pub redaction_class: MemoryRedactionClass,
    /// Lowercase BLAKE3 body hash.
    pub body_hash: &'a str,
    /// Prompt budget units requested by this descriptor.
    pub budget_units: u32,
}

/// Validated memory descriptor consumed by reusable prompt assembly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryDescriptor {
    id: String,
    kind: MemoryDescriptorKind,
    source_label: String,
    trust_scope: MemoryTrustScope,
    precedence: MemoryPrecedence,
    freshness: MemoryFreshness,
    inclusion_policy: MemoryInclusionPolicy,
    admission_status: MemoryAdmissionStatus,
    redaction_class: MemoryRedactionClass,
    body_hash: crate::artifact::Hash,
    budget_units: NonZeroU32,
}

impl MemoryDescriptor {
    /// Validates host-supplied instruction or learned-memory metadata.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when required fields are missing, body metadata is
    /// unsafe, budget units are zero, or freshness has expired.
    // r[impl agent_harness_runtime.claude_code_memory.descriptors]
    pub fn validate(draft: &MemoryDescriptorDraft<'_>) -> Result<Self, MemoryError> {
        let id = normalize(DESCRIPTOR_ID_FIELD, draft.id)?;
        let source_label = normalize(MEMORY_SOURCE_FIELD, draft.source_label)?;
        let trust_scope = required_trust_scope(&id, draft.trust_scope)?;
        reject_expired_descriptor(&id, draft.freshness)?;
        let inclusion_policy = required_inclusion_policy(&id, draft.inclusion_policy)?;
        let body_hash =
            crate::artifact::Hash::new(draft.body_hash).map_err(MemoryError::InvalidHash)?;
        let budget_units =
            NonZeroU32::new(draft.budget_units).ok_or_else(|| MemoryError::ZeroBudgetUnits {
                descriptor_id: id.clone(),
            })?;
        Ok(Self {
            id,
            kind: draft.kind,
            source_label,
            trust_scope,
            precedence: draft.precedence,
            freshness: draft.freshness,
            inclusion_policy,
            admission_status: draft.admission_status,
            redaction_class: draft.redaction_class,
            body_hash,
            budget_units,
        })
    }

    /// Returns the stable descriptor identity.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the descriptor kind.
    #[must_use]
    pub const fn kind(&self) -> MemoryDescriptorKind {
        self.kind
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns the trust scope.
    #[must_use]
    pub const fn trust_scope(&self) -> MemoryTrustScope {
        self.trust_scope
    }

    /// Returns the deterministic precedence rank.
    #[must_use]
    pub const fn precedence(&self) -> MemoryPrecedence {
        self.precedence
    }

    /// Returns the freshness state.
    #[must_use]
    pub const fn freshness(&self) -> MemoryFreshness {
        self.freshness
    }

    /// Returns the prompt-inclusion policy.
    #[must_use]
    pub const fn inclusion_policy(&self) -> MemoryInclusionPolicy {
        self.inclusion_policy
    }

    /// Returns the host admission status.
    #[must_use]
    pub const fn admission_status(&self) -> MemoryAdmissionStatus {
        self.admission_status
    }

    /// Returns the redaction class.
    #[must_use]
    pub const fn redaction_class(&self) -> MemoryRedactionClass {
        self.redaction_class
    }

    /// Returns the BLAKE3 body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }

    /// Returns requested prompt budget units.
    #[must_use]
    pub const fn budget_units(&self) -> NonZeroU32 {
        self.budget_units
    }
}

/// Borrowed memory prompt policy fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryPromptPolicyDraft<'a> {
    /// Prompt assembly mode to use for memory sources.
    pub mode: PromptAssemblyMode,
    /// Section order from first to last.
    pub section_order: &'a [PromptSectionKind],
    /// Maximum budget units for each section.
    pub section_budget_units: u32,
    /// Lowercase BLAKE3 policy hash.
    pub policy_hash: &'a str,
    /// Whether stale memory may influence prompts.
    pub include_stale: bool,
}

/// Validated memory prompt policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryPromptPolicy {
    prompt_policy: PromptAssemblyPolicy,
    include_stale: bool,
}

impl MemoryPromptPolicy {
    /// Validates selected memory prompt policy.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when underlying prompt assembly policy validation fails.
    pub fn validate(draft: &MemoryPromptPolicyDraft<'_>) -> Result<Self, MemoryError> {
        let prompt_policy = PromptAssemblyPolicy::validate(&PromptAssemblyPolicyDraft {
            mode: draft.mode,
            section_order: draft.section_order,
            section_budget_units: draft.section_budget_units,
            policy_hash: draft.policy_hash,
        })?;
        Ok(Self {
            prompt_policy,
            include_stale: draft.include_stale,
        })
    }

    /// Returns the underlying prompt assembly policy.
    #[must_use]
    pub const fn prompt_policy(&self) -> &PromptAssemblyPolicy {
        &self.prompt_policy
    }

    /// Returns true when stale descriptors may influence prompts.
    #[must_use]
    pub const fn include_stale(&self) -> bool {
        self.include_stale
    }
}

/// Assembles admitted memory descriptors using the reusable prompt assembly core.
///
/// # Errors
///
/// Returns [`MemoryError`] when the descriptor count exceeds the memory bound or
/// prompt source validation/assembly fails.
// r[impl agent_harness_runtime.claude_code_memory.prompt_assembly]
// r[impl agent_harness_runtime.claude_code_memory.adapter_boundary]
pub fn assemble_memory_prompt(
    descriptors: &[MemoryDescriptor],
    policy: &MemoryPromptPolicy,
) -> Result<PromptAssembly, MemoryError> {
    if descriptors.len() > MAX_MEMORY_DESCRIPTORS {
        return Err(MemoryError::TooManyItems {
            field: DESCRIPTOR_ID_FIELD.as_str(),
            max_items: MAX_MEMORY_DESCRIPTORS,
            actual_items: descriptors.len(),
        });
    }
    let mut sources = Vec::with_capacity(descriptors.len());
    for descriptor in descriptors {
        sources.push(prompt_source_for_descriptor(descriptor, policy)?);
    }
    assemble_prompt_sources(sources.as_slice(), policy.prompt_policy()).map_err(MemoryError::from)
}


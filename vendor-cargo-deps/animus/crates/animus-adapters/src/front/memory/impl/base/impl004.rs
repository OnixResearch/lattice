/// Descriptor context supplied when a candidate is admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryAdmissionContext {
    /// Trust scope assigned by host admission.
    pub trust_scope: MemoryTrustScope,
    /// Prompt precedence assigned by host admission.
    pub precedence: MemoryPrecedence,
    /// Freshness state assigned by host admission.
    pub freshness: MemoryFreshness,
    /// Prompt inclusion policy assigned by host admission.
    pub inclusion_policy: MemoryInclusionPolicy,
}

/// Host persistence intent for a candidate admission outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryPersistenceIntent {
    /// Persist the admitted candidate for future prompt influence.
    Persist,
    /// Do not persist or reuse the candidate.
    DoNotPersist,
}

/// Result of applying admission to a memory candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryAdmissionEffect {
    descriptor: Option<MemoryDescriptor>,
    persistence: MemoryPersistenceIntent,
}

impl MemoryAdmissionEffect {
    /// Returns the admitted descriptor when one may influence future prompts.
    #[must_use]
    pub fn descriptor(&self) -> Option<&MemoryDescriptor> {
        self.descriptor.as_ref()
    }

    /// Consumes the effect and returns the admitted descriptor, when present.
    #[must_use]
    pub fn into_descriptor(self) -> Option<MemoryDescriptor> {
        self.descriptor
    }

    /// Returns the host persistence intent.
    #[must_use]
    pub const fn persistence(&self) -> MemoryPersistenceIntent {
        self.persistence
    }
}

/// Applies a host admission decision to a candidate without ambient storage.
///
/// # Errors
///
/// Returns [`MemoryError`] when the decision does not reference the candidate or
/// admitted descriptor validation fails.
///
/// # Panics
///
/// Panics if callers bypass candidate validation before resolving admission.
// r[impl agent_harness_runtime.claude_code_memory.admission]
pub fn resolve_memory_admission(
    candidate: &MemoryCandidate,
    decision: &MemoryAdmissionDecision,
    context: MemoryAdmissionContext,
) -> Result<MemoryAdmissionEffect, MemoryError> {
    assert!(!candidate.id().is_empty(), "candidate id must be validated");
    assert!(
        !candidate.source_label().is_empty(),
        "candidate source label must be validated"
    );
    if candidate.id() != decision.candidate_id() {
        return Err(MemoryError::AdmissionCandidateMismatch {
            candidate_id: String::from(candidate.id()),
            decision_candidate_id: String::from(decision.candidate_id()),
        });
    }
    if decision.status() != MemoryAdmissionStatus::Admitted {
        return Ok(MemoryAdmissionEffect {
            descriptor: None,
            persistence: MemoryPersistenceIntent::DoNotPersist,
        });
    }
    let descriptor = MemoryDescriptor::validate(&MemoryDescriptorDraft {
        id: candidate.id(),
        kind: candidate.kind(),
        source_label: candidate.source_label(),
        trust_scope: Some(context.trust_scope),
        precedence: context.precedence,
        freshness: context.freshness,
        inclusion_policy: Some(context.inclusion_policy),
        admission_status: MemoryAdmissionStatus::Admitted,
        redaction_class: candidate.redaction_class(),
        body_hash: candidate.body_hash().as_str(),
        budget_units: candidate.budget_units().get(),
    })?;
    Ok(MemoryAdmissionEffect {
        descriptor: Some(descriptor),
        persistence: MemoryPersistenceIntent::Persist,
    })
}

/// Inclusion status recorded by a memory influence receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryReceiptInclusionStatus {
    /// Memory influenced prompt assembly.
    Included,
    /// Memory was skipped by policy or budget.
    Skipped,
    /// Memory was redacted before prompt influence.
    Redacted,
    /// Memory was denied by trust or admission.
    Denied,
    /// Required host memory service or adapter was unavailable.
    Unavailable,
}

/// Borrowed safe memory receipt metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryInfluenceReceiptDraft<'a> {
    /// Stable receipt label.
    pub receipt_label: &'a str,
    /// Descriptor identity.
    pub descriptor_id: &'a str,
    /// Safe source label.
    pub source_label: &'a str,
    /// Descriptor trust scope.
    pub trust_scope: MemoryTrustScope,
    /// Descriptor freshness state.
    pub freshness: MemoryFreshness,
    /// Inclusion status.
    pub inclusion_status: MemoryReceiptInclusionStatus,
    /// Redaction class.
    pub redaction_class: MemoryRedactionClass,
    /// Lowercase BLAKE3 descriptor body hash.
    pub body_hash: &'a str,
    /// Raw memory body, forbidden in receipts.
    pub raw_memory_body: Option<&'a str>,
    /// Credential-looking value, forbidden in receipts.
    pub credential: Option<&'a str>,
    /// Provider payload, forbidden in receipts.
    pub provider_payload: Option<&'a str>,
    /// Full host path, forbidden in receipts.
    pub full_path: Option<&'a str>,
    /// Unbounded prompt text, forbidden in receipts.
    pub unbounded_prompt_text: Option<&'a str>,
}

/// Safe metadata proving memory prompt influence without leaking bodies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryInfluenceReceipt {
    receipt_label: String,
    descriptor_id: String,
    source_label: String,
    trust_scope: MemoryTrustScope,
    freshness: MemoryFreshness,
    inclusion_status: MemoryReceiptInclusionStatus,
    redaction_class: MemoryRedactionClass,
    body_hash: crate::artifact::Hash,
}

impl MemoryInfluenceReceipt {
    /// Validates safe memory influence receipt metadata.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when forbidden sensitive material is present or
    /// safe labels/hashes fail validation.
    // r[impl agent_harness_runtime.claude_code_memory.receipts]
    pub fn validate(draft: &MemoryInfluenceReceiptDraft<'_>) -> Result<Self, MemoryError> {
        reject_receipt_leaks(draft)?;
        Ok(Self {
            receipt_label: normalize(RECEIPT_LABEL_FIELD, draft.receipt_label)?,
            descriptor_id: normalize(DESCRIPTOR_ID_FIELD, draft.descriptor_id)?,
            source_label: normalize(MEMORY_SOURCE_FIELD, draft.source_label)?,
            trust_scope: draft.trust_scope,
            freshness: draft.freshness,
            inclusion_status: draft.inclusion_status,
            redaction_class: draft.redaction_class,
            body_hash: crate::artifact::Hash::new(draft.body_hash)
                .map_err(MemoryError::InvalidHash)?,
        })
    }

    /// Returns the stable receipt label.
    #[must_use]
    pub fn receipt_label(&self) -> &str {
        self.receipt_label.as_str()
    }

    /// Returns the descriptor identity.
    #[must_use]
    pub fn descriptor_id(&self) -> &str {
        self.descriptor_id.as_str()
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns trust scope.
    #[must_use]
    pub const fn trust_scope(&self) -> MemoryTrustScope {
        self.trust_scope
    }

    /// Returns freshness state.
    #[must_use]
    pub const fn freshness(&self) -> MemoryFreshness {
        self.freshness
    }

    /// Returns inclusion status.
    #[must_use]
    pub const fn inclusion_status(&self) -> MemoryReceiptInclusionStatus {
        self.inclusion_status
    }

    /// Returns redaction class.
    #[must_use]
    pub const fn redaction_class(&self) -> MemoryRedactionClass {
        self.redaction_class
    }

    /// Returns the BLAKE3 descriptor body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }
}


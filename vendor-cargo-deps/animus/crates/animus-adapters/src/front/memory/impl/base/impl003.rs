/// Borrowed learned-memory candidate fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryCandidateDraft<'a> {
    /// Stable candidate identity.
    pub id: &'a str,
    /// Candidate kind.
    pub kind: MemoryDescriptorKind,
    /// Safe source label.
    pub source_label: &'a str,
    /// Redaction class selected before review.
    pub redaction_class: MemoryRedactionClass,
    /// Lowercase BLAKE3 candidate body hash.
    pub body_hash: &'a str,
    /// Prompt budget units requested if admitted.
    pub budget_units: u32,
}

/// Validated learned-memory candidate requiring host admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryCandidate {
    id: String,
    kind: MemoryDescriptorKind,
    source_label: String,
    redaction_class: MemoryRedactionClass,
    body_hash: crate::artifact::Hash,
    budget_units: NonZeroU32,
}

impl MemoryCandidate {
    /// Validates a learned-memory candidate before host admission.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when candidate identity, source label, hash, or
    /// budget metadata is invalid.
    pub fn validate(draft: &MemoryCandidateDraft<'_>) -> Result<Self, MemoryError> {
        let id = normalize(DESCRIPTOR_ID_FIELD, draft.id)?;
        let source_label = normalize(MEMORY_SOURCE_FIELD, draft.source_label)?;
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
            redaction_class: draft.redaction_class,
            body_hash,
            budget_units,
        })
    }

    /// Returns the candidate identity.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the candidate kind.
    #[must_use]
    pub const fn kind(&self) -> MemoryDescriptorKind {
        self.kind
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns the selected redaction class.
    #[must_use]
    pub const fn redaction_class(&self) -> MemoryRedactionClass {
        self.redaction_class
    }

    /// Returns the BLAKE3 candidate body hash.
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

/// Borrowed admission-decision fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryAdmissionDecisionDraft<'a> {
    /// Candidate identity being reviewed.
    pub candidate_id: &'a str,
    /// Host admission status.
    pub status: MemoryAdmissionStatus,
    /// Safe decision label for audit trails.
    pub decision_label: &'a str,
    /// Lowercase BLAKE3 hash for the reviewed candidate body.
    pub body_hash: &'a str,
    /// Raw candidate body, forbidden in admission metadata.
    pub raw_candidate_body: Option<&'a str>,
    /// Credential-looking value, forbidden in admission metadata.
    pub credential: Option<&'a str>,
    /// Provider payload, forbidden in admission metadata.
    pub provider_payload: Option<&'a str>,
    /// Full host path, forbidden in admission metadata.
    pub full_path: Option<&'a str>,
}

/// Validated host admission decision for a memory candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryAdmissionDecision {
    candidate_id: String,
    status: MemoryAdmissionStatus,
    decision_label: String,
    body_hash: crate::artifact::Hash,
}

impl MemoryAdmissionDecision {
    /// Validates host admission metadata without storing raw memory bodies.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when safe fields fail validation or forbidden
    /// sensitive material is present.
    pub fn validate(draft: &MemoryAdmissionDecisionDraft<'_>) -> Result<Self, MemoryError> {
        reject_admission_leaks(draft)?;
        Ok(Self {
            candidate_id: normalize(DESCRIPTOR_ID_FIELD, draft.candidate_id)?,
            status: draft.status,
            decision_label: normalize(DECISION_LABEL_FIELD, draft.decision_label)?,
            body_hash: crate::artifact::Hash::new(draft.body_hash)
                .map_err(MemoryError::InvalidHash)?,
        })
    }

    /// Returns the reviewed candidate identity.
    #[must_use]
    pub fn candidate_id(&self) -> &str {
        self.candidate_id.as_str()
    }

    /// Returns the host admission status.
    #[must_use]
    pub const fn status(&self) -> MemoryAdmissionStatus {
        self.status
    }

    /// Returns the safe decision label.
    #[must_use]
    pub fn decision_label(&self) -> &str {
        self.decision_label.as_str()
    }

    /// Returns the reviewed body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }
}


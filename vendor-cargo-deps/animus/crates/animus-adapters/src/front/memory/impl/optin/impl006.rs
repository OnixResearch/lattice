/// Borrowed opt-in memory policy fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OptInMemoryPolicyDraft<'a> {
    /// Independent memory-use controls.
    pub use_controls: MemoryScopeControls,
    /// Independent memory-generation controls.
    pub generation_controls: MemoryScopeControls,
    /// Lowercase BLAKE3 identity for the selected policy.
    pub policy_hash: &'a str,
}

/// Validated opt-in memory policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptInMemoryPolicy {
    use_controls: MemoryScopeControls,
    generation_controls: MemoryScopeControls,
    policy_hash: crate::artifact::Hash,
}

impl OptInMemoryPolicy {
    /// Validates opt-in memory controls and policy identity.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when the policy hash is invalid.
    // r[impl agent_harness_runtime.openai_codex_opt_in_memories.policy_controls]
    pub fn validate(draft: &OptInMemoryPolicyDraft<'_>) -> Result<Self, MemoryError> {
        Ok(Self {
            use_controls: draft.use_controls,
            generation_controls: draft.generation_controls,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash)
                .map_err(MemoryError::InvalidHash)?,
        })
    }

    /// Returns independent memory-use controls.
    #[must_use]
    pub const fn use_controls(&self) -> MemoryScopeControls {
        self.use_controls
    }

    /// Returns independent memory-generation controls.
    #[must_use]
    pub const fn generation_controls(&self) -> MemoryScopeControls {
        self.generation_controls
    }

    /// Returns true when memory use is explicitly enabled at every scope.
    #[must_use]
    pub const fn memory_use_enabled(&self) -> bool {
        self.use_controls.enabled()
    }

    /// Returns true when memory generation is explicitly enabled at every scope.
    #[must_use]
    pub const fn memory_generation_enabled(&self) -> bool {
        self.generation_controls.enabled()
    }

    /// Returns selected policy identity.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }
}

/// Opt-in memory record kind.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OptInMemoryRecordKind {
    /// Thread or project summary.
    Summary,
    /// Durable generated fact.
    DurableFact,
    /// Stable user or project preference.
    Preference,
    /// Known pitfall or recurring warning.
    Pitfall,
}

/// Redaction state for an opt-in memory record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRecordRedactionStatus {
    /// Record summary is safe to consider for bounded prompt injection.
    Clear,
    /// Record may surface only redacted metadata.
    Redacted,
}

/// Bounded confidence score for generated memory records.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MemoryConfidence(u8);

impl MemoryConfidence {
    /// Builds a bounded confidence score.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when the score is outside the supported percentage range.
    pub fn new(percent: u8) -> Result<Self, MemoryError> {
        if !(MIN_MEMORY_CONFIDENCE_PERCENT..=MAX_MEMORY_CONFIDENCE_PERCENT).contains(&percent) {
            return Err(MemoryError::InvalidConfidence {
                min_percent: MIN_MEMORY_CONFIDENCE_PERCENT,
                max_percent: MAX_MEMORY_CONFIDENCE_PERCENT,
                actual_percent: percent,
            });
        }
        Ok(Self(percent))
    }

    /// Returns the confidence percentage.
    #[must_use]
    pub const fn percent(self) -> u8 {
        self.0
    }
}

/// Borrowed opt-in memory record fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OptInMemoryRecordDraft<'a> {
    /// Stable record identity.
    pub id: &'a str,
    /// Record kind.
    pub kind: OptInMemoryRecordKind,
    /// Bounded safe summary label.
    pub summary_label: &'a str,
    /// Memory scope; missing scope is rejected.
    pub scope: Option<MemoryTrustScope>,
    /// Freshness state.
    pub freshness: MemoryFreshness,
    /// Safe source thread identity.
    pub source_thread_id: &'a str,
    /// Safe supporting evidence handle.
    pub evidence_handle: &'a str,
    /// Redaction status.
    pub redaction_status: MemoryRecordRedactionStatus,
    /// Bounded confidence percentage.
    pub confidence_percent: u8,
    /// Lowercase BLAKE3 identity for the redacted body or summary source.
    pub body_hash: &'a str,
    /// Prompt budget units requested when injected.
    pub budget_units: u32,
}

/// Validated evidence-backed opt-in memory record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptInMemoryRecord {
    id: String,
    kind: OptInMemoryRecordKind,
    summary_label: String,
    scope: MemoryTrustScope,
    freshness: MemoryFreshness,
    source_thread_id: String,
    evidence_handle: String,
    redaction_status: MemoryRecordRedactionStatus,
    confidence: MemoryConfidence,
    body_hash: crate::artifact::Hash,
    budget_units: NonZeroU32,
}

impl OptInMemoryRecord {
    /// Validates an evidence-backed generated memory record.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when required identity, scope, evidence, hash,
    /// confidence, or budget metadata is invalid.
    // r[impl agent_harness_runtime.openai_codex_opt_in_memories.record_contract]
    pub fn validate(draft: &OptInMemoryRecordDraft<'_>) -> Result<Self, MemoryError> {
        let id = normalize(DESCRIPTOR_ID_FIELD, draft.id)?;
        let summary_label = normalize(MEMORY_SUMMARY_FIELD, draft.summary_label)?;
        let source_thread_id = normalize(MEMORY_THREAD_FIELD, draft.source_thread_id)?;
        let evidence_handle = normalize(MEMORY_EVIDENCE_FIELD, draft.evidence_handle)?;
        let scope = draft.scope.ok_or_else(|| MemoryError::MissingRecordScope {
            record_id: id.clone(),
        })?;
        reject_expired_descriptor(&id, draft.freshness)?;
        let confidence = MemoryConfidence::new(draft.confidence_percent)?;
        let body_hash =
            crate::artifact::Hash::new(draft.body_hash).map_err(MemoryError::InvalidHash)?;
        let budget_units =
            NonZeroU32::new(draft.budget_units).ok_or_else(|| MemoryError::ZeroBudgetUnits {
                descriptor_id: id.clone(),
            })?;
        Ok(Self {
            id,
            kind: draft.kind,
            summary_label,
            scope,
            freshness: draft.freshness,
            source_thread_id,
            evidence_handle,
            redaction_status: draft.redaction_status,
            confidence,
            body_hash,
            budget_units,
        })
    }

    /// Returns record identity.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns record kind.
    #[must_use]
    pub const fn kind(&self) -> OptInMemoryRecordKind {
        self.kind
    }

    /// Returns safe summary label.
    #[must_use]
    pub fn summary_label(&self) -> &str {
        self.summary_label.as_str()
    }

    /// Returns memory scope.
    #[must_use]
    pub const fn scope(&self) -> MemoryTrustScope {
        self.scope
    }

    /// Returns freshness state.
    #[must_use]
    pub const fn freshness(&self) -> MemoryFreshness {
        self.freshness
    }

    /// Returns safe source thread identity.
    #[must_use]
    pub fn source_thread_id(&self) -> &str {
        self.source_thread_id.as_str()
    }

    /// Returns safe evidence handle.
    #[must_use]
    pub fn evidence_handle(&self) -> &str {
        self.evidence_handle.as_str()
    }

    /// Returns redaction status.
    #[must_use]
    pub const fn redaction_status(&self) -> MemoryRecordRedactionStatus {
        self.redaction_status
    }

    /// Returns confidence score.
    #[must_use]
    pub const fn confidence(&self) -> MemoryConfidence {
        self.confidence
    }

    /// Returns BLAKE3 body identity.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }

    /// Returns requested injection budget units.
    #[must_use]
    pub const fn budget_units(&self) -> NonZeroU32 {
        self.budget_units
    }
}


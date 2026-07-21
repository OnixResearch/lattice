/// Inputs for conservative memory-generation eligibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryGenerationEligibility<'a> {
    /// Selected opt-in policy.
    pub policy: &'a OptInMemoryPolicy,
    /// Whether the thread consumed external context that may be persisted.
    pub external_context_persistable: bool,
    /// Whether the thread contains secret-bearing content.
    pub contains_secret_bearing_content: bool,
    /// Whether redaction succeeded before generation.
    pub redaction_succeeded: bool,
    /// Remaining memory-generation budget units.
    pub remaining_budget_units: u32,
}

/// Memory-generation decision state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryGenerationStatus {
    /// Generation may proceed through an extraction adapter.
    Allowed,
    /// Generation is denied before extraction.
    Denied,
}

/// Reason a memory-generation request was denied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryGenerationDenialReason {
    /// One or more scopes did not opt into memory generation.
    MissingConsent,
    /// External context cannot be persisted into memory.
    ExternalContext,
    /// Secret-bearing content was present.
    SecretBearingContent,
    /// Redaction failed before generation.
    RedactionFailed,
    /// Memory generation budget was exhausted.
    BudgetExhausted,
}

/// Conservative memory-generation decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryGenerationDecision {
    status: MemoryGenerationStatus,
    denial_reason: Option<MemoryGenerationDenialReason>,
}

impl MemoryGenerationDecision {
    /// Returns generation status.
    #[must_use]
    pub const fn status(self) -> MemoryGenerationStatus {
        self.status
    }

    /// Returns denial reason when generation is denied.
    #[must_use]
    pub const fn denial_reason(self) -> Option<MemoryGenerationDenialReason> {
        self.denial_reason
    }
}

/// Evaluates conservative memory-generation eligibility without calling extractors.
///
/// # Panics
///
/// Panics if callers bypass policy validation before eligibility evaluation.
#[must_use]
// r[impl agent_harness_runtime.openai_codex_opt_in_memories.generation_eligibility]
pub fn evaluate_memory_generation(
    input: MemoryGenerationEligibility<'_>,
) -> MemoryGenerationDecision {
    assert!(
        !input.policy.policy_hash().as_str().is_empty(),
        "memory policy hash must be validated"
    );
    assert_eq!(
        input.policy.memory_generation_enabled(),
        input.policy.generation_controls().enabled(),
        "generation helper must mirror policy controls"
    );
    if !input.policy.memory_generation_enabled() {
        return denied_generation(MemoryGenerationDenialReason::MissingConsent);
    }
    if !input.external_context_persistable {
        return denied_generation(MemoryGenerationDenialReason::ExternalContext);
    }
    if input.contains_secret_bearing_content {
        return denied_generation(MemoryGenerationDenialReason::SecretBearingContent);
    }
    if !input.redaction_succeeded {
        return denied_generation(MemoryGenerationDenialReason::RedactionFailed);
    }
    if input.remaining_budget_units == u32::default() {
        return denied_generation(MemoryGenerationDenialReason::BudgetExhausted);
    }
    MemoryGenerationDecision {
        status: MemoryGenerationStatus::Allowed,
        denial_reason: None,
    }
}

/// Prompt injection budget for opt-in memories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryInjectionBudget(NonZeroU32);

impl MemoryInjectionBudget {
    /// Builds a memory injection budget.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when the budget is zero.
    pub fn new(units: u32) -> Result<Self, MemoryError> {
        let units = NonZeroU32::new(units).ok_or(MemoryError::ZeroInjectionBudget)?;
        Ok(Self(units))
    }

    /// Returns maximum injection units.
    #[must_use]
    pub const fn units(self) -> NonZeroU32 {
        self.0
    }
}

/// State recorded for one memory injection ledger entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryInjectionState {
    /// Memory summary was included.
    Included,
    /// Memory was skipped before consuming budget.
    Skipped,
    /// Memory was expired or stale for this injection policy.
    Expired,
    /// Memory was redacted before prompt injection.
    Redacted,
    /// Memory was skipped because the memory budget was exhausted.
    OverBudget,
}

/// Reason attached to a memory injection ledger entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryInjectionReason {
    /// Memory use was disabled by opt-in controls.
    MemoriesDisabled,
    /// Record was stale for prompt injection.
    StaleRecord,
    /// Record was redacted by policy.
    RedactedByPolicy,
    /// Record exceeded the remaining injection budget.
    OverBudget,
}

/// Safe ledger entry for memory prompt injection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryInjectionLedgerEntry {
    record_id: String,
    source_thread_id: String,
    evidence_handle: String,
    body_hash: crate::artifact::Hash,
    requested_units: NonZeroU32,
    consumed_units: u32,
    state: MemoryInjectionState,
    reason: Option<MemoryInjectionReason>,
}

impl MemoryInjectionLedgerEntry {
    /// Returns record identity.
    #[must_use]
    pub fn record_id(&self) -> &str {
        self.record_id.as_str()
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

    /// Returns BLAKE3 body identity.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
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

    /// Returns injection state.
    #[must_use]
    pub const fn state(&self) -> MemoryInjectionState {
        self.state
    }

    /// Returns optional injection reason.
    #[must_use]
    pub const fn reason(&self) -> Option<MemoryInjectionReason> {
        self.reason
    }
}

/// Pure opt-in memory injection plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryInjectionPlan {
    ledger: Vec<MemoryInjectionLedgerEntry>,
}

impl MemoryInjectionPlan {
    /// Returns safe injection ledger entries.
    #[must_use]
    pub fn ledger(&self) -> &[MemoryInjectionLedgerEntry] {
        self.ledger.as_slice()
    }
}


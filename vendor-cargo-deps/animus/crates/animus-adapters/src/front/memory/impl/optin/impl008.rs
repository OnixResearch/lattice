/// Plans bounded memory injection from validated records and opt-in policy.
///
/// # Errors
/// Returns [`MemoryError`] when the record count exceeds the memory bound.
///
/// # Panics
///
/// Panics if callers bypass budget or policy validation before planning injection.
// r[impl agent_harness_runtime.openai_codex_opt_in_memories.injection_ledger]
pub fn plan_opt_in_memory_injection(
    records: &[OptInMemoryRecord],
    policy: &OptInMemoryPolicy,
    budget: MemoryInjectionBudget,
) -> Result<MemoryInjectionPlan, MemoryError> {
    if records.len() > MAX_MEMORY_DESCRIPTORS {
        return Err(MemoryError::TooManyItems {
            field: DESCRIPTOR_ID_FIELD.as_str(),
            max_items: MAX_MEMORY_DESCRIPTORS,
            actual_items: records.len(),
        });
    }
    assert!(records.len() <= MAX_MEMORY_DESCRIPTORS, "record bound is enforced");
    assert!(budget.units().get() > u32::default(), "budget must be validated");
    assert!(
        !policy.policy_hash().as_str().is_empty(),
        "memory policy hash must be validated"
    );
    let mut ordered: Vec<&OptInMemoryRecord> = records.iter().collect();
    ordered.sort_by(|left, right| compare_opt_in_records(left, right));
    let mut used_units = u32::default();
    let mut ledger = Vec::with_capacity(records.len());
    for record in ordered {
        let decision = injection_decision(record, policy, budget, used_units);
        used_units = used_units.saturating_add(decision.consumed_units);
        ledger.push(MemoryInjectionLedgerEntry {
            record_id: record.id.clone(),
            source_thread_id: record.source_thread_id.clone(),
            evidence_handle: record.evidence_handle.clone(),
            body_hash: record.body_hash.clone(),
            requested_units: record.budget_units,
            consumed_units: decision.consumed_units,
            state: decision.state,
            reason: decision.reason,
        });
    }
    Ok(MemoryInjectionPlan { ledger })
}

/// Inputs for required-rule source resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequiredRuleInput<'a> {
    /// Selected opt-in memory policy.
    pub memory_policy: &'a OptInMemoryPolicy,
    /// Whether required guidance is present.
    pub guidance_present: bool,
    /// Whether required policy is present.
    pub policy_present: bool,
}

/// Resolution of required rules independently from memory state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequiredRuleResolution {
    required_rules_active: bool,
    memory_required: bool,
}

impl RequiredRuleResolution {
    /// Returns true when required guidance or policy remains active.
    #[must_use]
    pub const fn required_rules_active(self) -> bool {
        self.required_rules_active
    }

    /// Returns true when memory state is required for required rules.
    #[must_use]
    pub const fn memory_required(self) -> bool {
        self.memory_required
    }
}

/// Resolves required guidance/policy independently from optional memory controls.
#[must_use]
// r[impl agent_harness_runtime.openai_codex_opt_in_memories.non_authoritative]
pub fn resolve_required_rules(input: RequiredRuleInput<'_>) -> RequiredRuleResolution {
    let _ = input.memory_policy.memory_use_enabled();
    RequiredRuleResolution {
        required_rules_active: input.guidance_present || input.policy_present,
        memory_required: false,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MemoryInjectionDecision {
    state: MemoryInjectionState,
    reason: Option<MemoryInjectionReason>,
    consumed_units: u32,
}

fn injection_decision(
    record: &OptInMemoryRecord,
    policy: &OptInMemoryPolicy,
    budget: MemoryInjectionBudget,
    used_units: u32,
) -> MemoryInjectionDecision {
    assert!(record.budget_units().get() > u32::default());
    assert!(budget.units().get() > u32::default());
    if !policy.memory_use_enabled() {
        return memory_injection_hidden(
            MemoryInjectionState::Skipped,
            MemoryInjectionReason::MemoriesDisabled,
        );
    }
    if record.freshness() == MemoryFreshness::Stale {
        return memory_injection_hidden(
            MemoryInjectionState::Expired,
            MemoryInjectionReason::StaleRecord,
        );
    }
    if record.redaction_status() == MemoryRecordRedactionStatus::Redacted {
        return memory_injection_hidden(
            MemoryInjectionState::Redacted,
            MemoryInjectionReason::RedactedByPolicy,
        );
    }
    let remaining_units = budget.units().get().saturating_sub(used_units);
    let requested_units = record.budget_units().get();
    if requested_units <= remaining_units {
        return MemoryInjectionDecision {
            state: MemoryInjectionState::Included,
            reason: None,
            consumed_units: requested_units,
        };
    }
    memory_injection_hidden(
        MemoryInjectionState::OverBudget,
        MemoryInjectionReason::OverBudget,
    )
}

fn memory_injection_hidden(
    state: MemoryInjectionState,
    reason: MemoryInjectionReason,
) -> MemoryInjectionDecision {
    MemoryInjectionDecision {
        state,
        reason: Some(reason),
        consumed_units: u32::default(),
    }
}

fn denied_generation(reason: MemoryGenerationDenialReason) -> MemoryGenerationDecision {
    MemoryGenerationDecision {
        status: MemoryGenerationStatus::Denied,
        denial_reason: Some(reason),
    }
}

fn compare_opt_in_records(
    left: &OptInMemoryRecord,
    right: &OptInMemoryRecord,
) -> core::cmp::Ordering {
    left.scope()
        .cmp(&right.scope())
        .then(left.kind().cmp(&right.kind()))
        .then(left.id.cmp(&right.id))
}

fn prompt_source_for_descriptor(
    descriptor: &MemoryDescriptor,
    policy: &MemoryPromptPolicy,
) -> Result<super::prompt_assembly::PromptSource, MemoryError> {
    let admission_status = memory_prompt_admission(descriptor, policy);
    let prompt_source = PromptSourceDraft {
        kind: PromptSourceKind::Memory,
        scope: descriptor.trust_scope().prompt_scope(),
        trust_status: memory_prompt_trust(descriptor),
        admission_status,
        requirement: PromptSourceRequirement::Optional,
        priority: PromptSourcePriority::new(descriptor.precedence().rank()),
        budget_class: PromptBudgetClass::Normal,
        redaction_class: descriptor.redaction_class().prompt_redaction(),
        label: descriptor.id(),
        section: PromptSectionKind::Context,
        body_hash: descriptor.body_hash().as_str(),
        requested_units: descriptor.budget_units().get(),
    };
    super::prompt_assembly::PromptSource::validate(&prompt_source).map_err(MemoryError::from)
}

fn memory_prompt_admission(
    descriptor: &MemoryDescriptor,
    policy: &MemoryPromptPolicy,
) -> PromptSourceAdmissionStatus {
    if descriptor.admission_status() != MemoryAdmissionStatus::Admitted {
        return descriptor.admission_status().prompt_admission();
    }
    if descriptor.inclusion_policy() == MemoryInclusionPolicy::Never {
        return PromptSourceAdmissionStatus::Denied;
    }
    if descriptor.freshness() == MemoryFreshness::Stale && !policy.include_stale() {
        return PromptSourceAdmissionStatus::Denied;
    }
    PromptSourceAdmissionStatus::Admitted
}

fn memory_prompt_trust(descriptor: &MemoryDescriptor) -> PromptSourceTrustStatus {
    match descriptor.admission_status() {
        MemoryAdmissionStatus::Admitted => PromptSourceTrustStatus::Trusted,
        MemoryAdmissionStatus::Denied | MemoryAdmissionStatus::Unavailable => {
            PromptSourceTrustStatus::Untrusted
        }
    }
}

fn required_trust_scope(
    descriptor_id: &str,
    trust_scope: Option<MemoryTrustScope>,
) -> Result<MemoryTrustScope, MemoryError> {
    trust_scope.ok_or_else(|| MemoryError::MissingTrustScope {
        descriptor_id: String::from(descriptor_id),
    })
}

fn required_inclusion_policy(
    descriptor_id: &str,
    inclusion_policy: Option<MemoryInclusionPolicy>,
) -> Result<MemoryInclusionPolicy, MemoryError> {
    inclusion_policy.ok_or_else(|| MemoryError::MissingInclusionPolicy {
        descriptor_id: String::from(descriptor_id),
    })
}

fn reject_expired_descriptor(
    descriptor_id: &str,
    freshness: MemoryFreshness,
) -> Result<(), MemoryError> {
    if freshness == MemoryFreshness::Expired {
        return Err(MemoryError::ExpiredDescriptor {
            descriptor_id: String::from(descriptor_id),
        });
    }
    Ok(())
}

fn reject_admission_leaks(draft: &MemoryAdmissionDecisionDraft<'_>) -> Result<(), MemoryError> {
    if draft.raw_candidate_body.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::RawCandidateBody));
    }
    if draft.credential.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::Credential));
    }
    if draft.provider_payload.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::ProviderPayload));
    }
    if draft.full_path.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::FullPath));
    }
    assert!(draft.raw_candidate_body.is_none());
    assert!(draft.credential.is_none());
    assert!(draft.provider_payload.is_none());
    assert!(draft.full_path.is_none());
    Ok(())
}

fn reject_receipt_leaks(draft: &MemoryInfluenceReceiptDraft<'_>) -> Result<(), MemoryError> {
    if draft.raw_memory_body.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
    }
    if draft.credential.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::Credential));
    }
    if draft.provider_payload.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::ProviderPayload));
    }
    if draft.full_path.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::FullPath));
    }
    if draft.unbounded_prompt_text.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::UnboundedPromptText));
    }
    assert!(draft.raw_memory_body.is_none());
    assert!(draft.credential.is_none());
    assert!(draft.provider_payload.is_none());
    assert!(draft.full_path.is_none());
    assert!(draft.unbounded_prompt_text.is_none());
    Ok(())
}

fn memory_leak(leak: MemoryReceiptLeak) -> MemoryError {
    MemoryError::ReceiptLeak { leak }
}

fn normalize(field: crate::validation::FieldName, value: &str) -> Result<String, MemoryError> {
    crate::validation::normalized_adapter_field(field, value).map_err(MemoryError::Contract)
}

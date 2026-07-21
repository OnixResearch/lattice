/// Assembles prompt sections and safe ledger entries from in-memory sources.
///
/// # Errors
///
/// Returns [`PromptAssemblyError`] when the source count exceeds the adapter bound.
// r[impl agent_harness_runtime.prompt_assembly.sources]
// r[impl agent_harness_runtime.prompt_assembly.precedence]
// r[impl agent_harness_runtime.prompt_assembly.ledger]
// r[impl agent_harness_runtime.prompt_assembly.discovery]
pub fn assemble_prompt_sources(
    sources: &[PromptSource],
    policy: &PromptAssemblyPolicy,
) -> Result<PromptAssembly, PromptAssemblyError> {
    if sources.len() > MAX_PROMPT_SOURCES {
        return Err(PromptAssemblyError::TooManyItems {
            field: SOURCE_LABEL_FIELD.as_str(),
            max_items: MAX_PROMPT_SOURCES,
            actual_items: sources.len(),
        });
    }

    let mut ordered: Vec<&PromptSource> = sources.iter().collect();
    ordered.sort_by(|left, right| compare_sources(left, right, policy));

    let mut usages: Vec<SectionUsage> = Vec::with_capacity(policy.section_order().len());
    let mut selected: Vec<SelectedSource> = Vec::with_capacity(sources.len());
    let mut sections = Vec::with_capacity(sources.len());
    let mut ledger = Vec::with_capacity(sources.len());

    for source in ordered {
        let decision = prompt_source_decision(source, policy, &selected, &usages);
        if decision.visible {
            add_section_usage(&mut usages, source.section(), decision.consumed_units);
            selected.push(SelectedSource {
                section: source.section(),
                priority: source.priority(),
            });
            sections.push(PromptAssemblySection {
                section: source.section(),
                source_label: source.label.clone(),
                body_hash: source.body_hash.clone(),
                consumed_units: decision.consumed_units,
                truncated: decision.state == PromptAssemblyLedgerState::Truncated,
            });
        }
        ledger.push(PromptAssemblyLedgerEntry {
            source_kind: source.kind(),
            source_label: source.label.clone(),
            trust_status: source.trust_status(),
            admission_status: source.admission_status(),
            section: source.section(),
            budget_class: source.budget_class(),
            requested_units: source.requested_units(),
            consumed_units: decision.consumed_units,
            body_hash: source.body_hash.clone(),
            state: decision.state,
            reason: decision.reason,
        });
    }

    Ok(PromptAssembly {
        sections,
        ledger,
        policy_hash: policy.policy_hash.clone(),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PromptSourceDecision {
    state: PromptAssemblyLedgerState,
    reason: Option<PromptAssemblyLedgerReason>,
    consumed_units: u32,
    visible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SectionUsage {
    section: PromptSectionKind,
    used_units: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SelectedSource {
    section: PromptSectionKind,
    priority: PromptSourcePriority,
}

fn prompt_source_decision(
    source: &PromptSource,
    policy: &PromptAssemblyPolicy,
    selected: &[SelectedSource],
    usages: &[SectionUsage],
) -> PromptSourceDecision {
    if source.kind() == PromptSourceKind::Unsupported {
        return hidden_decision(
            PromptAssemblyLedgerState::Unsupported,
            PromptAssemblyLedgerReason::UnsupportedSourceKind,
        );
    }
    if policy.mode() == PromptAssemblyMode::HostContextOnly
        && source.kind().requires_discovery_adapter()
    {
        return hidden_decision(
            PromptAssemblyLedgerState::Unsupported,
            PromptAssemblyLedgerReason::DiscoveryUnavailable,
        );
    }
    if source.admission_status() == PromptSourceAdmissionStatus::Unavailable {
        return hidden_decision(
            PromptAssemblyLedgerState::Unsupported,
            PromptAssemblyLedgerReason::RequiredSourceUnavailable,
        );
    }
    if source.admission_status() == PromptSourceAdmissionStatus::Denied {
        return hidden_decision(
            PromptAssemblyLedgerState::Denied,
            PromptAssemblyLedgerReason::DeniedByAdmission,
        );
    }
    if source.trust_status() == PromptSourceTrustStatus::Untrusted {
        return hidden_decision(
            PromptAssemblyLedgerState::Denied,
            PromptAssemblyLedgerReason::UntrustedSource,
        );
    }
    if source.redaction_class() != PromptRedactionClass::Public {
        return hidden_decision(
            PromptAssemblyLedgerState::Redacted,
            PromptAssemblyLedgerReason::RedactedByPolicy,
        );
    }
    if has_conflict(selected, source.section(), source.priority()) {
        return hidden_decision(
            PromptAssemblyLedgerState::Conflicting,
            PromptAssemblyLedgerReason::SourceConflict,
        );
    }

    let requested_units = source.requested_units().get();
    let remaining_units = remaining_units(
        policy.section_budget_units().get(),
        usages,
        source.section(),
    );
    if requested_units <= remaining_units {
        return visible_decision(PromptAssemblyLedgerState::Included, requested_units, None);
    }
    if remaining_units > u32::default() {
        return visible_decision(
            PromptAssemblyLedgerState::Truncated,
            remaining_units,
            Some(PromptAssemblyLedgerReason::OverBudget),
        );
    }
    hidden_decision(
        PromptAssemblyLedgerState::Skipped,
        PromptAssemblyLedgerReason::OverBudget,
    )
}

fn visible_decision(
    state: PromptAssemblyLedgerState,
    consumed_units: u32,
    reason: Option<PromptAssemblyLedgerReason>,
) -> PromptSourceDecision {
    PromptSourceDecision {
        state,
        reason,
        consumed_units,
        visible: true,
    }
}

fn hidden_decision(
    state: PromptAssemblyLedgerState,
    reason: PromptAssemblyLedgerReason,
) -> PromptSourceDecision {
    PromptSourceDecision {
        state,
        reason: Some(reason),
        consumed_units: u32::default(),
        visible: false,
    }
}

fn compare_sources(
    left: &PromptSource,
    right: &PromptSource,
    policy: &PromptAssemblyPolicy,
) -> core::cmp::Ordering {
    section_index(policy, left.section())
        .cmp(&section_index(policy, right.section()))
        .then(left.priority().cmp(&right.priority()))
        .then(left.kind().cmp(&right.kind()))
        .then(left.label.cmp(&right.label))
}

fn section_index(policy: &PromptAssemblyPolicy, section: PromptSectionKind) -> usize {
    policy
        .section_order()
        .iter()
        .position(|candidate| *candidate == section)
        .unwrap_or(MAX_PROMPT_SECTIONS)
}

fn has_conflict(
    selected: &[SelectedSource],
    section: PromptSectionKind,
    priority: PromptSourcePriority,
) -> bool {
    selected
        .iter()
        .any(|candidate| candidate.section == section && candidate.priority == priority)
}

fn remaining_units(max_units: u32, usages: &[SectionUsage], section: PromptSectionKind) -> u32 {
    let used_units = usages
        .iter()
        .find(|usage| usage.section == section)
        .map_or(u32::default(), |usage| usage.used_units);
    max_units.saturating_sub(used_units)
}


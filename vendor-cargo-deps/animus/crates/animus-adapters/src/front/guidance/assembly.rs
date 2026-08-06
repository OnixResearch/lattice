use alloc::vec::Vec;

use super::GUIDANCE_NAME_FIELD;
use super::GuidanceAssembly;
use super::GuidanceBudget;
use super::GuidanceDisposition;
use super::GuidanceError;
use super::GuidanceLedgerEntry;
use super::GuidanceLedgerReason;
use super::GuidanceSource;
use super::MAX_GUIDANCE_SOURCES;

const ZERO_GUIDANCE_UNITS: u32 = 0;

/// Assembles guidance source metadata into a deterministic budget ledger.
///
/// # Errors
///
/// Returns [`GuidanceError`] when the source list exceeds the bounded adapter
/// limit or two non-redacted sources have the same scope and precedence.
// r[impl agent_harness_runtime.openai_codex_guidance_skill_budget.guidance_sources]
// r[impl agent_harness_runtime.openai_codex_guidance_skill_budget.budget_ledger]
pub fn assemble_guidance_sources(
    sources: &[GuidanceSource],
    budget: GuidanceBudget,
) -> Result<GuidanceAssembly, GuidanceError> {
    validate_guidance_source_count(sources)?;
    reject_ambiguous_guidance_precedence(sources)?;
    let ordered = ordered_sources(sources);
    Ok(GuidanceAssembly {
        ledger: build_guidance_ledger(&ordered, budget.max_units().get()),
    })
}

fn validate_guidance_source_count(sources: &[GuidanceSource]) -> Result<(), GuidanceError> {
    if sources.len() > MAX_GUIDANCE_SOURCES {
        return Err(GuidanceError::TooManyItems {
            field: GUIDANCE_NAME_FIELD.as_str(),
            max_items: MAX_GUIDANCE_SOURCES,
            actual_items: sources.len(),
        });
    }
    Ok(())
}

fn ordered_sources(sources: &[GuidanceSource]) -> Vec<&GuidanceSource> {
    let mut ordered: Vec<&GuidanceSource> = sources.iter().collect();
    ordered.sort_by(|left, right| {
        left.precedence
            .cmp(&right.precedence)
            .then(left.scope.cmp(&right.scope))
            .then(left.name.cmp(&right.name))
    });
    ordered
}

fn build_guidance_ledger(ordered: &[&GuidanceSource], budget_units: u32) -> Vec<GuidanceLedgerEntry> {
    let mut remaining_units = budget_units;
    let mut ledger = Vec::with_capacity(ordered.len());
    for source in ordered {
        let (disposition, consumed_units, reason) = guidance_budget_decision(source, remaining_units);
        remaining_units = remaining_units.saturating_sub(consumed_units);
        ledger.push(guidance_ledger_entry(source, disposition, consumed_units, reason));
    }
    ledger
}

fn guidance_ledger_entry(
    source: &GuidanceSource,
    disposition: GuidanceDisposition,
    consumed_units: u32,
    reason: Option<GuidanceLedgerReason>,
) -> GuidanceLedgerEntry {
    GuidanceLedgerEntry {
        source_name: source.name.clone(),
        source_origin: source.origin.clone(),
        content_hash: source.content_hash.clone(),
        scope: source.scope,
        precedence: source.precedence,
        disposition,
        requested_units: source.budget_units,
        consumed_units,
        reason,
    }
}

fn guidance_budget_decision(
    source: &GuidanceSource,
    remaining_units: u32,
) -> (GuidanceDisposition, u32, Option<GuidanceLedgerReason>) {
    if source.redacted {
        return redacted_decision();
    }
    let requested_units = source.budget_units.get();
    if requested_units <= remaining_units {
        return (GuidanceDisposition::Included, requested_units, None);
    }
    over_budget_decision(remaining_units)
}

const fn redacted_decision() -> (GuidanceDisposition, u32, Option<GuidanceLedgerReason>) {
    (GuidanceDisposition::Redacted, ZERO_GUIDANCE_UNITS, Some(GuidanceLedgerReason::RedactedByPolicy))
}

const fn over_budget_decision(remaining_units: u32) -> (GuidanceDisposition, u32, Option<GuidanceLedgerReason>) {
    if remaining_units > ZERO_GUIDANCE_UNITS {
        return (GuidanceDisposition::Truncated, remaining_units, Some(GuidanceLedgerReason::OverBudget));
    }
    (GuidanceDisposition::Skipped, ZERO_GUIDANCE_UNITS, Some(GuidanceLedgerReason::OverBudget))
}

fn reject_ambiguous_guidance_precedence(sources: &[GuidanceSource]) -> Result<(), GuidanceError> {
    for (left_index, left) in sources.iter().enumerate() {
        if left.redacted {
            continue;
        }
        for right in sources.iter().skip(left_index + 1) {
            if right.redacted {
                continue;
            }
            if left.scope == right.scope && left.precedence == right.precedence {
                return Err(GuidanceError::DuplicateGuidancePrecedence {
                    scope: left.scope,
                    precedence: left.precedence,
                });
            }
        }
    }
    Ok(())
}

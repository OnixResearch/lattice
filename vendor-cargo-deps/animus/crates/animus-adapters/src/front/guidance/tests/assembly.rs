use super::*;

#[test]
fn guidance_precedence_records_nested_after_root() {
    // r[verify agent_harness_runtime.openai_codex_guidance_skill_budget.guidance_sources.precedence]
    let root = source(ROOT_NAME, PromptResourceScope::Global, ROOT_PRECEDENCE, ORIGIN_ROOT);
    let nested = source_with(
        NESTED_NAME,
        PromptResourceScope::Project,
        NESTED_PRECEDENCE,
        ORIGIN_NESTED,
        SECOND_HASH,
        NESTED_UNITS,
        false,
    );
    let assembly = assemble_guidance_sources(&[nested, root], GuidanceBudget::new(nonzero(FULL_BUDGET_UNITS)))
        .expect("guidance assembly should succeed");

    assert_eq!(assembly.ledger().len(), EXPECTED_TWO_ITEMS);
    assert_eq!(assembly.ledger()[0].source_name(), ROOT_NAME);
    assert_eq!(assembly.ledger()[0].disposition(), GuidanceDisposition::Included);
    assert_eq!(assembly.ledger()[1].source_name(), NESTED_NAME);
    assert_eq!(assembly.ledger()[1].source_origin(), ORIGIN_NESTED);
    assert_eq!(assembly.ledger()[1].content_hash().as_str(), SECOND_HASH);
}

#[test]
fn duplicate_guidance_precedence_fails_closed() {
    // r[verify agent_harness_runtime.openai_codex_guidance_skill_budget.guidance_sources.precedence]
    let first = source(ROOT_NAME, PromptResourceScope::Project, ROOT_PRECEDENCE, ORIGIN_ROOT);
    let second = source(NESTED_NAME, PromptResourceScope::Project, ROOT_PRECEDENCE, ORIGIN_NESTED);
    let error = assemble_guidance_sources(&[first, second], GuidanceBudget::new(nonzero(FULL_BUDGET_UNITS)))
        .expect_err("duplicate active precedence should fail");

    assert!(matches!(error, GuidanceError::DuplicateGuidancePrecedence { .. }));
    assert_eq!(error, GuidanceError::DuplicateGuidancePrecedence {
        scope: PromptResourceScope::Project,
        precedence: ROOT_PRECEDENCE,
    });
}

#[test]
fn budget_ledger_records_truncated_skipped_and_redacted_sources() {
    // r[verify agent_harness_runtime.openai_codex_guidance_skill_budget.budget_ledger.truncation]
    let root = source(ROOT_NAME, PromptResourceScope::Global, ROOT_PRECEDENCE, ORIGIN_ROOT);
    let nested = source_with(
        NESTED_NAME,
        PromptResourceScope::Project,
        NESTED_PRECEDENCE,
        ORIGIN_NESTED,
        SECOND_HASH,
        NESTED_UNITS,
        false,
    );
    let redacted = source_with(
        REDACTED_NAME,
        PromptResourceScope::Session,
        REDACTED_PRECEDENCE,
        ORIGIN_REDACTED,
        THIRD_HASH,
        REDACTED_UNITS,
        true,
    );
    let assembly =
        assemble_guidance_sources(&[root, nested, redacted], GuidanceBudget::new(nonzero(SMALL_BUDGET_UNITS)))
            .expect("budget assembly should record overage");

    assert_eq!(assembly.ledger().len(), EXPECTED_THREE_ITEMS);
    assert_eq!(assembly.ledger()[0].disposition(), GuidanceDisposition::Included);
    assert_eq!(assembly.ledger()[1].disposition(), GuidanceDisposition::Truncated);
    assert_eq!(assembly.ledger()[1].reason(), Some(GuidanceLedgerReason::OverBudget));
    assert_eq!(assembly.ledger()[2].disposition(), GuidanceDisposition::Redacted);
    assert_eq!(assembly.ledger()[2].consumed_units(), u32::default());
}

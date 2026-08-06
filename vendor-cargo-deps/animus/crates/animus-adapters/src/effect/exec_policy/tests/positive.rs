use super::*;

#[test]
fn rule_carries_validation_examples_and_hashes() {
    let rule = allow_cargo_rule();

    assert_eq!(rule.id(), ALLOW_CARGO_ID);
    assert_eq!(rule.decision(), Decision::Allow);
    assert_eq!(rule.rationale(), SAFE_RATIONALE);
    assert_eq!(rule.command_prefix(), &[Text::from("cargo")]);
    assert!(!rule.rule_hash().is_empty());
}

#[test]
fn most_restrictive_matching_decision_wins_with_all_rule_evidence() {
    let command = command(&["cargo", "test"]);
    let rules = alloc::vec![allow_cargo_rule(), forbid_cargo_test_rule()];
    let evaluation = evaluate_command(&command, rules.as_slice());

    assert_eq!(evaluation.decision(), Decision::Forbid);
    assert_eq!(evaluation.shell_analysis(), AnalysisKind::Direct);
    assert_eq!(evaluation.matched_rules().len(), rules.len());
    assert!(
        evaluation
            .matched_rules()
            .iter()
            .any(|evidence| evidence.rule_id() == ALLOW_CARGO_ID && evidence.decision() == Decision::Allow)
    );
    assert!(
        evaluation
            .matched_rules()
            .iter()
            .any(|evidence| evidence.rule_id() == FORBID_CARGO_TEST_ID && evidence.decision() == Decision::Forbid)
    );
    assert_eq!(evaluation.redaction_marker(), REDACTION_COMMAND_HASH_ONLY);
}

#[test]
fn prompt_rule_requires_approval_without_denying_by_default() {
    let command = command(&["nix", "build"]);
    let rules = alloc::vec![allow_cargo_rule(), prompt_nix_rule()];
    let evaluation = evaluate_command(&command, rules.as_slice());

    assert_eq!(evaluation.decision(), Decision::Prompt);
    assert!(!evaluation.decision().dispatch_allowed());
    assert_eq!(evaluation.matched_rules().len(), 1);
    assert_eq!(evaluation.matched_rules()[0].rule_id(), PROMPT_NIX_ID);
    assert_eq!(evaluation.sandbox().active_sandbox(), SANDBOX);
    assert_eq!(evaluation.sandbox().requested_capability(), REQUESTED_CAPABILITY);
}

#[test]
fn simple_shell_chain_evaluates_each_command_and_blocks_smuggling() {
    let command = command(CARGO_TEST_WRAPPER);
    let rules = alloc::vec![allow_cargo_rule(), forbid_rm_rule()];
    let evaluation = evaluate_command(&command, rules.as_slice());
    let rendered = alloc::format!("{evaluation:?}");

    assert_eq!(evaluation.shell_analysis(), AnalysisKind::SimpleChain);
    assert_eq!(evaluation.decision(), Decision::Forbid);
    assert_eq!(evaluation.command_fingerprints().len(), rules.len());
    assert!(evaluation.matched_rules().iter().any(|evidence| evidence.rule_id() == ALLOW_CARGO_ID));
    assert!(evaluation.matched_rules().iter().any(|evidence| evidence.rule_id() == FORBID_RM_ID));
    assert!(!rendered.contains(RAW_REMOVE_SOURCE));
}

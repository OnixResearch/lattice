use super::*;

#[test]
fn unsupported_shell_construct_is_opaque_and_forbidden_by_default() {
    let command = command(UNSUPPORTED_PIPE_WRAPPER);
    let rules = alloc::vec![allow_cargo_rule()];
    let evaluation = evaluate_command(&command, rules.as_slice());
    let rendered = alloc::format!("{evaluation:?}");

    assert_eq!(evaluation.shell_analysis(), AnalysisKind::OpaqueUnsupported);
    assert_eq!(evaluation.decision(), Decision::Forbid);
    assert!(evaluation.matched_rules().is_empty());
    assert_eq!(evaluation.command_fingerprints().len(), 1);
    assert!(!rendered.contains(RAW_PIPE_SOURCE));
}

#[test]
fn empty_prefix_and_empty_examples_are_rejected() {
    let positive = [CommandIntentDraft { argv: &["cargo"] }];
    let negative = [CommandIntentDraft { argv: &["rm"] }];
    let empty_prefix = Rule::validate(&RuleDraft {
        id: Some(ALLOW_CARGO_ID),
        decision: Some(ALLOW_LABEL),
        rationale: Some(SAFE_RATIONALE),
        command_prefix: &[],
        positive_examples: &positive,
        negative_examples: &negative,
    })
    .expect_err("empty prefix should fail");
    let empty_examples = Rule::validate(&RuleDraft {
        id: Some(ALLOW_CARGO_ID),
        decision: Some(ALLOW_LABEL),
        rationale: Some(SAFE_RATIONALE),
        command_prefix: &["cargo"],
        positive_examples: &[],
        negative_examples: &negative,
    })
    .expect_err("empty examples should fail");

    assert_eq!(
        empty_prefix,
        Error::Contract(crate::AdapterContractError::EmptyCollection {
            field: PREFIX_FIELD.as_str(),
        })
    );
    assert_eq!(
        empty_examples,
        Error::Contract(crate::AdapterContractError::EmptyCollection {
            field: POSITIVE_EXAMPLES_FIELD.as_str(),
        })
    );
}

#[test]
fn unsupported_decision_and_missing_fields_are_rejected() {
    let positive = [CommandIntentDraft { argv: &["cargo"] }];
    let negative = [CommandIntentDraft { argv: &["rm"] }];
    let bad_decision_error = Rule::validate(&RuleDraft {
        id: Some(ALLOW_CARGO_ID),
        decision: Some("maybe"),
        rationale: Some(SAFE_RATIONALE),
        command_prefix: &["cargo"],
        positive_examples: &positive,
        negative_examples: &negative,
    })
    .expect_err("unsupported decision should fail");
    let missing = Rule::validate(&RuleDraft {
        id: None,
        decision: Some(ALLOW_LABEL),
        rationale: Some(SAFE_RATIONALE),
        command_prefix: &["cargo"],
        positive_examples: &positive,
        negative_examples: &negative,
    })
    .expect_err("missing id should fail");

    assert_eq!(bad_decision_error, Error::UnsupportedDecision {
        value: Text::from("maybe"),
    });
    assert_eq!(missing, Error::MissingField {
        field: RULE_ID_FIELD.as_str(),
    });
}

#[test]
fn rule_examples_must_prove_match_and_not_match_boundaries() {
    let positive_mismatch = Rule::validate(&RuleDraft {
        id: Some(ALLOW_CARGO_ID),
        decision: Some(ALLOW_LABEL),
        rationale: Some(SAFE_RATIONALE),
        command_prefix: &["cargo"],
        positive_examples: &[CommandIntentDraft { argv: &["rm"] }],
        negative_examples: &[CommandIntentDraft { argv: &["rm"] }],
    })
    .expect_err("positive example must match");
    let negative_match = Rule::validate(&RuleDraft {
        id: Some(ALLOW_CARGO_ID),
        decision: Some(ALLOW_LABEL),
        rationale: Some(SAFE_RATIONALE),
        command_prefix: &["cargo"],
        positive_examples: &[CommandIntentDraft { argv: &["cargo"] }],
        negative_examples: &[CommandIntentDraft {
            argv: &["cargo", "test"],
        }],
    })
    .expect_err("negative example must not match");

    assert!(matches!(positive_mismatch, Error::PositiveExampleMismatch { .. }));
    assert!(matches!(negative_match, Error::NegativeExampleMatched { .. }));
}

#![allow(
    unknown_lints,
    clippy::expect_used,
    assertion_density,
    non_trait_imports,
    no_unwrap,
    reason = "checked-in rule-table fixtures must stop at the first malformed or drifted row"
)]

use std::collections::BTreeSet;

use lattice_workflow_transport::*;
use serde::Deserialize;

const RULE_FIXTURES: &str = include_str!("../fixtures/exchange-recovery-fixtures-v1.json");
const FIXTURE_SCHEMA: &str = "lattice.workflow-exchange-recovery-fixtures.v1";
const OUTCOME_COUNT: usize = 16;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    command: CommandKind,
    outcome: ExchangeOutcome,
    same_incarnation: bool,
    step: RecoveryStep,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlRule {
    expected_sequence: u64,
    observed_sequence: u64,
    step: RecoveryStep,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RefusedRule {
    name: String,
    command: CommandKind,
    outcome: ExchangeOutcome,
    same_incarnation: bool,
    forbidden_step: RecoveryStep,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RefusedControlRule {
    name: String,
    expected_sequence: u64,
    observed_sequence: u64,
    forbidden_step: RecoveryStep,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixtures {
    schema: String,
    table: String,
    non_claims: Vec<String>,
    rules: Vec<Rule>,
    control_after_observe: Vec<ControlRule>,
    refused: Vec<RefusedRule>,
    refused_control_after_observe: Vec<RefusedControlRule>,
}

fn fixtures() -> Fixtures {
    serde_json::from_str(RULE_FIXTURES).expect("rule-table fixtures decode")
}

// Independent oracle: the C6 table as written in the relocation plan, row by row.
fn planned_step(command: CommandKind, outcome: ExchangeOutcome, same_incarnation: bool) -> Option<RecoveryStep> {
    let unknown = ExchangeOutcome::Failure(FailureClass::UnknownAfterWrite);
    let not_accepted = [
        ExchangeOutcome::Failure(FailureClass::NotAccepted),
        ExchangeOutcome::Failure(FailureClass::ConnectFailed),
    ];
    let read_only = [CommandKind::Observe, CommandKind::ReadLog, CommandKind::Reconcile];
    if command == CommandKind::Dispatch && not_accepted.contains(&outcome) {
        return Some(RecoveryStep::ResendIdentical);
    }
    if command == CommandKind::Dispatch && outcome == unknown {
        return Some(RecoveryStep::RecoverDispatch);
    }
    if command == CommandKind::RecoverDispatch && outcome == ExchangeOutcome::Rejected(RejectionCode::NotFound) {
        return Some(if same_incarnation {
            RecoveryStep::ResendDispatch
        } else {
            RecoveryStep::StopUnknown
        });
    }
    if command == CommandKind::Control && not_accepted.contains(&outcome) {
        return Some(RecoveryStep::ResendIdentical);
    }
    if command == CommandKind::Control && outcome == unknown {
        return Some(RecoveryStep::ObserveThenCompare);
    }
    if read_only.contains(&command) && (not_accepted.contains(&outcome) || outcome == unknown) {
        return Some(RecoveryStep::RetryBounded);
    }
    None
}

// r[verify lattice.workflow_interchange.reconnect_rules]
// r[verify lattice.workflow_interchange.reconnect_rules.scenario.fixtures]
#[test]
fn checked_in_rule_table_matches_the_pure_rules_exhaustively() {
    let fixtures = fixtures();
    assert_eq!(fixtures.schema, FIXTURE_SCHEMA);
    assert_eq!(fixtures.table, EXCHANGE_RECOVERY_SCHEMA);
    assert!(!fixtures.non_claims.is_empty());
    let mut seen = BTreeSet::new();
    for rule in &fixtures.rules {
        assert_eq!(next_step(rule.command, rule.outcome, rule.same_incarnation), rule.step);
        let key = (rule.command, serde_json::to_string(&rule.outcome).expect("outcome encodes"), rule.same_incarnation);
        assert!(seen.insert(key), "duplicate rule row");
    }
    assert_eq!(seen.len(), CommandKind::ALL.len() * OUTCOME_COUNT * 2);
    for rule in &fixtures.control_after_observe {
        assert_eq!(control_after_observe(rule.expected_sequence, rule.observed_sequence), rule.step);
    }
}

// r[verify lattice.workflow_interchange.reconnect_rules]
#[test]
fn checked_in_rule_table_matches_the_planned_c6_rows() {
    let mut planned_rows = 0_usize;
    for rule in fixtures().rules {
        if let Some(expected) = planned_step(rule.command, rule.outcome, rule.same_incarnation) {
            assert_eq!(rule.step, expected);
            planned_rows = planned_rows.saturating_add(1);
        }
    }
    assert!(planned_rows > 0);
}

// r[verify lattice.workflow_interchange.reconnect_rules.scenario.refused]
#[test]
fn refused_rule_rows_are_never_produced() {
    let fixtures = fixtures();
    assert!(!fixtures.refused.is_empty());
    for refused in &fixtures.refused {
        assert_ne!(
            next_step(refused.command, refused.outcome, refused.same_incarnation),
            refused.forbidden_step,
            "{}",
            refused.name
        );
    }
    assert!(!fixtures.refused_control_after_observe.is_empty());
    for refused in &fixtures.refused_control_after_observe {
        assert_ne!(
            control_after_observe(refused.expected_sequence, refused.observed_sequence),
            refused.forbidden_step,
            "{}",
            refused.name
        );
    }
}

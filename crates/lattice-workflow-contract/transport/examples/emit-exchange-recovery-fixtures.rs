#![allow(
    unknown_lints,
    clippy::expect_used,
    non_trait_imports,
    no_unwrap,
    assertion_density,
    allocation_in_loop,
    unbounded_collection_growth,
    reason = "the rule-table fixture generator enumerates the closed command and outcome classes once"
)]

use lattice_workflow_transport::*;
use serde::Serialize;

const FIXTURE_SCHEMA: &str = "lattice.workflow-exchange-recovery-fixtures.v1";
const NON_CLAIMS: [&str; 4] = [
    "the rule table does not prove which endpoint incarnation answered",
    "same_incarnation is gateway evidence and never Lattice wire data",
    "retry bounds and backoff are owned by the caller",
    "a recovery step does not prove that any workflow effect happened",
];

#[derive(Serialize)]
struct Rule {
    command: CommandKind,
    outcome: ExchangeOutcome,
    same_incarnation: bool,
    step: RecoveryStep,
}

#[derive(Serialize)]
struct ControlRule {
    expected_sequence: u64,
    observed_sequence: u64,
    step: RecoveryStep,
}

#[derive(Serialize)]
struct RefusedRule {
    name: &'static str,
    command: CommandKind,
    outcome: ExchangeOutcome,
    same_incarnation: bool,
    forbidden_step: RecoveryStep,
}

#[derive(Serialize)]
struct RefusedControlRule {
    name: &'static str,
    expected_sequence: u64,
    observed_sequence: u64,
    forbidden_step: RecoveryStep,
}

#[derive(Serialize)]
struct Fixtures {
    schema: &'static str,
    table: &'static str,
    non_claims: [&'static str; 4],
    rules: Vec<Rule>,
    control_after_observe: Vec<ControlRule>,
    refused: Vec<RefusedRule>,
    refused_control_after_observe: Vec<RefusedControlRule>,
}

fn outcomes() -> Vec<ExchangeOutcome> {
    let failures = FailureClass::ALL.into_iter().map(ExchangeOutcome::Failure);
    let rejections = REJECTION_CODES.into_iter().map(ExchangeOutcome::Rejected);
    failures.chain([ExchangeOutcome::Success]).chain(rejections).collect()
}

fn rules() -> Vec<Rule> {
    let mut rules = Vec::new();
    for command in CommandKind::ALL {
        for outcome in outcomes() {
            for same_incarnation in [false, true] {
                rules.push(Rule {
                    command,
                    outcome,
                    same_incarnation,
                    step: next_step(command, outcome, same_incarnation),
                });
            }
        }
    }
    rules
}

fn control_rules() -> Vec<ControlRule> {
    [(7, 7), (7, 8), (7, 9), (7, 6), (0, 0)]
        .into_iter()
        .map(|(expected_sequence, observed_sequence)| ControlRule {
            expected_sequence,
            observed_sequence,
            step: control_after_observe(expected_sequence, observed_sequence),
        })
        .collect()
}

fn refused() -> Vec<RefusedRule> {
    let unknown = ExchangeOutcome::Failure(FailureClass::UnknownAfterWrite);
    let not_found = ExchangeOutcome::Rejected(RejectionCode::NotFound);
    vec![
        RefusedRule {
            name: "unknown dispatch is never blindly resent",
            command: CommandKind::Dispatch,
            outcome: unknown,
            same_incarnation: true,
            forbidden_step: RecoveryStep::ResendIdentical,
        },
        RefusedRule {
            name: "unknown dispatch never redispatches before recovery",
            command: CommandKind::Dispatch,
            outcome: unknown,
            same_incarnation: true,
            forbidden_step: RecoveryStep::ResendDispatch,
        },
        RefusedRule {
            name: "missing acceptance on a new incarnation never redispatches",
            command: CommandKind::RecoverDispatch,
            outcome: not_found,
            same_incarnation: false,
            forbidden_step: RecoveryStep::ResendDispatch,
        },
        RefusedRule {
            name: "unknown control is never resent before observation",
            command: CommandKind::Control,
            outcome: unknown,
            same_incarnation: true,
            forbidden_step: RecoveryStep::ResendIdentical,
        },
        RefusedRule {
            name: "unknown is never reported as complete",
            command: CommandKind::Observe,
            outcome: unknown,
            same_incarnation: true,
            forbidden_step: RecoveryStep::Complete,
        },
    ]
}

fn refused_control() -> Vec<RefusedControlRule> {
    vec![
        RefusedControlRule {
            name: "a control whose expected sequence is behind is never resent",
            expected_sequence: 7,
            observed_sequence: 8,
            forbidden_step: RecoveryStep::ResendIdentical,
        },
        RefusedControlRule {
            name: "an observation older than the control basis never licenses a resend",
            expected_sequence: 7,
            observed_sequence: 6,
            forbidden_step: RecoveryStep::ResendIdentical,
        },
    ]
}

fn main() {
    let fixtures = Fixtures {
        schema: FIXTURE_SCHEMA,
        table: EXCHANGE_RECOVERY_SCHEMA,
        non_claims: NON_CLAIMS,
        rules: rules(),
        control_after_observe: control_rules(),
        refused: refused(),
        refused_control_after_observe: refused_control(),
    };
    println!("{}", serde_json::to_string_pretty(&fixtures).expect("recovery rule fixtures serialize"));
}

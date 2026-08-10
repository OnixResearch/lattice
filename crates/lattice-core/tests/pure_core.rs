use std::collections::HashMap;

use lattice_core::content_identity::ScheduleOccurrenceInput;
use lattice_core::content_identity::schedule_occurrence;
use lattice_core::control_flow::ConditionBranch;
use lattice_core::control_flow::ConditionNodeConfig;
use lattice_core::control_flow::ControlFlowError;
use lattice_core::control_flow::evaluate_condition;
use lattice_core::control_flow::parse_condition_predicate;
use lattice_core::expressions::ExpressionContext;
use lattice_core::expressions::evaluate_expression_template;
use lattice_core::expressions::parse_expression_template;
use lattice_core::value::ContentHash;
use lattice_core::value::Value;
use rat_nodegraph::model::PortId;

const NODE_ID: u64 = 7;
const PORT_ID: u64 = 11;
const SCHEDULE_ID: &str = "nightly";
const FIRST_DUE_EPOCH_MS: i64 = 1_000;
const SECOND_DUE_EPOCH_MS: i64 = 2_000;
const READY_VALUE: &str = "ready";

#[test]
fn expression_evaluation_reads_typed_context_values() {
    let mut context = ExpressionContext::new(HashMap::new());
    context.insert(NODE_ID, PortId(PORT_ID), Value::from_string("  ready  ".to_owned()));
    let source = format!("{{{{node:{NODE_ID}:port:{PORT_ID}|trim|upper}}}}");
    let template = parse_expression_template(&source).expect("valid expression");

    let evaluated = evaluate_expression_template(&template, &context).expect("expression evaluation");

    assert_eq!(evaluated.text, "READY");
}

#[test]
fn malformed_expression_fails_closed() {
    let source = format!("{{{{node:not-a-number:port:{PORT_ID}}}}}");
    assert!(parse_expression_template(&source).is_err());
}

#[test]
fn condition_evaluation_accepts_valid_input_and_rejects_bad_predicates() {
    let config = ConditionNodeConfig {
        branches: vec![ConditionBranch {
            label: READY_VALUE.to_owned(),
            predicate: parse_condition_predicate("eq:ready").expect("valid predicate"),
        }],
    };

    let matches = evaluate_condition(&config, Some(&Value::from_string(READY_VALUE.to_owned())));
    let malformed = parse_condition_predicate("gt:not-a-number");

    assert_eq!(matches, vec![READY_VALUE.to_owned()]);
    assert!(matches!(malformed, Err(ControlFlowError::InvalidPredicate(_))));
}

#[test]
fn identities_and_content_hashes_are_deterministic_and_input_sensitive() {
    let first_identity = schedule_occurrence(ScheduleOccurrenceInput {
        schedule_id: SCHEDULE_ID,
        due_epoch_ms: FIRST_DUE_EPOCH_MS,
    })
    .expect("valid schedule identity");
    let repeated_identity = schedule_occurrence(ScheduleOccurrenceInput {
        schedule_id: SCHEDULE_ID,
        due_epoch_ms: FIRST_DUE_EPOCH_MS,
    })
    .expect("repeated schedule identity");
    let second_identity = schedule_occurrence(ScheduleOccurrenceInput {
        schedule_id: SCHEDULE_ID,
        due_epoch_ms: SECOND_DUE_EPOCH_MS,
    })
    .expect("changed schedule identity");
    let first_hash = ContentHash::from_bytes(b"first");
    let repeated_hash = ContentHash::from_bytes(b"first");
    let second_hash = ContentHash::from_bytes(b"second");

    assert_eq!(first_identity, repeated_identity);
    assert_ne!(first_identity, second_identity);
    assert_eq!(first_hash, repeated_hash);
    assert_ne!(first_hash, second_hash);
}

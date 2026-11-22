#![allow(
    unknown_lints,
    clippy::expect_used,
    assertion_density,
    non_trait_imports,
    no_unwrap,
    reason = "checked-in continuity fixtures must stop at the first malformed or drifted case"
)]

use std::collections::BTreeSet;

use lattice_workflow_contract::ExecutionObservation;
use lattice_workflow_contract::observation_identity;
use lattice_workflow_transport::*;
use serde::Deserialize;

const CONTINUITY_FIXTURES: &str = include_str!("../fixtures/continuity-fixtures-v1.json");
const FIXTURE_SCHEMA: &str = "lattice.workflow-observation-continuity-fixtures.v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    previous: Option<ExecutionObservation>,
    next: ExecutionObservation,
    expected: Continuity,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixtures {
    schema: String,
    check: String,
    non_claims: Vec<String>,
    cases: Vec<Case>,
}

fn outcome_label(continuity: &Continuity) -> String {
    match continuity {
        Continuity::Continuous => "continuous".to_string(),
        Continuity::Gap { .. } => "gap".to_string(),
        Continuity::Reject { reason } => serde_json::to_string(reason).expect("reason encodes"),
    }
}

// r[verify lattice.workflow_interchange.observation_continuity]
// r[verify lattice.workflow_interchange.observation_continuity.scenario.fixtures]
#[test]
fn checked_in_continuity_cases_match_the_pure_check() {
    let fixtures: Fixtures = serde_json::from_str(CONTINUITY_FIXTURES).expect("continuity fixtures decode");
    assert_eq!(fixtures.schema, FIXTURE_SCHEMA);
    assert_eq!(fixtures.check, OBSERVATION_CONTINUITY_SCHEMA);
    assert!(!fixtures.non_claims.is_empty());
    let mut outcomes = BTreeSet::new();
    for case in &fixtures.cases {
        for observation in case.previous.iter().chain([&case.next]) {
            assert_eq!(observation.identity, observation_identity(observation), "{}", case.name);
        }
        assert_eq!(continuity(case.previous.as_ref(), &case.next), case.expected, "{}", case.name);
        outcomes.insert(outcome_label(&case.expected));
    }
    let required = [
        ContinuityRejection::RequestChanged,
        ContinuityRejection::ExecutionChanged,
        ContinuityRejection::WorkflowChanged,
        ContinuityRejection::SequenceDecreased,
        ContinuityRejection::TerminalReverted,
        ContinuityRejection::CursorRegressed,
    ]
    .iter()
    .map(|reason| serde_json::to_string(reason).expect("reason encodes"))
    .chain(["continuous".to_string(), "gap".to_string()]);
    for outcome in required {
        assert!(outcomes.contains(&outcome), "fixtures do not cover {outcome}");
    }
}

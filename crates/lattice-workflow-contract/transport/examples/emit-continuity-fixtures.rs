#![allow(
    unknown_lints,
    clippy::expect_used,
    non_trait_imports,
    no_unwrap,
    assertion_density,
    reason = "the continuity fixture generator derives every case from the checked-in producer observation"
)]

use lattice_workflow_contract::ExecutionObservation;
use lattice_workflow_contract::ExecutionState;
use lattice_workflow_contract::Identity;
use lattice_workflow_contract::Label;
use lattice_workflow_contract::LogReference;
use lattice_workflow_contract::LogStream;
use lattice_workflow_contract::NodeState;
use lattice_workflow_contract::observation_identity;
use lattice_workflow_transport::*;
use serde::Deserialize;
use serde::Serialize;

const CONTRACT_FIXTURES: &str = include_str!("../../fixtures/producer-fixtures-v1.json");
const FIXTURE_SCHEMA: &str = "lattice.workflow-observation-continuity-fixtures.v1";
const RUNNING_SEQUENCE: u64 = 5;
const LOG_CHUNK_BYTES: u64 = 32;
const NON_CLAIMS: [&str; 3] = [
    "continuity does not prove that an observation is authentic or current",
    "a continuous result does not prove that no output was lost outside the referenced logs",
    "a gap names incomplete streams and does not recover their bytes",
];

#[derive(Deserialize)]
struct ContractFixtures {
    observation: ExecutionObservation,
}

#[derive(Serialize)]
struct Case {
    name: &'static str,
    previous: Option<ExecutionObservation>,
    next: ExecutionObservation,
    expected: Continuity,
}

#[derive(Serialize)]
struct Fixtures {
    schema: &'static str,
    check: &'static str,
    non_claims: [&'static str; 3],
    cases: Vec<Case>,
}

fn label(value: &str) -> Label {
    Label::new(value.to_string()).expect("fixture label is canonical")
}

fn identity(digit: char) -> Identity {
    Identity::new(format!("b3:{}", digit.to_string().repeat(64))).expect("fixture identity is canonical")
}

fn sealed(mut observation: ExecutionObservation) -> ExecutionObservation {
    observation.identity = observation_identity(&observation);
    observation
}

fn running_from(completed: &ExecutionObservation, sequence: u64) -> ExecutionObservation {
    let mut observation = completed.clone();
    observation.sequence = sequence;
    observation.state = ExecutionState::Running;
    observation.terminal = false;
    observation.successful = false;
    observation.receipt_identity = None;
    for (index, node) in observation.nodes.iter_mut().enumerate() {
        node.state = if index == 0 {
            NodeState::Running
        } else {
            NodeState::Pending
        };
        node.terminal = false;
        node.successful = false;
        node.receipt_identity = None;
    }
    sealed(observation)
}

fn with_state(base: &ExecutionObservation, sequence: u64, state: ExecutionState) -> ExecutionObservation {
    let mut observation = base.clone();
    observation.sequence = sequence;
    observation.state = state;
    observation.terminal = state.is_terminal();
    observation.successful = state == ExecutionState::Completed;
    sealed(observation)
}

fn with_cursor(base: &ExecutionObservation, sequence: u64, cursor: u64) -> ExecutionObservation {
    let mut observation = base.clone();
    observation.sequence = sequence;
    for reference in &mut observation.logs {
        reference.cursor = cursor;
    }
    sealed(observation)
}

fn with_log_flags(base: &ExecutionObservation, sequence: u64, flags: (bool, bool, bool)) -> ExecutionObservation {
    let (gap_before, retained, truncated) = flags;
    let mut observation = with_cursor(base, sequence, LOG_CHUNK_BYTES.saturating_mul(2));
    for reference in &mut observation.logs {
        reference.gap_before = gap_before;
        reference.retained = retained;
        reference.truncated = truncated;
    }
    sealed(observation)
}

fn build_stdout_gap() -> Continuity {
    Continuity::Gap {
        nodes: vec![label("build")],
        streams: vec![LogStreamKey {
            node_id: label("build"),
            stream: LogStream::Stdout,
        }],
    }
}

fn reject(reason: ContinuityRejection) -> Continuity {
    Continuity::Reject { reason }
}

#[allow(
    unknown_lints,
    function_length,
    reason = "the closed case list is reviewed as one table"
)]
fn cases(completed: &ExecutionObservation) -> Vec<Case> {
    let running = running_from(completed, RUNNING_SEQUENCE);
    let running_next = with_cursor(&running, RUNNING_SEQUENCE.saturating_add(1), LOG_CHUNK_BYTES);
    let unknown = with_state(&running, RUNNING_SEQUENCE.saturating_add(1), ExecutionState::Unknown);
    let mut changed_request = running_next.clone();
    changed_request.request_identity = identity('5');
    let mut changed_execution = running_next.clone();
    changed_execution.execution_identity = identity('6');
    let mut changed_workflow = running_next.clone();
    changed_workflow.workflow_identity = identity('7');
    let mut two_stream_gap = with_log_flags(&running, RUNNING_SEQUENCE.saturating_add(1), (true, true, false));
    two_stream_gap.logs.push(LogReference {
        node_id: label("test"),
        stream: LogStream::Stderr,
        reference_identity: identity('8'),
        cursor: 0,
        bytes: LOG_CHUNK_BYTES,
        truncated: true,
        gap_before: false,
        retained: true,
    });
    let two_stream_gap = sealed(two_stream_gap);
    let advanced = with_cursor(&running, RUNNING_SEQUENCE.saturating_add(1), LOG_CHUNK_BYTES.saturating_mul(2));
    let regressed_with_gap = with_log_flags(&advanced, RUNNING_SEQUENCE.saturating_add(2), (true, true, false));
    let mut regressed_with_gap = regressed_with_gap;
    for reference in &mut regressed_with_gap.logs {
        reference.cursor = LOG_CHUNK_BYTES;
    }
    let regressed_with_gap = sealed(regressed_with_gap);
    vec![
        Case {
            name: "first observation without history",
            previous: None,
            next: running.clone(),
            expected: Continuity::Continuous,
        },
        Case {
            name: "identical replayed observation",
            previous: Some(running.clone()),
            next: running.clone(),
            expected: Continuity::Continuous,
        },
        Case {
            name: "log cursor resumes forward",
            previous: Some(running.clone()),
            next: running_next.clone(),
            expected: Continuity::Continuous,
        },
        Case {
            name: "running execution completes",
            previous: Some(running.clone()),
            next: completed.clone(),
            expected: Continuity::Continuous,
        },
        Case {
            name: "handler restart reports unknown",
            previous: Some(running.clone()),
            next: unknown.clone(),
            expected: Continuity::Continuous,
        },
        Case {
            name: "terminal observation replayed",
            previous: Some(completed.clone()),
            next: completed.clone(),
            expected: Continuity::Continuous,
        },
        Case {
            name: "explicit gap before resumed cursor",
            previous: Some(running.clone()),
            next: with_log_flags(&running, RUNNING_SEQUENCE.saturating_add(1), (true, true, false)),
            expected: build_stdout_gap(),
        },
        Case {
            name: "log no longer retained",
            previous: Some(running.clone()),
            next: with_log_flags(&running, RUNNING_SEQUENCE.saturating_add(1), (false, false, false)),
            expected: build_stdout_gap(),
        },
        Case {
            name: "log truncated",
            previous: None,
            next: with_log_flags(&running, RUNNING_SEQUENCE, (false, true, true)),
            expected: build_stdout_gap(),
        },
        Case {
            name: "gaps on two nodes",
            previous: Some(running.clone()),
            next: two_stream_gap,
            expected: Continuity::Gap {
                nodes: vec![label("build"), label("test")],
                streams: vec![
                    LogStreamKey {
                        node_id: label("build"),
                        stream: LogStream::Stdout,
                    },
                    LogStreamKey {
                        node_id: label("test"),
                        stream: LogStream::Stderr,
                    },
                ],
            },
        },
        Case {
            name: "request identity changed",
            previous: Some(running.clone()),
            next: sealed(changed_request),
            expected: reject(ContinuityRejection::RequestChanged),
        },
        Case {
            name: "execution identity changed",
            previous: Some(running.clone()),
            next: sealed(changed_execution),
            expected: reject(ContinuityRejection::ExecutionChanged),
        },
        Case {
            name: "workflow identity changed",
            previous: Some(running.clone()),
            next: sealed(changed_workflow),
            expected: reject(ContinuityRejection::WorkflowChanged),
        },
        Case {
            name: "sequence decreased",
            previous: Some(running_next.clone()),
            next: running.clone(),
            expected: reject(ContinuityRejection::SequenceDecreased),
        },
        Case {
            name: "terminal reverted to running",
            previous: Some(completed.clone()),
            next: running_from(completed, completed.sequence.saturating_add(1)),
            expected: reject(ContinuityRejection::TerminalReverted),
        },
        Case {
            name: "terminal outcome changed",
            previous: Some(completed.clone()),
            next: with_state(completed, completed.sequence.saturating_add(1), ExecutionState::Failed),
            expected: reject(ContinuityRejection::TerminalReverted),
        },
        Case {
            name: "terminal became unknown",
            previous: Some(completed.clone()),
            next: with_state(completed, completed.sequence.saturating_add(1), ExecutionState::Unknown),
            expected: reject(ContinuityRejection::TerminalReverted),
        },
        Case {
            name: "log cursor moved backwards",
            previous: Some(advanced.clone()),
            next: with_cursor(&advanced, RUNNING_SEQUENCE.saturating_add(2), LOG_CHUNK_BYTES),
            expected: reject(ContinuityRejection::CursorRegressed),
        },
        Case {
            name: "rejection takes precedence over a gap",
            previous: Some(advanced),
            next: regressed_with_gap,
            expected: reject(ContinuityRejection::CursorRegressed),
        },
    ]
}

fn main() {
    let contract: ContractFixtures = serde_json::from_str(CONTRACT_FIXTURES).expect("contract fixtures decode");
    let completed = contract.observation;
    let cases = cases(&completed);
    for case in &cases {
        assert_eq!(continuity(case.previous.as_ref(), &case.next), case.expected, "{}", case.name);
    }
    let fixtures = Fixtures {
        schema: FIXTURE_SCHEMA,
        check: OBSERVATION_CONTINUITY_SCHEMA,
        non_claims: NON_CLAIMS,
        cases,
    };
    println!("{}", serde_json::to_string_pretty(&fixtures).expect("continuity fixtures serialize"));
}

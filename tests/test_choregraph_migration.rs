//! Choregraph persisted-state and replay migration checks.

mod common;

use std::collections::BTreeMap;

use choregraph_core::DenialReason;
use choregraph_core::LocalState;
use choregraph_core::ReplayDecision;
use choregraph_core::ReplayFailureReason;
use lattice::engine::runner::PersistedMachineBinding;
use lattice::engine::runner::PersistedMachineEvent;
use lattice::engine::runner::PersistedReplayIdentity;
use lattice::engine::runner::PersistedStateIssue;
use lattice::engine::runner::PersistedStepState;
use lattice::engine::runner::PersistedStepStateRecord;
use lattice::engine::runner::RunnerStepAction;
use lattice::engine::runner::RunnerStepState;
use lattice::engine::runner::build_runner_plan;
use lattice::engine::runner::migrate_persisted_states;
use lattice::engine::runner::replay_persisted_events;
use pretty_assertions::assert_eq;

const WORKFLOW_REVISION: &str = "workflow-revision";
const MACHINE_IDENTITY: &str = "blake3:machine-identity";
const OTHER_WORKFLOW_REVISION: &str = "other-workflow-revision";
const OTHER_MACHINE_IDENTITY: &str = "blake3:other-machine-identity";
const UNSUPPORTED_SCHEMA: &str = "unsupported.state.v0";
const FOREIGN_NODE_ID: u64 = 999_777;
const MACHINE_BINDING_BYTE: u8 = 1;
const INITIAL_STATE_BINDING_BYTE: u8 = 2;
const STALE_BINDING_BYTE: u8 = 3;
const REPLAY_EVENT_COUNT: u64 = 4;

fn binding() -> PersistedMachineBinding {
    PersistedMachineBinding::expected(WORKFLOW_REVISION, MACHINE_IDENTITY)
}

fn record(binding: &PersistedMachineBinding, node_id: u64, state: PersistedStepState) -> PersistedStepStateRecord {
    PersistedStepStateRecord {
        binding: binding.clone(),
        node_id,
        state,
    }
}

#[test]
// r[verify lattice.choregraph_adoption.persistence]
fn persisted_state_migration_accepts_all_supported_states() {
    let (workflow, source, target) = common::two_node_pipeline();
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");
    let binding = binding();
    let cases = [
        (PersistedStepState::Pending, RunnerStepState::Pending),
        (PersistedStepState::Ready, RunnerStepState::Ready),
        (PersistedStepState::Running, RunnerStepState::Running),
        (PersistedStepState::Completed, RunnerStepState::Completed),
        (PersistedStepState::Failed, RunnerStepState::Failed),
        (PersistedStepState::Cancelled, RunnerStepState::Cancelled),
    ];

    for (persisted, expected) in cases {
        let states = migrate_persisted_states(&plan, &binding, &[
            record(&binding, source.0, persisted),
            record(&binding, target.0, persisted),
        ])
        .expect("supported state migrates");
        assert_eq!(states.get(&source.0), Some(&expected));
        assert_eq!(states.get(&target.0), Some(&expected));
    }
}

#[test]
// r[verify lattice.choregraph_adoption.persistence]
fn persisted_state_migration_rejects_stale_foreign_partial_and_ambiguous_records() {
    let (workflow, source, target) = common::two_node_pipeline();
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");
    let expected = binding();

    let mut stale_workflow = expected.clone();
    stale_workflow.workflow_revision = OTHER_WORKFLOW_REVISION.to_string();
    assert_eq!(
        migrate_persisted_states(&plan, &expected, &[
            record(&stale_workflow, source.0, PersistedStepState::Pending),
            record(&stale_workflow, target.0, PersistedStepState::Pending),
        ],),
        Err(PersistedStateIssue::StaleWorkflowRevision)
    );

    let mut stale_machine = expected.clone();
    stale_machine.machine_identity = OTHER_MACHINE_IDENTITY.to_string();
    assert_eq!(
        migrate_persisted_states(&plan, &expected, &[
            record(&stale_machine, source.0, PersistedStepState::Pending),
            record(&stale_machine, target.0, PersistedStepState::Pending),
        ],),
        Err(PersistedStateIssue::StaleMachineIdentity)
    );

    let mut unsupported = expected.clone();
    unsupported.old_schema = UNSUPPORTED_SCHEMA.to_string();
    assert_eq!(
        migrate_persisted_states(&plan, &expected, &[
            record(&unsupported, source.0, PersistedStepState::Pending),
            record(&unsupported, target.0, PersistedStepState::Pending),
        ],),
        Err(PersistedStateIssue::UnsupportedSchema)
    );

    assert_eq!(
        migrate_persisted_states(&plan, &expected, &[record(&expected, source.0, PersistedStepState::Pending)],),
        Err(PersistedStateIssue::PartialState)
    );
    assert_eq!(
        migrate_persisted_states(&plan, &expected, &[
            record(&expected, source.0, PersistedStepState::Pending),
            record(&expected, FOREIGN_NODE_ID, PersistedStepState::Pending),
        ],),
        Err(PersistedStateIssue::ForeignNode)
    );
    assert_eq!(
        migrate_persisted_states(&plan, &expected, &[
            record(&expected, source.0, PersistedStepState::Pending),
            record(&expected, source.0, PersistedStepState::Ready),
        ],),
        Err(PersistedStateIssue::AmbiguousNode)
    );
}

#[test]
// r[verify lattice.choregraph_adoption.replay]
fn replay_reconstructs_state_without_intents_and_rejects_stale_bindings() {
    let (workflow, source, _target) = common::two_node_pipeline();
    let plan = build_runner_plan(&workflow).plan.expect("runner plan builds");
    let states = BTreeMap::new();
    let events = [
        PersistedMachineEvent {
            node_id: source.0,
            action: RunnerStepAction::MarkReady,
        },
        PersistedMachineEvent {
            node_id: source.0,
            action: RunnerStepAction::Start,
        },
        PersistedMachineEvent {
            node_id: source.0,
            action: RunnerStepAction::Complete,
        },
        PersistedMachineEvent {
            node_id: source.0,
            action: RunnerStepAction::Cancel,
        },
    ];
    let identity = PersistedReplayIdentity {
        expected_machine: [MACHINE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES],
        expected_initial_state: [INITIAL_STATE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES],
        observed_machine: [MACHINE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES],
        observed_initial_state: [INITIAL_STATE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES],
    };

    let decision = replay_persisted_events(&plan, &states, identity, &events).expect("replay inputs convert");
    let ReplayDecision::Failed(failure) = decision else {
        panic!("terminal mutation must stop replay");
    };
    assert_eq!(failure.consumed.get(), REPLAY_EVENT_COUNT - 1);
    assert!(matches!(failure.reason, ReplayFailureReason::TransitionDenied(DenialReason::TerminalState { .. })));
    let source_id = choregraph_core::NodeId::parse(source.0.to_string(), choregraph_core::DEFAULT_MAX_IDENTIFIER_BYTES)
        .expect("runner id is valid");
    assert_eq!(failure.last_valid_state.state().state(&source_id), Some(LocalState::Completed));

    let stale_identity = PersistedReplayIdentity {
        observed_machine: [STALE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES],
        ..identity
    };
    let stale = replay_persisted_events(&plan, &states, stale_identity, &events).expect("stale replay inputs convert");
    assert!(matches!(
        stale,
        ReplayDecision::Failed(choregraph_core::ReplayFailure {
            reason: ReplayFailureReason::MachineBindingMismatch,
            ..
        })
    ));
}

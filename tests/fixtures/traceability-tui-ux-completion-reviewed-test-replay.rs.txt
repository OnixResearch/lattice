//! Tests for replay timeline reconstruction and bundle safety.

use std::collections::BTreeSet;

use lattice::engine::RunId;
use lattice::engine::Value;
use lattice::engine::Workflow;
use lattice::engine::runner::RunnerEventKind;
use lattice::replay::ReplayArtifactPayloadState;
use lattice::replay::ReplayArtifactReference;
use lattice::replay::ReplayBundleError;
use lattice::replay::ReplayDiagnosticCode;
use lattice::replay::ReplayEventRecord;
use lattice::replay::ReplayNodeState;
use lattice::replay::active_events;
use lattice::replay::artifact_view;
use lattice::replay::build_replay_timeline;
use lattice::replay::export_replay_bundle;
use lattice::replay::import_replay_bundle_json;
use lattice::replay::move_replay_cursor;
use lattice::replay::replay_bundle_to_pretty_json;
use lattice::replay::replay_cursor;
use lattice::replay::summarize_value;
use lattice::replay::validate_replay_events;
use lattice::store::Store;
use pretty_assertions::assert_eq;

const RUN_ID: RunId = RunId(7);
const NODE_ID: u64 = 42;
const SECOND_NODE_ID: u64 = 43;
const FIRST_SEQUENCE: u64 = 0;
const SECOND_SEQUENCE: u64 = 1;
const THIRD_SEQUENCE: u64 = 2;
const FOURTH_SEQUENCE: u64 = 3;
const FIFTH_SEQUENCE: u64 = 4;
const EXPECTED_FRAME_COUNT: usize = 5;
const CURSOR_FORWARD_DELTA: i32 = 9;
const CURSOR_BACK_DELTA: i32 = -2;
const ARTIFACT_BYTE_LENGTH: usize = 12;

fn replay_event(sequence: u64, node_id: Option<u64>, kind: RunnerEventKind) -> ReplayEventRecord {
    ReplayEventRecord {
        run_id: RUN_ID,
        sequence,
        timestamp_epoch_ms: Some(sequence as i64),
        node_id,
        port_id: None,
        parent_run_id: None,
        kind,
        summary: format!("event {sequence}"),
        redacted: false,
        value: None,
        artifact: None,
        contains_plaintext_secret_material: false,
    }
}

fn ordered_events() -> Vec<ReplayEventRecord> {
    vec![
        replay_event(FIRST_SEQUENCE, None, RunnerEventKind::RunStarted),
        replay_event(SECOND_SEQUENCE, Some(NODE_ID), RunnerEventKind::NodeReady),
        replay_event(THIRD_SEQUENCE, Some(NODE_ID), RunnerEventKind::NodeStarted),
        replay_event(FOURTH_SEQUENCE, Some(NODE_ID), RunnerEventKind::NodeCompleted),
        replay_event(FIFTH_SEQUENCE, None, RunnerEventKind::RunCompleted),
    ]
}

fn known_nodes() -> BTreeSet<u64> {
    BTreeSet::from([NODE_ID, SECOND_NODE_ID])
}

#[test]
fn active_replay_events_expose_run_start_and_ready_nodes_without_terminal_claims() {
    let mut workflow = Workflow::new("active-replay");
    let node_id = workflow.add_shell_node("ready", "sleep 1");

    let events = active_events(RUN_ID, &workflow, 0);
    let timeline = build_replay_timeline(RUN_ID, &events, None);

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind, RunnerEventKind::RunStarted);
    assert_eq!(events[1].kind, RunnerEventKind::NodeReady);
    assert_eq!(events[1].node_id, Some(node_id.0));
    assert!(!events.iter().any(|event| event.kind == RunnerEventKind::RunCompleted));
    assert!(timeline.diagnostics.iter().all(|diagnostic| !diagnostic.fatal));
    assert_eq!(
        timeline.frames.last().expect("ready frame").node_states.get(&node_id.0),
        Some(&ReplayNodeState::Ready)
    );

    let empty_events = active_events(RUN_ID, &Workflow::new("empty-active-replay"), 0);
    assert_eq!(empty_events.len(), 1);
    assert_eq!(empty_events[0].kind, RunnerEventKind::RunStarted);
}

fn artifact_reference() -> ReplayArtifactReference {
    ReplayArtifactReference {
        content_hash: "b3:artifact".to_string(),
        type_tag: "bytes".to_string(),
        media_type: Some("application/octet-stream".to_string()),
        byte_length: ARTIFACT_BYTE_LENGTH,
        backend_ref: "store://artifact".to_string(),
        sensitive: false,
        summary: "artifact bytes".to_string(),
    }
}

// r[verify n8nclone.workflow_execution.replay.core.fold]
// r[verify n8nclone.workflow_execution.replay.core.no_side_effects]
// r[verify n8nclone.workflow_execution.replay.core]
// r[verify n8nclone.tui_ux.replay_debugger.timeline]
#[test]
fn replay_timeline_folds_events_and_moves_cursor() {
    let events = ordered_events();
    let timeline = build_replay_timeline(RUN_ID, &events, Some(&known_nodes()));
    let mut cursor = replay_cursor(timeline.frames.len());
    cursor = move_replay_cursor(cursor, CURSOR_FORWARD_DELTA);
    cursor = move_replay_cursor(cursor, CURSOR_BACK_DELTA);

    assert!(timeline.diagnostics.is_empty());
    assert_eq!(timeline.frames.len(), EXPECTED_FRAME_COUNT);
    assert_eq!(timeline.frames[THIRD_SEQUENCE as usize].node_states[&NODE_ID], ReplayNodeState::Running);
    assert_eq!(timeline.frames[FOURTH_SEQUENCE as usize].node_states[&NODE_ID], ReplayNodeState::Completed);
    assert_eq!(cursor.position, THIRD_SEQUENCE as usize);
}

// r[verify n8nclone.workflow_validation.replay_records]
// r[verify n8nclone.workflow_validation.replay_records.order]
#[test]
fn duplicate_and_out_of_order_events_are_rejected() {
    let events = vec![
        replay_event(FIRST_SEQUENCE, None, RunnerEventKind::RunStarted),
        replay_event(SECOND_SEQUENCE, Some(NODE_ID), RunnerEventKind::NodeStarted),
        replay_event(SECOND_SEQUENCE, Some(NODE_ID), RunnerEventKind::NodeCompleted),
    ];
    let diagnostics = validate_replay_events(&events, Some(&known_nodes()));

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == ReplayDiagnosticCode::DuplicateSequence));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == ReplayDiagnosticCode::OutOfOrderEvent));
}

// r[verify n8nclone.workflow_validation.replay_records]
// r[verify n8nclone.workflow_validation.replay_records.terminal]
#[test]
fn terminal_state_mutation_is_rejected() {
    let events = vec![
        replay_event(FIRST_SEQUENCE, None, RunnerEventKind::RunStarted),
        replay_event(SECOND_SEQUENCE, Some(NODE_ID), RunnerEventKind::NodeCompleted),
        replay_event(THIRD_SEQUENCE, Some(NODE_ID), RunnerEventKind::NodeStarted),
    ];
    let diagnostics = validate_replay_events(&events, Some(&known_nodes()));

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == ReplayDiagnosticCode::TerminalStateMutation));
}

// r[verify n8nclone.workflow_validation.replay_records]
#[test]
fn unknown_nodes_are_rejected_when_snapshot_ids_are_known() {
    let events = vec![
        replay_event(FIRST_SEQUENCE, None, RunnerEventKind::RunStarted),
        replay_event(SECOND_SEQUENCE, Some(SECOND_NODE_ID), RunnerEventKind::NodeStarted),
    ];
    let diagnostics = validate_replay_events(&events, Some(&BTreeSet::from([NODE_ID])));

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == ReplayDiagnosticCode::UnknownNode));
}

// r[verify n8nclone.workflow_execution.replay.artifact_placeholders.missing]
// r[verify n8nclone.workflow_execution.replay.artifact_placeholders]
// r[verify n8nclone.workflow_validation.replay_records.artifact]
#[test]
fn missing_artifact_payload_renders_placeholder_without_validation_failure() {
    let mut events = ordered_events();
    events[THIRD_SEQUENCE as usize].artifact = Some(artifact_reference());
    let diagnostics = validate_replay_events(&events, Some(&known_nodes()));
    let view = artifact_view(artifact_reference(), false);

    assert!(diagnostics.is_empty());
    assert_eq!(view.payload_state, ReplayArtifactPayloadState::Missing);
    assert!(view.display_summary.contains("payload unavailable"));
}

// r[verify n8nclone.tui_ux.replay_debugger.redaction]
#[test]
fn sensitive_values_are_redacted_in_replay_summaries() {
    let summary = summarize_value(&Value::from_sensitive_string("secret-token".to_string()));

    assert!(summary.redacted);
    assert_eq!(summary.summary, "[redacted]");
}

// r[verify n8nclone.workflow_format.replay_bundles]
// r[verify n8nclone.workflow_format.replay_bundles.export]
// r[verify n8nclone.workflow_format.replay_bundles.import]
#[test]
fn replay_bundle_round_trips_safe_events() {
    let bundle = export_replay_bundle(RUN_ID, "wf", None, ordered_events());
    let json = replay_bundle_to_pretty_json(&bundle).expect("bundle serializes");
    let imported = import_replay_bundle_json(&json).expect("bundle imports");

    assert_eq!(imported.run_id, RUN_ID);
    assert_eq!(imported.events.len(), EXPECTED_FRAME_COUNT);
}

// r[verify n8nclone.workflow_format.replay_bundles]
// r[verify n8nclone.workflow_format.replay_bundles.reject_unsafe]
// r[verify n8nclone.workflow_validation.replay_records]
#[test]
fn replay_bundle_rejects_plaintext_secret_markers() {
    let mut events = ordered_events();
    events[SECOND_SEQUENCE as usize].contains_plaintext_secret_material = true;
    let bundle = export_replay_bundle(RUN_ID, "wf", None, events);
    let json = replay_bundle_to_pretty_json(&bundle).expect("bundle serializes");
    let error = import_replay_bundle_json(&json).expect_err("unsafe bundle rejected");

    match error {
        ReplayBundleError::Validation(diagnostics) => assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == ReplayDiagnosticCode::PlaintextSecretMaterial)
        ),
        other => panic!("expected validation error, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.replay.event_store.completed]
// r[verify n8nclone.workflow_execution.replay.event_store.partial]
// r[verify n8nclone.workflow_execution.replay.event_store]
#[test]
fn store_persists_replay_events_for_run() {
    let temp_dir = tempfile::tempdir().expect("temp dir created");
    let store = Store::new(temp_dir.path().join("store.redb")).expect("store opens");
    let events = ordered_events();

    store.save_run_replay_events(RUN_ID, &events).expect("events save");
    let loaded = store.load_run_replay_events(RUN_ID).expect("events load").expect("events exist");

    assert_eq!(loaded, events);
}

#![allow(unknown_lints, path_segment_repetition)]

use super::support::*;

const THREAD_ID: &str = "thread:foreground";
const SNAPSHOT_ID: &str = "snap:handoff";
const RESTORE_HASH: &str = "blake3:handoff-restore";
const INCLUDED_TRACKED_FILE: &str = "tracked:src/lib.rs";
const DIRECTION_TO_FOREGROUND: &str = "isolated_to_foreground";
const CONFLICT_CLEAR: &str = "clear";
const CONFLICT_DIRTY: &str = "dirty_state_conflict";
const EXPECTED_INCLUDED_COUNT: usize = 1;

#[test]
fn snapshot_handoff_allows_clear_isolated_to_foreground_move() {
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.handoff_snapshot]
    let record = crate::WorkspaceSnapshotHandoffRecord::new(valid_handoff_draft(CONFLICT_CLEAR))
        .expect("clear handoff should validate");

    assert!(record.handoff_allowed());
    assert_eq!(record.thread_id(), THREAD_ID);
    assert_eq!(record.source_workspace_id(), WORKSPACE_ID);
    assert_eq!(record.target_workspace_id(), PARENT_ID);
    assert_eq!(record.snapshot_id(), SNAPSHOT_ID);
    assert_eq!(record.included_resources().len(), EXPECTED_INCLUDED_COUNT);
    assert_eq!(record.restore_manifest_hash(), RESTORE_HASH);
}

#[test]
fn dirty_state_conflict_blocks_handoff_but_preserves_restore_evidence() {
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.handoff_snapshot.conflict]
    let record = crate::WorkspaceSnapshotHandoffRecord::new(valid_handoff_draft(CONFLICT_DIRTY))
        .expect("conflict record should still validate as review evidence");

    assert!(!record.handoff_allowed());
    assert_eq!(record.conflict_status(), crate::WorkspaceHandoffConflictStatus::DirtyStateConflict);
    assert_eq!(record.restore_manifest_hash(), RESTORE_HASH);
}

#[test]
fn handoff_rejects_empty_included_resource_manifest() {
    let error = crate::WorkspaceSnapshotHandoffRecord::new(crate::WorkspaceSnapshotHandoffRecordDraft {
        included_resources: &[],
        ..valid_handoff_draft(CONFLICT_CLEAR)
    })
    .expect_err("handoff without included resource evidence should fail");

    assert_eq!(error, crate::AdapterContractError::EmptyCollection {
        field: "handoff_included_resources",
    });
}

fn valid_handoff_draft(conflict_status: &'static str) -> crate::WorkspaceSnapshotHandoffRecordDraft<'static> {
    crate::WorkspaceSnapshotHandoffRecordDraft {
        thread_id: THREAD_ID,
        direction: DIRECTION_TO_FOREGROUND,
        source_workspace_id: WORKSPACE_ID,
        target_workspace_id: PARENT_ID,
        snapshot_id: SNAPSHOT_ID,
        included_resources: &[INCLUDED_TRACKED_FILE],
        conflict_status,
        restore_manifest_hash: RESTORE_HASH,
    }
}

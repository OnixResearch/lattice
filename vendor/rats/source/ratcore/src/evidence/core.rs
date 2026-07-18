//! Renderer-neutral interaction replay scripts and deterministic runner.
//!
//! Renderer crates translate input events into [`ReplayEvent`] values and drive
//! model transitions through [`ReplayTarget`] implementations.

const FIELD_METADATA_NAME: &str = "metadata.name";
const FIELD_EVENT_KIND: &str = "event.kind";
const FIELD_EVENT_TARGET: &str = "event.target";
const FIELD_EVENT_TEXT: &str = "event.text";
const FIELD_MOUSE_BOUNDS: &str = "event.mouse";
const FIELD_ACTION: &str = "event.action";
const FIELD_WAIT: &str = "event.wait";
const FIELD_ASSERTION: &str = "assertion";
const STATE_ENTRY_SEPARATOR_BYTES: usize = 2;

mod event;
pub use event::ReplayEvent;
mod fixture;
pub use fixture::{
    ReplayAssertion, ReplayMetadata, ReplayScript, ReplaySnapshotExpectation, ReplayWaitCondition,
    ReplayWaitPredicate,
};
mod outcome;
pub use outcome::{ReplayIssue, ReplayIssueCode, ReplayReport, ReplayTarget};

mod runner;
pub use runner::{blake3_hex, run_replay, state_snapshot_hash};
mod widgets;
pub use widgets::{
    CommandPaletteReplayTarget, LeaderReplayRow, LeaderReplaySummary, LeaderReplayTarget,
    PointerMouseRendererArtifact, PointerMouseReplayProof, PointerMouseReplayProofRow,
    PointerMouseReplayTarget, ReplayCommandAction, ReplayTreeNode, TreeReplayTarget,
    WorkspaceShellReplayEvidence, WorkspaceShellReplayEvidenceRow, WorkspaceShellReplayTarget,
    command_palette_replay_script, leader_key_negative_replay_script,
    leader_key_positive_replay_script, leader_key_replay_summary, leader_key_replay_target,
    pointer_mouse_negative_replay_script, pointer_mouse_positive_replay_script,
    pointer_mouse_renderer_artifacts, pointer_mouse_replay_proof, pointer_mouse_replay_target,
    tree_navigation_replay_script, workspace_shell_negative_replay_script,
    workspace_shell_positive_replay_script, workspace_shell_replay_evidence,
    workspace_shell_replay_target,
};
mod outputs;
pub use outputs::{
    WorkspaceShellRendererSnapshot, WorkspaceShellRendererSnapshotIssue,
    WorkspaceShellRendererSnapshotIssueCode, WorkspaceShellRendererSnapshotReport,
    validate_workspace_shell_renderer_snapshots, workspace_shell_renderer_snapshots,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

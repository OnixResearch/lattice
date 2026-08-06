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
pub use fixture::ReplayAssertion;
pub use fixture::ReplayMetadata;
pub use fixture::ReplayScript;
pub use fixture::ReplaySnapshotExpectation;
pub use fixture::ReplayWaitCondition;
pub use fixture::ReplayWaitPredicate;
mod outcome;
pub use outcome::ReplayIssue;
pub use outcome::ReplayIssueCode;
pub use outcome::ReplayReport;
pub use outcome::ReplayTarget;

mod runner;
pub use runner::blake3_hex;
pub use runner::run_replay;
pub use runner::state_snapshot_hash;
mod widgets;
pub use widgets::CommandPaletteReplayTarget;
pub use widgets::LeaderReplayRow;
pub use widgets::LeaderReplaySummary;
pub use widgets::LeaderReplayTarget;
pub use widgets::PointerMouseRendererArtifact;
pub use widgets::PointerMouseReplayProof;
pub use widgets::PointerMouseReplayProofRow;
pub use widgets::PointerMouseReplayTarget;
pub use widgets::ReplayCommandAction;
pub use widgets::ReplayTreeNode;
pub use widgets::TreeReplayTarget;
pub use widgets::WorkspaceShellReplayEvidence;
pub use widgets::WorkspaceShellReplayEvidenceRow;
pub use widgets::WorkspaceShellReplayTarget;
pub use widgets::command_palette_replay_script;
pub use widgets::leader_key_negative_replay_script;
pub use widgets::leader_key_positive_replay_script;
pub use widgets::leader_key_replay_summary;
pub use widgets::leader_key_replay_target;
pub use widgets::pointer_mouse_negative_replay_script;
pub use widgets::pointer_mouse_positive_replay_script;
pub use widgets::pointer_mouse_renderer_artifacts;
pub use widgets::pointer_mouse_replay_proof;
pub use widgets::pointer_mouse_replay_target;
pub use widgets::tree_navigation_replay_script;
pub use widgets::workspace_shell_negative_replay_script;
pub use widgets::workspace_shell_positive_replay_script;
pub use widgets::workspace_shell_replay_evidence;
pub use widgets::workspace_shell_replay_target;
mod outputs;
pub use outputs::WorkspaceShellRendererSnapshot;
pub use outputs::WorkspaceShellRendererSnapshotIssue;
pub use outputs::WorkspaceShellRendererSnapshotIssueCode;
pub use outputs::WorkspaceShellRendererSnapshotReport;
pub use outputs::validate_workspace_shell_renderer_snapshots;
pub use outputs::workspace_shell_renderer_snapshots;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

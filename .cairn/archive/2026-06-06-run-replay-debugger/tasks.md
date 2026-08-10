## Phase 1: Replay core and event persistence

- [x] [serial] r[n8nclone.workflow_execution.replay.event_store] Persist ordered runner events, safe value summaries, redaction metadata, and artifact references for every run.
- [x] [parallel] r[n8nclone.workflow_execution.replay.core] Add pure replay timeline reconstruction, cursor movement, and node-state folding with positive and negative tests.
- [x] [parallel] r[n8nclone.workflow_validation.replay_records] Validate replay event order, terminal-state mutation attempts, unknown nodes, malformed artifact refs, and bad parent/child correlation.

## Phase 2: Replay bundles

- [x] [serial] r[n8nclone.workflow_format.replay_bundles] Add redaction-safe replay bundle export/import with deterministic JSON or Nickel representation and fixture tests.
- [x] [parallel] r[n8nclone.workflow_execution.replay.artifact_placeholders] Resolve available artifact summaries during replay and display missing artifact placeholders without failing the replay.

## Phase 3: TUI debugger

- [x] [serial] r[n8nclone.tui_ux.replay_debugger.timeline] Add TUI replay timeline with event cursor, node-state map, cache markers, and selected-event details.
- [x] [parallel] r[n8nclone.tui_ux.replay_debugger.active_run] Feed active runner events into the same timeline model used by historical replay.
- [x] [parallel] r[n8nclone.tui_ux.replay_debugger.redaction] Ensure replay views and exported bundles preserve credential and payload redaction boundaries.

## Phase 4: Gates

- [x] [serial] r[n8nclone.tui_ux.replay_debugger.timeline] Run focused replay/debugger tests, `./scripts/check.rs`, and archive or report blockers.

## ADDED Requirements

### Requirement: Replay debugger timeline
r[n8nclone.tui_ux.replay_debugger.timeline] The system MUST let users inspect active or historical workflow runs as ordered replay timelines.

#### Scenario: Open historical replay
r[n8nclone.tui_ux.replay_debugger.timeline.open]
- GIVEN a run has persisted replay events
- WHEN the user opens that run in the replay debugger
- THEN the system displays event order, node status, cache markers, failure summaries, and selected-event details

#### Scenario: Step replay cursor
r[n8nclone.tui_ux.replay_debugger.timeline.step]
- GIVEN the replay debugger is open
- WHEN the user moves the replay cursor forward or backward
- THEN the system updates the displayed node states from recorded events without mutating the workflow or run history

### Requirement: Active-run replay view
r[n8nclone.tui_ux.replay_debugger.active_run] The system SHOULD feed active runner events into the same timeline model used for historical replay.

#### Scenario: Active events stream into timeline
r[n8nclone.tui_ux.replay_debugger.active_run.stream]
- GIVEN a workflow run is active in the TUI
- WHEN runner events are emitted
- THEN the replay timeline updates with the new safe event summaries while the TUI remains responsive

### Requirement: Replay redaction boundaries
r[n8nclone.tui_ux.replay_debugger.redaction] The system MUST preserve redaction boundaries in replay views.

#### Scenario: Secret remains redacted in replay
r[n8nclone.tui_ux.replay_debugger.redaction.secret]
- GIVEN a recorded run used secret or sensitive payload material
- WHEN the replay debugger renders event details or value summaries
- THEN the system shows redacted summaries and does not reveal plaintext secret material

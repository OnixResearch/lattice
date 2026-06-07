## ADDED Requirements

### Requirement: Replay event persistence
r[n8nclone.workflow_execution.replay.event_store] The system MUST persist ordered runner events and redaction-safe value summaries for workflow run replay.

#### Scenario: Persist completed run events
r[n8nclone.workflow_execution.replay.event_store.completed]
- GIVEN a workflow run emits runner events and reaches a terminal state
- WHEN run history is persisted
- THEN the system stores the ordered event sequence, node identities, optional port identities, safe summaries, redaction metadata, and artifact references

#### Scenario: Partial run remains replayable
r[n8nclone.workflow_execution.replay.event_store.partial]
- GIVEN a workflow run is interrupted before terminal completion
- WHEN the operator opens replay for that run
- THEN the system reconstructs a partial timeline from the persisted events without requiring side-effect re-execution

### Requirement: Replay core
r[n8nclone.workflow_execution.replay.core] The system MUST reconstruct replay timelines from immutable event records without executing workflow side effects.

#### Scenario: Fold events into node states
r[n8nclone.workflow_execution.replay.core.fold]
- GIVEN a valid ordered event sequence for a run
- WHEN replay reconstruction runs
- THEN the system produces deterministic timeline frames and node status at each cursor position

#### Scenario: Replay does not rerun side effects
r[n8nclone.workflow_execution.replay.core.no_side_effects]
- GIVEN a historical run contains shell, HTTP, file, image, agent, trigger, or subworkflow events
- WHEN replay mode opens that run
- THEN the system displays recorded events and summaries without invoking any runtime or adapter

### Requirement: Replay artifact placeholders
r[n8nclone.workflow_execution.replay.artifact_placeholders] The system SHOULD keep replay usable when artifact payloads are unavailable.

#### Scenario: Missing artifact shows placeholder
r[n8nclone.workflow_execution.replay.artifact_placeholders.missing]
- GIVEN a replay event references an artifact whose payload cannot be loaded
- WHEN the replay view requests artifact details
- THEN the system displays hash, media metadata, and a missing-payload placeholder instead of failing the replay

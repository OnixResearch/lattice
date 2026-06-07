## ADDED Requirements

### Requirement: Replay bundle format
r[n8nclone.workflow_format.replay_bundles] The system MUST export and import redaction-safe replay bundles for workflow run debugging.

#### Scenario: Export replay bundle
r[n8nclone.workflow_format.replay_bundles.export]
- GIVEN a persisted run has runner events and safe summaries
- WHEN the operator exports a replay bundle
- THEN the system writes run metadata, workflow snapshot or workflow hash, ordered events, redaction metadata, and artifact reference metadata without plaintext secrets

#### Scenario: Import replay bundle
r[n8nclone.workflow_format.replay_bundles.import]
- GIVEN a replay bundle satisfies the replay bundle contract
- WHEN the operator imports it
- THEN the system stores or opens the bundle as a replayable historical run without executing workflow side effects

#### Scenario: Reject unsafe replay bundle
r[n8nclone.workflow_format.replay_bundles.reject_unsafe]
- GIVEN a replay bundle includes malformed events or fields marked as plaintext secret material
- WHEN import validation runs
- THEN the system rejects the bundle with deterministic diagnostics

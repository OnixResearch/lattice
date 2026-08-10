## ADDED Requirements

### Requirement: Replay record validation
r[n8nclone.workflow_validation.replay_records] The system MUST validate replay records and bundles before presenting them as trusted run timelines.

#### Scenario: Out-of-order event is rejected
r[n8nclone.workflow_validation.replay_records.order]
- GIVEN a replay bundle contains duplicate or out-of-order event sequence numbers
- WHEN replay validation runs
- THEN the system reports a fatal diagnostic and refuses to import the malformed timeline

#### Scenario: Terminal-state mutation is rejected
r[n8nclone.workflow_validation.replay_records.terminal]
- GIVEN replay events attempt to move a completed, failed, or cancelled node back to a non-terminal state
- WHEN replay validation runs
- THEN the system reports a fatal diagnostic and does not display the invalid state transition as trusted

#### Scenario: Unknown artifact reference is recoverable
r[n8nclone.workflow_validation.replay_records.artifact]
- GIVEN a replay record references an artifact whose payload is not included
- WHEN replay validation runs
- THEN the system accepts the record if the reference metadata is well formed and marks the payload unavailable

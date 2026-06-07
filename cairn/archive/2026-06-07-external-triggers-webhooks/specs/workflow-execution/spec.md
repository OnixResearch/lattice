## ADDED Requirements

### Requirement: Trigger payload binding
r[n8nclone.workflow_execution.triggers.payload_binding] The system MUST bind admitted trigger payload data to declared workflow input values before execution.

#### Scenario: JSON payload binds to input port
r[n8nclone.workflow_execution.triggers.payload_binding.json]
- GIVEN a webhook trigger maps a JSON body field to a workflow input port
- WHEN a valid request is admitted
- THEN the system supplies the mapped field as a typed input value for the workflow run

#### Scenario: Missing required payload field fails
r[n8nclone.workflow_execution.triggers.payload_binding.missing]
- GIVEN a trigger input binding requires a payload field that is absent
- WHEN trigger admission runs
- THEN the system rejects the event before starting the workflow

#### Scenario: Payload type mismatch fails
r[n8nclone.workflow_execution.triggers.payload_binding.type_mismatch]
- GIVEN a trigger input binding maps a payload value to an incompatible workflow input type tag
- WHEN trigger admission runs
- THEN the system reports a deterministic diagnostic and does not execute workflow side effects

### Requirement: Trigger run history
r[n8nclone.scheduling.triggers.history] The system SHOULD record trigger run outcomes with safe payload summaries.

#### Scenario: Persist trigger outcome
r[n8nclone.scheduling.triggers.history.persist]
- GIVEN a trigger event is accepted and starts a workflow run
- WHEN that run reaches a terminal state
- THEN the system stores trigger id, workflow name, run id, idempotency key, status, and redaction-safe payload summary

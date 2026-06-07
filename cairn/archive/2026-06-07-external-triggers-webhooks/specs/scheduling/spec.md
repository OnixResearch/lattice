## ADDED Requirements

### Requirement: Trigger definitions
r[n8nclone.scheduling.triggers.model] The system MUST persist trigger definitions that bind external events to saved workflows.

#### Scenario: Save enabled trigger
r[n8nclone.scheduling.triggers.model.save]
- GIVEN an operator creates a valid trigger for a saved workflow
- WHEN the trigger is saved
- THEN the system persists trigger identity, workflow reference, kind, enabled state, input binding, overlap policy, and idempotency policy

#### Scenario: Reject trigger for missing workflow
r[n8nclone.scheduling.triggers.model.missing_workflow]
- GIVEN an operator creates a trigger for a workflow that cannot be resolved
- WHEN trigger validation runs
- THEN the system reports a diagnostic and refuses to enable the trigger

### Requirement: Trigger admission
r[n8nclone.scheduling.triggers.admission] The system MUST admit trigger runs through deterministic policy before starting workflow execution.

#### Scenario: Disabled trigger is skipped
r[n8nclone.scheduling.triggers.admission.disabled]
- GIVEN an inbound event targets a disabled trigger
- WHEN trigger admission runs
- THEN the system rejects the event without starting the workflow

#### Scenario: Duplicate idempotency key is rejected
r[n8nclone.scheduling.triggers.admission.idempotency]
- GIVEN a trigger event carries an idempotency key that was already accepted for that trigger
- WHEN trigger admission runs
- THEN the system rejects the duplicate event and preserves the previous run record

#### Scenario: Overlap policy prevents duplicate run
r[n8nclone.scheduling.triggers.admission.overlap]
- GIVEN a trigger's workflow is already running and overlap is forbidden
- WHEN another event arrives for the same trigger
- THEN the system rejects or delays the event according to policy without starting an overlapping run

### Requirement: Webhook trigger adapter
r[n8nclone.scheduling.triggers.webhook] The system SHOULD accept HTTP webhook events for triggers that explicitly declare webhook admission policy.

#### Scenario: Valid webhook starts workflow
r[n8nclone.scheduling.triggers.webhook.valid]
- GIVEN a webhook trigger is enabled and a request satisfies method, path, content type, authentication, and payload binding rules
- WHEN the webhook adapter receives the request
- THEN the system starts the configured workflow with typed trigger payload inputs

#### Scenario: Invalid webhook is rejected
r[n8nclone.scheduling.triggers.webhook.invalid]
- GIVEN a webhook request fails method, path, content type, authentication, or payload binding checks
- WHEN the webhook adapter receives the request
- THEN the system reports a request diagnostic and does not start workflow execution

### Requirement: Schedule trigger compatibility
r[n8nclone.scheduling.triggers.scheduler_compat] The system MUST preserve existing scheduled-run behavior when schedules are represented as trigger definitions.

#### Scenario: Existing schedule remains due
r[n8nclone.scheduling.triggers.scheduler_compat.due]
- GIVEN an existing recurring schedule is migrated or adapted to a schedule trigger
- WHEN due-run evaluation runs at a due time
- THEN the system returns the same workflow run plan as the previous schedule model

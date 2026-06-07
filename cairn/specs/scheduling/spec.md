# Scheduling Specification

## Purpose

Defines the `scheduling` capability.

## Requirements

### Requirement: Schedule model
r[n8nclone.scheduling.schedule_model] The system MUST persist schedule definitions that bind saved workflows to recurring run rules.

#### Scenario: Save workflow schedule
r[n8nclone.scheduling.schedule_model.save]
- GIVEN an operator creates a schedule for a saved workflow
- WHEN the schedule is saved
- THEN the system persists the workflow reference, recurrence rule, enabled state, and overlap policy

#### Scenario: Reject missing workflow schedule
r[n8nclone.scheduling.schedule_model.missing_workflow]
- GIVEN an operator creates a schedule for a workflow that does not exist
- WHEN schedule validation runs
- THEN the system reports a schedule diagnostic and refuses to enable the schedule

### Requirement: Due-run evaluator
r[n8nclone.scheduling.due_evaluator] The system MUST determine due scheduled runs from schedule state and an explicit current-time input.

#### Scenario: Schedule is due
r[n8nclone.scheduling.due_evaluator.due]
- GIVEN an enabled schedule whose next run time is at or before the current-time input
- WHEN due-run evaluation runs
- THEN the system returns a due-run plan for that workflow

#### Scenario: Schedule is not due
r[n8nclone.scheduling.due_evaluator.not_due]
- GIVEN an enabled schedule whose next run time is after the current-time input
- WHEN due-run evaluation runs
- THEN the system returns no due-run plan for that schedule

### Requirement: Scheduler daemon run loop
r[n8nclone.scheduling.daemon.run_due] The system SHOULD provide a daemon mode that starts due workflow runs without the TUI.

#### Scenario: Daemon starts due workflow
r[n8nclone.scheduling.daemon.run_due.start]
- GIVEN a schedule is due and the referenced workflow can be loaded
- WHEN the daemon evaluates schedules
- THEN the system starts a headless workflow run and records the outcome

#### Scenario: Daemon reports load failure
r[n8nclone.scheduling.daemon.run_due.load_failure]
- GIVEN a due schedule references a workflow that cannot be loaded
- WHEN the daemon evaluates schedules
- THEN the system records a schedule error without disabling unrelated schedules

### Requirement: Scheduled run overlap policy
r[n8nclone.scheduling.overlap_policy] The system MUST prevent overlapping scheduled runs unless the schedule explicitly permits overlap.

#### Scenario: Overlap prevented
r[n8nclone.scheduling.overlap_policy.prevent]
- GIVEN a scheduled workflow is already running and overlap is disabled
- WHEN the same schedule becomes due again
- THEN the daemon skips or delays the new run according to policy without starting a second overlapping run

### Requirement: Missed-run handling
r[n8nclone.scheduling.missed_runs] The system SHOULD handle daemon restart and missed schedules deterministically.

#### Scenario: Restart after missed run
r[n8nclone.scheduling.missed_runs.restart]
- GIVEN the daemon was stopped while a schedule became due
- WHEN the daemon restarts with a current-time input after the missed run time
- THEN the evaluator applies the configured missed-run policy and reports the resulting plan

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

## ADDED Requirements

### Requirement: Trigger Nickel format
r[n8nclone.workflow_format.triggers.nickel] The system MUST preserve trigger definitions in typed Nickel workflow import and export.

#### Scenario: Export workflow triggers
r[n8nclone.workflow_format.triggers.nickel.export]
- GIVEN a workflow has manual, schedule, or webhook trigger definitions
- WHEN the operator exports the workflow to Nickel
- THEN the system writes trigger definitions with workflow references, input bindings, auth references, overlap policy, and idempotency policy

#### Scenario: Import workflow triggers
r[n8nclone.workflow_format.triggers.nickel.import]
- GIVEN a Nickel workflow file declares valid trigger definitions
- WHEN the operator imports it
- THEN the system saves equivalent trigger definitions without storing plaintext credential material

#### Scenario: Workflow without triggers remains valid
r[n8nclone.workflow_format.triggers.nickel.compatibility]
- GIVEN an existing Nickel workflow file contains no trigger section
- WHEN the operator imports it after trigger support is added
- THEN the system imports the workflow with no enabled triggers and preserves existing runnable meaning

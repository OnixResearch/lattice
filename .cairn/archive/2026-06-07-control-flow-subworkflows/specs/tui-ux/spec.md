## ADDED Requirements

### Requirement: Control-flow add-node flow
r[n8nclone.tui_ux.control_flow.add_flow] The system SHOULD let users create condition, merge, and subworkflow nodes through guided TUI flows.

#### Scenario: Create condition node
r[n8nclone.tui_ux.control_flow.add_flow.condition]
- GIVEN the user starts the add-node flow
- WHEN the user chooses a condition node and enters valid branch predicates
- THEN the system creates a condition node with branch output ports and marks the workflow as unsaved

#### Scenario: Create merge node
r[n8nclone.tui_ux.control_flow.add_flow.merge]
- GIVEN the user starts the add-node flow
- WHEN the user chooses a merge node and selects a fan-in policy
- THEN the system creates a merge node with compatible input and output ports

#### Scenario: Create subworkflow node
r[n8nclone.tui_ux.control_flow.add_flow.subworkflow]
- GIVEN saved workflows are available
- WHEN the user chooses a subworkflow node and selects a valid child workflow
- THEN the system creates a subworkflow node with the selected workflow reference and editable interface mapping fields

### Requirement: Control-flow property editing
r[n8nclone.tui_ux.control_flow.properties] The system MUST keep control-flow node property edits recoverable.

#### Scenario: Reject invalid predicate edit
r[n8nclone.tui_ux.control_flow.properties.invalid_predicate]
- GIVEN a condition node property edit is in progress
- WHEN the user saves a malformed predicate
- THEN the system rejects the edit, reports the validation error, and preserves the previous predicate

#### Scenario: Missing subworkflow remains visible
r[n8nclone.tui_ux.control_flow.properties.missing_subworkflow]
- GIVEN a subworkflow node references a workflow that is no longer available
- WHEN the properties panel renders
- THEN the system displays the missing reference and keeps the node editable

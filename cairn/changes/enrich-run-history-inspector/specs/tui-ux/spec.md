## ADDED Requirements

### Requirement: Persist run details
r[n8nclone.tui_ux.run_history.details.persist] The system SHOULD persist node-level details for completed workflow runs.

#### Scenario: Persist node detail records
r[n8nclone.tui_ux.run_history.details.persist.nodes]
- GIVEN a workflow run reaches a terminal state
- WHEN run history is persisted
- THEN the system stores each node's final status, cache marker, output summary, and error summary for later inspection

### Requirement: Run history timeline
r[n8nclone.tui_ux.run_history.timeline] The system SHOULD display a node-by-node timeline for an inspected run.

#### Scenario: Show run timeline
r[n8nclone.tui_ux.run_history.timeline.show]
- GIVEN a run has persisted node details
- WHEN the user inspects that run
- THEN the system lists node outcomes in execution order with status and cache context

### Requirement: Run history node outputs
r[n8nclone.tui_ux.run_history.node_outputs] The system MUST let users inspect stdout, stderr, exit code, and error details for a selected run-history node.

#### Scenario: Inspect node output
r[n8nclone.tui_ux.run_history.node_outputs.inspect]
- GIVEN the run-history timeline is open
- WHEN the user selects a node with persisted outputs
- THEN the system displays that node's stdout, stderr, exit code, and error details without changing the current workflow

#### Scenario: Missing output remains recoverable
r[n8nclone.tui_ux.run_history.node_outputs.missing]
- GIVEN the selected node has no persisted output details
- WHEN the user inspects the node
- THEN the system displays a clear empty-detail message and keeps history navigation usable

### Requirement: Run history rerun actions
r[n8nclone.tui_ux.run_history.rerun] The system MAY let users rerun a workflow from run history through an explicit action.

#### Scenario: Rerun selected workflow
r[n8nclone.tui_ux.run_history.rerun.workflow]
- GIVEN a run-history record references a workflow that is still available
- WHEN the user confirms rerun for that record
- THEN the system starts a new run without mutating the historical record

#### Scenario: Rerun unavailable workflow
r[n8nclone.tui_ux.run_history.rerun.unavailable]
- GIVEN a run-history record references a workflow that cannot be loaded
- WHEN the user confirms rerun
- THEN the system reports the load failure and keeps the current workflow unchanged

### Requirement: Run detail error handling
r[n8nclone.tui_ux.run_history.details.empty_error] The system MUST handle missing or unavailable run detail storage without breaking the TUI.

#### Scenario: Detail storage unavailable
r[n8nclone.tui_ux.run_history.details.empty_error.storage]
- GIVEN run detail storage returns an error
- WHEN the user inspects a run
- THEN the system reports the error and keeps the run-history list usable

## ADDED Requirements

### Requirement: Background workflow execution
r[n8nclone.tui_ux.background_runs.responsive] The system MUST keep the TUI responsive while workflow execution is in progress.

#### Scenario: Long-running workflow remains interactive
r[n8nclone.tui_ux.background_runs.long_running]
- GIVEN a workflow contains a long-running shell node
- WHEN the user starts the workflow
- THEN the system continues to render and handle input while the node runs

#### Scenario: Render node run status
r[n8nclone.tui_ux.background_runs.status]
- GIVEN a workflow run is active
- WHEN node execution state changes
- THEN the system displays current node status in the TUI

#### Scenario: Cancel active run
r[n8nclone.tui_ux.background_runs.cancel]
- GIVEN a workflow run is active
- WHEN the user requests cancellation
- THEN the system stops pending work, reports cancellation, and leaves the TUI usable

#### Scenario: Failed node does not freeze TUI
r[n8nclone.tui_ux.background_runs.failure]
- GIVEN a shell node exits with an error
- WHEN the workflow run processes that failure
- THEN the system reports the error and continues handling TUI input

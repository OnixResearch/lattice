## ADDED Requirements

### Requirement: Workflow browser
r[n8nclone.tui_ux.workflow_browser.open] The system MUST let users browse saved workflows from inside the TUI.

#### Scenario: Load saved workflow
r[n8nclone.tui_ux.workflow_browser.load]
- GIVEN at least one workflow is saved in the store
- WHEN the user opens the workflow browser and confirms a workflow
- THEN the system loads that workflow into the canvas and closes the browser

#### Scenario: Empty workflow store
r[n8nclone.tui_ux.workflow_browser.empty]
- GIVEN no workflows are saved in the store
- WHEN the user opens the workflow browser
- THEN the system displays that no saved workflows are available

#### Scenario: Load failure preserves current workflow
r[n8nclone.tui_ux.workflow_browser.load_failure]
- GIVEN a workflow selection cannot be loaded from storage
- WHEN the load fails
- THEN the system reports the failure and keeps the current workflow active

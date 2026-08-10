## ADDED Requirements

### Requirement: Shell node property editing
r[n8nclone.tui_ux.properties.edit_shell] The system MUST let users edit selected shell node properties from the properties panel.

#### Scenario: Edit shell command
r[n8nclone.tui_ux.properties.edit_command]
- GIVEN exactly one shell node is selected
- WHEN the user edits and saves the command field
- THEN the workflow stores the new command and the properties panel displays it

#### Scenario: Reject invalid timeout
r[n8nclone.tui_ux.properties.validate_timeout]
- GIVEN exactly one shell node is selected
- WHEN the user enters an invalid timeout value
- THEN the system rejects the edit and preserves the previous timeout

#### Scenario: Cancel property edit
r[n8nclone.tui_ux.properties.cancel]
- GIVEN a property edit is in progress
- WHEN the user cancels the edit
- THEN the system restores the original value and leaves the workflow unchanged

#### Scenario: Property editing test coverage
r[n8nclone.tui_ux.properties.tests]
- GIVEN property editing behavior is implemented
- WHEN the test suite runs
- THEN positive saves and negative validation failures are both verified

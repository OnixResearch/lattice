# TUI UX Specification

## Purpose

Define the accepted user-experience contract for n8nclone's terminal workflow editor.

## Requirements

### Requirement: Terminal workflow workspace
r[n8nclone.tui_ux.workspace] The system MUST provide a terminal workflow workspace where users can create, inspect, persist, and run node-based workflows.

#### Scenario: Workspace starts
r[n8nclone.tui_ux.workspace.starts]
- GIVEN a valid terminal environment
- WHEN the user starts n8nclone
- THEN the system displays the workflow canvas, properties panel, output panel, and status bar

### Requirement: Keyboard-first operation
r[n8nclone.tui_ux.keyboard_first] The system SHOULD keep core workflow authoring actions available from the keyboard.

#### Scenario: Keyboard controls are discoverable
r[n8nclone.tui_ux.keyboard_first.discoverable]
- GIVEN the user is in the TUI
- WHEN the user reviews the visible hints or opens the command palette
- THEN the system exposes the available workflow authoring commands

### Requirement: Interactive command palette
r[n8nclone.tui_ux.command_palette.interactive] The system MUST let users search, navigate, and execute command palette actions from the keyboard.

#### Scenario: Execute selected palette action
r[n8nclone.tui_ux.command_palette.execute]
- GIVEN the command palette is open
- WHEN the user filters to an available command and confirms the selected result
- THEN the system dispatches the corresponding application action and closes the palette

#### Scenario: Close palette without action
r[n8nclone.tui_ux.command_palette.escape]
- GIVEN the command palette is open
- WHEN the user presses Escape
- THEN the system closes the palette without changing the workflow

#### Scenario: Confirm with no selectable match
r[n8nclone.tui_ux.command_palette.no_match]
- GIVEN the command palette is open and the current query has no selectable command
- WHEN the user confirms the selection
- THEN the system leaves the workflow unchanged and keeps the user in a recoverable palette state

### Requirement: Contextual help footer
r[n8nclone.tui_ux.help_footer.contextual] The system SHOULD display footer help that reflects the active panel, modal, or graph interaction mode.

#### Scenario: Canvas help
r[n8nclone.tui_ux.help_footer.canvas]
- GIVEN the canvas is focused and no modal is open
- WHEN the status bar renders
- THEN the system shows canvas-relevant navigation and workflow shortcuts

#### Scenario: Wiring help
r[n8nclone.tui_ux.help_footer.wiring]
- GIVEN the graph is in wiring mode
- WHEN the status bar renders
- THEN the system shows how to complete or cancel wiring

#### Scenario: Properties and output help
r[n8nclone.tui_ux.help_footer.properties_output]
- GIVEN the properties or output panel is focused
- WHEN the status bar renders
- THEN the system shows shortcuts relevant to the focused panel

#### Scenario: Footer truncates safely
r[n8nclone.tui_ux.help_footer.truncate]
- GIVEN terminal width is constrained
- WHEN footer help is longer than the available area
- THEN the system renders a compact hint without overflowing the status bar

### Requirement: Empty-state onboarding
r[n8nclone.tui_ux.empty_state.show] The system SHOULD display onboarding guidance when the workflow canvas has no nodes.

#### Scenario: Empty workflow shows next actions
r[n8nclone.tui_ux.empty_state.next_actions]
- GIVEN the workflow contains no nodes
- WHEN the TUI renders the canvas
- THEN the system shows how to add a node and open the command palette

#### Scenario: Non-empty workflow hides onboarding
r[n8nclone.tui_ux.empty_state.hide]
- GIVEN the workflow contains at least one node
- WHEN the TUI renders the canvas
- THEN the system does not show the empty-state onboarding block

#### Scenario: Constrained terminal uses compact copy
r[n8nclone.tui_ux.empty_state.compact]
- GIVEN the terminal canvas is too small for the full onboarding block
- WHEN the TUI renders the empty workflow
- THEN the system shows a compact hint instead of overflowing the canvas

#### Scenario: Empty-state rendering is covered
r[n8nclone.tui_ux.empty_state.tests]
- GIVEN empty-state rendering is implemented
- WHEN the render tests run
- THEN empty and non-empty workflow cases are both verified

### Requirement: Unsaved-change tracking
r[n8nclone.tui_ux.unsaved.dirty_indicator] The system MUST visibly indicate when the current workflow has unsaved changes.

#### Scenario: Mutation marks workflow unsaved
r[n8nclone.tui_ux.unsaved.mutation_marks_dirty]
- GIVEN the current workflow is saved or newly clean
- WHEN the user mutates the workflow
- THEN the system marks the workflow as unsaved and displays an unsaved indicator

#### Scenario: Successful save clears unsaved indicator
r[n8nclone.tui_ux.unsaved.save_clears]
- GIVEN the current workflow has unsaved changes
- WHEN the workflow saves successfully
- THEN the system clears the unsaved indicator

#### Scenario: Quit with unsaved changes asks for confirmation
r[n8nclone.tui_ux.unsaved.quit_confirm]
- GIVEN the current workflow has unsaved changes
- WHEN the user requests quit
- THEN the system asks for confirmation before exiting

#### Scenario: Failed save keeps workflow dirty
r[n8nclone.tui_ux.unsaved.failed_save]
- GIVEN the current workflow has unsaved changes
- WHEN saving fails
- THEN the system reports the failure and keeps the unsaved indicator visible

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

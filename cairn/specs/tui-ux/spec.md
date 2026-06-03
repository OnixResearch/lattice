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

### Requirement: Guided add-node flow
r[n8nclone.tui_ux.add_node.guided] The system MUST guide users through node creation instead of inserting only a hardcoded placeholder node.

#### Scenario: Create shell node from entered fields
r[n8nclone.tui_ux.add_node.shell_fields]
- GIVEN the user starts the add-node flow
- WHEN the user provides a shell node label and command and confirms creation
- THEN the system creates a shell node with those values and selects it

#### Scenario: Create node from template
r[n8nclone.tui_ux.add_node.template]
- GIVEN node templates are configured
- WHEN the user selects a template in the add-node flow
- THEN the system creates a node using the selected template's ports and defaults

#### Scenario: Cancel add-node flow
r[n8nclone.tui_ux.add_node.cancel]
- GIVEN the add-node flow is open
- WHEN the user cancels before confirming
- THEN the system creates no node and preserves the existing workflow

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

### Requirement: Run history inspection
r[n8nclone.tui_ux.run_history.persist] The system SHOULD persist workflow run records and expose recent runs for inspection in the TUI.

#### Scenario: Persist run record
r[n8nclone.tui_ux.run_history.persist_record]
- GIVEN a workflow run reaches a terminal state
- WHEN the run finishes, fails, or is canceled
- THEN the system stores a run record with summary status for later inspection

#### Scenario: List recent runs
r[n8nclone.tui_ux.run_history.list]
- GIVEN run records exist in the store
- WHEN the user opens run history
- THEN the system lists recent runs with enough summary information to choose one

#### Scenario: Inspect selected run
r[n8nclone.tui_ux.run_history.inspect]
- GIVEN the run history list is open
- WHEN the user selects a run
- THEN the system displays that run's summary without changing the current workflow

#### Scenario: Empty or unavailable history
r[n8nclone.tui_ux.run_history.empty_error]
- GIVEN no run history is available or storage returns an error
- WHEN the user opens run history
- THEN the system displays a clear message and keeps the TUI usable

### Requirement: Mouse wiring
r[n8nclone.tui_ux.mouse_wiring.drag_release] The system MUST create a workflow edge when a user drags from an output port and releases on a compatible input port.

#### Scenario: Drag-release creates compatible edge
r[n8nclone.tui_ux.mouse_wiring.drag_release.compatible]
- GIVEN the TUI canvas contains two nodes with compatible ports
- WHEN the user starts wiring from the first node's output port and releases over the second node's input port
- THEN the system creates an edge between those ports and marks the workflow as unsaved

#### Scenario: Invalid drag-release cancels wiring
r[n8nclone.tui_ux.mouse_wiring.invalid_release]
- GIVEN the TUI canvas is in wiring mode
- WHEN the user releases the mouse over a non-port target
- THEN the system cancels wiring without creating an edge

### Requirement: Visible port snapping
r[n8nclone.tui_ux.mouse_wiring.visible_port_clicks] The system MUST treat visible rendered port dots as valid mouse targets for wiring.

#### Scenario: Click visible output dot starts wiring
r[n8nclone.tui_ux.mouse_wiring.visible_port_clicks.start]
- GIVEN the TUI canvas contains a node with a rendered output port dot
- WHEN the user clicks the visible output port dot
- THEN the system starts wiring from that output port

#### Scenario: Click visible input dot completes wiring
r[n8nclone.tui_ux.mouse_wiring.visible_port_clicks.complete]
- GIVEN the TUI canvas is wiring from an output port and a compatible input port dot is visible
- WHEN the user clicks or releases on the visible input port dot
- THEN the system creates an edge to that input port

### Requirement: Mouse wiring snap radius
r[n8nclone.tui_ux.mouse_wiring.snap_near_ports] The system SHOULD snap active mouse wiring to a nearby visible port dot.

#### Scenario: Release near visible input dot snaps
r[n8nclone.tui_ux.mouse_wiring.snap_near_ports.release]
- GIVEN the TUI canvas is wiring from an output port and a compatible input port dot is nearby
- WHEN the user releases the mouse within the port snap radius
- THEN the system creates an edge to the nearby input port

#### Scenario: Far release still cancels wiring
r[n8nclone.tui_ux.mouse_wiring.snap_near_ports.far_release]
- GIVEN the TUI canvas is wiring from an output port
- WHEN the user releases the mouse outside the port snap radius and not on a port
- THEN the system cancels wiring without creating an edge

### Requirement: Discoverable selected-node editing
r[n8nclone.tui_ux.properties.discoverable_entrypoints] The system MUST expose selected-node editing from the normal canvas workflow and the command palette.

#### Scenario: Canvas shortcut edits command
r[n8nclone.tui_ux.properties.discoverable_entrypoints.canvas_command]
- GIVEN exactly one shell node is selected on the canvas
- WHEN the user presses the edit-command shortcut
- THEN the system starts editing that node's command without requiring a focus change

#### Scenario: Palette action edits command
r[n8nclone.tui_ux.properties.discoverable_entrypoints.palette_command]
- GIVEN exactly one shell node is selected
- WHEN the user opens the command palette and confirms the edit-command action
- THEN the system starts editing that node's command

#### Scenario: Missing selection reports edit requirement
r[n8nclone.tui_ux.properties.discoverable_entrypoints.no_selection]
- GIVEN no node is selected
- WHEN the user invokes a selected-node edit action
- THEN the system leaves the workflow unchanged and reports that one node must be selected

### Requirement: Modal property text entry
r[n8nclone.tui_ux.properties.modal_text_entry] The system MUST route property-edit text input to the active edit draft regardless of which panel had focus when editing started.

#### Scenario: Save command edit from canvas focus
r[n8nclone.tui_ux.properties.modal_text_entry.save_command]
- GIVEN command editing was started while the canvas was focused
- WHEN the user edits the draft and presses Enter
- THEN the system saves the new command and exits edit mode

#### Scenario: Cancel modal edit from canvas focus
r[n8nclone.tui_ux.properties.modal_text_entry.cancel]
- GIVEN property editing was started while the canvas was focused
- WHEN the user presses Escape
- THEN the system cancels the edit and leaves the node unchanged

### Requirement: Discoverable editing help
r[n8nclone.tui_ux.properties.discoverable_help] The system SHOULD show selected-node edit shortcuts in visible help surfaces.

#### Scenario: Footer includes edit shortcuts
r[n8nclone.tui_ux.properties.discoverable_help.footer]
- GIVEN the canvas or properties panel is active
- WHEN the footer renders
- THEN the system shows how to edit the selected node's command, label, or timeout

#### Scenario: Palette lists edit actions
r[n8nclone.tui_ux.properties.discoverable_help.palette]
- GIVEN the command palette is open
- WHEN the available actions are listed
- THEN the system includes selected-node edit actions

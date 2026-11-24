# TUI UX Specification

## Purpose

Define the accepted user-experience contract for Lattice's terminal workflow editor.

## Requirements

### Requirement: Terminal workflow workspace
r[n8nclone.tui_ux.workspace] The system MUST provide a terminal workflow workspace where users can create, inspect, persist, and run node-based workflows.

#### Scenario: Workspace starts
r[n8nclone.tui_ux.workspace.starts]
- GIVEN a valid terminal environment
- WHEN the user starts Lattice
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

### Requirement: Generic agent node add flow
r[n8nclone.tui_ux.agent_nodes.add_flow] The system MUST let users create image input nodes and generic Pi or Clankers agent nodes from the TUI add-node flow.

#### Scenario: Create image input node
r[n8nclone.tui_ux.agent_nodes.add_flow.input]
- GIVEN the user starts the add-node flow
- WHEN the user chooses an image input node, enters an image label, and selects a local image path
- THEN the system creates an image input node with image output ports and marks the workflow as unsaved

#### Scenario: Create Pi generic agent node
r[n8nclone.tui_ux.agent_nodes.add_flow.pi]
- GIVEN the user starts the add-node flow
- WHEN the user chooses a generic agent node, selects Pi as the target runtime, and enters required prompt and output fields
- THEN the system creates a generic agent node with Pi target settings, media-aware inputs, and typed outputs

#### Scenario: Create Clankers generic agent node
r[n8nclone.tui_ux.agent_nodes.add_flow.clankers]
- GIVEN the user starts the add-node flow
- WHEN the user chooses a generic agent node, selects Clankers as the target runtime, and enters required prompt and output fields
- THEN the system creates a generic agent node with Clankers target settings, media-aware inputs, and typed outputs

#### Scenario: Cancel generic agent node creation
r[n8nclone.tui_ux.agent_nodes.add_flow.cancel]
- GIVEN image input or generic agent node creation is in progress
- WHEN the user cancels the add-node flow
- THEN the system creates no node and preserves the existing workflow

### Requirement: Generic agent node property editing
r[n8nclone.tui_ux.agent_nodes.properties] The system MUST let users edit selected image input and generic agent node properties from the properties surface.

#### Scenario: Edit generic agent prompt
r[n8nclone.tui_ux.agent_nodes.properties.prompt]
- GIVEN exactly one generic agent node is selected
- WHEN the user edits and saves the prompt field
- THEN the workflow stores the new prompt, marks the workflow as unsaved, and displays the updated value

#### Scenario: Edit generic agent target settings
r[n8nclone.tui_ux.agent_nodes.properties.target]
- GIVEN exactly one generic agent node is selected
- WHEN the user edits target runtime, model hints, tool policy, context references, output mode, credential references, or session references with valid values
- THEN the workflow stores the updated generic agent configuration and keeps the node runnable

#### Scenario: Edit image input path
r[n8nclone.tui_ux.agent_nodes.properties.image_path]
- GIVEN exactly one image input node is selected
- WHEN the user edits the local image path with a valid value
- THEN the workflow stores the updated image input configuration and keeps the node routable to generic agent inputs

#### Scenario: Reject invalid generic agent property
r[n8nclone.tui_ux.agent_nodes.properties.invalid]
- GIVEN exactly one image input or generic agent node is selected
- WHEN the user enters an invalid local path, unsupported output mode, unsupported target runtime, or missing required prompt field
- THEN the system rejects the edit, preserves the previous value, and keeps the edit flow recoverable

### Requirement: Image previews
r[n8nclone.tui_ux.image_nodes.preview] The system SHOULD render bounded image previews or metadata fallbacks for image nodes and generic agent image outputs.

#### Scenario: Canvas shows image thumbnail or fallback
r[n8nclone.tui_ux.image_nodes.preview.canvas]
- GIVEN an image input node or generic agent node has a loaded or generated image value
- WHEN the canvas renders that node
- THEN the system shows a bounded image thumbnail when supported or a compact metadata summary when thumbnail rendering is unavailable

#### Scenario: Output panel shows generated image details
r[n8nclone.tui_ux.image_nodes.preview.output_panel]
- GIVEN the selected run output contains a generated image value from a generic agent node
- WHEN the output panel renders
- THEN the system shows media type, dimensions when available, content hash summary, adapter metadata, and a bounded preview or fallback

#### Scenario: Preview never dumps raw bytes
r[n8nclone.tui_ux.image_nodes.preview.no_raw_bytes]
- GIVEN an image value contains binary content
- WHEN any TUI surface renders the value
- THEN the system does not dump raw image bytes into the terminal

### Requirement: Generic agent error feedback
r[n8nclone.tui_ux.agent_nodes.errors] The system MUST surface image load, validation, preflight, and Pi or Clankers adapter errors without breaking TUI navigation.

#### Scenario: Missing image path feedback
r[n8nclone.tui_ux.agent_nodes.errors.missing_path]
- GIVEN an image input node references a missing local path
- WHEN the workflow validates or runs from the TUI
- THEN the system displays a clear node-scoped error and keeps the canvas interactive

#### Scenario: Unsupported image format feedback
r[n8nclone.tui_ux.agent_nodes.errors.unsupported_format]
- GIVEN an image input node references an unsupported image format
- WHEN validation reports the format error
- THEN the system displays the diagnostic near the node or output surface and keeps editing recoverable

#### Scenario: Invalid generic agent target feedback
r[n8nclone.tui_ux.agent_nodes.errors.invalid_target]
- GIVEN a generic agent node declares an unsupported target runtime
- WHEN validation reports the target error
- THEN the system displays the diagnostic near the node or properties surface and keeps editing recoverable

#### Scenario: Pi or Clankers adapter error feedback
r[n8nclone.tui_ux.agent_nodes.errors.adapter]
- GIVEN a Pi or Clankers adapter fails while running a generic agent node
- WHEN the run state updates in the TUI
- THEN the system marks the node failed, shows the adapter error summary, and continues handling input

### Requirement: Node-kind provider property metadata
r[n8nclone.tui_ux.node_kind_provider] The system SHOULD render and edit node-specific TUI metadata through node-kind providers instead of scattered presentation-only `NodeConfig` matches.

#### Scenario: Provider-backed property panel
r[n8nclone.tui_ux.node_kind_provider.property_metadata]
- GIVEN a selected node has a registered node-kind provider
- WHEN the TUI renders the properties panel or opens a supported property editor
- THEN the system SHOULD use provider-supplied labels, editable fields, display values, and validation hints while preserving current built-in node behavior

#### Scenario: Unsupported field edit remains recoverable
r[n8nclone.tui_ux.node_kind_provider.unsupported_edit]
- GIVEN a selected node does not support a requested editable field through its provider
- WHEN the user attempts to edit that field
- THEN the system MUST reject the edit without mutating workflow configuration and keep the user in a recoverable TUI state

#### Scenario: Provider fallback preserves display
r[n8nclone.tui_ux.node_kind_provider.fallback]
- GIVEN a built-in node kind has not yet migrated every display path to provider metadata
- WHEN the TUI renders that node during the migration
- THEN the system MUST keep the existing display branch active as a guarded fallback and MUST NOT hide existing node details

### Requirement: Replay debugger timeline
r[n8nclone.tui_ux.replay_debugger.timeline] The system MUST let users inspect active or historical workflow runs as ordered replay timelines.

#### Scenario: Open historical replay
r[n8nclone.tui_ux.replay_debugger.timeline.open]
- GIVEN a run has persisted replay events
- WHEN the user opens that run in the replay debugger
- THEN the system displays event order, node status, cache markers, failure summaries, and selected-event details

#### Scenario: Step replay cursor
r[n8nclone.tui_ux.replay_debugger.timeline.step]
- GIVEN the replay debugger is open
- WHEN the user moves the replay cursor forward or backward
- THEN the system updates the displayed node states from recorded events without mutating the workflow or run history

### Requirement: Active-run replay view
r[n8nclone.tui_ux.replay_debugger.active_run] The system SHOULD feed active runner events into the same timeline model used for historical replay.

#### Scenario: Active events stream into timeline
r[n8nclone.tui_ux.replay_debugger.active_run.stream]
- GIVEN a workflow run is active in the TUI
- WHEN runner events are emitted
- THEN the replay timeline updates with the new safe event summaries while the TUI remains responsive

### Requirement: Replay redaction boundaries
r[n8nclone.tui_ux.replay_debugger.redaction] The system MUST preserve redaction boundaries in replay views.

#### Scenario: Secret remains redacted in replay
r[n8nclone.tui_ux.replay_debugger.redaction.secret]
- GIVEN a recorded run used secret or sensitive payload material
- WHEN the replay debugger renders event details or value summaries
- THEN the system shows redacted summaries and does not reveal plaintext secret material

### Requirement: Credential provider UX
r[n8nclone.tui_ux.credentials.providers] The system SHOULD expose safe credential-provider reachability and reference-editing helpers without revealing secret values.

#### Scenario: Test provider reachability
r[n8nclone.tui_ux.credentials.providers.test]
- GIVEN a credential provider is configured
- WHEN the operator requests a provider test
- THEN the system reports reachable, missing, unauthorized, or timed-out status without printing resolved secret material

#### Scenario: Edit credential reference safely
r[n8nclone.tui_ux.credentials.providers.edit]
- GIVEN a node credential reference is editable
- WHEN the user changes provider id, key identity, or optional field identity
- THEN the system validates the reference shape and preserves the previous reference on invalid input

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

### Requirement: Trigger management surface
r[n8nclone.tui_ux.triggers.manage] The system MAY expose trigger definitions and recent trigger errors in operator-facing UI surfaces.

#### Scenario: List triggers
r[n8nclone.tui_ux.triggers.manage.list]
- GIVEN trigger definitions exist for the current workflow
- WHEN the operator opens the trigger management surface
- THEN the system lists trigger id, kind, enabled state, and last safe status summary

#### Scenario: Show trigger error safely
r[n8nclone.tui_ux.triggers.manage.error]
- GIVEN a trigger has a recent admission or execution error
- WHEN the operator inspects that trigger
- THEN the system shows the safe diagnostic without exposing plaintext secrets or raw sensitive payloads

### Requirement: Webhook listener CLI controls
r[n8nclone.tui_ux.webhook_listener.cli] The system SHOULD expose operator CLI controls for running and inspecting the webhook listener.

#### Scenario: Start listener command
r[n8nclone.tui_ux.webhook_listener.cli.start]
- GIVEN the operator has configured persisted webhook triggers
- WHEN the operator runs the listener command
- THEN the command prints the bind address, loaded trigger count, and safe operational status

#### Scenario: Listener dry-run command
r[n8nclone.tui_ux.webhook_listener.cli.dry_run]
- GIVEN the operator wants to validate listener configuration without accepting network requests
- WHEN the operator runs the listener in dry-run mode
- THEN the command reports trigger matching and credential-reference diagnostics without starting a server

### Requirement: Process-boundary TUI failure recovery
r[lattice.tui_ux.process_failure_recovery] Lattice MUST keep workflow failure, validation diagnostics, explicit rerun, active-run cancellation, and terminal resize paths usable through the real PTY process boundary, and MUST preserve durable run evidence without automatic retry or historical mutation.

#### Scenario: Failed run remains inspectable and explicitly rerunnable
r[lattice.tui_ux.process_failure_recovery.scenario.rerun]
- GIVEN a saved workflow's first real TUI run reaches a deterministic node failure
- WHEN the operator opens run history, inspects the failed node, and explicitly requests rerun
- THEN the TUI MUST remain responsive, MUST start one fresh run identity, MUST preserve the failed historical record unchanged, and MUST display and persist the second run's independent outcome

#### Scenario: Fatal validation prevents effects
r[lattice.tui_ux.process_failure_recovery.scenario.diagnostics]
- GIVEN a saved workflow has a fatal validation diagnostic and a potential observable effect
- WHEN the operator requests execution and opens diagnostics in the real TUI
- THEN Lattice MUST display the bounded diagnostic, MUST start no workflow effect or run record, and MUST continue accepting overlay-close and navigation input

#### Scenario: Active cancellation prevents pending work
r[lattice.tui_ux.process_failure_recovery.scenario.cancel]
- GIVEN a real TUI workflow has a bounded child process that reported readiness and has pending work not yet performed
- WHEN the operator requests active-run cancellation
- THEN Lattice MUST stop pending workflow work, MUST render and persist the bounded cancellation outcome, MUST prevent the pending observable effect, and MUST keep history and quit controls usable

#### Scenario: Resize preserves interaction
r[lattice.tui_ux.process_failure_recovery.scenario.resize]
- GIVEN the real TUI has an active workspace or modal surface
- WHEN the PTY is resized below the surface's preferred dimensions and later restored
- THEN Lattice MUST render without panic or overflow, MUST preserve the current workflow and modal/navigation state, and MUST continue handling keyboard input after each resize

#### Scenario: PTY evidence remains bounded
r[lattice.tui_ux.process_failure_recovery.scenario.non-claim]
- GIVEN the process-boundary TUI matrix passes on a supported Unix/Linux host
- WHEN the result is reported as continuous evidence
- THEN it MUST NOT claim byte-identical rendering across terminal emulators, unsupported-platform behavior, arbitrary child cleanup, workflow semantic correctness, or production readiness

### Requirement: Topological rows and branch lanes

r[lattice.tui_ux.rows.topology] The TUI MUST render each workflow node once in deterministic dependency order, using horizontal lane columns and connectors for forks and joins, including unequal-length branches and disconnected roots.

#### Scenario: Workflow topology snapshots

r[lattice.tui_ux.rows.topology.snapshots]
- GIVEN linear, fork-join, and diamond workflow fixtures
- WHEN the rows surface renders
- THEN maintained screen-column snapshots MUST cover their node order and lane connectors
- AND every dependency MUST precede its child without changing graph coordinates.

### Requirement: Live observed metadata

r[lattice.tui_ux.rows.metadata] Rows MUST display state, local observed start time, elapsed or final observed duration, observed dispatch-attempt count, and a bounded meaningful output summary, updating from actual per-node execution observations before whole-run completion. Provider usage MAY occupy additional columns at wider widths; absent observations MUST NOT be invented.

#### Scenario: Completed and non-successful nodes update during a run

r[lattice.tui_ux.rows.metadata.live]
- GIVEN the workflow still has active or pending work
- WHEN a node's final admitted result is observed
- THEN rows MUST display its actual completed, failed, or skipped state and available output/usage without waiting for the final run event
- AND preparation or admission failures MUST NOT be presented as completed
- AND a skipped or cache-only result without a dispatch-start observation MUST NOT invent an attempt
- AND whole-run completion MUST NOT duplicate already-observed output.

### Requirement: Shared selection and viewport

r[lattice.tui_ux.rows.selection] The canvas `v` binding MUST switch graph and rows layouts without changing selected node identity. Row navigation and clicks MUST update the shared selection and keep it visible; row drags and lateral arrows MUST NOT mutate the hidden spatial graph.

#### Scenario: Navigate beyond the visible rows

r[lattice.tui_ux.rows.selection.viewport]
- GIVEN a workflow has more nodes than fit in the canvas
- WHEN the operator uses arrows, page navigation, Home/End, or the mouse wheel
- THEN the selected row MUST remain visible and its properties/output ownership MUST remain the selected node
- AND clicking a scrolled row MUST select that displayed node, not its unscrolled index.

### Requirement: Bounded visible lanes

r[lattice.tui_ux.rows.wide] Visible lane columns MUST be bounded by terminal width and six lanes, with a `+N` overflow marker and a shared-lane marker, while preserving one independently selectable row per node.

#### Scenario: Fan-out exceeds lane capacity

r[lattice.tui_ux.rows.wide.overflow]
- GIVEN a workflow fan-out exceeds available lane columns
- WHEN rows render or the terminal resizes
- THEN connector width MUST remain bounded, overflow MUST be explicit, and every node MUST remain present and navigable.

### Requirement: Portable workflow authoring parity

r[lattice.tui_ux.portable_authoring_parity] The TUI MUST provide typed create, edit, remove or reset where meaningful, validation, cancellation, persistence, and Nickel round-trip paths for every field in the current portable workflow schema rather than relying on preservation of unexposed imported state.

#### Scenario: Complete portable workflow is authored

r[lattice.tui_ux.portable_authoring_parity.scenario.complete]
- GIVEN the current portable schema includes workflow metadata, graph nodes/ports/edges, all built-in node configs, credentials, runner settings/policies, and triggers
- WHEN an operator authors a workflow through TUI typed flows
- THEN every portable field MUST have a discoverable create/edit/remove-or-reset path with validation and cancellation
- AND the resulting in-memory workflow MUST be representable by typed Nickel without hidden manual source edits.

#### Scenario: Imported preservation does not satisfy authoring

r[lattice.tui_ux.portable_authoring_parity.scenario.preservation_only]
- GIVEN an imported workflow contains a portable field that whole-workflow store serialization would preserve
- WHEN the parity catalog finds no typed TUI authoring action for that field
- THEN parity validation MUST fail with the missing field identity
- AND display-only or untouched save/load behavior MUST NOT satisfy the requirement.

### Requirement: Checked authoring capability catalog

r[lattice.tui_ux.portable_authoring_parity.catalog] Lattice MUST maintain a pure checked capability catalog that covers every current portable field and binds field owner, value kind, create/edit/remove-or-reset support, parser, validation, TUI action, persistence, and Nickel round-trip evidence.

#### Scenario: Catalog covers schema exactly

r[lattice.tui_ux.portable_authoring_parity.catalog.scenario.complete]
- GIVEN the portable schema and authoring catalog from the same source revision
- WHEN completeness validation runs
- THEN every schema field MUST map to exactly one owned capability row with all required evidence classes
- AND duplicate, stale, or unknown rows MUST fail.

#### Scenario: Missing capability blocks closeout

r[lattice.tui_ux.portable_authoring_parity.catalog.scenario.missing]
- GIVEN a portable field is added or an existing capability row is removed or marked display-only
- WHEN parity validation runs
- THEN it MUST report the exact uncovered field and fail closeout.

### Requirement: All built-in node kinds are creatable

r[lattice.tui_ux.portable_authoring_parity.node_kinds] The guided TUI add flow MUST create Shell, HTTP, File, ImageInput, Agent, Condition, Merge, and Subworkflow nodes from validated typed drafts and MUST create no node when a draft is cancelled or invalid.

#### Scenario: HTTP and file nodes are created

r[lattice.tui_ux.portable_authoring_parity.node_kinds.scenario.http_file]
- GIVEN the operator opens the guided add flow
- WHEN valid HTTP or file node fields are entered and confirmed
- THEN the TUI MUST create the selected node kind with matching portable configuration and typed ports
- AND mark the workflow unsaved.

#### Scenario: Invalid or cancelled creation preserves workflow

r[lattice.tui_ux.portable_authoring_parity.node_kinds.scenario.cancel]
- GIVEN any built-in node creation draft is active
- WHEN validation fails or the operator cancels
- THEN the workflow graph, configs, credentials, runner settings, triggers, and dirty state MUST preserve their prior values.

### Requirement: Every built-in node field is editable

r[lattice.tui_ux.portable_authoring_parity.node_fields] Node-kind provider metadata and typed editors MUST cover every current portable Shell, HTTP, File, ImageInput, Agent, Condition, Merge, and Subworkflow configuration field with pure parsing/validation before atomic mutation.

#### Scenario: Valid node field edit commits

r[lattice.tui_ux.portable_authoring_parity.node_fields.scenario.edit]
- GIVEN one selected built-in node and a supported configuration field
- WHEN the operator supplies a valid typed scalar, enum, optional, list, map, mapping, policy, path, or bounded number value
- THEN the TUI MUST update only that field, preserve unrelated imported state, mark the workflow unsaved, and display the new typed value.

#### Scenario: Invalid node field edit is recoverable

r[lattice.tui_ux.portable_authoring_parity.node_fields.scenario.invalid]
- GIVEN a node field edit contains malformed syntax, unsupported enum, duplicate key, unsafe path, empty required value, overflow, or out-of-range number
- WHEN the operator attempts to save
- THEN the pure validator MUST reject it without mutating any workflow field
- AND the TUI MUST preserve the prior value and a recoverable edit state.

### Requirement: Workflow and graph fields are editable

r[lattice.tui_ux.portable_authoring_parity.workflow_graph] The TUI MUST expose typed editors for current portable workflow metadata, node labels/positions, input/output port declarations, compatible edges, and per-node credential references with checked identity allocation and collision rejection.

#### Scenario: Graph metadata and ports are authored

r[lattice.tui_ux.portable_authoring_parity.workflow_graph.scenario.edit]
- GIVEN a workflow draft and selected workflow or node graph owner
- WHEN the operator edits metadata, position, a typed port, edge, or opaque credential reference with valid values
- THEN Lattice MUST apply the exact typed change and preserve all unrelated graph and workflow state.

#### Scenario: Invalid graph edit is atomic

r[lattice.tui_ux.portable_authoring_parity.workflow_graph.scenario.invalid]
- GIVEN an edit would duplicate an id or label where prohibited, overflow identity allocation, create a wrong-direction/self/incompatible edge, orphan an incident edge, or introduce a malformed credential reference
- WHEN validation runs
- THEN Lattice MUST reject the complete edit without partial graph, config, credential, runner, or trigger mutation.

### Requirement: Runner policy fields are editable

r[lattice.tui_ux.portable_authoring_parity.runner] The TUI MUST expose typed per-node editing for current portable runtime kind/key, ability and effect requirements, cache policy, retry and backoff policy, artifact-reference policy, and associated named bounds.

#### Scenario: Valid runner policy is stored

r[lattice.tui_ux.portable_authoring_parity.runner.scenario.edit]
- GIVEN a selected node and a valid supported runtime/policy draft
- WHEN the operator confirms the edit
- THEN Lattice MUST store equivalent runner settings and preserve them through save/load and Nickel export/import.

#### Scenario: Unsupported runner policy is rejected

r[lattice.tui_ux.portable_authoring_parity.runner.scenario.invalid]
- GIVEN a runner draft has a missing binding, unknown policy, duplicate ability/effect, zero or excessive bound, overflow, or incompatible artifact rule
- WHEN the operator attempts to save
- THEN Lattice MUST reject it without changing the previous runner settings.

### Requirement: Trigger definitions are editable without activation

r[lattice.tui_ux.portable_authoring_parity.triggers] The TUI MUST let operators browse, add, edit, disable, and remove current portable manual, schedule, and webhook trigger definitions through typed validated drafts without starting a listener, schedule, workflow run, or other effect during authoring.

#### Scenario: Valid trigger draft commits

r[lattice.tui_ux.portable_authoring_parity.triggers.scenario.edit]
- GIVEN the current workflow and a valid manual, schedule, or webhook trigger draft
- WHEN the operator confirms the draft
- THEN Lattice MUST store the trigger definition with equivalent workflow reference, input, authentication reference, overlap, and idempotency policy
- AND it MUST perform no trigger or workflow effect.

#### Scenario: Invalid or cancelled trigger draft is inert

r[lattice.tui_ux.portable_authoring_parity.triggers.scenario.invalid]
- GIVEN a trigger draft is malformed, duplicates an identity, targets the wrong workflow, contains an invalid schedule/path/auth reference/policy, or is cancelled
- WHEN the draft closes
- THEN the prior trigger set and store MUST remain unchanged and no listener, scheduler, run, or credential resolution may start.

### Requirement: Aggregate node deletion

r[lattice.tui_ux.aggregate_node_deletion] TUI node deletion MUST use one workflow-level lifecycle mutation that removes the graph node, incident edges, node configuration, credentials, runner settings, invalid node-owned references, and matching TUI focus/selection/edit state.

#### Scenario: Configured node is deleted cleanly

r[lattice.tui_ux.aggregate_node_deletion.scenario.delete]
- GIVEN a selected node has incident edges, configuration, credential references, runner settings, and an active property draft
- WHEN the operator confirms deletion
- THEN one aggregate mutation MUST remove every state item owned by that node and clear its TUI state
- AND workflow validation, store save/load, and Nickel export/import MUST report no orphan state.

#### Scenario: Missing deletion target changes nothing

r[lattice.tui_ux.aggregate_node_deletion.scenario.missing]
- GIVEN the requested node identity is absent or the delete plan is stale
- WHEN deletion is applied
- THEN Lattice MUST reject or no-op deterministically without removing state owned by another node or changing dirty state.

### Requirement: Condition topology reconciliation

r[lattice.tui_ux.condition_topology_reconciliation] Condition branch editing MUST reconcile output ports by validated branch label identity, preserving matching port ids and edges, creating ports for new labels, removing deleted ports and incident edges, treating rename as remove-plus-add, and applying no mutation on failure or cancellation.

#### Scenario: Branch cardinality changes atomically

r[lattice.tui_ux.condition_topology_reconciliation.scenario.change]
- GIVEN a condition node has valid branches, output ports, and outgoing edges
- WHEN a valid edit adds, removes, renames, or reorders branches
- THEN unchanged labels MUST retain port identities and incident edges, new labels MUST receive fresh collision-free ports, deleted labels MUST lose ports and incident edges, and output order MUST match the edited branch order
- AND workflow validation MUST observe one matching output per branch.

#### Scenario: Invalid branch edit preserves topology

r[lattice.tui_ux.condition_topology_reconciliation.scenario.reject]
- GIVEN a branch edit has an empty or duplicate label, malformed predicate, identifier exhaustion, stale node, or is cancelled
- WHEN reconciliation runs
- THEN it MUST reject the edit without changing branch config, ports, edges, focus, or dirty state.

### Requirement: TUI-to-Nickel parity round trip

r[lattice.tui_ux.portable_authoring_parity.round_trip] Lattice MUST maintain exhaustive fixtures that author every current capability through TUI actions and prove equivalent in-memory, store, exported Nickel, and re-imported workflow meaning.

#### Scenario: Complete authored workflow round trips

r[lattice.tui_ux.portable_authoring_parity.round_trip.scenario.complete]
- GIVEN a TUI-authored workflow exercises all eight node kinds, every portable field family, typed ports/edges, credentials, runner policies, control-flow mappings, and all trigger kinds
- WHEN it is saved, loaded, exported to Nickel, and imported
- THEN the resulting workflow MUST preserve equivalent portable meaning and validate without fatal diagnostics.

#### Scenario: Invalid authored workflow does not partially persist

r[lattice.tui_ux.portable_authoring_parity.round_trip.scenario.invalid]
- GIVEN an authoring draft, deletion plan, or condition reconciliation would make the workflow invalid
- WHEN validation or persistence admission runs
- THEN Lattice MUST preserve the prior in-memory workflow and store bytes and MUST NOT emit a misleading importable export.

### Requirement: Complete authoring is discoverable

r[lattice.tui_ux.portable_authoring_parity.discoverability] Command palette, contextual help, and owner-specific property surfaces MUST make every authoring field family discoverable while remaining usable at constrained terminal dimensions.

#### Scenario: Operator finds every editor family

r[lattice.tui_ux.portable_authoring_parity.discoverability.scenario.help]
- GIVEN the TUI is focused on the canvas, workflow, node, runner, credential, or trigger owner
- WHEN contextual help or the command palette is rendered
- THEN it MUST expose the corresponding create/edit/remove actions without requiring undocumented key knowledge.

### Requirement: Authoring parity verification includes failures

r[lattice.tui_ux.portable_authoring_parity.verification] Completion MUST pair positive field-family, aggregate deletion, condition reconciliation, store, Nickel, and TUI/PTY fixtures with negative malformed, duplicate, overflow, stale, cancelled, orphan, dangling-edge, hidden-field, save-failure, and constrained-render fixtures.

#### Scenario: Closeout evidence covers the parity matrix

r[lattice.tui_ux.portable_authoring_parity.verification.scenario.closeout]
- GIVEN maintainers intend to archive the parity change
- WHEN closeout validation runs
- THEN evidence MUST show a complete capability matrix, focused positive and negative tests, repository checks, Octet result, Cairn validation/gates, and selected Nix checks
- AND every skipped rail MUST include an exact blocker and next-best check.

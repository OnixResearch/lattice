## ADDED Requirements

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

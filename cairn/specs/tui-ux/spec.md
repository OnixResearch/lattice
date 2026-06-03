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

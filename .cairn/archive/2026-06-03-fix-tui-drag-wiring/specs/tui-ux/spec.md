## ADDED Requirements

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

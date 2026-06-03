## ADDED Requirements

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

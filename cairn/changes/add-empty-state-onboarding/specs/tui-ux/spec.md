## ADDED Requirements

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

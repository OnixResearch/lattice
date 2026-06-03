## ADDED Requirements

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

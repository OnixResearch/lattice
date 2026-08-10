## ADDED Requirements

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

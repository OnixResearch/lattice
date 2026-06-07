## ADDED Requirements

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

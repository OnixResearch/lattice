## ADDED Requirements

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

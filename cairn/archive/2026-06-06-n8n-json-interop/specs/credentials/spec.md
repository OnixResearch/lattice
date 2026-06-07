## ADDED Requirements

### Requirement: n8n credential placeholders
r[n8nclone.credentials.n8n_json.placeholders] The system MUST import n8n credential references as opaque placeholders and reject plaintext credential material.

#### Scenario: Import credential reference placeholder
r[n8nclone.credentials.n8n_json.placeholders.reference]
- GIVEN an n8n JSON node references a credential identity without plaintext secret material
- WHEN the workflow is imported
- THEN the system stores an opaque credential reference placeholder that must be resolved through a configured provider before execution

#### Scenario: Reject plaintext n8n credential material
r[n8nclone.credentials.n8n_json.placeholders.plaintext]
- GIVEN an n8n JSON import contains plaintext credential values or decrypted credential material
- WHEN import validation runs
- THEN the system rejects the credential data and reports a fatal diagnostic without saving plaintext secrets

#### Scenario: Export keeps credential references opaque
r[n8nclone.credentials.n8n_json.placeholders.export]
- GIVEN a n8nclone workflow with credential references is exported to n8n JSON
- WHEN export succeeds for supported nodes
- THEN the system writes only opaque credential reference identities and omits plaintext secret material

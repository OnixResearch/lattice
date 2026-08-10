## ADDED Requirements

### Requirement: n8n parity secret safety
r[n8nclone.credentials.n8n_parity_secret_safety] The system MUST keep n8n parity fixtures, reports, diagnostics, and test output free of plaintext credential material.

#### Scenario: Credential placeholders stay opaque
r[n8nclone.credentials.n8n_parity_secret_safety.placeholders]
- GIVEN a supported n8n parity fixture contains credential references or placeholders
- WHEN the fixture imports and the mapping report is generated
- THEN the report preserves opaque credential identity without exposing plaintext credential values

#### Scenario: Plaintext credential fixture is rejected safely
r[n8nclone.credentials.n8n_parity_secret_safety.plaintext_rejected]
- GIVEN an unsupported n8n parity fixture contains plaintext credential material
- WHEN import validation runs
- THEN the fixture is rejected and diagnostics omit the plaintext value while explaining how to use credential references

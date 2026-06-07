## ADDED Requirements

### Requirement: n8n parity diagnostics
r[n8nclone.workflow_validation.n8n_parity_diagnostics] The system MUST reject unsupported n8n parity fixtures with deterministic, actionable diagnostics that identify the unsupported feature class.

#### Scenario: Unsupported node diagnostic
r[n8nclone.workflow_validation.n8n_parity_diagnostics.unsupported_node]
- GIVEN a parity fixture contains an unsupported n8n node kind
- WHEN import validation runs
- THEN the diagnostic identifies the unsupported node kind without panicking or persisting a partial workflow

#### Scenario: Unsupported expression diagnostic
r[n8nclone.workflow_validation.n8n_parity_diagnostics.unsupported_expression]
- GIVEN a parity fixture contains unsupported n8n expression syntax
- WHEN import validation runs
- THEN the diagnostic identifies the unsupported expression class and keeps the original expression out of unsafe execution paths

#### Scenario: Malformed connection diagnostic
r[n8nclone.workflow_validation.n8n_parity_diagnostics.malformed_connection]
- GIVEN a parity fixture contains malformed n8n connection data
- WHEN import validation runs
- THEN the diagnostic identifies the malformed connection and rejects the fixture deterministically

#### Scenario: Plaintext credential diagnostic
r[n8nclone.workflow_validation.n8n_parity_diagnostics.plaintext_credential]
- GIVEN a parity fixture contains plaintext credential material
- WHEN import validation runs
- THEN the diagnostic rejects the fixture and reports only safe credential-placeholder guidance

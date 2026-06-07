## ADDED Requirements

### Requirement: Credential provider format
r[n8nclone.workflow_format.credentials.providers] The system MUST import and export credential provider references without plaintext secret material.

#### Scenario: Export provider references
r[n8nclone.workflow_format.credentials.providers.export]
- GIVEN a workflow contains credential references targeting configured providers
- WHEN the workflow is exported
- THEN the system writes provider id, key identity, and optional field identity without resolved secret values

#### Scenario: Import provider references
r[n8nclone.workflow_format.credentials.providers.import]
- GIVEN a workflow file declares credential provider references with valid shapes
- WHEN the workflow is imported
- THEN the system saves the references as opaque credential identities and does not require resolving plaintext secrets during import

#### Scenario: Reject plaintext provider payload
r[n8nclone.workflow_format.credentials.providers.reject_plaintext]
- GIVEN a workflow file attempts to include local vault payloads or resolved secret values inline
- WHEN import validation runs
- THEN the system rejects the file and preserves existing saved workflows

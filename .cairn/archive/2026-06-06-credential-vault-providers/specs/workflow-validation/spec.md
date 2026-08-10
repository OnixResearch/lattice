## ADDED Requirements

### Requirement: Credential provider validation
r[n8nclone.workflow_validation.credentials.providers] The system MUST validate credential provider references and capabilities before node execution.

#### Scenario: Unsupported field access is fatal
r[n8nclone.workflow_validation.credentials.providers.field]
- GIVEN a credential reference requests a field from a provider that does not support field access
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic before execution starts

#### Scenario: Malformed provider key is fatal
r[n8nclone.workflow_validation.credentials.providers.key]
- GIVEN a credential reference contains a malformed key for its provider
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic with safe provider and key context

#### Scenario: Missing provider config is fatal
r[n8nclone.workflow_validation.credentials.providers.config]
- GIVEN workflow execution requires a credential provider that is absent from runtime configuration
- WHEN validation or runner admission runs
- THEN the system reports a fatal diagnostic before invoking side-effecting runtimes

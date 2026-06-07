## ADDED Requirements

### Requirement: Webhook authentication credentials
r[n8nclone.credentials.webhook_auth] The system MUST resolve webhook authentication material through credential references instead of storing plaintext secrets in trigger definitions.

#### Scenario: Valid webhook secret authenticates request
r[n8nclone.credentials.webhook_auth.valid]
- GIVEN a webhook trigger references a credential provider secret for authentication
- WHEN the provider resolves the secret and the inbound request signature is valid
- THEN the system admits the request without persisting the plaintext secret

#### Scenario: Missing webhook secret rejects request
r[n8nclone.credentials.webhook_auth.missing]
- GIVEN a webhook trigger references a credential provider secret that cannot be resolved
- WHEN an inbound webhook request arrives
- THEN the system rejects the request and does not start the workflow

#### Scenario: Webhook diagnostics redact secret material
r[n8nclone.credentials.webhook_auth.redaction]
- GIVEN webhook authentication fails after resolving secret material
- WHEN the diagnostic is rendered or persisted
- THEN the system omits plaintext secret material and includes only safe trigger and credential reference context

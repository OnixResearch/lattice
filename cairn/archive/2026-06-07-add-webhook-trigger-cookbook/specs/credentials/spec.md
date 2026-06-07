## ADDED Requirements

### Requirement: Webhook cookbook secret safety
r[n8nclone.credentials.webhook_cookbook_secret_safety] The system MUST keep webhook cookbook authentication examples credential-reference based and must not expose plaintext secret material in cookbook artifacts or smoke evidence.

#### Scenario: Cookbook uses credential references
r[n8nclone.credentials.webhook_cookbook_secret_safety.references]
- GIVEN the webhook cookbook demonstrates authenticated webhook requests
- WHEN the workflow, docs, and tests are inspected
- THEN they show credential references and setup placeholders instead of plaintext webhook secret values

#### Scenario: Cookbook redaction assertions
r[n8nclone.credentials.webhook_cookbook_secret_safety.redaction]
- GIVEN cookbook smoke tests exercise accepted and rejected authenticated webhook requests
- WHEN responses, diagnostics, run history, logs, and documentation snippets are checked
- THEN authorization headers, bearer values, signature values, raw sensitive payload fields, and credential provider values are absent

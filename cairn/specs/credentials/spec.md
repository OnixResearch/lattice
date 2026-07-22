# Credentials Specification

## Purpose

Defines the `credentials` capability.

## Requirements

### Requirement: Secret reference model
r[n8nclone.credentials.secret_refs.model] The system MUST represent workflow credentials as opaque secret references instead of plaintext secret values and MUST reject plaintext credential fields through a stable redaction-safe credential diagnostic before store access.

#### Scenario: Store secret reference
r[n8nclone.credentials.secret_refs.model.store_ref]
- GIVEN a node resolves an execution-only credential and the workflow is subsequently saved or exported
- WHEN repository-owned store and Nickel export boundaries serialize the workflow
- THEN they store only provider, key, optional field, and binding identities and do not serialize the resolved plaintext value

#### Scenario: Reject inline secret field
r[n8nclone.credentials.secret_refs.model.reject_plaintext]
- GIVEN an imported workflow provides plaintext secret material in a credential field
- WHEN workflow import runs
- THEN the system returns a stable credential diagnostic, omits the plaintext value from diagnostics, and does not create or mutate the workflow store

### Requirement: Credential provider interface
r[n8nclone.credentials.provider_interface] The system MUST resolve secret references through a credential-provider interface at execution boundaries.

#### Scenario: Resolve secret for node execution
r[n8nclone.credentials.provider_interface.resolve]
- GIVEN a node references a credential that the configured provider can resolve
- WHEN the node is prepared for execution
- THEN the system supplies the resolved secret to the node adapter without storing it in the workflow

#### Scenario: Missing secret fails safely
r[n8nclone.credentials.provider_interface.missing]
- GIVEN a node references a credential that the configured provider cannot resolve
- WHEN the workflow runs
- THEN the system fails the node with a clear credential error and does not execute the side-effecting adapter

### Requirement: Secret redaction
r[n8nclone.credentials.redaction] The system MUST redact resolved secret material before user-visible output, tracing logs, run history, JSON summaries, or persisted run-detail surfaces can observe it.

#### Scenario: Redact run output
r[n8nclone.credentials.redaction.output]
- GIVEN a node emits a resolved secret or overlapping secret fragment through stdout and stderr
- WHEN execution output, tracing logs, run history, JSON summaries, and persisted run details are observed
- THEN every surface MUST omit the plaintext values, output/history/JSON/persisted details MUST preserve explicit redaction markers, and tracing logs MUST retain safe non-secret node context

#### Scenario: Redact validation message
r[n8nclone.credentials.redaction.validation]
- GIVEN a credential diagnostic mentions a secret reference and a separate plaintext value exists
- WHEN text and JSON diagnostics are rendered
- THEN both renderings MUST preserve the reference identity and MUST omit the plaintext value

### Requirement: Credential validation
r[n8nclone.credentials.validation] The system SHOULD validate secret-reference shape before execution.

#### Scenario: Malformed secret reference
r[n8nclone.credentials.validation.malformed]
- GIVEN a node contains a malformed secret reference
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic before execution starts

### Requirement: Credential cache identity
r[n8nclone.credentials.cache_identity] The system MUST avoid hashing plaintext secret material into cache keys.

#### Scenario: Cache key uses reference identity
r[n8nclone.credentials.cache_identity.reference]
- GIVEN a node uses a credential during execution
- WHEN the system computes the node cache key
- THEN the key includes the credential reference identity and excludes the resolved secret value

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
- GIVEN a Lattice workflow with credential references is exported to n8n JSON
- WHEN export succeeds for supported nodes
- THEN the system writes only opaque credential reference identities and omits plaintext secret material

### Requirement: Credential provider registry
r[n8nclone.credentials.provider_registry] The system MUST resolve credential references through an explicit configured provider registry.

#### Scenario: Resolve through registered provider
r[n8nclone.credentials.provider_registry.resolve]
- GIVEN a node references a credential provider id and key that exist in the configured registry
- WHEN the node is prepared for execution
- THEN the system resolves the secret through that provider and does not persist the plaintext value

#### Scenario: Unknown provider fails safely
r[n8nclone.credentials.provider_registry.unknown]
- GIVEN a node references a credential provider id that is not configured
- WHEN workflow validation or execution preparation runs
- THEN the system reports a fatal credential diagnostic before invoking the side-effecting node runtime

### Requirement: Provider diagnostics redaction
r[n8nclone.credentials.provider_diagnostics.redaction] The system MUST redact resolved secret material from provider diagnostics and reachability checks.

#### Scenario: Provider test hides secret
r[n8nclone.credentials.provider_diagnostics.redaction.test]
- GIVEN a provider reachability check successfully resolves a secret
- WHEN the result is rendered to a user or persisted
- THEN the system reports success without displaying the secret value

#### Scenario: Provider error hides secret fragments
r[n8nclone.credentials.provider_diagnostics.redaction.error]
- GIVEN a provider failure message contains text derived from secret material
- WHEN the diagnostic is rendered
- THEN the system redacts the secret fragments while preserving safe provider and key context

### Requirement: Environment credential provider
r[n8nclone.credentials.providers.env] The system SHOULD provide an environment-variable credential provider for development and deployment environments.

#### Scenario: Environment secret resolves
r[n8nclone.credentials.providers.env.resolve]
- GIVEN a credential reference targets an environment variable that is present
- WHEN credential resolution runs
- THEN the provider returns the environment value only to the execution boundary

#### Scenario: Missing environment secret fails
r[n8nclone.credentials.providers.env.missing]
- GIVEN a credential reference targets an environment variable that is absent
- WHEN credential resolution runs
- THEN the provider returns a missing-secret error without executing the node runtime

### Requirement: Local encrypted credential vault
r[n8nclone.credentials.providers.local_vault] The system MAY provide a local encrypted vault provider for operator-managed secrets.

#### Scenario: Vault secret resolves with valid key
r[n8nclone.credentials.providers.local_vault.resolve]
- GIVEN a local vault contains an encrypted secret and the configured key source unlocks it
- WHEN credential resolution runs
- THEN the provider returns the plaintext only to the execution boundary and keeps the workflow file free of secret material

#### Scenario: Tampered vault entry is rejected
r[n8nclone.credentials.providers.local_vault.tamper]
- GIVEN a local vault entry fails integrity validation
- WHEN credential resolution runs
- THEN the provider rejects the entry and reports a redaction-safe diagnostic

### Requirement: External command credential provider
r[n8nclone.credentials.providers.command] The system MAY resolve secrets through an explicitly configured external command provider.

#### Scenario: Command provider returns secret
r[n8nclone.credentials.providers.command.resolve]
- GIVEN an external command provider is configured and returns a valid secret response within limits
- WHEN credential resolution runs
- THEN the system supplies the secret to the execution boundary and redacts command output from diagnostics

#### Scenario: Command provider timeout fails safely
r[n8nclone.credentials.providers.command.timeout]
- GIVEN an external command provider exceeds its configured timeout
- WHEN credential resolution runs
- THEN the system fails the node preparation and does not invoke the side-effecting runtime

### Requirement: Provider cache identity
r[n8nclone.credentials.provider_cache_identity] The system MUST include credential provider reference identity but exclude resolved secret values from cache keys.

#### Scenario: Provider-backed cache key avoids plaintext
r[n8nclone.credentials.provider_cache_identity.no_plaintext]
- GIVEN a node resolves a credential through any supported provider
- WHEN the system computes the node cache key
- THEN the key includes provider id, key identity, optional field identity, and excludes the resolved secret bytes

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

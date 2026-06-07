## ADDED Requirements

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

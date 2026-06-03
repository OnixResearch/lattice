# Credentials Specification

## Purpose

Defines the `credentials` capability.

## Requirements

### Requirement: Secret reference model
r[n8nclone.credentials.secret_refs.model] The system MUST represent workflow credentials as opaque secret references instead of plaintext secret values.

#### Scenario: Store secret reference
r[n8nclone.credentials.secret_refs.model.store_ref]
- GIVEN a node configuration needs a credential
- WHEN the workflow is saved or exported
- THEN the system stores only a secret reference and does not serialize plaintext secret material

#### Scenario: Reject inline secret field
r[n8nclone.credentials.secret_refs.model.reject_plaintext]
- GIVEN an imported workflow attempts to provide plaintext secret material in a credential field
- WHEN workflow validation runs
- THEN the system rejects the field and reports a credential diagnostic

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
r[n8nclone.credentials.redaction] The system MUST redact secret material from user-visible and persisted diagnostic surfaces.

#### Scenario: Redact run output
r[n8nclone.credentials.redaction.output]
- GIVEN a node uses a resolved secret during execution
- WHEN output, logs, run history, or JSON summaries are rendered
- THEN the system masks the secret material while preserving useful non-secret context

#### Scenario: Redact validation message
r[n8nclone.credentials.redaction.validation]
- GIVEN a credential diagnostic mentions a secret reference
- WHEN the diagnostic is rendered
- THEN the system displays the reference identity without displaying secret material

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

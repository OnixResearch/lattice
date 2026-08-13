# UCAN Token Flow OIDC Exchange Delta

## ADDED Requirements

### Requirement: ucan_token_flow.oidc.versioned_policy

r[ucan_token_flow.oidc.versioned_policy]

Basalt SHALL evaluate OIDC exchange under a typed, versioned Nickel policy that names trusted issuers, audiences, algorithms, claim predicates, freshness, network bounds, cache bounds, Mantle capabilities, and UCAN lifetime limits.

#### Scenario: Policy export is valid

- **GIVEN** an accepted OIDC exchange policy
- **WHEN** Basalt exports it for runtime use
- **THEN** the export SHALL be canonical and BLAKE3-identified
- **AND** it SHALL contain no private key, provider credential, OIDC token, or secret value

#### Scenario: Required policy field is absent

- **GIVEN** a policy without an issuer, audience, algorithm, freshness, claim, capability, or network bound required by its profile
- **WHEN** the contract validates it
- **THEN** validation SHALL fail before service startup

### Requirement: ucan_token_flow.oidc.pure_verification

r[ucan_token_flow.oidc.pure_verification]

Basalt SHALL verify OIDC structure, algorithm, key use, signature, issuer, audience, time claims, selected identity claims, and requested capability in a pure core with explicit time and public-key inputs.

#### Scenario: GitHub token satisfies policy

- **GIVEN** a correctly signed token from the configured issuer with the required audience, fresh time claims, accepted repository and workflow claims, and an allowed capability request
- **WHEN** the core verifies it
- **THEN** it SHALL return bounded identity facts and an attenuated issuance plan

#### Scenario: One verification condition fails

- **GIVEN** a malformed token, disallowed algorithm, wrong key use, bad signature, wrong issuer or audience, stale time claim, wrong claim type, claim mismatch, or excessive request
- **WHEN** the core verifies it
- **THEN** it SHALL return a stable typed rejection
- **AND** it SHALL NOT produce an issuance plan

### Requirement: ucan_token_flow.oidc.bounded_jwks

r[ucan_token_flow.oidc.bounded_jwks]

Basalt SHALL retrieve OIDC discovery and JWKS documents only from configured HTTPS origins under named transport, redirect, document, key-count, freshness, and refresh bounds.

#### Scenario: Unknown key rotates safely

- **GIVEN** a token with an unknown `kid` and a configured issuer
- **WHEN** the current key set is still eligible for bounded refresh
- **THEN** the shell MAY fetch one fresh bounded key set
- **AND** the core SHALL verify only a policy-allowed candidate key

#### Scenario: Discovery or key retrieval is unsafe

- **GIVEN** an unapproved redirect, issuer mismatch, TLS failure, timeout, oversized document, malformed document, excessive key set, or stale unavailable cache
- **WHEN** the shell retrieves verification material
- **THEN** exchange SHALL fail closed
- **AND** it SHALL NOT use an ambient or unconfigured key

### Requirement: ucan_token_flow.oidc.attenuated_ucan

r[ucan_token_flow.oidc.attenuated_ucan]

Basalt SHALL issue a UCAN only after successful OIDC verification, and the UCAN audience, operations, project, resource scope, and lifetime SHALL NOT exceed policy or verified OIDC validity.

#### Scenario: Allowed build capability is exchanged

- **GIVEN** verified identity facts and a request for an allowed Mantle build resource
- **WHEN** Basalt issues the UCAN
- **THEN** the token SHALL target the configured Mantle audience
- **AND** it SHALL contain only the granted capability scope
- **AND** it SHALL expire no later than both applicable limits

#### Scenario: Caller requests broader authority

- **GIVEN** a request for an unapproved operation, project, resource, audience, or lifetime
- **WHEN** attenuation evaluates it
- **THEN** exchange SHALL reject or reduce it only as the explicit policy defines
- **AND** it SHALL never silently grant the broader request

### Requirement: ucan_token_flow.oidc.replay_boundary

r[ucan_token_flow.oidc.replay_boundary]

Basalt SHALL bind an exchange to the requested Mantle audience and capability and SHALL apply the configured replay policy before issuance.

#### Scenario: One-time exchange repeats

- **GIVEN** a policy that requires one-time exchange and a replay key already committed for the same public token facts and request
- **WHEN** the caller repeats exchange
- **THEN** Basalt SHALL reject the replay
- **AND** it SHALL NOT issue another UCAN

#### Scenario: Replay storage is unavailable

- **GIVEN** a policy that requires replay prevention and an unavailable replay store
- **WHEN** exchange reaches replay admission
- **THEN** exchange SHALL fail closed

### Requirement: ucan_token_flow.oidc.redacted_evidence

r[ucan_token_flow.oidc.redacted_evidence]

Basalt SHALL record bounded exchange decisions without raw OIDC tokens, UCAN bearer tokens, private keys, provider credentials, full JWK sets, or secret-derived hashes.

#### Scenario: Exchange succeeds or fails

- **GIVEN** any terminal exchange outcome
- **WHEN** Basalt emits evidence
- **THEN** evidence MAY include policy, issuer profile, selected public subject, requested and granted capability, key identifier, and reason code
- **AND** it SHALL omit bearer and secret material

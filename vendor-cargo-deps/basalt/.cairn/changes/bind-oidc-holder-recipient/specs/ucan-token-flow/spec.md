## MODIFIED Requirements

### Requirement: ucan_token_flow.oidc.replay_boundary

r[ucan_token_flow.oidc.replay_boundary]

Basalt SHALL admit at most one exchange per OIDC token. The replay key SHALL cover only the token issuer and `jti`, so a later request for another audience, project, operation, resource or holder SHALL be a duplicate. The shell SHALL reserve the key until `max(exp, iat + max_token_age_seconds) + clock_skew_seconds`, the last time at which the same token can still pass verification, and SHALL apply the configured replay policy before issuance.

#### Scenario: One OIDC token is exchanged again

- **GIVEN** a policy that requires one-time exchange and a token that was already exchanged
- **WHEN** the caller exchanges the same token for the same or another operation, resource or holder
- **THEN** Basalt SHALL reject the replay
- **AND** it SHALL NOT issue another UCAN

#### Scenario: Reservation outlives the issued UCAN

- **GIVEN** a token that remains acceptable after the UCAN issued from it expires
- **WHEN** the caller exchanges the same token after that UCAN expired
- **THEN** the reservation SHALL still be held and Basalt SHALL reject the replay

#### Scenario: A fresh token is exchanged

- **GIVEN** a token with a new `jti`
- **WHEN** the caller exchanges it
- **THEN** Basalt SHALL admit it once

#### Scenario: Replay storage is unavailable

- **GIVEN** a policy that requires replay prevention and an unavailable replay store
- **WHEN** exchange reaches replay admission
- **THEN** exchange SHALL fail closed

## ADDED Requirements

### Requirement: OIDC holder delivery is bound to one exact Mantle request

Explicit OIDC operation selections MAY include the native Mantle event-read and result-read actions. Shipped default policy MUST NOT acquire these actions implicitly, and the policy contract MUST reject an unsupported, empty or duplicated operation selection.
r[ucan_token_flow.oidc.holder_recipient] Basalt MUST support explicit typed holder delivery that carries the typed Mantle request the holder will sign and its policy artifact reference. Before key load and replay, the request subject MUST equal the recipient, its audience and project MUST equal the policy's service audience and project, its ability MUST match the requested operation, and the requested resource MUST equal the request's canonical resource exactly. The issued UCAN MUST name the recipient and carry exactly that canonical resource. A recipient equal to the issuing signer MUST be refused before key load and replay. Recipient selection MUST NOT claim holder possession or bypass issuer admission. Metadata-only receipt capability identities MUST bind the recipient without including bearer tokens. The legacy exchange MUST keep issuing to the policy's service audience.

#### Scenario: Holder delivery carries the exact canonical grant
- GIVEN an accepted OIDC token, admitted service policy and a holder's Mantle request naming that holder, service and project
- WHEN the recipient exchange issues a UCAN
- THEN the signed token audience is the holder, its only capability is the request's canonical resource and ability, and expiration remains bounded by the service policy
- AND the native Mantle verifier admits the holder-signed invocation for that request and denies it for another project

#### Scenario: Unbound holder requests are refused before effects
- GIVEN a holder request with another project, service audience or subject, or a requested resource that is a bare or truncated prefix of the canonical resource, or a recipient equal to the issuer
- WHEN the recipient exchange is attempted
- THEN it MUST deny before key load and replay and issue no UCAN

### Requirement: Shared native Mantle signing canonicalization
r[ucan_token_flow.oidc.native_signing] Basalt MUST expose pure Mantle canonicalization using an explicit request and policy artifact reference without requiring fabricated receiver policy or trust roots. The native verifier MUST retain operator-configured generation, endpoint and audience admission before invoking this canonicalization, and for an admitted request both MUST return the same enforcement request. Canonicalization MUST preserve exact length framing, all signed request fields, nonzero generation and per-field/aggregate bounds, MUST reject fields with surrounding whitespace, and MUST NOT confer authority.

#### Scenario: Holder and receiver share one signed representation
- GIVEN an explicit native Mantle request and operator-pinned policy reference
- WHEN a holder constructs its signing resource and the native receiver admits the invocation
- THEN the actual signature MUST cover the same canonical fields and request substitutions MUST fail before replay consumption

# Nostr Identity Bindings Specification

## ADDED Requirements

### Requirement: Basalt accepts only reviewed verified Nostr inputs [r[nostr_identity_bindings.verified_input]]

Basalt MUST accept Nostr authentication only through the exact reviewed `VerifiedNostrCredential` contract. Basalt MUST reject caller-supplied validity booleans.

#### Scenario: Reviewed verified input passes the boundary

- GIVEN the UCAN Nostr adapter verified an event for the exact request
- AND the adapter schema and profile identities match Basalt policy
- WHEN Basalt evaluates the credential
- THEN it MAY continue to subject resolution
- AND it MUST preserve the adapter and protocol identities

#### Scenario: Forged verification fact fails

- GIVEN a caller supplies `signature_valid = true`, `authenticated = true`, an unknown adapter profile, or equivalent unverified metadata
- WHEN Basalt evaluates the input
- THEN evaluation MUST fail before subject resolution or authorization

### Requirement: Nickel policy owns stable subject bindings [r[nostr_identity_bindings.policy_contract]]

Basalt MUST define versioned Nickel policy for stable subjects and typed Nostr authenticators. Generated runtime artifacts MUST remain bound to reviewed policy bytes.

#### Scenario: Valid binding policy loads

- GIVEN each subject and authenticator has a unique stable identifier
- AND each Nostr key has canonical form and one active subject owner
- WHEN policy validation runs
- THEN validation MUST produce a deterministic subject index

#### Scenario: Invalid binding policy fails

- GIVEN a policy has malformed keys, duplicate identifiers, ambiguous active ownership, unknown lifecycle states, or weakened non-claims
- WHEN policy validation runs
- THEN validation MUST fail before authentication or enforcement

### Requirement: Subject resolution is exact and deterministic [r[nostr_identity_bindings.subject_resolution]]

Basalt MUST resolve one canonical active Nostr key to one stable subject. It MUST bind the result to the exact credential and request.

#### Scenario: Active key resolves

- GIVEN a verified credential uses one active key bound to one subject
- AND its audience, purpose, challenge, and request match the expected values
- WHEN subject resolution runs
- THEN Basalt MUST construct `AuthenticatedSubject`
- AND the result MUST identify the exact subject, binding, policy, event, and request

#### Scenario: Missing or ambiguous key denies

- GIVEN a verified key is absent, inactive, revoked, duplicated, or bound to multiple subjects
- WHEN subject resolution runs
- THEN resolution MUST deny with a deterministic reason
- AND no fallback subject MUST be selected

### Requirement: Authentication never grants authority [r[nostr_identity_bindings.authority_separation]]

`AuthenticatedSubject` MUST NOT grant a resource or ability. Authorization MUST remain a separate Basalt and UCAN decision for the exact request.

#### Scenario: Matching authentication and authorization compose

- GIVEN Nostr authentication and UCAN authorization both pass
- AND both results bind the same request and configured subject policy
- WHEN composition runs
- THEN Basalt MAY return a composed allow
- AND the allow MUST not exceed the UCAN resource, ability, caveats, or time bounds

#### Scenario: Authentication without authority denies

- GIVEN Nostr authentication passes
- AND UCAN authorization is missing, denied, stale, or scoped to another subject or request
- WHEN composition runs
- THEN the requested action MUST remain denied

### Requirement: Key lifecycle is explicit [r[nostr_identity_bindings.key_lifecycle]]

Basalt MUST support active, retired, and revoked Nostr bindings. Stable subject identity MUST remain separate from authenticator keys.

#### Scenario: Rotation preserves the subject

- GIVEN a subject retires an old key and activates a new key under a new policy version
- WHEN the new key authenticates
- THEN the stable subject ID MUST remain unchanged
- AND receipts MUST identify the new binding and policy

#### Scenario: Retired or revoked key cannot authenticate

- GIVEN a valid Nostr credential uses a retired or revoked binding
- WHEN subject resolution runs
- THEN resolution MUST deny
- AND historical binding metadata MUST NOT become current authority

### Requirement: Receipts preserve roles and redaction [r[nostr_identity_bindings.receipts]]

Basalt MUST emit BLAKE3-bound authentication and composition receipts. External Nostr SHA-256 event IDs MUST remain in a distinct protocol role.

#### Scenario: Safe receipt is emitted

- GIVEN authentication or composition reaches a terminal decision
- WHEN Basalt emits a receipt
- THEN it MUST contain bounded public identities, decision, reason, and required non-claims
- AND it MUST exclude private keys, credentials, proof bytes, challenges, and secret metadata

#### Scenario: Digest or claim substitution fails

- GIVEN a receipt swaps SHA-256 and BLAKE3 roles or claims that authentication proves authorization
- WHEN receipt validation runs
- THEN validation MUST fail

### Requirement: Positive and negative evidence is complete [r[nostr_identity_bindings.final_validation]]

The change MUST include positive and negative fixtures for policy, subject, lifecycle, request, authority, receipt, and dependency boundaries.

#### Scenario: Complete matrix passes

- GIVEN implementation, policy, generated artifacts, fixtures, documentation, and dependency evidence are complete
- WHEN focused Basalt, downstream, Cairn, and release verification runs
- THEN valid cases MUST pass
- AND each malformed, ambiguous, stale, revoked, forged, mismatched, secret-bearing, or overclaiming case MUST fail

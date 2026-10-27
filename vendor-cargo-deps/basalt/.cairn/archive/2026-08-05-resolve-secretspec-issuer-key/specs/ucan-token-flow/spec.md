# UCAN Token Flow Specification Delta

## ADDED Requirements

### Requirement: ucan_token_flow.oidc.secretspec_issuer_key

r[ucan_token_flow.oidc.secretspec_issuer_key]

Basalt SHALL resolve the UCAN issuer signing key through the pinned SecretSpec Rust SDK in an imperative shell under an explicit profile and `basalt-oidc-exchange` scope.

#### Scenario: Issuer key resolves

- GIVEN an allowed metadata-only declaration and provider
- WHEN the exchange service starts or rotates keys
- THEN the imperative shell SHALL obtain the key
- AND the pure verification core SHALL remain independent of provider access

#### Scenario: Issuer key cannot resolve safely

- GIVEN a missing declaration, wrong scope, provider error, timeout, cancellation, or oversized provider output
- WHEN Basalt loads the key
- THEN issuance SHALL remain disabled
- AND Basalt SHALL NOT fall back to a static or ambient key

#### Scenario: Provider process is bounded

- GIVEN a selected SecretSpec provider can launch a child process
- WHEN Basalt resolves the key
- THEN resolution SHALL run inside an owned bounded worker with an explicit timeout and output-size bound

#### Scenario: Writes and caches are disabled

- GIVEN a SecretSpec boundary used for issuer-key resolution
- WHEN Basalt configures the boundary
- THEN write operations and plaintext value caches SHALL be disabled

#### Scenario: Rotation keeps an overlap window

- GIVEN an explicit key identifier is rotated
- WHEN Basalt overlaps the public verification key window
- THEN private material SHALL NOT be exposed

### Requirement: ucan_token_flow.oidc.secretspec_dependency

r[ucan_token_flow.oidc.secretspec_dependency]

Basalt SHALL pin the SecretSpec Rust SDK to a reviewed revision and record its source identity so builds resolve a durable boundary.

#### Scenario: Dependency is pinned

- GIVEN Basalt declares the SecretSpec dependency
- WHEN Cargo and Nix resolve it
- THEN the resolved revision SHALL match the recorded reviewed source identity

### Requirement: ucan_token_flow.oidc.secretspec_fixtures

r[ucan_token_flow.oidc.secretspec_fixtures]

Basalt SHALL maintain positive and negative fixtures for issuer-key resolution from a bounded SecretSpec boundary.

#### Scenario: Fixture suite covers key health

- GIVEN key-load and rotation fixtures run against a bounded SecretSpec declaration
- WHEN resolution is exercised
- THEN valid loads pass and missing, wrong-scope, timeout, oversized, and provider-failure loads fail closed

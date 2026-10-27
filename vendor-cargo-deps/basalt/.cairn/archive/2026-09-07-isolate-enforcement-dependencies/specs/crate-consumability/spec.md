# Minimal enforcement profile

## ADDED Requirements

### Requirement: Explicit enforcement-only dependency profile
r[crate_consumability.enforcement_profile] Basalt SHALL preserve its default public modules and verification behavior. Explicit no-default consumers MUST omit OIDC HTTP and durable-state adapter dependencies while retaining exact UCAN enforcement. No dependency downgrade or verification bypass is permitted.

#### Scenario: Default consumer
- GIVEN default features
- WHEN Basalt compiles and tests
- THEN OIDC and durable-authority modules retain their existing behavior

#### Scenario: Minimal consumer
- GIVEN default features disabled
- WHEN enforcement compiles and tests on pinned Rust 1.95
- THEN signed execution and document verification remain available without bounded-http, bounded-http-core, replay-ledger-core or revocation-view-core

### Requirement: Maintained isolation evidence
r[crate_consumability.enforcement_profile_evidence] Release verification MUST test the minimal profile and reject empty or forbidden normal dependency observations. Evidence SHALL distinguish actual compiler results from configured feature intent.

#### Scenario: Invalid dependency observation
- GIVEN empty output or a forbidden package
- WHEN the pure dependency validator runs
- THEN it rejects the observation

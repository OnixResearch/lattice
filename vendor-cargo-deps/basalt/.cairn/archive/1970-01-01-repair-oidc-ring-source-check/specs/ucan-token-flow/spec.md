# UCAN Token Flow

## ADDED Requirements

### Requirement: ucan_token_flow.oidc.source_boundary

r[ucan_token_flow.oidc.source_boundary]

Basalt SHALL verify OIDC RS256 signatures with the exact `ring` release that its Nickel source contract pins. The `oidc-exchange-sources` check SHALL require the same version and checksum in the runtime dependencies, in the contract, in the exported source boundary and in the `Cargo.lock` entry for that crate. `jsonwebtoken` SHALL remain only an exact dev-dependency and SHALL NOT appear in the runtime dependencies.

#### Scenario: Pinned ring source verifies

- **GIVEN** the runtime dependency, source contract, exported boundary and lock entry all name `ring` 0.17.14 with its reviewed checksum
- **WHEN** the source check runs
- **THEN** it SHALL pass

#### Scenario: Ring checksum drift fails

- **GIVEN** the `ring` checksum differs in `Cargo.lock` or in the source contract
- **WHEN** the source check runs
- **THEN** it SHALL fail

#### Scenario: jsonwebtoken returns to the runtime closure

- **GIVEN** `jsonwebtoken` is declared under the runtime `[dependencies]`
- **WHEN** the source check runs
- **THEN** it SHALL fail

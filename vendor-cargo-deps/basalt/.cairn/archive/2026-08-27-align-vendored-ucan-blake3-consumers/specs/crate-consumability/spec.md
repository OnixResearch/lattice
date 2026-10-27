# Crate Consumability Specification Delta

## ADDED Requirements

### Requirement: Vendored UCAN composes with reviewed exact BLAKE3 consumers

r[crate_consumability.blake3_alignment] Basalt MUST import a reviewed UCAN revision whose exact peer-authentication and Nostr BLAKE3 pins compose with supported consumer graphs. The source revision, vendored snapshot manifest, and Cargo lock MUST move together through owning tools.

#### Scenario: Molten consumes branch-authority policy

- GIVEN Molten requires the reviewed exact BLAKE3 cohort
- WHEN Cargo resolves Basalt and its vendored UCAN peer-authentication crates
- THEN one compatible BLAKE3 version MUST resolve without a patch override

#### Scenario: Vendored source is edited by hand

- GIVEN a vendored manifest differs from the recorded UCAN source revision
- WHEN the snapshot check runs
- THEN verification MUST fail before publication

### Requirement: Compatibility evidence remains bounded

r[crate_consumability.blake3_alignment.verification] Verification MUST cover snapshot parity, Basalt policy APIs, supported feature sets, and one downstream composition. It MUST NOT claim cryptographic correctness or universal compatibility.

#### Scenario: Compatibility rail passes

- GIVEN the imported revision and lock agree
- WHEN focused, release, Nix, and lifecycle checks run
- THEN the receipt MAY report tested dependency compatibility only

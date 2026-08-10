# Project Identity Specification

## Purpose

Defines the `project-identity` capability.

## Requirements

### Requirement: Canonical Lattice identity
r[lattice.project_identity.canonical] The project MUST expose Lattice as its canonical product identity and MUST use `lattice` for repository-owned Rust package, crate, executable, and Nix package/check names.

#### Scenario: Operator builds and invokes the product
r[lattice.project_identity.canonical.scenario.build-and-run]
- GIVEN an operator builds the repository-owned package
- WHEN Cargo or Nix exposes the resulting executable
- THEN the canonical package and executable MUST be named `lattice` and current documentation MUST use that command

#### Scenario: Stale public branding remains
r[lattice.project_identity.canonical.scenario.stale-branding]
- GIVEN a current non-compatibility source, test, example, script, or operator document still presents `n8nclone` as the product name
- WHEN the maintained repository identity audit runs
- THEN validation MUST fail until the stale public branding is removed

### Requirement: Legacy state compatibility
r[lattice.project_identity.compatibility] Lattice MUST prefer its canonical state identities while continuing to admit explicitly configured or already-existing `n8nclone` state without silently discarding operator data.

#### Scenario: Canonical and legacy environment overrides are both set
r[lattice.project_identity.compatibility.scenario.environment-precedence]
- GIVEN both `LATTICE_DATA_DIR` and `N8NCLONE_DATA_DIR` are configured
- WHEN Lattice selects its data directory
- THEN `LATTICE_DATA_DIR` MUST win deterministically

#### Scenario: Only legacy state exists
r[lattice.project_identity.compatibility.scenario.legacy-state]
- GIVEN the canonical default path or database does not exist and the corresponding legacy `n8nclone` state does exist
- WHEN Lattice selects its default state
- THEN it MUST reuse the legacy state rather than creating an empty parallel state store

#### Scenario: Neither state identity exists
r[lattice.project_identity.compatibility.scenario.canonical-default]
- GIVEN neither canonical nor legacy state exists and no environment override is configured
- WHEN Lattice selects its default state
- THEN it MUST choose the canonical `.lattice` directory and `lattice.redb` filename

### Requirement: Stable protocol and evidence identities
r[lattice.project_identity.stable-identities] The rename MUST preserve versioned protocol domains, accepted Cairn requirement IDs, evidence markers, and historical archives whose identity stability is required for verification or receipt continuity.

#### Scenario: Existing webhook signature is verified after rename
r[lattice.project_identity.stable-identities.scenario.webhook-domain]
- GIVEN a webhook signature was generated with the version-one `n8nclone` signature domain
- WHEN Lattice verifies the same signed request after the product rename
- THEN verification MUST use the unchanged version-one domain and MUST retain the existing result

#### Scenario: Historical evidence is validated after rename
r[lattice.project_identity.stable-identities.scenario.cairn-history]
- GIVEN accepted requirements use durable `n8nclone.*` identifiers and the original rename archive records their preservation boundary
- WHEN the maintained identity audit and Cairn validation run
- THEN the accepted identifier manifest and the recorded rename archive paths and bytes MUST remain unchanged

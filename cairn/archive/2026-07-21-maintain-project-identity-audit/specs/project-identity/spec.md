# Project Identity Specification Delta

## MODIFIED Requirements

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

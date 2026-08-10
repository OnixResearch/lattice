# Repository Validation Delta

## ADDED Requirements

### Requirement: Native lifecycle directory
r[lattice.repository_validation.native_lifecycle] Lattice MUST use `.cairn/` as its single repository lifecycle directory.

#### Scenario: Hash-bound migration preserves content
r[lattice.repository_validation.native_lifecycle.scenario.parity]
- GIVEN the reviewed legacy lifecycle tree and migration plan
- WHEN the operator executes the exact plan hash
- THEN each migrated destination MUST match its declared source BLAKE3 identity

#### Scenario: Active validation uses the native path
r[lattice.repository_validation.native_lifecycle.scenario.active_paths]
- GIVEN the legacy tree has been removed
- WHEN repository, traceability, and Nix checks run
- THEN they MUST read accepted specifications and archives from `.cairn/`

#### Scenario: Legacy-only layout fails closed
r[lattice.repository_validation.native_lifecycle.scenario.legacy_rejected]
- GIVEN a repository has lifecycle content only under `cairn/`
- WHEN current Cairn validation runs
- THEN it MUST reject the root with `cairn.lifecycle_directory.migration_required`

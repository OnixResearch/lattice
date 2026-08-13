# Crate Consumability Delta

## ADDED Requirements

### Requirement: The crate MUST retain `AGPL-3.0-or-later` package license metadata.

r[crate_consumability.preserve_agpl_license.license_preserved] The crate MUST retain `AGPL-3.0-or-later` package license metadata.

#### Scenario: Manifest review

r[crate_consumability.preserve_agpl_license.license_preserved.manifest_review]

- GIVEN package metadata is reviewed

- WHEN Cargo reads the manifest

- THEN the license remains `AGPL-3.0-or-later` and is not silently weakened or removed

### Requirement: Consumer documentation MUST state the license boundary without making legal advice claims.

r[crate_consumability.preserve_agpl_license.license_documented] Consumer documentation MUST state the license boundary without making legal advice claims.

#### Scenario: Consumer reads docs

r[crate_consumability.preserve_agpl_license.license_documented.consumer_reads_docs]

- GIVEN a downstream application author evaluates adoption

- WHEN they read README/package docs

- THEN the AGPL license is visible and no alternate license or compatibility promise is implied

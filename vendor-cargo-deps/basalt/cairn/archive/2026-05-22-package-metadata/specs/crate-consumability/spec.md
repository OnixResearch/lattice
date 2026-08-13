# Crate Consumability Delta

## ADDED Requirements

### Requirement: The crate manifest MUST include enough package metadata for applications and registry tooling to identify source, docs, README, license, and purpose.

r[crate_consumability.package_metadata.metadata_complete] The crate manifest MUST include enough package metadata for applications and registry tooling to identify source, docs, README, license, and purpose.

#### Scenario: Cargo metadata

r[crate_consumability.package_metadata.metadata_complete.cargo_metadata]

- GIVEN a consumer or registry reads package metadata

- WHEN Cargo emits metadata for the crate

- THEN repository/readme/license/description/discoverability fields are present and accurate

### Requirement: The release rail MUST detect missing or invalid public package metadata before publication.

r[crate_consumability.package_metadata.metadata_verification] The release rail MUST detect missing or invalid public package metadata before publication.

#### Scenario: Package warning regression

r[crate_consumability.package_metadata.metadata_verification.package_warning_regression]

- GIVEN a maintainer prepares a package

- WHEN package metadata checks run

- THEN missing repository/documentation/homepage-style warnings are fixed or explicitly justified

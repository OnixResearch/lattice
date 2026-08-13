# Crate Consumability Delta

## ADDED Requirements

### Requirement: The crate MUST express its UCAN dependency in a way that downstream applications can resolve without a sibling `../ucan` checkout.

r[crate_consumability.use_remote_ucan_dependency.dependency_source] The crate MUST express its UCAN dependency in a way that downstream applications can resolve without a sibling `../ucan` checkout.

#### Scenario: Remote resolution

r[crate_consumability.use_remote_ucan_dependency.dependency_source.remote_resolution]

- GIVEN an application adds this crate as a dependency

- WHEN Cargo resolves the dependency graph

- THEN UCAN is fetched from the configured remote source or an explicit consumer override, not from this repo's parent directory

#### Scenario: Local override

r[crate_consumability.use_remote_ucan_dependency.dependency_source.local_override]

- GIVEN a maintainer wants to test against a local UCAN checkout

- WHEN they use the documented local override path

- THEN the public manifest remains valid for remote consumers

### Requirement: The crate MUST package successfully with the remote dependency contract.

r[crate_consumability.use_remote_ucan_dependency.package_verification] The crate MUST package successfully with the remote dependency contract.

#### Scenario: Cargo package

r[crate_consumability.use_remote_ucan_dependency.package_verification.cargo_package]

- GIVEN release verification runs

- WHEN `cargo package` executes

- THEN the package manifest passes dependency-source validation and does not fail because UCAN is path-only

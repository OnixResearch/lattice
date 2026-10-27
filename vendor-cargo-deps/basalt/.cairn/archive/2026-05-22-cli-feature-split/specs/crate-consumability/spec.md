# Crate Consumability Delta

## ADDED Requirements

### Requirement: The library crate MUST be consumable without enabling CLI-only dependencies.

r[crate_consumability.cli_feature_split.library_without_cli_deps] The library crate MUST be consumable without enabling CLI-only dependencies.

#### Scenario: Consumer library build

r[crate_consumability.cli_feature_split.library_without_cli_deps.consumer_library_build]

- GIVEN a downstream app depends on the crate for enforcement APIs only

- WHEN Cargo builds the dependency graph without the CLI feature

- THEN CLI parser dependencies are not required for the library path

### Requirement: The CLI binary MUST remain buildable and runnable when the CLI feature is enabled.

r[crate_consumability.cli_feature_split.cli_feature_preserves_binary] The CLI binary MUST remain buildable and runnable when the CLI feature is enabled.

#### Scenario: Cli enabled build

r[crate_consumability.cli_feature_split.cli_feature_preserves_binary.cli_enabled_build]

- GIVEN an operator enables the CLI feature

- WHEN the binary target builds and the enforce command is smoke-tested

- THEN existing CLI behavior and JSON receipt shape are preserved

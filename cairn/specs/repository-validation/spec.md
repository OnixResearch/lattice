# Repository Validation Specification

## Purpose

Defines the `repository-validation` capability.

## Requirements

### Requirement: Host-secret-independent repository checks
r[n8nclone.repository_validation.flake_check] The repository MUST provide a full Nix flake validation path that does not require a machine signing secret and MUST NOT weaken signing for ordinary package, cache, or release builds.

#### Scenario: Missing host signing key
r[n8nclone.repository_validation.flake_check.scenario.missing-signing-key]
- GIVEN the host configures a runtime `secret-key-files` path whose key is unavailable
- WHEN an operator runs the documented check-scoped flake validation command
- THEN validation MUST proceed with signing disabled only for that invocation and MUST NOT require a placeholder key

#### Scenario: Normal signing policy remains external
r[n8nclone.repository_validation.flake_check.scenario.normal-signing]
- GIVEN an operator runs an ordinary Nix build without the check-scoped override
- WHEN the host has configured signing policy
- THEN the repository MUST leave that policy unchanged and MUST NOT embed a flake-level signing override

### Requirement: Reproducible vendored source snapshot
r[n8nclone.repository_validation.vendor_snapshot] The Git and Nix source snapshots MUST include every Rust source file required by configured vendored path dependencies.

#### Scenario: Nested source resembles an output name
r[n8nclone.repository_validation.vendor_snapshot.scenario.nested-output-name]
- GIVEN a vendored Rust source path contains a basename or directory named like a repository output such as `result` or `coverage`
- WHEN Git ignore admission and Nix source snapshotting run
- THEN the source file MUST remain admitted and available to Cargo

### Requirement: Complete Nix test source
r[n8nclone.repository_validation.test_source] The Nix source closure MUST include tracked non-Rust fixtures required by repository-owned test targets.

#### Scenario: Integration test embeds a JSON fixture
r[n8nclone.repository_validation.test_source.scenario.json-fixture]
- GIVEN a repository-owned Rust test includes a tracked JSON fixture
- WHEN the Nix Clippy or Nextest check compiles that target
- THEN the fixture MUST be present in the filtered source closure

### Requirement: Formatter ownership boundary
r[n8nclone.repository_validation.format_scope] The flake format check MUST validate repository-owned Rust packages without requiring imported vendored snapshots to conform to the consumer toolchain's formatter output.

#### Scenario: Vendored source uses upstream formatting
r[n8nclone.repository_validation.format_scope.scenario.vendor-format]
- GIVEN a vendored path dependency uses formatting that differs from the current project toolchain
- WHEN the flake format check runs
- THEN the check MUST format-check `n8nclone` and MUST NOT fail solely because vendored source would be reformatted

### Requirement: Clippy ownership boundary
r[n8nclone.repository_validation.clippy_scope] The flake Clippy check MUST deny warnings for repository-owned targets without applying repository lint policy to vendored dependencies.

#### Scenario: Vendored dependency has upstream lint findings
r[n8nclone.repository_validation.clippy_scope.scenario.vendor-lints]
- GIVEN a vendored path dependency triggers Clippy lints under the consumer's warning policy
- WHEN the flake Clippy check runs
- THEN the check MUST lint `n8nclone` with dependencies excluded from lint admission

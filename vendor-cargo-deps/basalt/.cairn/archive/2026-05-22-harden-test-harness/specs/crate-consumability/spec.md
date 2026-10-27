# Crate Consumability Delta

## ADDED Requirements

### Requirement: The repo MUST expose a flake release check that delegates to the canonical Rust release verifier.

r[crate_consumability.harden_test_harness.flake_release_check] The repo MUST expose a flake release check that delegates to the canonical Rust release verifier and records any sandbox-only deferrals in the verifier receipt.

#### Scenario: Flake release check

r[crate_consumability.harden_test_harness.flake_release_check.default_flake_gate]

- GIVEN a maintainer runs the default flake checks
- WHEN the release check executes in a sandbox that cannot fetch the private UCAN Git dependency
- THEN it compiles and runs the Rust release verifier with policy freshness prevalidated as a Nix input and records Cargo-dependent rails as deferred in the receipt

### Requirement: The release verifier MUST emit machine-readable and human-readable receipts.

r[crate_consumability.harden_test_harness.release_receipts] The release verifier MUST emit machine-readable and human-readable receipts.

#### Scenario: Receipt materialization

r[crate_consumability.harden_test_harness.release_receipts.receipt_materialization]

- GIVEN the release verifier runs
- WHEN each step completes or a step fails
- THEN JSON and Markdown summaries record step names, commands, statuses, elapsed time, and bounded output tails

### Requirement: The release rail MUST run integration tests through nextest while retaining doctest coverage.

r[crate_consumability.harden_test_harness.nextest_rail] The release rail MUST run integration tests through nextest while retaining doctest coverage.

#### Scenario: Nextest and doctests

r[crate_consumability.harden_test_harness.nextest_rail.nextest_and_doctests]

- GIVEN the release rail executes no-default and CLI-feature tests
- WHEN integration tests and doctests are selected
- THEN nextest runs the integration/unit tests and Cargo runs doctests explicitly

### Requirement: Enforcement tests MUST cover boundary, fail-closed, and multi-capability invariants.

r[crate_consumability.harden_test_harness.enforcement_invariants] Enforcement tests MUST cover boundary, fail-closed, and multi-capability invariants.

#### Scenario: Enforcement invariant coverage

r[crate_consumability.harden_test_harness.enforcement_invariants.boundary_and_capabilities]

- GIVEN the enforcement regression suite runs
- WHEN resource prefix siblings, unknown contracts, id mismatches, malformed contract shape, and multiple capabilities are tested
- THEN the crate returns deterministic allowed, denied, or error outcomes without panics

### Requirement: CLI tests MUST cover common negative operator paths.

r[crate_consumability.harden_test_harness.cli_negative_smokes] CLI tests MUST cover common negative operator paths.

#### Scenario: CLI negative coverage

r[crate_consumability.harden_test_harness.cli_negative_smokes.operator_failures]

- GIVEN the CLI feature test rail runs
- WHEN denied requests, malformed policy files, and bad capability syntax are exercised
- THEN the CLI emits the expected JSON receipt or nonzero parse/error result

### Requirement: Public API and package-content guardrails MUST protect consumer-facing surfaces.

r[crate_consumability.harden_test_harness.api_and_package_guards] Public API and package-content guardrails MUST protect consumer-facing surfaces.

#### Scenario: API and package guards

r[crate_consumability.harden_test_harness.api_and_package_guards.consumer_surface]

- GIVEN public API tests and the package-content verifier run
- WHEN DTO accessors/serialization and package file lists are checked
- THEN required public API behavior and packaged source/policy artifacts remain present

### Requirement: Release verification MUST use a pinned Rust toolchain selection.

r[crate_consumability.harden_test_harness.toolchain_pin] Release verification MUST use a pinned Rust toolchain selection.

#### Scenario: Toolchain reproducibility

r[crate_consumability.harden_test_harness.toolchain_pin.explicit_toolchain]

- GIVEN a maintainer enters or runs the repo toolchain
- WHEN Rust tooling is selected
- THEN the toolchain file names an explicit Rust version rather than a moving stable channel

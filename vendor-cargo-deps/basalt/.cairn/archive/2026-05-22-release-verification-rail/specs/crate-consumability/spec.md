# Crate Consumability Delta

## ADDED Requirements

### Requirement: The repo MUST expose one documented release-readiness rail for crate consumers.

r[crate_consumability.release_verification_rail.single_gate] The repo MUST expose one documented release-readiness rail for crate consumers.

#### Scenario: Maintainer release check

r[crate_consumability.release_verification_rail.single_gate.maintainer_release_check]

- GIVEN a maintainer prepares the crate for downstream use

- WHEN they run the documented rail

- THEN fmt, tests, clippy, policy freshness, feature builds, and package validation are covered

### Requirement: The verification rail MUST avoid stale shared target/sccache artifacts causing false failures.

r[crate_consumability.release_verification_rail.target_hygiene] The verification rail MUST avoid stale shared target/sccache artifacts causing false failures.

#### Scenario: Clean environment

r[crate_consumability.release_verification_rail.target_hygiene.clean_environment]

- GIVEN verification runs in a fresh or repo-local target directory

- WHEN Cargo builds dependencies

- THEN missing `.rlib`/`.rmeta` artifacts from prior stale state do not affect the result

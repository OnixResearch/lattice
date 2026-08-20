# Verification: Publish the Kiln workflow contract

Date: 2026-08-20

## Baseline

The isolated worktree started from `origin/main` at `e7987b6a11c257dcb13deff4f998c8b2fc4d8255`.

Before implementation:

- `nix develop -c cargo test -p lattice --lib` passed with 111 tests.
- Cairn validation passed with no issues.
- The existing root and `lattice-core` Octet baselines had no new or stale findings.

## Focused contract checks

These commands passed:

```text
cargo test --manifest-path crates/lattice-workflow-contract/Cargo.toml --workspace --all-targets --all-features
cargo clippy --manifest-path crates/lattice-workflow-contract/Cargo.toml --workspace --all-targets --all-features -- -D warnings
cargo check --manifest-path crates/lattice-workflow-contract/Cargo.toml --target wasm32-unknown-unknown
nix run .#workflow-contract-octet-deny-all
```

The contract test rail passed 20 tests. It includes positive and negative unit, producer-fixture, and source-boundary cases.

The pinned Octet deny-all rail checked the workspace, all targets, and all features. It reported `Status: clean` with zero findings.

## Nickel checks

The contract and positive request fixture exported successfully.

The malformed identity and zero-limit fixtures failed as required.

## Repository checks

These commands passed after the repository-validation evidence snapshot was refreshed:

```text
scripts/check.rs
nix develop -c octet-precommit-check
nix flake check -L --option secret-key-files ''
```

The broad rail included operator E2E, tests, strict Clippy, source checks, nested contract checks, and Nix builds.

## Cairn and Tracey

Proposal, design, and tasks gates passed. Cairn validation passed.

Raw Tracey coverage remains nonzero because Lattice has reviewed pre-existing debt. The sorted `missing` and `dangling` sets exactly matched the clean `origin/main` baseline.

All six `lattice.workflow_interchange.*` requirements have implementation or verification markers. None entered the missing or dangling sets.

## Claim boundary

These checks validate bounded contract shape, identity, linkage, lifecycle, controls, logs, fixtures, and repository integration.

They do not prove workflow correctness, host confinement, external effect success, credential freshness, or release readiness.

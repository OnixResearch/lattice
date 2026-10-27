# Proposal: Rename the crate to Basalt

## Summary

Rename `ucan-nickel-contracts` to `basalt` across public package, library, CLI, docs, and release-verification surfaces while preserving the crate's purpose: hardened UCAN contract boundaries for Nickel and Steel policy backends.

## Motivation

The current name is accurate but narrow and unwieldy. `Basalt` gives the repo a durable nature-themed identity that fits the contract-bedrock role: Rust-owned authority enforcement, Nickel policy artifacts, Steel DTO boundaries, and deterministic receipts. The rename should improve usability without weakening consumer guardrails.

## Scope

- Rename Cargo package/crate and CLI binary to `basalt`.
- Update README, flake descriptions/check names, release verifier paths, examples, and tests.
- Keep policy schema identifiers stable unless a schema-breaking semantic change requires migration.
- Document that Basalt was formerly `ucan-nickel-contracts` for discoverability.
- Verify package metadata, CLI smoke tests, docs, policy freshness, and release gates.

## Non-goals

- Changing UCAN/Nickel/Steel policy semantics.
- Reworking the crate's public DTO model beyond import-path/name changes.
- Adding a compatibility wrapper crate or publishing alias in this slice.
- Configuring remote repository hosting; this repo currently has no Git remote configured.

# Harden test harness and verification receipts

## Why

The crate has a useful release rail, but the default Nix check only proves policy freshness and the test suite leaves several auth, CLI, package-content, receipt, nextest, and reproducibility seams implicit. Downstream consumers need one canonical gate plus evidence that the public API, CLI failures, generated artifacts, and packaging contract remain stable.

## What Changes

- Promote the release rail into a flake check and keep a direct maintained command for local runs.
- Emit machine-readable and Markdown release-verification receipts with step results and timing.
- Run integration tests through nextest while keeping doctests explicit.
- Add table-style enforcement coverage for boundary, fail-closed, and multiple-capability cases.
- Add CLI negative smoke coverage and public DTO/API guardrails.
- Assert package contents for required source, policy, generated artifact, README, and tests.
- Pin the Rust toolchain instead of tracking moving stable.

## Scope

In scope: Cargo/Nix harness, tests, release verifier, README verification docs, and crate-consumability Cairn requirements. Out of scope: hosted CI provider setup, release tagging, network publication, or changing policy semantics.

## Verification

Validate this planning package with Cairn proposal/design/tasks gates. Implementation must run the updated release rail, `nix flake check`, and focused tests before archive.

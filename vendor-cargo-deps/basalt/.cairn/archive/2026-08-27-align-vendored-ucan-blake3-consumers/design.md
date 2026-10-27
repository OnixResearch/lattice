## Context

Basalt builds against a checked-in UCAN snapshot. The source owner now aligns both exact BLAKE3 consumers with version `1.8.5`.

## Decision

Use `tools/sync-vendored-ucan.rs --import-revision` with one full source object ID. Do not edit the vendored tree or snapshot manifest by hand.

Let the importer replace the reviewed paths and regenerate their BLAKE3 manifest. Then let Cargo regenerate `Cargo.lock`.

Basalt uses `iddqd` APIs available in `0.4.5`. Set that compatible minimum while retaining `0.4.6` in Basalt's own lock. Molten's reviewed source patch can then resolve `0.4.5` without changing map semantics.

## Verification

Run importer self-tests and source checks. Run branch-authority tests, no-default and CLI suites, strict Clippy, formatting, the release verifier, Nix evaluation, and Cairn gates.

## Non-claims

Snapshot parity and tests do not prove UCAN cryptographic correctness, every downstream graph, or release eligibility.

# Proposal: property and fuzz test harness

## Summary

Add property-based and fuzz-style test coverage for Basalt's pure contract-boundary logic so boundary, fail-closed, hashing, and descriptor invariants are exercised beyond hand-written examples.

## Problem

The current suite has good example coverage, but it does not systematically generate resource/ability/capability combinations, malformed policy shapes, Steel descriptor variants, or receipt hashing permutations. That leaves accidental edge-case regressions harder to catch.

## Scope

- Add a repo-local property test harness for deterministic pure functions.
- Cover enforcement invariants, fail-closed parser behavior, Steel callable descriptor validation, and stable hash behavior.
- Keep generated cases bounded and deterministic in normal Cargo/Nix rails.

## Non-goals

- No live network UCAN service dependency.
- No interpreter embedding or Steel runtime execution.
- No probabilistic CI-only fuzzing that cannot be reproduced locally.

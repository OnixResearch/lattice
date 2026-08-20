# Verification evidence

Date: 2026-08-20

## Focused Rust evidence

The nested workflow workspace passed all targets and all features.

- `lattice-workflow-contract`: 20 tests
- `lattice-workflow-transport`: 24 tests, including producer and source-boundary tests
- strict Clippy with `-D warnings`: pass
- format check: pass

Recovery tests cover linked acceptance, changed request identity, observation-only output, stale observation output, not-found, and response disconnect.

## Strict Octet evidence

The pinned `workflow-transport-octet` Nix check passed with zero findings, zero warnings, and zero errors.

## Focused Nix evidence

The workflow transport test, Clippy, format, and strict Octet derivations passed.

## Lifecycle evidence

Cairn proposal, design, and task gates passed. Cairn validation passed.

Cairn sync updated the accepted workflow-interchange specification. The accepted-spec test passed afterward.

The live Tracey report contains no dispatch-recovery requirement in its missing or dangling sets. Existing unrelated repository debt remains unchanged.

## Full repository gate

The first full run hit the existing timing-sensitive operation-pack analyzer test. The isolated `operation-packs`, Nextest, and Lattice derivations then passed.

`nix flake check -L --option secret-key-files ''`: pass

## Claim boundary

This evidence proves bounded local recovery-envelope shape, request linkage, accepted-response linkage, and tested failure classification.

It does not prove durable lookup, non-execution, exactly-once behavior, provider availability, or a terminal workflow outcome.

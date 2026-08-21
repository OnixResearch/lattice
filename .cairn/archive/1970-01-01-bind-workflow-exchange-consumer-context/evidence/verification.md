# Verification evidence

Date: 2026-08-20

## Focused behavior

- `lattice-workflow-contract`: 17 unit tests, 2 producer tests, and 2 source-boundary tests pass.
- `lattice-workflow-transport`: 19 unit tests, 3 producer and recovery tests, and 2 source-boundary tests pass.
- The workflow exchange application rail passes nine positive and negative tests.
- The source-context test proves identity drift, fixed environment binding, collision rejection, instantiated revision binding, and successful execution.
- The contract remains `no_std + alloc` and builds for `wasm32-unknown-unknown`.
- Positive Nickel profiles export. Negative identity and limit profiles fail.

## Strict checks

- Root, contract, and transport Clippy pass with `-D warnings` and no dependency linting.
- Contract and transport Octet deny-all checks report clean with zero findings.
- `octet-precommit-check` reports zero new and zero stale root and core identities.
- `scripts/check.rs` passes after preserving the intentionally nonformatted repository-validation fixture.
- `nix flake check path:$PWD -L --option secret-key-files ''` passes all checks, including 1,036 nextest cases.
- Cairn validation and proposal, design, and task gates pass.

## Traceability

Tracey remains an expected repository-wide failure. Exact set comparison against base revision `7073242403acce1fccb8c58a5775d25da4d6aed9` reports:

```text
missing: base=139 current=139 added=0 removed=0
dangling: base=3 current=3 added=0 removed=0
```

No `lattice.workflow_interchange.source_*` requirement is missing or dangling. The current receipt BLAKE3 is `af184fbfea469b73d3858643f4d5d807c261512a267ca8dd9bc3311dff98262c`.

## Claim boundary

This evidence proves the checked source-context contract, route binding, instantiated revision, durable linkage, and local execution behavior.

It does not prove source trust, revision availability, checkout correctness, workflow correctness, host confinement, external effect success, deployment, or release readiness.

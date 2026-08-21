# Verification evidence

Date: 2026-08-20

## Focused application evidence

The focused `workflow_exchange` rail passes eight positive and negative tests.

It covers exact route admission, revision drift, durable dispatch, duplicate dispatch, idempotency conflict, acceptance recovery, route-exact execution, unavailable authority, observation, cancellation, stale controls, unsupported pause, active limits, bounded logs, missing logs, restart uncertainty, and Unix transport composition.

The typed handler profile exports successfully. Zero active executions and empty route arrays fail Nickel export.

## Rust checks

- focused application tests: pass
- `lattice-workflow-contract`: 16 unit tests, 4 integration tests, and source-boundary checks pass
- `lattice-workflow-transport`: 19 unit tests, 5 integration tests, and source-boundary checks pass
- root package Clippy with all targets, no dependencies, and `-D warnings`: pass
- contract and transport Clippy with all targets, all features, no dependencies, and `-D warnings`: pass
- package format checks: pass

The repository-owned Clippy rail uses `--no-deps`. This keeps vendored Rats findings outside Lattice-owned acceptance.

## Octet evidence

The root Octet semantic baseline migration reported:

```text
old=855 candidate=855 matched=855 unmatched_old=0 unmatched_candidate=0
```

The `lattice-core` migration reported:

```text
old=24 candidate=24 matched=24 unmatched_old=0 unmatched_candidate=0
```

The fail-closed `octet-precommit-check` passed with zero new and zero stale identities.

The independent contract and transport deny-all commands each reported `clean`, zero findings, zero warnings, and zero errors.

## Broad repository evidence

- `scripts/check.rs`: pass
- `nix flake check path:$PWD -L --option secret-key-files ''`: pass
- focused contract, transport, wasm, Clippy, format, Nickel, and Octet Nix checks: pass
- the isolated operation-CLI and operation-pack timing-sensitive rails: pass

The unconstrained parallel workspace run exposed the known process-fixture collision in operation CLI and one timing-sensitive operation-pack classification. Both isolated rails passed. The maintained serial repository check and final full Nix check passed.

Cairn validation and the task gate passed before and after accepted-spec sync.

Traceability remains an expected repository-wide failure. After merging current remote `main`, a detached comparison with base revision `71628cd` proved exact set equality:

```text
missing: base=139 current=139 added=0 removed=0
dangling: base=3 current=3 added=0 removed=0
```

No `lattice.workflow_interchange.application_*` requirement is missing or dangling. The merged traceability receipt BLAKE3 is `1c12ad62b3296e1862d7914e51fd582c3d93ad0fe9c1fd1f790a93909f1e75d5`.

## Claim boundary

This evidence proves the checked local profile, route, durable admission, runtime composition, lifecycle, recovery, log, and command behavior.

It does not prove workflow correctness, host confinement, external effect success, credential freshness, peer intent, provider availability, deployment, or release readiness.

# Evidence: First lattice-core crate slice

Establishes the compiler-enforced FCIS boundary for
`r[lattice.repository_validation.fcis_crate_boundary]`.

## What was added

- New `crates/lattice-core` workspace member. It depends only on `serde` and
  `chrono`; it has no store, clock, filesystem, process, network, provider,
  async-runtime, CLI, or UI dependency, so the crate is host-capability free by
  its dependency set.
- `WorkflowMeta` and `TemporalError` moved into `lattice-core::workflow`.
  `WorkflowMeta::new_at` is the deterministic constructor; the clock-reading
  `WorkflowMeta::new` convenience remains a thin wrapper that delegates to it.
- The `lattice` shell re-exports `WorkflowMeta` and `TemporalError`
  (`pub use lattice_core::workflow::*`), so existing public paths and callers
  are unchanged. Shell `Workflow` continues to own the graph and node configs.
- Octet: `lattice-core` added to the workspace `default_scope`; the baselined
  gate now checks it. `octet-baseline.json` was rekeyed to admit the new
  crate's four findings.

## Verification

- `cargo check --workspace`: compiles.
- `cargo test -p lattice-core`: 1 passed; `cargo test -p lattice --lib`:
  101 passed (includes the temporal-facts tests via the core re-export).
- `cargo clippy -p lattice-core --all-targets --no-deps`: clean.
- `scripts/check.rs`: exit 0 (script format, operator E2E, format, tests,
  clippy, cairn specs).
- `octet-precommit-check`: exit 0, status warning-only with the rekeyed
  baseline (Lattice held to the accepted baseline, `lattice-core` admitted).

## Scope note

This slice extracts only the workflow metadata DTO. Scheduler-admission and
replay decisions remain in the shell crate for a later staged move, and the
explicit forbidden-dependency architecture guard (source-policy fixture) is a
follow-on enforcement step. Evidence here covers source shape and dependency
direction only, not semantic correctness or release readiness.

## Why

[Issue #21](https://github.com/OnixResearch/lattice/issues/21) loses author helper,
function, parameter and module bytes when workflow import retains only its
materialized graph. Source editing, materialized editing and export must not
silently share the meaning of “save”.

## What Changes

- Retain bounded captured inputs and explicit source-declared parameter overrides
  alongside, rather than inside, portable graph interchange.
- Persist independent editable source drafts, safe preview categories and exact
  candidates; atomically publish workflow plus source association using durable
  expected-revision CAS and monotonic catalog generations.
- Preserve last admission on failed/stale/concurrent drafts and preserve old
  workflow clones. Route materialized edit/save paths through one explicit fork
  rule with retained source lineage, without source rewriting.
- Register effect-free CLI and TUI source editing, separate exact `.author.json`
  archive export from explicit flat `--expanded` export, and preserve legacy
  no-source materialized workflows.
- Define focused regression and integration commands in
  `docs/source-authoring.md` and `tests/test_source_authoring.rs`.

## Scope and Non-claims

This is the reachable Lattice-owned portion of #21, not issue closure. The
selected nickel-eval does not expose exclusive captured-module import resolution
or complete closure receipts. File imports remain compatible and are explicitly
retained-entry snapshots, not reproducible captured programs. Choregraph #6 has
not supplied bounded construction/origin mappings; attribution reports
unavailable. Registry names are not executable digests. No inferred origins,
translation receipts, inverse compiler, alternate runtime scheduler, logical
identity normalizer, authority, credential delivery or semantic equivalence
proof is introduced.

No builds, tests, formatters, evaluator fixtures or lifecycle gates were run in
the delegated implementation. Parent integration owns observed verification and
root CLI/README/changelog wiring. The issue remains open for its named producer
contracts and unqualified full-closure/origin/execution criteria.

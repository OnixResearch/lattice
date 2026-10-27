# Nickel evaluator source

Lattice uses `nickel-eval` from canonical Radicle RID `rad:z21miuQLqLwe1HkaMu2mEZy5fdG9F`.

Cargo and Nix pin reviewed object `f23be09e41337de8fa729274f50023f861ae0e8a` through `https://seed.radicle.garden/z21miuQLqLwe1HkaMu2mEZy5fdG9F.git`.

The supported cohort is Nickel CLI `1.17.0` at upstream commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`, `nickel-lang 2.2.0`, and `nickel-lang-core 0.18.0`.

r[impl lattice.nickel_eval_adoption.boundary]

Lattice retains configuration discovery, defaults, product decoding, warning policy, store-opening order, and rollback authority.

A source outage fails dependency acquisition. There is no GitHub, sibling-path, branch, or runtime fallback.

r[impl lattice.nickel_eval_adoption.cutover]

Rollback reverts the dependency, Cargo and Nix locks, adapter, and call-site changes in one VCS revert.

r[impl lattice.nickel_eval_adoption.cutover.scenario.non-claim]

Passing checks prove selected fixture compatibility. They do not prove Nickel soundness, evaluator equivalence, import-closure completeness, contract correctness, configuration safety, or release readiness.

## Imported JSON record merges

r[impl lattice.nickel_import_merge.values]
r[impl lattice.nickel_import_merge.boundary]

The Lattice adapter continues passing source unchanged to the native embedded
evaluator. Nested JSON/YAML record imports in core `0.18.0` lose the runtime's
closurization invariant when converted from recursive AST records to plain
records. A merge can then reach a nonconstant, non-thunk field during recursive
fixpoint construction. This is a producer deserialization bug, not a reason to
reconstruct profile fields, remove assertions, intercept panics, or invoke an
external evaluator at runtime.

The owned producer repair backports upstream
[PR 2677](https://github.com/nickel-lang/nickel/pull/2677), source commit
`aa94d9b1f7a0981b4f0d6e8dff79f0a74ff1bb1a`, to the exact published core `0.18.0`
package. It adds the existing closurization operation at JSON/YAML conversion and
generic serde container creation. Runtime fixpoint and merge behavior remain
unchanged. The producer owns `scripts/import-nickel-core.py`, the source patch,
MIT notice, complete package snapshot at `vendor/nickel-lang-core-0.18.0`, and
exact provenance checks; Lattice does not carry an independently edited copy.

Cargo patch tables are root-only. Integration must select `nickel-lang-core`
from the **same immutable public producer revision** as `nickel-eval`, align its
Cargo/Nix source closure, and verify anonymous acquisition before the correction
is admitted. The original pin above describes the current baseline; this document
does not claim that a new producer object has already been published or that
Lattice's final admitted dependency path has already passed verification.

The producer's owning Rust `1.90.0` environment has passed all 43 tests, including
the five imported-container regressions, and strict Clippy. Its native retained
fixture smoke produced the complete expected value below, matching the pinned
external CLI's observed value. These local producer checks do not replace
immutable publication, exact consumer dependency admission, or Lattice's final
native profile smoke.

Native adapter regression command:

```sh
cargo test --lib nickel::tests::imported_json_merge
```

The producer retains `fixtures/imported-merge/profile.ncl` and its relative JSON
input, plus a native `export_json` example. From the exact producer checkout,
compare the JSON values from these commands, with the CLI path set explicitly
to the pinned `1.17.0` executable:

```sh
cargo test --test imported_merge
cargo run --example export_json -- fixtures/imported-merge/profile.ncl
"$PINNED_NICKEL" export --format json fixtures/imported-merge/profile.ncl
```

Expected complete value: `{"attachment_ref":{"token_id":1},"base":5,"data":{"extra":"kept","string":"value"},"derived":6,"items":[{"flag":true}]}`.
Tests also cover both merge operand orders, captured source, nested record/array
contracts, recursive overriding, malformed/missing imports, and safe contract
errors. Selected agreement is not proof of arbitrary evaluator equivalence,
Nickel soundness, import closure completeness, or workflow correctness.

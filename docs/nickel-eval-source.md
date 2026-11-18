# Nickel evaluator source

Lattice uses `nickel-eval` from canonical Radicle RID `rad:z21miuQLqLwe1HkaMu2mEZy5fdG9F`.

Cargo and Nix pin reviewed object `f0ea0d2a31d3482ddf4e575482f6ddf7a7e7d4e9` through `https://seed.radicle.garden/z21miuQLqLwe1HkaMu2mEZy5fdG9F.git`.

The supported cohort is Nickel CLI `1.17.0` at upstream commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`, `nickel-lang 2.2.0`, and `nickel-lang-core 0.18.0`.

r[impl lattice.nickel_eval_adoption.boundary]

Lattice retains configuration discovery, defaults, product decoding, warning policy, store-opening order, and rollback authority.

Evaluation and observed cohort metadata both cross the `src/nickel` adapter.
Source-authoring compiler records read that adapter's selected source, library,
and core versions; they do not import the evaluator directly or identify compiled
executable bytes.

A source outage fails dependency acquisition. There is no GitHub, sibling-path, branch, or runtime fallback.

r[impl lattice.nickel_eval_adoption.cutover]

Rollback reverts the dependency, Cargo and Nix locks, adapter, and call-site changes in one VCS revert.

r[impl lattice.nickel_eval_adoption.cutover.scenario.non-claim]

Passing checks prove selected fixture compatibility. They do not prove Nickel soundness, evaluator equivalence, import-closure completeness, contract correctness, configuration safety, or release readiness.

r[verify lattice.nickel_eval_adoption.compatibility]
r[verify lattice.nickel_eval_adoption.compatibility.scenario.agree]
r[verify lattice.nickel_eval_adoption.cutover.scenario.accepted]
r[verify lattice.nickel_toolchain.fixture_cohort.scenario.aligned]

The original direct/shared migration comparison is retained in VCS at Lattice
`3539ade3d6c6f5ea0c37a896761dfc5da1deb856`, whose repository harness and Nix
receipt are recorded in `target/publication-candidate-current.json`. It is
historical evidence for that selected cohort, not qualification of the new
captured construction profile. The duplicate direct-core test path and direct
dependency are removed; ongoing product regressions assert expected admitted
values and safe failures through the Lattice adapter. The cohort guard checks
all resolved native package versions and the pinned external CLI, and requires
`nickel-eval`, `nickel-lang` and `nickel-lang-core` to resolve from the same
pinned producer source and revision.

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

The producer repair backports upstream
[PR 2677](https://github.com/nickel-lang/nickel/pull/2677), source commit
`aa94d9b1f7a0981b4f0d6e8dff79f0a74ff1bb1a`, to the exact published core `0.18.0`
package. It adds the existing closurization operation at JSON/YAML conversion and
generic serde container creation. Runtime fixpoint and merge behavior remain
unchanged. The producer owns `scripts/import-native-captured.sh`, its
repository-owned patches (import closurization, the captured-module seam and the
local-core wrapper), the retained MIT notice, and the complete package snapshots
at `vendor/nickel-lang-core` and `vendor/nickel-lang`. The import script checks
the pristine crate archives' SHA-256 and BLAKE3 identities and the declared
upstream core revision before applying those patches.

### Admitted producer cohort

Cargo patch tables are root-only, but Lattice needs none: `nickel-eval` selects
both packages through path dependencies inside its own repository, so Cargo
resolves `nickel-eval`, `nickel-lang` and `nickel-lang-core` from the single
pinned producer source and revision. Lattice carries no consumer-owned snapshot
or root patch. The Nix cohort guard rejects a reintroduced `nickel-lang-core`
path patch and requires all three packages to resolve from that exact source.

Lattice previously vendored a byte-exact snapshot of this repair from producer
revision `b4ddff052105eb4b75827e228a65b823dc1a0444` because the published
evaluator did not yet carry it. The adopted producer revision carries the repair
and the captured-module seam itself, so that snapshot and its root patch are
removed; rollback restores them with the previous pins in one VCS revert.

The first published captured-module revision, `2d1fb2b59f373967cc69e13ec289759d9cd4df46`,
kept the crate archives' `.cargo_vcs_info.json` and `Cargo.toml.orig` in its
vendored package trees. Cargo reserves those names, so Lattice's crane vendoring
of the git dependency failed before compilation. The adopted revision is the
producer's import fix: it validates those files inside the pinned archives,
omits them from the vendored trees, and gates `cargo package --list` on both
packages. Evaluator sources, patches and the language cohort are unchanged.

The retained core remains `0.18.0`, the evaluator cohort remains Nickel CLI
`1.17.0` / `nickel-lang 2.2.0`, and the native imported-merge regressions pass
against the producer-selected core.

Native adapter regressions:

```sh
cargo test --lib nickel::tests::imported_json_merge
```

They write the JSON imports and Nickel sources to a temporary directory and
assert these complete values through the Lattice adapter:

- `identity & { machine.reviewed = true, expires = 1234 }` and the reversed
  operand order, over `{"machine":{"id":"machine-1"},"grants":[{"operation":"shell","allowed":true}]}`:
  `{"expires":1234,"grants":[{"allowed":true,"operation":"shell"}],"machine":{"id":"machine-1","reviewed":true}}`.
- The recursive default/force override over
  `{"base":3,"nested":{"answer":42},"items":[{"enabled":true}]}`:
  `{"base":5,"derived":6,"items":[{"enabled":true}],"nested":{"answer":42,"extra":"retained"}}`.

To compare against the pinned external CLI, write the same files and run
`nickel export --format json <file>` with the dev shell's Nickel `1.17.0`
(rev `1320a98`), then compare `jq -cS` canonical output with the values above.
On 2026-11-18 all three CLI exports matched exactly. The producer's own
`tests/imported_merge.rs` at the pinned revision covers JSON/YAML merges in both
orders, nested record/array contracts, deserialized and serde-created containers,
and safe failure categories; it passed with `cargo test --locked --test
imported_merge` from that checkout. Tests also cover malformed/missing imports and
safe contract errors through the adapter. Selected agreement is not proof of
arbitrary evaluator equivalence, Nickel soundness, import closure completeness,
or workflow correctness.

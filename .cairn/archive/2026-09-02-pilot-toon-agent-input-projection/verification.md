# Verification

## Baseline

Before implementation, the focused command, OpenAI-compatible, and bounded Animus route suite passed 53 tests.

## Upstream review

The reviewed Rust release is `toon-format` 0.5.0 at commit `2136cb1a35f3ed63be733bb74d36b89d3b4592dd` under the MIT license.

The release README advertises TOON specification v3.0. The current TOON website publishes specification v4.1 as a Working Draft dated 2026-07-25. The Lattice profile is therefore named `toon_v3` and makes no current-spec claim.

The public benchmark compares TOON with pretty and compact JSON. It shows that TOON can be larger than compact JSON for mixed, semi-uniform, and deeply nested data. Lattice does not import those results as model evidence.

The crates.io package checksum is `8f89570c1a68d73941f728cca32a4345b2ffca36667ad921af336c60309a3e7e`. The selected upstream file manifest identity is `b3:7aef44e975bdab7a0a705bf824f5ffa0e292bd2d1cb122c6f1585364ad33dc01`.

## Vendored codec boundary

A direct Cargo dependency on `toon-format` unifies `serde_json/preserve_order` across the product graph. That changes Lattice's existing JSON object ordering, prompt bytes, unrelated Clippy outcomes, and every accepted Octet baseline identity. Lattice therefore rejects the crates.io dependency.

Instead, the exact reviewed upstream library source is vendored under `src/vendor-cargo-deps/toon-format/`. That directory is the Octet-reviewed vendored-source boundary: file-level naming lints skip it, and the vendored bytes stay upstream-exact.

- `UPSTREAM-FILES.b3` verifies 53 selected upstream files byte-for-byte.
- `scripts/sync_toon_format.rs` reproduces the vendored snapshot from an extracted upstream crate directory and fails on revision, license, or manifest drift.
- `SOURCE.ncl` records the repository, revision, crates.io checksum, and manifest identities.
- `PATCH.md` records that no upstream Rust source file is modified.
- `Cargo.lock` and `Cargo.toml` are byte-identical to `origin/main`; the codec adds no dependency-graph edge.
- The five vendored modules compile inside the `lattice` crate behind `#[path]` declarations with narrow, reasoned style allows; the strict round-trip corpus and route tests carry the behavioral gate.
- A local `IndexMap` type alias maps the codec's ordered map to `BTreeMap`, matching the crate-wide deterministic JSON ordering.

## Implementation evidence

- Omitted policy retains the existing pretty-JSON heading and body.
- The closed target policy admits only `json_pretty`, `json_compact`, and `toon_v3` for Pi, Clankers, or Animus.
- Command, OpenAI-compatible, and Animus routes produce the same selected structured-input sections.
- TOON v3 output must strictly decode to the original typed JSON value.
- An empty root object and malformed TOON counts, widths, or indentation fail closed.
- Projection failure occurs before command or network dispatch and never falls back to JSON.
- Provider envelopes and requested model output remain JSON.
- Response metadata contains only profile, implementation, and input-count facts.
- The pure pilot comparator blocks binding drift, duplicate observations, missing usage, missing outcomes, task regression, non-improving token totals, and arithmetic overflow.
- The omission default remains `json_pretty`. No default promotion is part of this change.

## Controlled pilot

The declared local `VibeThinker-3B` OpenAI-compatible service at `127.0.0.1:13305` was unavailable on 2026-09-02. The endpoint probe returned transport exit code 7 before task dispatch.

No model task outcome and no provider prompt-token count was observed. The checked report is `inconclusive_missing_outcome` and recommends no default promotion.

This result makes no billing, accuracy, cost, latency, provider, or production claim.

## Octet identity evidence

Adding the codec as a Cargo dependency replaced all 846 accepted Octet baseline identities (0 current accepted, 864 new, 846 stale). The vendored in-crate compilation preserves them.

The structured-input projection moved the prompt display-index addition out of `agent_node` into the saturating-add renderer, resolving exactly one accepted `raw_arithmetic_overflow` finding. The guarded baseline migration tool was extended so an allowlisted resolution may only shrink a finding group, keeps the remaining candidates, and refuses any candidate-count increase. Its self-test and pinned-fragment guard test cover both directions.

After migration, `octet-precommit-check` reports `warning-only` with 845 accepted, 845 current accepted, 0 new, and 0 stale.

## Checks

Passed:

- 8 pure projection and comparison tests
- 60 focused command, OpenAI-compatible, Animus, and projection integration tests
- 2 baseline-migration guard tests and the migration script self-test
- exact sync reproduction from the reviewed extracted crate
- strict Clippy for the Lattice package, all targets, with warnings denied
- `scripts/check.rs`, including repository-validation boundaries and the full Cargo test suite
- `octet-precommit-check` with 845 accepted/current findings, 0 new, and 0 stale
- `checks.x86_64-linux.structured-input-projection-policy`
- serialized local `nix flake check -L --option secret-key-files '' --builders '' --max-jobs 1`, including 1,091 passing Nextest tests
- Cairn validation and proposal, design, and tasks gates with the canonical generated policy

The full flake check omitted incompatible Darwin and AArch64 systems. This result does not claim support for those systems.

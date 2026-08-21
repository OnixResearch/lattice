# Validation

## Focused behavior

The focused ready-frontier suite passes 10 tests and keeps one comparative benchmark ignored by default. The Choregraph effect-authority suite passes 20 tests. The approval suite passes 14 tests and keeps its crash child ignored except when the parent test starts it.

The focused rails cover serial compatibility, typed capacity, stable planning, expected-failure refinement fixtures, concurrent authorized effects, conflict serialization, completion-driven release, canonical result order, observed completion order, integration ownership, evidence non-claims, independent uncertain attempts, and durable approval suspension.

## Repository gates

- `scripts/check.rs` passes its format, boundary, test, Clippy, and Cairn-spec checks.
- Scoped owned-code Clippy passes with warnings denied. Broad workspace Clippy remains outside this acceptance claim because vendored source has pre-existing findings.
- The root Octet gate accepts exactly 846 baseline findings with zero new or stale findings.
- The pure-core Octet gate accepts exactly 23 baseline findings with zero new or stale findings.
- Cairn repository validation and proposal, design, and task gates pass.
- Final `nix flake check -L --option secret-key-files ''` passes all local flake checks.

The first clean Nix run exposed a transient saved-child shell test failure. The isolated debug and release tests passed, and the complete Nix check passed on the immediate retry without a source change. This evidence does not claim that unrelated historical test flakiness is absent.

## Benchmark evidence

The maintained workload identity is `blake3:55604e2765f89046f53ba81f44514e782e53436905613e9912b7d143dfc1543a`.

The local observed fixture recorded serial time `626.427825ms` and bounded ready-frontier time `313.475304ms`. The deterministic duration model records serial `370ms`, dependency-level barrier `340ms`, and ready frontier `310ms`.

These values apply only to the maintained local fixtures. They do not adopt the paper's accelerator percentages or establish production latency.

## Source and proof identities

- Choregraph runtime revision: `0b2000edaf86d5a7a8601246afbc0d526f4a2a07`
- Choregraph readiness-evidence revision: `b3e08e19750f53bdbcae970cdf58a47a791ed20b`
- Runtime source archive BLAKE3: `8c35370153e249cc2fe9454746a82950d830e1451b3cd77cece96321acf1ea1e`
- Proof revision archive BLAKE3: `f66a229ca2098621e9ae445e11a2d6712261d64ce23948a5ad7809b8b902b9fe`

Lattice claims exact runtime-frontier compatibility checks and separate accepted proof evidence. It does not claim direct proof linkage between these two revisions.

## Traceability boundary

All 17 top-level and 22 scenario markers have maintained implementation or verification references. The review reports 1,463 requirements and 1,324 referenced requirements. Its receipt hash is `9ca7c64d7098458b7b75eaae00243a40a29b5a9959b527c3b5bf9befec2e0899`.

The three dangling markers are unrelated pre-existing lifecycle and Octet migration debt. No ready-frontier marker is dangling or missing.

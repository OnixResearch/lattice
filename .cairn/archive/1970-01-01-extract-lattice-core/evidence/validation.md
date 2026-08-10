# Validation evidence

Date: 2026-08-10

## Result

The extraction preserves the existing `lattice::*` facade and gives deterministic core logic a separate internal crate. The new crate does not receive process, filesystem, network, database, environment, clock, UI, scheduler, webhook, credential, provider, or orchestration authority.

## Passed checks

- Package-scoped formatting passed for `lattice` and `lattice-core`.
- `cargo test -p lattice-core` passed with four positive and negative pure-core tests.
- The compatibility suites for values, expressions, control flow, content identity, workflow format, and workflow validation passed.
- The dependency-boundary suite passed its admitted, forbidden-host, and malformed-manifest cases.
- `scripts/check.rs` passed the repository check set.
- Scoped Clippy passed with `--workspace --all-targets --no-deps`.
- The dual Octet gate passed for `lattice` and `lattice-core` with no new or stale identities.
- The Octet migration checker passed its positive migration and negative drift self-tests.
- Strict Cairn validation passed after the accepted requirement was synced.
- The identity suite passed all seven tests.
- The isolated subworkflow suite passed five exact release retries and ten full-file release retries after one non-reproducible Nix-build failure.
- `nix flake check path:$PWD -L --option secret-key-files ''` passed all host checks, including 817 nextest cases.
- `git diff --cached --check` passed before lifecycle completion.

## Octet migration

The migration is bound to source-result and baseline-file BLAKE3 identities in `octet-baseline-migration.json`.

- Old root: 1,141 records and 890 stable identities.
- New root: 872 stable identities and 1,108 retained occurrences.
- New core: 24 stable identities and 32 occurrences.
- The migration moved 33 old occurrences into the core scope.
- One moved warning was removed by the reviewed control-flow filename correction.
- No unreviewed identity or occurrence growth was accepted.

## Traceability boundary

Cairn Tracey reports the new `lattice.architecture.core_crate` requirement and all five scenarios as referenced after sync. It reports no missing or dangling ID from this extraction.

The repository still has inherited Tracey debt: 59 accepted requirement IDs are missing direct evidence, and three direct markers are dangling. The same classes were present before this extraction. This change does not claim to close unrelated traceability debt.

## Inherited validation limits

A direct workspace Clippy command without `--no-deps` reaches vendored repositories and fails on their existing warnings. The repository-owned Clippy contract uses `--no-deps`; that scoped contract passed.

One full Nix build failed once in `saved_child_executes_and_routes_declared_output_to_the_real_parent_port`. The same test then passed five exact release runs, ten full-file release runs, a debug run, the repository check set, and the final Nix check. No source change was made for the transient result.

Two later Nix attempts found stale lifecycle fixture evidence after specification sync. The first found non-canonical requirement text. The second found the expected exact-byte hash change from the new validation marker. Both fixtures were corrected, their focused negative and positive tests passed, and the final full Nix check passed.

These checks prove the bounded extraction and repository contracts. They do not prove external provider behavior, remote trust, or whole-workflow correctness.

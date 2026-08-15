# Typed operation contract validation

Date: 2026-08-15

## Implementation checks

The following checks passed:

```text
CARGO_TARGET_DIR=target/operation-contracts cargo test --test test_cairn_specs
CARGO_TARGET_DIR=target/operation-contracts cargo test --test test_operation_cli
CARGO_TARGET_DIR=target/operation-contracts cargo test --test test_operation_contracts
CARGO_TARGET_DIR=target/operation-contracts cargo clippy -p lattice --lib --bin lattice --tests --no-deps -- -D warnings
CARGO_TARGET_DIR=target/operation-contracts scripts/check.rs
octet-precommit-check
git diff --check
nickel typecheck policy/operation-contracts.ncl
nickel export --format json builtins/operations/catalog.ncl
nickel export --format json builtins/operations/instances.example.ncl
nix flake check -L --option secret-key-files '' --builders ''
```

The focused Cairn spec suite passed three tests. The operation CLI suite passed four tests. The operation contract suite passed 15 tests.

The repository script passed its format, boundary, focused, full test, Clippy, and Cairn spec checks. The local Nix gate passed all 13 checks and 978 nextest cases.

The repository-wide Cargo format check reports only the intentional fixture difference at `tests/fixtures/repository-validation/vendor-dependency/src/coverage/result.rs`. The package format check in the Nix gate passed.

The strict Nickel catalog contract rejected `tests/fixtures/operations/catalog-unknown-field.ncl`. Nickel accepted the structural instance override fixture. Strict Rust instance admission remains authoritative for that case.

The first distributed Nix check exposed an untracked-source closure issue. Staging the new files fixed that issue and proved the operation catalog check.

The second distributed check failed while importing `rust-src` into `aspen1`. Its specified and observed Nix store hashes differed. The authoritative local-builder check then passed.

## Product evidence

Positive tests cover catalog and instance admission, deterministic identities, pure planning, direct execution, workflow parity, read-only Cairn methods, promotion, persistence, doctor facts, and safe host metadata.

Negative tests cover malformed identities, unknown fields, duplicate data, behavior overrides, secret-bearing values, stale plans, catalog drift, unavailable runtimes, unsafe executables, and unsafe repository roots.

They also cover capability widening, payload drift, missing authority, duplicate dispatch, timeout, cancellation, truncation, malformed output, nonzero exit, output overclaims, traversal, symlinks, and publication collisions.

The direct execution path persists `Started` before dispatch. It blocks a second dispatch after any durable start, including an unknown outcome.

Direct and promoted workflow execution use one exact `InvocationPlan`. Direct authority also reproduces catalog and instance admission before it permits the process effect.

The process authority grant binds these exact values:

- operation: `lattice/operation`;
- ability: `process/execute`;
- policy contract: `lattice-process`;
- resource: `lattice://process/0`;
- request: the exact identity-bound process payload.

Direct execution requires `--confirm-read-only`. Promotion requires `--confirm-create`. Discovery and preview paths do not load executors, resolve credentials, open stores, or create data directories.

A real Cairn executable completed `change_list` and `validate` through the confirmed operation path. Its executable identity was `b3:4a9ba978a408aea676782a75dd79cfe1d55bf802313714d79bef390e0b0d7d1f`.

The built-in fixture executor identity is `b3:025bc96d707d911025ad5f5026b61c66c7f75c8777f7096a4eb58ed09e4249ac`. The built-in `cairn.repository` revision is `b3:04f4f2b5e880435887fd6e8d124b80b9e94c9037ff310aacca88f065d6bf6cc9`.

## Exact traceability refresh

Implementation changed exact reviewed source bytes. The affected positive receipt tests and their fail-closed drift tests passed after the reviewed hash refresh.

After spec sync, no `lattice.operations` requirement or scenario identity is missing or dangling.

The full advisory profile still reports 105 missing identities and three dangling identities from unrelated work. Its input hash is `f417ea939d353ba36f9dff96f95a2b08a31145577097ab66d0d4945f803a0743`.

Its receipt hash is `16dddc70d405e75e684fd834d202d49c257fbdd592bab08bd023157de3f549b4`. The unrelated dangling identities are:

- `cairn.lifecycle_directory.hidden_default`;
- `lattice.repository_validation.octet_baseline_migration.scenario.dynamic`;
- `lattice.repository_validation.octet_baseline_migration.scenario.resolved`.

## Octet evidence

The pinned Octet gate completed with warning-only status and exit code zero.

- `lattice`: 855 accepted, 855 current, zero new, zero stale, and zero error findings;
- baseline: `b3:990b93716039ed0608a4a3614b46808ff359591f0ea75b4973460c95ef726ad7`;
- `lattice-core`: 24 accepted and unchanged;
- `lattice-core` baseline: `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

The reviewed migration paired 37 semantic identity changes. It also recorded one repaired stale finding. See `octet-migration-2026-08-15.md`.

## Cairn lifecycle

Lattice policy provenance pins Cairn revision `b6759012db4dab0158936e995b7bad39ac10465b`. Validation used the built executable from that source and the `lattice-default` policy.

Strict validation passed with no issues. Proposal, design, and task gates passed with these receipt hashes:

- proposal: `a50b1557d029b464bbf4efef449727e9a58dead127264978102ceaadc7673b22`;
- design: `4f8ec72d359ea878128a10e3dcfc84d77fa1177a79cae8808affcd1259573ef1`;
- tasks: `5a4d552f6ecfac157319b0eb5f4072fcea52feabe33638acb15058a6f2fd27f6`.

All 26 tasks are complete. Sync added six requirements and 12 scenarios to `.cairn/specs/operation-contracts/spec.md`.

The accepted spec content hash is `4b2dfa84b3969ceabb724d9803ddc1dd8867f4d4a98dfcd5d03837f6a1c4ba4c`. The executed sync receipt is `bfaf556d7e166c4553b00cc49b59c511d31087bcfc05082c104412c04fd4c834`.

Post-sync strict validation passed. The accepted Cairn spec shape suite also passed.

The explicit `2026-08-15` archive completed with receipt `3ca07fe3f19f5045bba14f889a643494fae6f170080918f5ad35d589e69d2334`.

## Claim boundary

These checks prove the listed behavior for the checked source, fixtures, policy, tool revisions, and local platform.

They do not prove external-system truth, arbitrary executor safety, credential freshness, effect uniqueness, production readiness, release eligibility, or release readiness.

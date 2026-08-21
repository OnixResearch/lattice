# Shared reconciliation core pilot

## Source

- RID: `rad:z4Tky6zvC8w4Y6c4YBzNxVbq5n752`
- Git source: `https://seed.radicle.garden/z4Tky6zvC8w4Y6c4YBzNxVbq5n752.git`
- Revision: `606489b5f40298181214bb76bc3457b607f225d9`

Cargo and Nix select the same full revision. The source-admission check rejects GitHub fallback and ambient sibling paths.

## Lattice pilot

The scheduler shell maps an exact claimed occurrence into the shared immutable plan and reservation contract immediately before effect execution.

Passed checks:

- Two positive and negative adapter tests.
- Fourteen scheduler occurrence tests.
- Fourteen scheduler daemon and operator tests.
- Repository-owned Clippy with `--no-deps` and `-D warnings`.
- `transactional-reconciliation-core-source` Nix source-agreement check.

Lattice retains workflow identity, Redb schema, transactions, run allocation, authority, effects, operator actions, and receipts.

## Mantle pilot

Mantle maps one canonical GC plan into the shared immutable plan and freshness contract.

Passed checks:

- Positive exact GC-plan freshness and dispatch admission.
- Negative changed-plan freshness rejection.
- `transactional-reconciliation-source-admission` Nix source-agreement check.

Mantle retains GC semantics, store mutation, effect authority, and evidence.

## Non-claims

These pilots do not prove persistence truth, transaction atomicity, external effect truth, exactly-once behavior, or release readiness.

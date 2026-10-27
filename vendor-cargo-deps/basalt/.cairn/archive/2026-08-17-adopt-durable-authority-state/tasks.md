## Phase 1: Exact source and composition

- [x] [serial] [task:source] Pin both cores to the canonical Radicle adapter and exact reviewed object without fallback. r[basalt.durable_authority.source]
- [x] [serial] [task:composition] [after:source] Add the generic Basalt composition helper over UCAN's published durable adapters. r[basalt.durable_authority.composition]
- [x] [parallel] [task:docs] [after:source] Document source, identity, authority, recovery, and non-claim mappings. r[basalt.durable_authority.authority]

## Phase 2: Consumer pilot

- [x] [serial] [task:replay] [after:composition] Prove first use reserves once, duplicate use denies, and Basalt performs no state work after replay. r[basalt.durable_authority.replay_order]
- [x] [parallel] [task:revocation] [after:composition] Prove current views answer and incomplete, unauthenticated, stale, missing, or unavailable views deny before replay. r[basalt.durable_authority.revocation]
- [x] [parallel] [task:source-tests] [after:source] Add exact-source and no-fallback tests for Cargo and the lockfile. r[basalt.durable_authority.validation]

## Phase 3: Validation and lifecycle

- [x] [serial] [task:gates] [after:replay] [after:revocation] [after:source-tests] Run focused tests, Clippy, release verification, Cairn gates, traceability, and `nix flake check -L`. r[basalt.durable_authority.validation]
- [x] [serial] [task:archive] [after:gates] Sync, archive, push the verified branch, fast-forward `main`, and verify `origin/main`. r[basalt.durable_authority.lifecycle]

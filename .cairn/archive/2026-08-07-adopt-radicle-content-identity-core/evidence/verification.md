# Radicle content identity adoption verification

Date: 2026-08-07

## Source admission

- RID: `rad:zA14jhTdwST4PxPBdsryc4zS7Kpq`
- Read-only adapter: `https://seed.radicle.garden/zA14jhTdwST4PxPBdsryc4zS7Kpq.git`
- Reviewed object: `7f55597b5dc879b7601856e8d7fd0dbacaa2a498`
- Source archive BLAKE3: `af024d70e39195a018e54cf31fdbba0dc64928af30e108f99d5988a3b10d0b7c`
- License: MIT
- Encoding: `lattice-tagged-le.v1`
- BLAKE3 dependency: 1.8.5

Producer host, strict Clippy, `wasm32-unknown-unknown`, Cairn, Nix, positive, negative, fresh-clone, and no-fallback checks passed.

Cargo, Nix, and Crane pin the same object. The Nix source gate rejects GitHub fallback, lock drift, and shared imports in credential or trigger security owners.

## Inventory and boundary

`evidence/inventory.md` records every production BLAKE3 family and repository tooling role at base revision `250f7d588d5a7a5ec76bc456d5dc57821714bd1d`.

Only durable schedule-occurrence framing is admitted. Its current encoding exactly matches the producer's hashed-part operation.

Lattice retains the domain, tags, big-endian due-time projection, `b3:` spelling, persistence, claim, validation, retry, and recovery meaning.

Value, node-execution, replay, orchestration, inspection, agent-program, webhook-claim, and evidence/tooling hashes remain local. Vault streams, vault integrity, webhook signatures, credential correlation, password hashing, KDFs, authentication, and Valence evidence identities remain outside the shared package.

## Baseline and cutover

Before dependency changes, `test_scheduler_occurrences` passed 14 tests.

After cutover:

- `test_content_identity_adoption`: 4 passed;
- `test_scheduler_occurrences`: 14 passed;
- fixed legacy/shared schedule identity: `b3:0f3b3ea4c2e1cb44d54ab759e6c69cd6e1d91af09a8bef1b503aa81dc176596c`;
- negative wrong-domain, wrong-order, field-omission, duplicate-map, unversioned-domain, security-domain, and security-owner import checks passed;
- Nix source check passed;
- Nix format check passed;
- Nix strict Clippy check passed for all Lattice targets with `--no-deps`;
- pinned Cairn proposal and design gates passed.

The scheduler's duplicate `blake3::Hasher` and hashed-part helper are removed. No stored or wire identity changed.

## Toolchain and repository baselines

Lattice's pinned Rust toolchain is older than a maintained Choregraph dependency. Focused tests use the installed nightly Rust 1.99 toolchain with the `cc` linker.

Direct focused Clippy reaches pre-existing deny-level findings in vendored Rats and Animus packages before it checks Lattice. The bounded Nix strict Clippy rail passed and is the clean consumer-owned signal.

The Octet rail fails on the unchanged identity baseline: 891 accepted entries map to 0 current identities, with 890 new and 891 stale findings. This exactly matches the repository-owned baseline recorded by the prior Nickel adoption; the content-identity change files add no findings. `scripts/check.rs` passes earlier rails, including the 101 library tests, then fails all three stale `test_execution_graph_source_admission` cases because they still require retired execution-graph pins while the accepted base uses Choregraph.

## Rollback

One reviewed VCS revert restores the Cargo and Nix source pins, locks, role wrapper, scheduler helper, tests, and docs. There is no runtime or acquisition fallback.

## Non-claims

These checks prove selected source, role, and byte-framing observations. They do not prove semantic equality, canonicalization correctness, confidentiality, authenticity, authorization, cache safety, deduplication safety, or release readiness.

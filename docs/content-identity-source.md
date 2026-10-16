# Content identity source and roles

Lattice uses `content-identity-core` for one admitted family: durable schedule-occurrence identity framing.

## Reviewed source

- Radicle RID: `rad:zA14jhTdwST4PxPBdsryc4zS7Kpq`
- Read-only adapter: `https://seed.radicle.garden/zA14jhTdwST4PxPBdsryc4zS7Kpq.git`
- Reviewed object: `7f55597b5dc879b7601856e8d7fd0dbacaa2a498`
- Source archive BLAKE3: `af024d70e39195a018e54cf31fdbba0dc64928af30e108f99d5988a3b10d0b7c`
- License: MIT

r[impl lattice.content_identity_adoption.source]

Cargo, Nix, and Crane use this exact object. They do not use GitHub, a sibling path, a branch, or a runtime fallback.

The reviewed object uses BLAKE3 1.8.5. Its fixed vectors match the earlier reviewed encoding and align with maintained Lattice dependencies.

## Admitted role

r[impl lattice.content_identity_adoption.boundary]

`crates/lattice-core/src/content_identity/mod.rs` owns the role-typed schedule-occurrence wrapper, re-exported by `src/content_identity/mod.rs`. Lattice keeps:

- domain `lattice-schedule-occurrence-v1`;
- `schedule-id` and `due-epoch-ms` tags;
- big-endian `i64` due-time projection;
- `b3:` output spelling;
- persistence, claim, validation, retry, and recovery meaning.

The shared core performs the existing hashed-part framing and BLAKE3 finalization. The fixed legacy and shared vectors are identical.

## Retained local roles

Lattice keeps value, node-execution, replay, orchestration, inspection, agent-program, webhook-claim, and traceability hashes local. Their encodings or product roles need separate migrations.

Vault streams, vault integrity, webhook signatures, credential correlation, password hashing, KDFs, authentication, and Valence evidence identities must not use the shared content-identity package.

See the Cairn inventory for the full compatibility table.

## Rollback

r[impl lattice.content_identity_adoption.cutover]

Rollback uses reviewed VCS state. Revert the Cargo and Nix pins, locks, wrapper, scheduler call site, tests, and docs together. Do not add a second runtime implementation or source fallback.

## Non-claims

r[impl lattice.content_identity_adoption.cutover.scenario.non-claim]

Fixed vector parity proves only the selected byte-framing result. It does not prove canonicalization correctness, semantic equality, confidentiality, authenticity, authorization, cache safety, deduplication safety, or release readiness.

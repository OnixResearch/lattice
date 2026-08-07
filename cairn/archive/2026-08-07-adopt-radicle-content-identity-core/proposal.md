## Why

Lattice has several independent BLAKE3 encodings for values, node execution, schedules, webhooks, orchestration, replay, and inspection. The proposed `content-identity-core` can centralize unambiguous framing, but Lattice already persists and compares many of these identities.

Adoption must preserve each compatibility surface or introduce a separately reviewed domain/version migration. Security-sensitive vault streams, signatures, credential handling, password hashing, and KDF behavior are outside this shared content-identity change.

## What Changes

- Inventory every BLAKE3 call, domain/tag, canonical input, output spelling, persisted/wire use, security role, and test vector.
- Verify the shared repository's licensed canonical Radicle source and narrow framing-only claim boundary.
- Classify each Lattice identity as eligible framing, retained local security mechanism, or separately versioned migration.
- Add a Lattice-owned versioned domain registry and explicit canonical projection helpers around the shared core.
- Dual-run fixed positive and negative vectors before replacing any framing helper.
- Align Cargo and Nix on one immutable Radicle revision without GitHub or sibling fallback.

## Success Criteria

- Every existing identity has an owner, role, domain/version, canonical input definition, and compatibility decision.
- Unchanged eligible surfaces produce identical full-width digests and serialized metadata.
- Ambiguous framing, unversioned domains, wrong-role digests, duplicate inputs, overflow, and security-role misuse fail closed.
- Vault encryption streams, webhook signatures, credentials, password hashes, KDFs, and Valence evidence identities remain outside the shared core.
- Lattice can revert all dependency and framing changes through reviewed VCS state.

## Non-Goals

- A universal artifact reference, canonical serializer, evidence identity, cryptographic protocol, cache-safety decision, or authorization mechanism.
- Changing stable `n8nclone` domain strings merely to rename them.
- Extracting keyed redaction or security-sensitive hash uses in this slice.
- Claiming digest equality proves semantic equality, authenticity, or truth.

## Impact

- **Source**: value/executor, trigger, schedule, orchestration, replay, and inspection identity helpers selected by inventory.
- **Dependencies**: exact Radicle source and generated Cargo/Nix lock updates.
- **Testing**: fixed vectors, persisted compatibility, role separation, source admission, Octet, Cairn, and Nix.

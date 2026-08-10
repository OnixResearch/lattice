## Context

Lattice uses raw `blake3::Hasher` and `blake3::hash` in several modules. The encodings are not uniform. Some are durable compatibility identities; others are display fingerprints, cache inputs, signatures, vault mechanics, or internal hints. Uniform syntax is not proof that they share one semantic owner.

## Success Contract

Completion requires a complete identity inventory, accepted producer evidence, role-specific eligibility decisions, fixed vector parity, exact source alignment, removal of selected duplicate framing, and tested rollback. Replacing every BLAKE3 call or hashing serde output through one helper is false completion.

## Decisions

### Inventory by role before changing code

Each current use records its purpose, domain/tag bytes, field order, length encoding, canonical projection, output spelling, persistence/wire status, consumers, and compatibility tests. Unknown roles remain local and block broad migration.

### Delegate framing, not domains

`content-identity-core` may write validated length-delimited material and structured digest metadata. Lattice owns versioned workflow, value, node execution, schedule, webhook claim, orchestration, replay, and inspection domain registries and canonical projections.

Stable legacy domain strings remain unchanged unless an explicit migration introduces a new version and compatibility plan. Product renaming alone is not a reason to change identity bytes.

### Exclude security-sensitive roles

Vault stream generation, vault integrity checks tied to encryption, webhook signatures, credential processing, password hashing, KDFs, key derivation, and authentication remain in their existing reviewed owners. The shared content identity is unkeyed content framing, not a cryptographic protocol toolkit.

### Preserve identities through fixed vectors

Each eligible surface gains positive vectors from current code and negative vectors for field omission, order change, wrong domain, wrong role, duplicate map entries, malformed metadata, and overflow. Legacy and shared paths dual-run before production replacement.

### Version deliberate identity changes

If current framing is ambiguous or defective, parity is not the goal. A separate domain/canonicalization version, migration rule, old/new read behavior, persisted-state treatment, and rollback evidence are required. This change cannot silently rewrite stored keys.

### Pin Radicle source exactly

Cargo and Nix use one canonical RID adapter and exact revision. No GitHub, branch, sibling, or runtime fallback is allowed. Owning tools generate lock changes; `flake.lock` is not manually edited.

## Validation

The inventory drives focused tests per identity family. Repository checks reject duplicate source packages, product domains in the shared crate, security-role imports, source mismatch, unknown RID, missing revision, and fallback configuration.

## Claim Boundary

Passing vectors prove only selected byte-framing compatibility. They do not prove canonicalization correctness, semantic equality, confidentiality, authenticity, authorization, cache safety, deduplication safety, or release readiness.

## Risks

- The identity inventory can miss persistence or external consumers. Source and schema searches must accompany tests.
- A framing helper can make wrong domain reuse easier. Role-typed Lattice wrappers remain mandatory.
- Legacy ambiguous encodings may need separate migrations rather than shared adoption.
- Producer licensing can block adoption.

# Lattice content identity inventory

Date: 2026-08-07

Source revision: `250f7d588d5a7a5ec76bc456d5dc57821714bd1d`

## Decision

Adopt the reviewed core for the schedule-occurrence family only. This family already uses the producer's canonical `update_hashed_part` encoding. Keep all other families local until a separate migration proves their role and storage behavior.

## Production identity families

| Owner | Role and material | Output and compatibility surface | Decision |
|---|---|---|---|
| `src/scheduler/admission.rs` | `lattice-schedule-occurrence-v1`; hashed `schedule-id` string and `due-epoch-ms` big-endian `i64` parts | `b3:<64 hex>`; stored schedule occurrence key, claim, validation, retry, and recovery identity | Eligible. Move framing to a role-typed Lattice wrapper. Preserve every byte. |
| `src/engine/executor.rs` | `node-execution:v1`; node ID, config variants, paths, HTTP fields, agent settings, values, ordered/sorted collections, and secret references | Raw 32-byte `ContentHash`; skip/reuse and execution behavior | Keep local. It mixes many product projections and secret-reference roles. A complete family migration is larger than this pilot. |
| `src/engine/value.rs` | Raw bytes, ordered hash concatenation, `image-value:v1`, and unversioned `error:` material | Raw 32-byte `ContentHash`; redb value keys, deduplication, input trees, and serialized hex | Keep local. Raw and ambiguous legacy encodings need separate versioned migration and storage policy. |
| `src/replay.rs` | Serde JSON workflow bytes, or a hash of a serialization-error string | Unprefixed 64-hex workflow snapshot field in replay state | Keep local. Serde output and error fallback are product policy and are not an admitted canonical projection. |
| `src/triggers.rs` credential reference | `lattice-webhook-credential-reference-context-v1`; hashed provider, key, and optional field parts | `b3:<64 hex>`; safe credential-reference display/correlation | Keep local. Credential correlation is security-sensitive and outside the shared package. |
| `src/triggers.rs` webhook claim | `lattice-webhook-idempotency-claim-v1`; hashed trigger and idempotency-key parts | `b3:<64 hex>`; persisted claim and duplicate-effect admission | Keep local. Claim identity has authorization and effect-admission meaning. |
| `src/triggers.rs` webhook signature | `n8nclone-webhook-signature:v1`; secret and raw body | Unprefixed 64 hex; webhook authentication | Excluded security mechanism. |
| `src/credentials.rs` | `n8nclone-local-vault-stream-v1` stream blocks and `n8nclone-local-vault-integrity-v1` plaintext integrity | Vault ciphertext mechanics and stored integrity value | Excluded security mechanism. |
| `src/orchestration/core.rs` | `lattice-orchestration:v1`, role label, and serde JSON bytes | Unprefixed 64 hex; orchestration facts, events, and replay checks | Keep local. Labels and serde projection remain orchestration policy. |
| `src/run/inspection/core.rs` | `lattice-run-workflow-name-v1`, NUL, workflow name | `b3:<64 hex>`; safe inspection identity | Keep local. Existing NUL framing differs from the reviewed encoding and needs a versioned migration. |
| `src/animus_agent/core.rs` | Raw program text | `b3:<64 hex>`; agent program identity | Keep local. It has no versioned domain and needs separate migration. |

## Tooling and evidence hashes

`scripts/traceability-inventory.rs`, `scripts/traceability_inventory/core.rs`, and traceability tests hash repository evidence or sets. Archived Cairn migration scripts also hash frozen evidence. These are tool/evidence owners, not production content identities. They remain local. Archived source is immutable.

## Shared source admission

- License: MIT.
- RID: `rad:zA14jhTdwST4PxPBdsryc4zS7Kpq`.
- Read-only adapter: `https://seed.radicle.garden/zA14jhTdwST4PxPBdsryc4zS7Kpq.git`.
- Reviewed object: `7f55597b5dc879b7601856e8d7fd0dbacaa2a498`.
- Source archive BLAKE3: `af024d70e39195a018e54cf31fdbba0dc64928af30e108f99d5988a3b10d0b7c`.
- Dependency compatibility: the reviewed object moves to BLAKE3 1.8.5 without changing fixed vectors, which aligns with maintained Lattice dependencies.
- Producer checks: positive and negative vectors, strict Clippy, formatting, Cairn validation, Nix checks, and `wasm32-unknown-unknown` `no_std + alloc` build passed.
- Fresh-clone exact-object acquisition passed; an unknown ref returned no object; no fallback exists.

## Rollback

Revert the Lattice dependency, lock, wrapper, scheduler call-site, Nix source, and tests together through reviewed VCS state. Do not add a branch, sibling path, GitHub mirror, runtime fallback, or second framing implementation.

## Non-claims

Schedule vector parity does not prove canonicalization correctness, semantic equality, confidentiality, authenticity, authorization, cache safety, deduplication safety, or release readiness.

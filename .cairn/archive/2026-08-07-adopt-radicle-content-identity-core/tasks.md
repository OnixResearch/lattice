## Phase 1: Complete identity inventory

- [x] [serial] Run existing focused identity tests before core dependency changes and inventory every BLAKE3 use, role, domain/tag, canonical input, order/length encoding, output spelling, persistence/wire surface, consumer, and rollback fact. r[lattice.content_identity_adoption.compatibility]
  - Baseline: 14 `test_scheduler_occurrences` tests passed. See `evidence/inventory.md`.
- [x] [serial] Classify each use as eligible unkeyed framing, retained local security mechanism, evidence-owned identity, or separate versioned migration; reject a blanket replacement. r[lattice.content_identity_adoption.boundary]
  - Only schedule-occurrence framing is eligible. Other product, security, and evidence families remain local.
- [x] [serial] Verify the shared source's explicit license, provenance, canonical RID, authority policy, exact object, source archive, framing boundary, embedded-target checks, and no-fallback receipt. r[lattice.content_identity_adoption.source]
  - Accepted reviewed object `7f55597b5dc879b7601856e8d7fd0dbacaa2a498`; see `evidence/inventory.md`.

## Phase 2: Lattice wrappers and dual run

- [x] [serial] Add Lattice-owned role-typed versioned domain and canonical-projection wrappers over the exact shared framing API. r[lattice.content_identity_adoption.boundary]
  - `src/content_identity/mod.rs` fixes the schedule role, domain, tags, integer projection, and output spelling.
- [x] [parallel] Add fixed positive legacy vectors for every eligible identity and negative wrong-domain, wrong-role, field omission, order change, duplicate input, malformed metadata, overflow, and unversioned-label vectors. r[lattice.content_identity_adoption.compatibility]
  - The eligible schedule family has fixed parity plus wrong-domain, order, omission, duplicate, unversioned, and security-role negatives. Producer validation covers bounded lengths and metadata generation; schedule output remains its existing `b3:<64 hex>` string, not shared metadata.
- [x] [parallel] Add guards proving vault streams, signatures, credentials, password hashes, KDFs, Valence evidence identities, product domains, and canonical projections do not move into the shared package. r[lattice.content_identity_adoption.boundary]
  - Rust and Nix guards reject shared imports in credential and trigger security owners.

## Phase 3: Exact source and incremental cutover

- [x] [serial] Align Cargo and Nix on the same Radicle adapter and immutable revision and regenerate locks with owning tools without manually editing `flake.lock`. r[lattice.content_identity_adoption.cutover]
  - Cargo, Nix, and Crane use the exact read-only adapter and object `7f55597b5dc879b7601856e8d7fd0dbacaa2a498`.
- [x] [serial] Replace only eligible framing helpers one identity family at a time after dual-run parity; keep incompatible or security-sensitive mechanisms local. r[lattice.content_identity_adoption.cutover]
  - Schedule occurrence is cut over. All other inventoried families remain local.
- [x] [serial] For any deliberate identity correction, stop and create a separate versioned migration with old/new storage behavior, compatibility, and rollback rather than changing bytes in this cutover. r[lattice.content_identity_adoption.compatibility]
  - No identity correction was made. The fixed schedule digest is unchanged.
- [x] [serial] Remove selected duplicate helpers only after every caller and persisted vector is covered; record VCS rollback for dependencies, locks, wrappers, and removals. r[lattice.content_identity_adoption.cutover]
  - The scheduler's duplicate hashed-part helper is removed. `docs/content-identity-source.md` records VCS rollback.

## Phase 4: Closeout

- [x] [parallel] Update README references, identity registry documentation, exact Radicle source, compatibility table, exclusions, rollback, and non-claims. r[lattice.content_identity_adoption.source] r[lattice.content_identity_adoption.boundary]
  - See `README.md`, `docs/content-identity-source.md`, and `evidence/inventory.md`.
- [x] [serial] Run focused tests again, `scripts/check.rs`, Octet, Cairn validation and gates, source/role guards, and bounded `nix flake check` before sync and archive. r[lattice.content_identity_adoption.compatibility] r[lattice.content_identity_adoption.cutover]
  - Focused tests, source/role guards, Nix format, strict consumer Clippy, and Cairn gates pass. `scripts/check.rs` and Octet reach only the unchanged repository baselines recorded in `evidence/verification.md`.

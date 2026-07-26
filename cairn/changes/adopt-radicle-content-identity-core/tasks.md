## Phase 1: Complete identity inventory

- [ ] [serial] Run existing focused identity tests before core dependency changes and inventory every BLAKE3 use, role, domain/tag, canonical input, order/length encoding, output spelling, persistence/wire surface, consumer, and rollback fact. r[lattice.content_identity_adoption.compatibility]
- [ ] [serial] Classify each use as eligible unkeyed framing, retained local security mechanism, evidence-owned identity, or separate versioned migration; reject a blanket replacement. r[lattice.content_identity_adoption.boundary]
- [ ] [serial] Verify the shared source's explicit license, provenance, canonical RID, authority policy, exact object, source archive, framing boundary, embedded-target checks, and no-fallback receipt. r[lattice.content_identity_adoption.source]

## Phase 2: Lattice wrappers and dual run

- [ ] [serial] Add Lattice-owned role-typed versioned domain and canonical-projection wrappers over the exact shared framing API. r[lattice.content_identity_adoption.boundary]
- [ ] [parallel] Add fixed positive legacy vectors for every eligible identity and negative wrong-domain, wrong-role, field omission, order change, duplicate input, malformed metadata, overflow, and unversioned-label vectors. r[lattice.content_identity_adoption.compatibility]
- [ ] [parallel] Add guards proving vault streams, signatures, credentials, password hashes, KDFs, Valence evidence identities, product domains, and canonical projections do not move into the shared package. r[lattice.content_identity_adoption.boundary]

## Phase 3: Exact source and incremental cutover

- [ ] [serial] Align Cargo and Nix on the same Radicle adapter and immutable revision and regenerate locks with owning tools without manually editing `flake.lock`. r[lattice.content_identity_adoption.cutover]
- [ ] [serial] Replace only eligible framing helpers one identity family at a time after dual-run parity; keep incompatible or security-sensitive mechanisms local. r[lattice.content_identity_adoption.cutover]
- [ ] [serial] For any deliberate identity correction, stop and create a separate versioned migration with old/new storage behavior, compatibility, and rollback rather than changing bytes in this cutover. r[lattice.content_identity_adoption.compatibility]
- [ ] [serial] Remove selected duplicate helpers only after every caller and persisted vector is covered; record VCS rollback for dependencies, locks, wrappers, and removals. r[lattice.content_identity_adoption.cutover]

## Phase 4: Closeout

- [ ] [parallel] Update README references, identity registry documentation, exact Radicle source, compatibility table, exclusions, rollback, and non-claims. r[lattice.content_identity_adoption.source] r[lattice.content_identity_adoption.boundary]
- [ ] [serial] Run focused tests again, `scripts/check.rs`, Octet, Cairn validation and gates, source/role guards, and bounded `nix flake check` before sync and archive. r[lattice.content_identity_adoption.compatibility] r[lattice.content_identity_adoption.cutover]

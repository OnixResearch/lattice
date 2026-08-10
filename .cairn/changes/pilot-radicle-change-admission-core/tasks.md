## Phase 1: Lattice baseline and producer review

- [ ] [serial] Run focused orchestration proposal tests before core changes and record proposal/approval identity vectors, successor validation, epoch/base checks, authority, events, replay, serialized fixtures, CLI output, and rollback. r[lattice.change_admission_adoption.compatibility]
- [ ] [serial] Review the producer's lossless alignment, simplifying two-consumer pilot, explicit license, provenance, canonical RID, exact source, endpoint, and no-fallback evidence; reject adoption when extraction was not selected. r[lattice.change_admission_adoption.source]
- [ ] [parallel] Add an exact mapping table for proposal, base, candidate, admissibility, current epoch/revision, supplied authorization, disposition, and retained Lattice authority. r[lattice.change_admission_adoption.boundary]

## Phase 2: Disposable adapter

- [ ] [serial] Build an in-memory orchestration adapter with role-typed opaque identities and no change to product hashes, persisted facts/events, workflow revisions, safe summaries, or CLI JSON. r[lattice.change_admission_adoption.boundary]
- [ ] [parallel] Add parity fixtures for proposal submission, exact successor validation, authorized admission, rejection, discard where mapped, and replay. r[lattice.change_admission_adoption.compatibility]
- [ ] [parallel] Add negative malformed/mismatched identity, stale epoch/base, changed candidate, failed validation, authority mismatch, unauthorized actor, repeated terminal decision, terminal mutation, and persisted-schema fixtures. r[lattice.change_admission_adoption.compatibility]
- [ ] [parallel] Measure removed branches, adapter size, callbacks, assertions, and retained product logic. r[lattice.change_admission_adoption.cutover]

## Phase 3: Adoption decision and conditional source

- [ ] [serial] Adversarially audit preview-as-approval, actor-authority leakage, stale acceptance, candidate substitution, mutation-success invention, event/replay compatibility, and adapter complexity. r[lattice.change_admission_adoption.cutover]
- [ ] [serial] Select adopt or reject; reject when adapters retain equivalent product logic or any mapping weakens current identity, stale, authority, or terminal behavior. r[lattice.change_admission_adoption.cutover]
- [ ] [serial] If adopted, align Cargo and Nix on the same Radicle adapter and immutable revision and regenerate locks with owning tools without manually editing `flake.lock`. r[lattice.change_admission_adoption.source]
- [ ] [serial] Remove duplicate local decisions only after full parity and record one VCS rollback; do not retain runtime, sibling, vendor, or GitHub fallback. r[lattice.change_admission_adoption.cutover]

## Phase 4: Closeout

- [ ] [parallel] Update README references, lifecycle mapping, exact source, identity/persistence compatibility, rollback, and bounded non-claims. r[lattice.change_admission_adoption.source] r[lattice.change_admission_adoption.boundary]
- [ ] [serial] Run focused tests again, `scripts/check.rs`, Octet, Cairn validation and gates, identity/schema/source guards, and bounded `nix flake check` before sync and archive, or archive a truthful rejection with its blocker. r[lattice.change_admission_adoption.compatibility] r[lattice.change_admission_adoption.cutover]

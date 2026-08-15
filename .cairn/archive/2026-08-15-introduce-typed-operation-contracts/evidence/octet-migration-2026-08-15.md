# Typed-operation Octet migration evidence

Date: 2026-08-15

## Result

The typed-operation implementation passes the pinned Octet precommit gate.

The first final-review run reported 151 new identities and 38 stale identities. A semantic comparison found 38 identity pairs and 113 unpaired new findings. The implementation removed all 113 new findings before baseline migration.

The repairs:

- split operation code into bounded catalog, CLI, model, planning, runtime, and evidence components;
- kept no more than seven Rust source files in each operation directory;
- bounded collection growth and documented narrow cases where caller-owned diagnostic vectors must grow;
- replaced raw arithmetic with named byte limits;
- made Boolean names explicit;
- reduced oversized integration functions;
- moved the operation process adapter into its own imperative-shell component;
- kept strict Clippy and direct positive and negative tests;
- used narrow source attributes with specific reasons for bounded validation and external schema names.

No lint was disabled. No warning budget was added. The migrated baseline accepts no new typed-operation finding.

## Reviewed identity transition

After repair, the exact drift was:

- 37 semantically equivalent identity pairs for pre-existing findings in reviewed integration files;
- one repaired stale finding;
- zero unpaired new findings;
- zero lint-category growth.

The repaired finding was `b3:2d079e31dff7e630`. It was the `catch_all_on_enum` finding in `src/tui/app.rs`. The repair replaced a catch-all branch with an exhaustive `NodeConfig` match, including the new operation variant.

The exact reviewed drift-set identities are:

- new set: `b3:8e66eb49ab81a39c7bfaceb32401a7555d3a55d01ab97db39e3be768ff00a7ef`;
- stale set: `b3:a3b627677cf634efaf441859d2980b9be2a09d4165ac232ea15154151a877448`.

The migration compared 856 old identities with 855 current identities. Its positive and negative self-tests passed before execute mode.

## Accepted baselines

The new top-level baseline contains 855 accepted identities. It has BLAKE3 identity `b3:990b93716039ed0608a4a3614b46808ff359591f0ea75b4973460c95ef726ad7`.

The independent `lattice-core` baseline remains unchanged with 24 accepted identities. It has BLAKE3 identity `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

The prior top-level baseline identity `b3:3212f34e0b57ed9e3e49aff1f5f7cf6267b627b9e4b9498a0da09934eb04a642` remains historical evidence.

## Commands

```text
cargo -q -Zscript \
  .cairn/changes/introduce-typed-operation-contracts/evidence/migrate-octet-baseline.rs \
  --self-test

cargo -q -Zscript \
  .cairn/changes/introduce-typed-operation-contracts/evidence/migrate-octet-baseline.rs \
  octet-baseline.json \
  target/octet-precommit/results.jsonl \
  target/octet-precommit/status.json \
  octet-baseline.json \
  --execute

octet-precommit-check
```

The final Octet status is `warning-only` with zero new and zero stale findings.

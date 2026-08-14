# Guarded-agent Octet migration evidence

## Result

The guarded-agent implementation passes the pinned Octet precommit gate.

The first implementation run compared 856 accepted top-level identities with 883 current identities. It reported 27 new findings and no stale findings. The implementation repaired all 27 findings before baseline migration.

The repairs removed these finding classes from the new guarded source and integration code:

- assertion density and oversized functions;
- oversized or crowded modules;
- unbounded collection growth;
- compound conditions;
- raw arithmetic overflow;
- repeated owner-path names;
- non-predicate Boolean names;
- unnamed numeric units.

The repair pass split the guarded core and shell into narrow files. It bounded collections, decomposed policy checks, added checked arithmetic, and added explicit invariants. It did not disable a lint, add a warning budget, or accept a new guarded finding.

## Reviewed identity transition

After repairs, the only drift was one semantically equivalent identity pair for the pre-existing `src/orchestration/shell.rs` file-length finding:

- stale identity: `b3:192cf045d87b7709`;
- current identity: `b3:fdf9d7c7d2e61698`.

Both findings have lint `excessive_file_length`, owner path `src/orchestration/shell.rs`, and message `source file is 786 lines (limit: 300)`. The identity changed because reviewed guarded-status integration changed the exact source bytes. The migration pairs only this finding.

The exact reviewed drift-set identities are:

- new set: `b3:0d6a54edc74b22e60eb4d28dffa444fee472ee573eed197664f80b1d06653189`;
- stale set: `b3:818c56b4c38398bdf5360888ec54e229d78cb5625dbd1489e829eafcf8bc1a6f`.

The migration compared 856 old identities with 856 current identities. It paired one identity, recorded zero repaired identities in the migration itself, and found no lint-category growth. Its positive and negative self-tests passed before execute mode.

## Accepted baselines

The new top-level baseline contains 856 accepted identities. It has BLAKE3 identity `b3:36d7c9cd79a3c05d314212ed40957568497d0aabec4a5327d568d8d760714da5`.

The independent `lattice-core` baseline remains unchanged with 24 accepted identities. It has BLAKE3 identity `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

The prior top-level baseline identity `b3:ea0aec4872a3d77db6bdc0cdb39fec49cbc9e4fce2e5fdcb6c3abeef396984bf` remains historical evidence.

## Commands

```text
cargo -q -Zscript \
  .cairn/changes/add-guarded-agent-loop-templates/evidence/migrate-octet-baseline.rs \
  --self-test

cargo -q -Zscript \
  .cairn/changes/add-guarded-agent-loop-templates/evidence/migrate-octet-baseline.rs \
  octet-baseline.json \
  target/octet-precommit/results.jsonl \
  target/octet-precommit/status.json \
  octet-baseline.json \
  --execute

octet-precommit-check
```

The final Octet run reported zero new and zero stale findings for both checked baseline scopes.

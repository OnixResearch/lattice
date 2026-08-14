# Agent-evaluation Octet migration evidence

Date: 2026-08-14

## Result

The manifest-driven agent-evaluation lane passes the pinned Octet precommit gate.

The first implementation run reported 114 new project-owned findings. The implementation repaired them before baseline migration. Repairs split large modules, bounded collection growth, reduced function and condition size, used checked arithmetic, qualified ownership paths, and added explicit invariants. No lint was disabled. No warning budget or finding baseline accepted new evaluation-lane debt.

## Reviewed identity transition

After repair, the only drift was one semantically equivalent identity pair for the pre-existing `src/store/mod.rs` file-length finding:

- stale identity: `b3:e1323bfbcba495ab`;
- current identity: `b3:fb2a41230779ce0b`.

Both findings use lint `excessive_file_length`, owner path `src/store/mod.rs`, and message `source file is 1494 lines (limit: 300)`. The identity changed because the evaluation tables and delegating store methods changed the exact source bytes. The migration pairs only this finding.

The exact reviewed drift-set identities are:

- new set: `b3:8509ce5915a85697c96e58e84c4ed57c8fe9f87f08fe7f89546bbebdd7427be0`;
- stale set: `b3:fa6e9257759af1a03200e21989be03e995e0c83d7d015ad4f0993d034f0fec72`.

The migration compared 856 old identities with 856 current identities. It paired one identity, recorded zero repaired identities in the migration itself, and found no lint-category growth. Its positive and negative self-tests passed before execute mode.

## Accepted baselines

The new top-level baseline contains 856 accepted identities. It has BLAKE3 identity `b3:3212f34e0b57ed9e3e49aff1f5f7cf6267b627b9e4b9498a0da09934eb04a642`.

The independent `lattice-core` baseline remains unchanged with 24 accepted identities. It has BLAKE3 identity `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

The prior top-level baseline identity `b3:36d7c9cd79a3c05d314212ed40957568497d0aabec4a5327d568d8d760714da5` remains historical evidence.

## Commands

```text
cargo -q -Zscript \
  .cairn/changes/add-manifest-driven-agent-evaluation-lane/evidence/migrate-octet-baseline.rs \
  --self-test

cargo -q -Zscript \
  .cairn/changes/add-manifest-driven-agent-evaluation-lane/evidence/migrate-octet-baseline.rs \
  octet-baseline.json \
  target/octet-precommit/results.jsonl \
  target/octet-precommit/status.json \
  octet-baseline.json \
  --execute

octet-precommit-check
```

The final Octet run reported zero new and zero stale findings for both checked baseline scopes.

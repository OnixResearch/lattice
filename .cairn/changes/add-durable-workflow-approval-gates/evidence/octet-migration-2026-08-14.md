# Octet migration evidence

## Result

The approval implementation passes the pinned Octet precommit gate.

The reviewed migration compared 859 old top-level identities with 858 current identities. It paired all 47 new identities with 47 stale identities by lint, owner path, and digit-normalized diagnostic meaning.

There were no unpaired new identities and no lint-category growth. One old `unbounded_collection_growth` identity, `b3:acd72b84ffa72a11`, was removed after the approval executor built checkpoint runner state from the admitted bounded checkpoint.

The exact reviewed drift identities are:

- new set: `b3:19d25f319d1c2a23a354ff023755cb7074f55e4ec84a8564ecc8f6de4f0ae532`;
- stale set: `b3:bc3c37604b93f9992c64a672af1128399e6b830918b946259204ddaf51ae4199`.

The migration script has positive and negative self-tests. Its dry run and execute mode both passed.

## Accepted baselines

The new top-level baseline contains 858 accepted identities. It has BLAKE3 identity `b3:de11a299574a712aaa04b45eb266abf14480a20e389ea6eeb9f37283fac3cc28`.

The independent `lattice-core` baseline remains unchanged with 24 accepted identities. It has BLAKE3 identity `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

The historical top-level baseline identity `b3:06a05f687a1e64bf1e28ddffecc77aa49bc09e901a15e60663701aaf6cc967e1` remains historical evidence. This change does not rewrite earlier receipts.

## Commands

```text
nix develop -c cargo -q -Zscript \
  .cairn/changes/add-durable-workflow-approval-gates/evidence/migrate-octet-baseline.rs \
  --self-test

nix develop -c cargo -q -Zscript \
  .cairn/changes/add-durable-workflow-approval-gates/evidence/migrate-octet-baseline.rs \
  octet-baseline.json \
  target/octet-precommit/results.jsonl \
  target/octet-precommit/status.json \
  octet-baseline.json \
  --execute

nix develop -c octet-precommit-check
```

The final Octet run reported zero new and zero stale findings for both checked baseline scopes.

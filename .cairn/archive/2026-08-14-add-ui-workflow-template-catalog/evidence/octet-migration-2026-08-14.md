# Workflow-template Octet migration evidence

## Result

The workflow-template implementation passes the pinned Octet precommit gate.

The reviewed migration compared 858 old top-level identities with 856 current identities. It paired all eight new identities with eight stale identities by lint, owner path, and digit-normalized diagnostic meaning.

There were no unpaired new identities and no lint-category growth. Two old assertion-density findings were repaired:

- `b3:17423351ffdcecac` for `src/tui/panels.rs::render`;
- `b3:50a5898367240b62` for `src/tui/input.rs::handle_key`.

The eight paired transitions cover only line-count or owner-file identity changes in reviewed files. They include existing file-length findings, the existing `workflow_format.rs` module-name finding, and the existing `App` and `AppAction` owner-path findings.

The exact reviewed drift identities are:

- new set: `b3:7803098feba352ecf61a66af48a1402fc9ed85c647cebd720e63b3b5ed7ca5fb`;
- stale set: `b3:42e4b245ad17172759cf2be8e238be2c49b4c5ad06dc6156663912b22eef6aba`.

The migration script has positive and negative self-tests. Its dry-run and execute modes passed.

## Accepted baselines

The new top-level baseline contains 856 accepted identities. It has BLAKE3 identity `b3:ea0aec4872a3d77db6bdc0cdb39fec49cbc9e4fce2e5fdcb6c3abeef396984bf`.

The independent `lattice-core` baseline remains unchanged with 24 accepted identities. It has BLAKE3 identity `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

The prior top-level baseline identity `b3:de11a299574a712aaa04b45eb266abf14480a20e389ea6eeb9f37283fac3cc28` remains historical evidence. This change does not rewrite earlier receipts.

## Commands

```text
nix develop -c cargo -q -Zscript \
  .cairn/changes/add-ui-workflow-template-catalog/evidence/migrate-octet-baseline.rs \
  --self-test

nix develop -c cargo -q -Zscript \
  .cairn/changes/add-ui-workflow-template-catalog/evidence/migrate-octet-baseline.rs \
  octet-baseline.json \
  target/octet-precommit/results.jsonl \
  target/octet-precommit/status.json \
  octet-baseline.json \
  --execute

nix develop -c octet-precommit-check
```

The final Octet run reported zero new and zero stale findings for both checked baseline scopes.

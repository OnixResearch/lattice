## Phase 1: Baseline and admission core

- [x] [serial] r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.nested-io] Run the existing focused file-node suite before core changes; baseline is 11/11 passing.
- [x] [serial] r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.missing-root] Add deterministic root/relative-path selection that rejects missing authority and empty targets.
- [x] [serial] r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.traversal] Reject parent, root, and prefix components in admitted relative targets.

## Phase 2: Capability shell

- [x] [serial] r[n8nclone.workflow_execution.file_nodes.local_capabilities] Route local read, write, and append operations through an open `cap-std` directory and relative target.
- [x] [serial] r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.symlink-escape] Verify static target and intermediate symlink escapes fail without outside effects.
- [x] [serial] r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.path-swap] Add a deterministic descriptor-root replacement test and verify outside content remains unchanged.

## Phase 3: Validation and lifecycle

- [x] [serial] r[n8nclone.workflow_execution.file_nodes.local_capabilities.scenario.nested-io] Run focused and full repository validation, audit the surviving mechanism, sync the accepted specification, archive the change, and commit locally. Evidence: focused file-node tests pass 13/13 plus 3/3 capability-core tests; `scripts/check.rs`, Clippy, Octet exact parity at 831/831, Cairn gates, and `nix flake check -L --option secret-key-files ''` all pass.

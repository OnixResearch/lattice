## Phase 1: Baseline and shared core

- [x] [serial] r[n8nclone.workflow_execution.capability_reads] Run focused image, workflow-format, and agent-node suites before core changes; all focused baselines pass.
- [x] [serial] r[n8nclone.workflow_execution.capability_reads.scenario.file-effects] Extract deterministic rooted-path admission and descriptor acquisition from the file-effect shell into a shared module.
- [x] [serial] r[n8nclone.workflow_execution.capability_reads.scenario.path-swap] Add bounded same-handle reads and deterministic post-root-open replacement tests.

## Phase 2: Reader migrations

- [x] [serial] r[n8nclone.workflow_execution.image_input_nodes.capability_read] Migrate production image inputs with explicit roots, same-handle metadata, and bounded reads.
- [x] [serial] r[n8nclone.workflow_format.markdown_agent_prompts.capability_read] Migrate Markdown prompt hydration while preserving source, UTF-8, non-empty, and byte-limit behavior.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.capability_read] Migrate agent image/artifact handoffs while preserving cwd confinement, limits, media checks, and diagnostics.

## Phase 3: Validation and lifecycle

- [x] [serial] r[n8nclone.workflow_execution.capability_reads] Run focused and full checks, audit Octet context migration if dependency metadata changes, sync accepted specs, archive, commit, and push. Evidence: focused reader and capability tests, `scripts/check.rs`, Clippy, Octet exact at 828/828 with four findings removed, Cairn gates, and `nix flake check -L --option secret-key-files ''` pass.

## Phase 1

- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.command_runtimes.config] Replace dedicated Pi/Clankers command fields with a generic runtime map and defaults.
- [x] [parallel] r[n8nclone.workflow_execution.agent_nodes.command_runtimes.planning] Refactor production command planning to select runtime entries and render generic argument placeholders.
- [x] [parallel] r[n8nclone.workflow_execution.agent_nodes.command_runtimes.errors] Add missing-runtime and empty-command negative coverage.
- [x] [parallel] r[n8nclone.workflow_execution.agent_nodes.command_runtimes.compatibility] Preserve existing Pi/Clankers text, JSON, image, artifact, TUI, and headless behavior through runtime presets.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.command_runtimes] Run `./scripts/check.rs` and archive or report blockers.

## Phase 1

- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.production_adapters.config] Add production runtime adapter configuration and defaults for Pi and Clankers command execution.
- [x] [parallel] r[n8nclone.workflow_execution.agent_nodes.production_adapters.route] Add a production adapter router that dispatches generic agent requests to the matching configured runtime without cross-target fallback.
- [x] [parallel] r[n8nclone.workflow_execution.agent_nodes.production_adapters.command] Add pure command planning and response decoding helpers for text and JSON output modes.
- [x] [parallel] r[n8nclone.workflow_execution.agent_nodes.production_adapters.errors] Add negative coverage for missing commands, unsupported output modes, invalid JSON, spawn failures, and non-zero exits.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.production_adapters.default_executor] Wire `Executor::new()` into the production router while preserving injected fake-adapter constructors for tests.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.production_adapters] Run `./scripts/check.rs` and archive or report remaining blockers.

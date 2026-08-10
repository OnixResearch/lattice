## Phase 1: Runtime and process ports

- [x] [serial] r[n8nclone.workflow_execution.trait_boundaries.runtime_dispatch] Define `NodeRuntime`, runtime invocation/result types, and a registry adapter for built-in runtime bindings.
- [x] [parallel] r[n8nclone.workflow_execution.trait_boundaries.runtime_dispatch] Route shell, HTTP, file, image, and generic agent execution through registered built-in runtime implementations while preserving existing executor APIs.
- [x] [parallel] r[n8nclone.workflow_execution.trait_boundaries.process_runner] Add `ProcessRunner`, `ProcessRequest`, `ProcessOutput`, and fake/production runners for shell-like and command-backed agent execution.
- [x] [parallel] r[n8nclone.workflow_execution.trait_boundaries.process_runner] Add positive and negative process-runner tests for successful output, timeout, nonzero status, spawn failure, stdin, cwd, env, and output-limit behavior.

## Phase 2: Events and persistence ports

- [x] [serial] r[n8nclone.workflow_execution.trait_boundaries.event_sink] Define `RunnerEvent`, ordered sequence assignment, redaction-safe summaries, and `RunnerEventSink` adapters for test capture and no-op behavior.
- [x] [parallel] r[n8nclone.workflow_execution.trait_boundaries.event_sink] Route TUI, headless, scheduler, and run-history reporting through event-sink adapters without changing visible run results.
- [x] [serial] r[n8nclone.workflow_execution.trait_boundaries.storage_ports] Split concrete store dependencies into workflow repository, run history, execution cache, value store, and schedule store traits.
- [x] [parallel] r[n8nclone.workflow_execution.trait_boundaries.storage_ports] Update headless, scheduler, and runner-shell entrypoints to accept the narrowest storage trait required and add fake-store positive/negative coverage.

## Phase 3: Node-kind metadata providers

- [x] [serial] r[n8nclone.tui_ux.node_kind_provider.property_metadata] Define node-kind provider metadata for labels, port schemas, editable fields, and property rendering.
- [x] [parallel] r[n8nclone.workflow_validation.node_kind_provider.config_validation] Move built-in node-specific validation into provider-backed validators with positive and negative diagnostics tests.
- [x] [parallel] r[n8nclone.workflow_format.node_kind_provider.handlers] Move built-in Nickel import/export node-config mapping into provider-backed handlers with compatibility and malformed-input tests.
- [x] [parallel] r[n8nclone.tui_ux.node_kind_provider.property_metadata] Replace broad TUI property-panel `NodeConfig` matches with provider-backed field descriptors where built-in providers exist.

## Phase 4: Compatibility and gates

- [x] [serial] r[n8nclone.workflow_execution.trait_boundaries.compatibility] Keep `Executor`, TUI, headless, scheduler, and tests as compatibility facades over trait-backed boundaries.
- [x] [parallel] r[n8nclone.workflow_validation.node_kind_provider.runtime_binding] Add runner-admission diagnostics for missing runtime bindings and provider/runtime mismatches.
- [x] [serial] r[n8nclone.workflow_execution.trait_boundaries] Run focused runtime/process/storage/event/provider tests, `./scripts/check.rs`, and archive or report blockers.

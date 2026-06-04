## Phase 1

- [ ] [serial] r[n8nclone.workflow_execution.image_input_nodes.production_io.local] Add `LocalImageIo` with filesystem metadata/read plumbing and pure byte-to-image planning helpers.
- [ ] [parallel] r[n8nclone.workflow_execution.image_input_nodes.production_io.default_executor] Wire `Executor::new()` to use production local image I/O while preserving explicit injected `ImageIo` constructors for tests.
- [ ] [parallel] r[n8nclone.workflow_execution.image_input_nodes.production_io.headless] Add a headless/default-executor test proving a temp local PNG workflow loads and produces typed image outputs.
- [ ] [parallel] r[n8nclone.workflow_execution.image_input_nodes.production_io.tui] Add a TUI/default-executor regression test proving image input nodes no longer fail with `NoopImageIo` when a valid local file is configured.
- [ ] [parallel] r[n8nclone.workflow_execution.image_input_nodes.production_io.errors] Add negative tests for missing file, unsupported bytes, oversized file, permission/denied read where practical, and outside-root paths.
- [ ] [serial] r[n8nclone.workflow_execution.image_input_nodes.production_io.no_import_io] Verify Nickel import/export and workflow validation do not read image bytes during static operations.
- [ ] [serial] r[n8nclone.workflow_execution.image_input_nodes.production_io.validation] Run `cargo fmt --check`, `cargo check`, `cargo test --workspace`, and the Cairn spec shape test.

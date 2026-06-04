## Phase 1

- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.contract] Add stdout JSON handoff envelope parsing for image and artifact production command modes.
- [x] [parallel] r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.image] Load image handoff files into typed image values with sniffed media type and stable hashes.
- [x] [parallel] r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.artifact] Load artifact handoff files into bytes values.
- [x] [parallel] r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.safety] Enforce working-directory containment, file-size limits, invalid envelope, missing file, media mismatch, and unsupported image failures.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.output_file_handoff.default_runtime] Add a headless/default-runtime regression for image handoff through workflow output ports.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.output_file_handoff] Run `./scripts/check.rs` and archive or report blockers.

## Phase 1

- [ ] [serial] r[n8nclone.workflow_execution.image_values.model] Add image value and image port modeling with stable content hashing and metadata summaries.
- [ ] [serial] r[n8nclone.workflow_execution.image_input_nodes.load] Add image input node configs, load plans, validation diagnostics, and in-memory positive/negative tests.
- [ ] [serial] r[n8nclone.workflow_execution.agent_nodes.config] Add generic agent node configs for prompt/AI/LLM-style work with text, image, JSON, and artifact ports.
- [ ] [parallel] r[n8nclone.workflow_execution.agent_nodes.execute] Execute generic agent requests through adapter traits and map text, JSON, image, and artifact outputs onto real graph ports.
- [ ] [parallel] r[n8nclone.workflow_execution.agent_nodes.pi_adapter] Add a Pi adapter implementation seam with deterministic fake-adapter tests.
- [ ] [parallel] r[n8nclone.workflow_execution.agent_nodes.clankers_adapter] Add a Clankers adapter implementation seam with deterministic fake-adapter tests.
- [ ] [parallel] r[n8nclone.workflow_execution.agent_nodes.cache_key] Include upstream hashes, prompt text, target runtime, model hints, tool policy, context references, and output mode in generic agent cache identity.
- [ ] [parallel] r[n8nclone.workflow_execution.agent_nodes.errors] Cover unsupported format, missing image, oversized input, decode failure, invalid target, credential/session failure, preflight rejection, and adapter failure cases.
- [ ] [parallel] r[n8nclone.workflow_format.agent_nodes.nickel] Extend Nickel import/export and round-trip tests for image input and generic agent nodes.
- [ ] [parallel] r[n8nclone.workflow_format.agent_nodes.invalid] Add invalid workflow fixtures for malformed generic agent configs and incompatible image/agent ports.
- [ ] [parallel] r[n8nclone.workflow_examples.image] Add an importable and headlessly runnable selfie-to-fine-art example using image input nodes and a generic agent node backed by fake Pi/Clankers adapters.
- [ ] [parallel] r[n8nclone.tui_ux.agent_nodes.add_flow] Update the TUI add-node flow so users can create image input nodes and generic Pi/Clankers agent nodes.
- [ ] [parallel] r[n8nclone.tui_ux.agent_nodes.properties] Add editable image path, prompt, target runtime, model hints, tool policy, context, output mode, and credential/session fields.
- [ ] [parallel] r[n8nclone.tui_ux.image_nodes.preview] Render bounded image previews or metadata fallbacks in the canvas/output surfaces.
- [ ] [parallel] r[n8nclone.tui_ux.agent_nodes.errors] Add TUI tests for recoverable missing-file, invalid-format, invalid-target, preflight, and adapter-error feedback.

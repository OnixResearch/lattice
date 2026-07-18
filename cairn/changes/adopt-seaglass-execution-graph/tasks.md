# Tasks

- [ ] [serial] r[n8nclone.workflow_execution.seaglass_graph_core] Vendor the Seaglass execution-graph core with exact source revision and a repo-owned Rust sync/import tool.
- [ ] [serial] r[n8nclone.workflow_execution.seaglass_graph_core.scenario.compatibility] Adapt runner admission, order, readiness, lifecycle, resume, critical path, and bottleneck queries while preserving typed runtime contracts.
- [ ] [parallel] r[n8nclone.workflow_format.markdown_agent_prompts.scenario.valid] Add bounded contained Markdown prompt-file loading and round-trip source metadata.
- [ ] [parallel] r[n8nclone.workflow_format.markdown_agent_prompts.scenario.invalid] Add negative tests for empty, multiple, absolute, traversal, escaping symlink, wrong extension, missing, oversized, and invalid UTF-8 prompt sources.
- [ ] [serial] r[n8nclone.workflow_execution.llm_operating_graph] Add a runnable role-agent operating-graph fixture with explicit fan-out, state/effect boundaries, and aggregation.
- [ ] [serial] r[n8nclone.workflow_execution.llm_operating_graph.scenario.feedback-boundary] Document feedback and dynamic graph changes as between-run inputs rather than hidden intra-run cycles.
- [ ] [serial] r[n8nclone.workflow_execution.rats_dependency] Adopt the newest consumable Rats source and retain evidence for any source-owner packaging repair.
- [ ] [serial] r[n8nclone.workflow_execution.llm_operating_graph] Add Callee and FarmTable to README references because their designs were used.
- [ ] [serial] r[n8nclone.workflow_execution.seaglass_graph_core] Run focused positive/negative tests, full repo checks, formatting, Clippy/Octet where available, and Cairn validation; record exact blockers.

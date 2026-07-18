# Tasks

- [x] [serial] r[n8nclone.workflow_execution.seaglass_graph_core] Vendor the Seaglass execution-graph core with exact source revision and a repo-owned Rust sync/import tool.
- [x] [serial] r[n8nclone.workflow_execution.seaglass_graph_core.scenario.compatibility] Adapt runner admission, order, readiness, lifecycle, resume, critical path, and bottleneck queries while preserving typed runtime contracts.
- [x] [parallel] r[n8nclone.workflow_format.markdown_agent_prompts.scenario.valid] Add bounded contained Markdown prompt-file loading and round-trip source metadata.
- [x] [parallel] r[n8nclone.workflow_format.markdown_agent_prompts.scenario.invalid] Add negative tests for empty, multiple, absolute, traversal, escaping symlink, wrong extension, missing, oversized, and invalid UTF-8 prompt sources.
- [x] [serial] r[n8nclone.workflow_execution.llm_operating_graph] Add a runnable role-agent operating-graph fixture with explicit fan-out, state/effect boundaries, and aggregation.
- [x] [serial] r[n8nclone.workflow_execution.llm_operating_graph.scenario.feedback-boundary] Document feedback and dynamic graph changes as between-run inputs rather than hidden intra-run cycles.
- [x] [serial] r[n8nclone.workflow_execution.rats_dependency] Adopt the newest consumable Rats source and retain evidence for any source-owner packaging repair.
- [x] [serial] r[n8nclone.workflow_execution.llm_operating_graph] Add Callee and FarmTable to README references because their designs were used.
- [x] [serial] r[n8nclone.workflow_execution.seaglass_graph_core] Run focused positive/negative tests, full repo checks, formatting, Clippy/Octet where available, and Cairn validation; record exact blockers.

## Validation evidence

- `scripts/sync-vendored-sibling.rs --self-test` passed.
- `scripts/check.rs` passed formatting, the full default-member test suite, Clippy with warnings denied, and repository spec-reference checks.
- Focused runner, workflow-format, executor, and LLM operating-graph tests passed, including failure propagation and every Markdown prompt rejection case.
- `octet-precommit-check` reached `integration-failure`: changing Rats from Git sources to path-vendored source made the existing baseline report `843 accepted, 0 current accepted, 871 new, 843 stale` and included vendored Rats findings. The baseline was not broadly re-accepted because those source-identity and upstream findings are outside this consumer change's claim boundary.
- Native Cairn validation remains blocked before change evaluation by the repository's pre-existing generated policy: `failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field gate_policy.substance`.

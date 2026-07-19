# Tasks

- [x] [serial] r[n8nclone.workflow_execution.seaglass_graph_core] Vendor the Seaglass execution-graph core with exact source revision and a repo-owned Rust sync/import tool.
- [x] [serial] r[n8nclone.workflow_execution.seaglass_graph_core.scenario.compatibility] Adapt runner admission, order, readiness, lifecycle, resume, critical path, and bottleneck queries while preserving typed runtime contracts.
- [x] [parallel] r[n8nclone.workflow_format.markdown_agent_prompts.scenario.valid] Add bounded contained Markdown prompt-file loading and round-trip source metadata.
- [x] [parallel] r[n8nclone.workflow_format.markdown_agent_prompts.scenario.invalid] Add negative tests for empty, multiple, absolute, traversal, escaping symlink, wrong extension, missing, oversized, and invalid UTF-8 prompt sources.
- [x] [serial] r[n8nclone.workflow_execution.llm_operating_graph] Add a runnable role-agent operating-graph fixture with explicit fan-out, state/effect boundaries, and aggregation.
- [x] [serial] r[n8nclone.workflow_execution.llm_operating_graph.scenario.production-file-effects] Configure the default execution shell with contained local file I/O and smoke-test the full graph through a real model endpoint.
- [x] [serial] r[n8nclone.workflow_execution.llm_operating_graph.scenario.feedback-boundary] Document feedback and dynamic graph changes as between-run inputs rather than hidden intra-run cycles.
- [x] [serial] r[n8nclone.workflow_execution.rats_dependency] Adopt the newest consumable Rats source and retain evidence for any source-owner packaging repair.
- [x] [serial] r[n8nclone.workflow_execution.llm_operating_graph] Add Callee and FarmTable to README references because their designs were used.
- [x] [serial] r[n8nclone.workflow_execution.seaglass_graph_core] Run focused positive/negative tests, full repo checks, formatting, Clippy/Octet where available, and Cairn validation; record exact blockers.

## Validation evidence

- `scripts/sync-vendored-sibling.rs --self-test` passed.
- `scripts/check.rs` passed formatting, the full default-member test suite, Clippy with warnings denied, and repository spec-reference checks.
- Focused runner, workflow-format, executor, file-node, and LLM operating-graph tests passed, including local-file containment, failure propagation, and every Markdown prompt rejection case.
- Headless run `3` used `ProductionAgentAdapter` against the live `user.Ornith-1.0-35B-BF16` endpoint and completed all six nodes with zero failures; replay recorded typed Researcher text, Builder JSON, Reviewer text, Publisher text, and a 2,060-byte final report write.
- Negative replay evidence remained fail-closed: a brief-read failure cancelled every downstream role and effect, and a Builder model-output failure cancelled Reviewer, Publisher, and the report write.
- The vendored dependency boundary moved to `vendor-cargo-deps/`, which Octet excludes as dependency source; final `octet-precommit-check` passed with `834 accepted, 834 current accepted, 0 new, 0 stale`, baseline hash `b3:52d96f7cb91398acd33024ac593e3a683d336752ecd0809993f096af74ed03d8`, and no vendored Rats findings.
- The Octet baseline migration was audited by stable finding context before generation: 824 contexts were unchanged, nine changed only through expected local size/context shifts, ten prior findings were retired, and three existing shell-owned ambient-environment accesses became visible under the updated Octet revision. The production filesystem follow-up replaced only the expected executor-length and existing module-name identities while retiring the obsolete `NoopFileSystem` import/default findings. No upstream vendored findings were accepted.
- Canonical Cairn policy generation restored `gate_policy.substance`; lifecycle validation and the proposal, design, and tasks gates passed.
- `nix flake check -L` reached the repository rail but was blocked before derivation checks by missing host secret `/run/secrets/vars/nix-signing-key/key`; `scripts/check.rs`, focused dev-shell checks, and `octet-precommit-check` passed independently.

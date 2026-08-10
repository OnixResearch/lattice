## Phase 1: Runtime core and shell

- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.route] Add backward-compatible typed runtime routing with deterministic missing/ambiguous-route rejection.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.plan] Add pure bounded chat-completions planning and unsupported-mode/config rejection.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.credentials] Pass resolved node credentials through a non-serializable execution context and enforce HTTPS for bearer use.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.success] Add strict text/JSON response decoding and non-secret metadata.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.failure] Add the bounded no-redirect production HTTP shell and positive/negative transport and protocol tests.

## Phase 2: Configuration and evidence

- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.plan] Add typed Nickel example/default documentation without plaintext secrets.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.real-graph] Run the full Markdown role graph through the native adapter against a real model endpoint and record bounded evidence. Evidence: `/tmp/n8nclone-native-openai/run.json` and `run-evidence.json` record run 0 against Ornith with all 6 nodes completed, 0 failures, and a 1,223-byte contained report write.
- [x] [serial] r[n8nclone.workflow_execution.agent_nodes.openai_compatible] Run full repository, Clippy, Octet, and Cairn validation; audit any baseline identity migration, sync accepted specs, and archive the change. Evidence: `scripts/check.rs`, Clippy with warnings denied, and the 831/831 Octet baseline at `b3:34648a91f594d728477e21bf4d8e56db7bc5a8a4cdc2575dc4cbe4ac4cd1589e` passed before final Cairn sync/archive.

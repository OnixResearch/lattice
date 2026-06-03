## Phase 1: Implementation

- [x] [serial] r[n8nclone.workflow_execution.pipeline_stdin.real_port_outputs] Map shell outputs to real graph output port IDs.
- [x] [serial] r[n8nclone.workflow_execution.pipeline_stdin.stdout_to_stdin] Add a regression proving stdout reaches downstream stdin.
- [x] [serial] r[n8nclone.workflow_execution.pipeline_stdin.cache_real_ports] Keep cache helpers aligned with real output port IDs.
- [x] [serial] r[n8nclone.workflow_execution.pipeline_stdin.stdout_to_stdin.connected] r[n8nclone.workflow_execution.pipeline_stdin.real_port_outputs.completed] r[n8nclone.workflow_execution.pipeline_stdin.cache_real_ports.routable] Verify with focused executor tests, full tests, Cairn validation, then archive the change.

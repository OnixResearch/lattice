## Phase 1: Runner IR and pure core

- [x] [serial] r[n8nclone.workflow_execution.runner_core.plan_ir] Add a pure adapter from persisted `Workflow`/`rat_nodegraph` data into a stable runner IR with typed nodes, ports, edges, runtime bindings, policies, and credential refs.
- [x] [parallel] r[n8nclone.workflow_execution.runner_core.trellis_graph_admission] Add pure graph admission and ready-set planning aligned with Trellis `graph_adjacency`, `job_dag`, `job_state`, and `dependency_scheduler` semantics.
- [x] [parallel] r[n8nclone.workflow_execution.runner_core.lifecycle] Add step lifecycle transition checks with positive/negative tests for pending, ready, running, completed, failed, and cancelled states.
- [x] [parallel] r[n8nclone.workflow_validation.generic_runner_admission] Extend validation with runner-admission diagnostics for cycles, missing runtime bindings, invalid effect requirements, and terminal-state mutation attempts.

## Phase 2: Runtime registry

- [x] [serial] r[n8nclone.workflow_execution.runtime_registry.dispatch] Introduce a `NodeRuntime` registry and route built-in shell, HTTP, file, image, and agent execution through runtime dispatch.
- [x] [parallel] r[n8nclone.workflow_execution.runtime_registry.typed_output_contract] Enforce that runtime outputs are keyed by declared port ids and match declared type tags before storing values.
- [x] [parallel] r[n8nclone.workflow_execution.runtime_registry.errors] Add missing-runtime, wrong-output-port, output-type-mismatch, and runtime-failure negative coverage.
- [x] [parallel] r[n8nclone.workflow_execution.runtime_registry.command_runtime] Promote command-backed execution to a reusable runtime while preserving generic agent Pi/Clankers presets.

## Phase 3: Events, artifacts, and durability

- [x] [serial] r[n8nclone.workflow_execution.runner_events.stream] Emit a unified runner event stream for TUI, headless, scheduled, and test execution paths.
- [x] [parallel] r[n8nclone.workflow_execution.runner_events.redaction] Preserve sensitivity/redaction metadata in events and summaries.
- [x] [parallel] r[n8nclone.workflow_execution.artifact_references.model] Add typed durable/artifact references for large values using BLAKE3 content hashes and media/codec metadata.
- [x] [parallel] r[n8nclone.workflow_execution.artifact_references.trellis_identity] Align artifact/reference admission with Trellis `typed_durable_reference` and `content_addressed_code` predicates where build and license fit.
- [x] [parallel] r[n8nclone.workflow_execution.durable_runner.resume] Add deterministic checkpoint/resume/idempotency admission aligned with Trellis durable job primitives.

## Phase 4: Format compatibility and gates

- [x] [serial] r[n8nclone.workflow_format.generic_pipeline_ir.nickel] Extend typed Nickel import/export with optional runtime bindings, policies, and artifact-reference fields while preserving existing workflow meaning.
- [x] [parallel] r[n8nclone.workflow_format.generic_pipeline_ir.compatibility] Add round-trip tests proving existing examples and generic agent workflows remain compatible.
- [x] [serial] r[n8nclone.workflow_execution.runner_core.compatibility] Keep `Executor`, TUI, headless, and scheduler entrypoints as compatibility facades over the generic runner shell.
- [x] [serial] r[n8nclone.workflow_execution.generic_pipeline_runner] Run focused tests, `./scripts/check.rs`, and archive or report blockers.

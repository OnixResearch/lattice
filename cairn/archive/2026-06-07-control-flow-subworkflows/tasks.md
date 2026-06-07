## Phase 1: Pure control-flow core

- [x] [serial] r[n8nclone.workflow_execution.control_flow.condition] Add condition-node config data, typed predicate model, pure predicate parser/evaluator, and positive/negative predicate tests.
- [x] [parallel] r[n8nclone.workflow_execution.control_flow.merge] Add merge-node config data, fan-in readiness decisions, output shaping, and positive/negative merge tests.
- [x] [parallel] r[n8nclone.workflow_validation.control_flow] Add validation diagnostics for invalid predicates, missing branches, incompatible merge inputs, and unreachable branch ports.

## Phase 2: Subworkflow runtime

- [x] [serial] r[n8nclone.workflow_execution.subworkflows.invocation] Add subworkflow config, interface mapping, recursion admission, and fake repository tests.
- [x] [parallel] r[n8nclone.workflow_execution.subworkflows.events] Emit parent/child runner-event correlation and persist child run summaries without changing parent workflow state.
- [x] [parallel] r[n8nclone.workflow_validation.subworkflows] Validate missing workflow references, invalid interface mappings, recursive references, and child fatal diagnostics before side effects.

## Phase 3: Format and TUI

- [x] [serial] r[n8nclone.workflow_format.control_flow] Extend typed Nickel import/export for condition, merge, and subworkflow node configs while preserving old workflows.
- [x] [parallel] r[n8nclone.tui_ux.control_flow.add_flow] Add TUI add-node/property flows for condition, merge, and subworkflow nodes with recoverable validation failures.
- [x] [parallel] r[n8nclone.workflow_format.control_flow.compatibility] Add round-trip tests for existing examples and new control-flow examples.

## Phase 4: Gates

- [x] [serial] r[n8nclone.workflow_execution.control_flow.condition] Run focused control-flow/subworkflow tests, `./scripts/check.rs`, and archive or report blockers.

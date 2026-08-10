## Phase 1: Baseline and capability inventory

- [ ] [serial] Record focused pre-change baselines for TUI add/edit/delete, workflow validation/store save/load, workflow-format/control-flow/trigger round trips, and operator PTY behavior, including the known HTTP/File creation, orphan deletion, and condition cardinality gaps. r[lattice.tui_ux.portable_authoring_parity] r[lattice.workflow_validation.aggregate_mutation]
- [ ] [serial] Add a pure checked capability catalog/matrix covering every field in the current portable workflow schema and classify create, edit, remove/reset, validation, TUI input, store, and Nickel round-trip evidence. r[lattice.tui_ux.portable_authoring_parity.catalog]
- [ ] [parallel] Add a negative completeness fixture proving a newly added or intentionally omitted portable field fails the parity matrix instead of defaulting to display-only or preservation-only status. r[lattice.tui_ux.portable_authoring_parity.catalog.scenario.complete] r[lattice.tui_ux.portable_authoring_parity.catalog.scenario.missing]

## Phase 2: Aggregate lifecycle fixes

- [x] [serial] Implement pure node-removal planning and one workflow-level mutation that removes the graph node, incident edges, config, credentials, runner settings, and invalid node-owned references, then clears matching TUI focus/selection/edit state. r[lattice.workflow_validation.aggregate_mutation] r[lattice.tui_ux.aggregate_node_deletion]
- [ ] [parallel] Add positive deletion tests for every built-in node/config kind and negative missing-node, stale-map, incident-edge, credential, runner, selected/editing, save/load, validation, and Nickel export/import cases. r[lattice.workflow_validation.aggregate_mutation.scenario.delete] r[lattice.workflow_validation.aggregate_mutation.scenario.reject_orphan]
- [x] [serial] Implement pure label-based condition branch/output reconciliation with checked id allocation, preserving matching port ids/edges, creating new ports, removing deleted ports/incident edges, treating rename as remove-plus-add, and applying only after full validation. r[lattice.tui_ux.condition_topology_reconciliation]
- [ ] [parallel] Add positive and negative add/remove/rename/reorder/unchanged/duplicate/empty/malformed/overflow/cancel tests, including unchanged-edge preservation and removed-edge cleanup. r[lattice.tui_ux.condition_topology_reconciliation.scenario.change] r[lattice.tui_ux.condition_topology_reconciliation.scenario.reject]

## Phase 3: Complete built-in node authoring

- [x] [serial] Extend guided creation with typed HTTP and file drafts and preserve existing shell, image, Pi/Clankers agent, condition, merge, and subworkflow creation/cancellation behavior. r[lattice.tui_ux.portable_authoring_parity.node_kinds]
- [ ] [serial] Extend node-kind provider metadata and pure typed parsers/apply functions for every current Shell, HTTP, File, ImageInput, Agent, Condition, Merge, and Subworkflow configuration field. r[lattice.tui_ux.portable_authoring_parity.node_fields]
- [ ] [parallel] Add positive and negative TUI input tests for every node field, enum, optional value, repeated/map row, numeric bound, malformed value, cancellation, and prior-value preservation path. r[lattice.tui_ux.portable_authoring_parity.node_fields.scenario.edit] r[lattice.tui_ux.portable_authoring_parity.node_fields.scenario.invalid]

## Phase 4: Workflow, graph, runner, credential, and trigger authoring

- [ ] [serial] Add catalog-driven typed drafts and pure apply functions for workflow metadata, node labels/positions, typed port declarations, compatible edges, and per-node credential reference maps. r[lattice.tui_ux.portable_authoring_parity.workflow_graph]
- [ ] [serial] Add catalog-driven typed drafts and pure apply functions for runtime kind/key, ability/effect requirements, cache, retry/backoff, artifact-reference policy, and every current named limit. r[lattice.tui_ux.portable_authoring_parity.runner]
- [ ] [serial] Add typed manual/schedule/webhook trigger browse/add/edit/remove drafts with existing trigger validation, secret-reference safety, cancellation, and no listener/scheduler effect during authoring. r[lattice.tui_ux.portable_authoring_parity.triggers]
- [ ] [parallel] Add positive and negative tests for metadata/ports/edges/credentials/runners/triggers, including duplicate/collision/overflow, incompatible types, malformed refs, unsupported policy, invalid schedule/webhook fields, cancellation, and no side effects. r[lattice.tui_ux.portable_authoring_parity.workflow_graph] r[lattice.tui_ux.portable_authoring_parity.runner] r[lattice.tui_ux.portable_authoring_parity.triggers]

## Phase 5: Cross-surface parity and completion

- [ ] [serial] Add exhaustive TUI-to-`Workflow`-to-store-to-Nickel-to-`Workflow` fixtures containing all eight node kinds, every capability row, typed ports/edges, credentials, runner policies, control-flow mappings, and all trigger kinds. r[lattice.tui_ux.portable_authoring_parity.round_trip]
- [ ] [parallel] Add negative cross-surface fixtures proving invalid drafts, deletion orphans, condition topology mismatch, unsupported fields, stale selections, save failure, and malformed Nickel leave prior workflow/store bytes unchanged. r[lattice.tui_ux.portable_authoring_parity.round_trip.scenario.invalid]
- [ ] [serial] Update command palette/help/property rendering and operator documentation so every authoring family is discoverable and constrained terminal layouts remain recoverable. r[lattice.tui_ux.portable_authoring_parity.discoverability]
- [ ] [serial] Run focused unit/integration/TUI/PTTY tests, `scripts/check.rs`, `octet-precommit-check`, Cairn validation and proposal/design/tasks gates, and bounded `nix flake check -L --option secret-key-files ''`; preserve exact evidence or blockers before sync/archive. r[lattice.tui_ux.portable_authoring_parity.verification]

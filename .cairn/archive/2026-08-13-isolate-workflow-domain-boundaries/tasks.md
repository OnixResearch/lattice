# Tasks: Isolate workflow domain boundaries

## Phase 1: Inventory and baselines

- [x] [serial] I1 Inventory every `rat_nodegraph` type in workflow, validation, execution, replay, history, expression, persistence, and non-TUI public signatures. r[lattice.workflow_validation.domain_boundary.inventory]
- [x] [serial] V1 Record focused workflow, Nickel, redb, replay, TUI, and Octet baselines, including the two accepted `ambient_clock` finding identities. r[lattice.workflow_validation.domain_boundary.compatibility]
- [x] [serial] I2 Classify each graph decision as Lattice product policy, Choregraph-owned meaning, Subwayrat UI state, or boundary conversion. r[lattice.workflow_validation.domain_boundary.ownership]

## Phase 2: Application types and explicit time

- [x] [serial] I3 Add Lattice-owned workflow identifiers, port declarations, graph-edit inputs, mutation plans, and domain errors. r[lattice.workflow_validation.domain_boundary.application_model]
- [x] [serial] I4 Change workflow creation and metadata mutation to require an explicit validated UTC timestamp. r[lattice.workflow_validation.domain_boundary.time]
- [x] [parallel] V2 Add positive and negative core tests for fixed time, invalid time, unknown nodes, incompatible ports, stale edits, and aggregate invariants. r[lattice.workflow_validation.domain_boundary.time] r[lattice.workflow_validation.domain_boundary.validation]

## Phase 3: Boundary adapters

- [x] [serial] I5 Add a pure Choregraph adapter that maps Lattice values into the pinned graph contract and maps failures into Lattice adapter errors. r[lattice.workflow_validation.domain_boundary.choregraph]
- [x] [serial] I6 Add Subwayrat TUI adapters for gestures, snapshots, render values, and selection state without exposing UI types to the workflow core. r[lattice.workflow_validation.domain_boundary.subwayrat]
- [x] [parallel] I7 Add explicit Nickel, JSON, and redb conversion adapters for supported workflow versions. r[lattice.workflow_validation.domain_boundary.compatibility]
- [x] [parallel] V3 Add accepted and rejected Choregraph, Subwayrat, Nickel, JSON, and redb adapter fixtures. r[lattice.workflow_validation.domain_boundary.validation]

## Phase 4: Call-site migration

- [x] [serial] I8 Migrate workflow construction, validation, expressions, execution, replay, run history, and persistence to Lattice-owned values. r[lattice.workflow_validation.domain_boundary.application_model]
- [x] [serial] I9 Move current-time acquisition to CLI, headless, scheduler, webhook, import, and TUI shells. r[lattice.workflow_validation.domain_boundary.time]
- [x] [serial] I10 Keep `rat_nodegraph` model and view use inside declared TUI adapter scopes. r[lattice.workflow_validation.domain_boundary.subwayrat]
- [x] [parallel] V4 Prove supported persisted workflows and replay fixtures remain compatible without rerunning effects. r[lattice.workflow_validation.domain_boundary.compatibility]

## Phase 5: Enforcement and closeout

- [x] [serial] I11 Add source and Octet guards that reject ambient clock reads and Subwayrat type leakage in declared workflow-core scopes. r[lattice.workflow_validation.domain_boundary.enforcement]
- [x] [serial] I12 Document the Lattice, Choregraph, Subwayrat, persistence, and time ownership boundaries. r[lattice.workflow_validation.domain_boundary.docs]
- [x] [parallel] V5 Run focused tests, `scripts/check.rs`, `octet-precommit-check`, Cairn validation and gates, operator E2E, and relevant Nix checks. r[lattice.workflow_validation.domain_boundary.final_checks]

## Verification Coverage

- `Scenario: Ownership inventory classifies each selected graph value` -> I1, I2
- `Scenario: Explicit timestamp creates deterministic metadata` -> I4, V2
- `Scenario: Core attempts an ambient clock read` -> I11, V5
- `Scenario: Lattice-owned mutation is accepted` -> I3, V2
- `Scenario: UI type enters a workflow-core signature` -> I6, I10, I11
- `Scenario: Choregraph rejects the mapped graph` -> I5, V3
- `Scenario: Supported persisted workflow remains readable` -> I7, V4
- `Scenario: Malformed boundary record does not mutate workflow state` -> I7, V3
- `Scenario: Shell supplies one timestamp per mutation` -> I9, V2
- `Scenario: Boundary claims remain scoped` -> I12, V5

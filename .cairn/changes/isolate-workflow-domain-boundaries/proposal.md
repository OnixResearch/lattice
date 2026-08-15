# Proposal: Isolate workflow domain boundaries

## Why

`src/engine/workflow.rs` owns workflow metadata and aggregate mutations. Its constructors read `chrono::Utc::now()` directly.

The same module exposes `rat_nodegraph` graph, node, port, and error types. `rat-nodegraph` is a Subwayrat UI component dependency.

These dependencies make time an ambient decision input. They also let a UI adapter model define Lattice domain signatures and persisted state.

The Octet baseline records both clock reads as `ambient_clock` findings. Direct source inspection confirms that these reads occur in `WorkflowMeta::new` and `Workflow::touch`.

## What Changes

- Add Lattice-owned workflow graph identifiers, port declarations, mutation inputs, plans, and errors.
- Keep Choregraph responsible for graph admission, lifecycle, replay, and machine decisions.
- Convert between Lattice workflow values, Choregraph inputs, portable Nickel records, redb records, and Subwayrat UI values at explicit boundaries.
- Require workflow creation and metadata mutation to receive an explicit timestamp value.
- Let CLI and TUI shells obtain current time before they call the workflow core.
- Remove `rat_nodegraph` types from non-UI workflow, validation, execution, replay, history, expression, and persistence signatures.
- Preserve supported Nickel, JSON, redb, CLI, TUI, and replay behavior.
- Add source and Octet guards for ambient time and Subwayrat type leakage in the workflow core.

## Impact

- **Core**: `src/engine/workflow.rs`, workflow validation, expressions, replay, and run-history projections.
- **Shells**: CLI, headless, scheduler, webhook, and TUI timestamp acquisition.
- **Adapters**: Subwayrat UI conversion, Choregraph lowering, Nickel conversion, and redb conversion.
- **Compatibility**: Existing workflow files and supported persisted records remain readable.
- **Testing**: Positive and negative core, adapter, persistence, replay, and TUI tests.

## Non-goals

- Do not move graph lifecycle or machine semantics from Choregraph into Lattice.
- Do not fork the portable workflow schema without a separate versioned change.
- Do not remove Subwayrat from the TUI.
- Do not add ports around pure internal helpers.
- Do not claim workflow correctness, provider correctness, effect safety, or release readiness.

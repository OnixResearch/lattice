# Evidence: Ownership inventory for the first extraction cohort

Phase 1 inventory for `r[lattice.repository_validation.fcis_crate_boundary]`
and `r[lattice.repository_validation.fcis_compatibility]`.

## Candidate first-cohort modules and their coupling

| Domain | Coupling (crate-local refs) | Clock reads | Notes |
|---|---|---|---|
| `src/engine/workflow.rs` | `rat_nodegraph::Graph`, `crate::engine::value::TypeTag`, node-config types | only shell conveniences (`new`/`touch`) that delegate to `_at` | Temporal facts already made explicit in Change A |
| `src/replay.rs` | `crate::engine::`, `crate::engine::executor::`, `crate::engine::runner::` | none | Deterministic timeline/cursor/bundle decisions |
| `src/scheduler/admission.rs` + `model.rs` | `crate::engine::`, `crate::engine::value::` | none | Due-run, occurrence admission, settlement, retry, abandonment, recovery |
| `src/orchestration/core.rs` | heavy (store, trigger, shells, effects) | none | DEFERRED for extraction after active pilots |

## Active-change ownership of the cohort

Cross-referenced every active change in `cairn/changes/` against the cohort
source and symbols:

- Only `complete-tui-nickel-authoring-parity` references `workflow.rs`,
  `WorkflowMeta`, or `Workflow::`; it is a TUI-authoring change and is the
  single owner to reconcile for the workflow module. No orchestration or
  scheduler pilot owns `workflow.rs`.
- `replay.rs` and `scheduler/admission*` are referenced by no active change
  that mutates their deterministic decisions.
- `src/orchestration/core.rs` is entangled with several active changes
  (`type-orchestration-identities`, `persist-versioned-operation-data`,
  `authorize-choregraph-effect-intents`, `pilot-radicle-effect-attempt-core`,
  `pilot-radicle-change-admission-core`), so orchestration state-transition
  extraction is DEFERRED until those reconcile, per the design.

## Conclusion

The first-cohort extraction boundary (workflow metadata/mutation, replay
decisions, scheduler-admission decisions) is not occupied by a conflicting
active pilot. A `lattice-core` crate hosting these three domains can be
introduced without colliding with in-flight orchestration/scheduler pilots.
The orchestration state-transition portion of the cohort remains gated on
active ownership and is left for a later staged move.

This is a source-shape and ownership inventory only. It does not claim
correctness, crate-boundary enforcement, or release readiness.

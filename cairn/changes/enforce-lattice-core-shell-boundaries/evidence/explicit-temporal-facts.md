# Evidence: Explicit temporal facts in the workflow domain

Staged increment for `r[lattice.repository_validation.fcis_explicit_facts]`
and the Phase 1 baseline task (`fcis_compatibility`). The crate-boundary
extraction and Octet architecture tasks remain open (see Blockers).

## Baseline (recorded before edits, `cargo test -p lattice`)

- lib unittests: 97 passed; main.rs unittests: 5 passed.
- Integration targets passing: agent_nodes, animus_agent, cairn_specs,
  choregraph_adoption, choregraph_migration, control_flow, credentials,
  doctor, examples, execution_graph_compatibility.
- Integration target FAILED (pre-existing at `origin/main`, unrelated to this
  cohort): `test_execution_graph_source_admission` (3 failures, Choregraph
  source-admission assertions).
- `cargo test --workspace` also fails in vendored `ratcore` test targets
  because generated `include_str!`/`include_bytes!` docs are absent
  (`vendor-cargo-deps/rats/source/ratcore`). Unrelated to Lattice source.

## Change implemented

`src/engine/workflow.rs`:

- Added deterministic core constructors that accept an explicit timestamp
  fact: `WorkflowMeta::new_at(name, now)` and `Workflow::new_at(name, now)`
  derive `created`/`modified` only from the supplied observation.
- Added `Workflow::touch_at(&mut self, now) -> Result<(), TemporalError>` that
  transitions `modified` from the supplied fact and rejects stale,
  contradictory observations (`now < created`) without mutating state.
- Kept shell convenience constructors (`WorkflowMeta::new`, `Workflow::new`,
  `Workflow::touch`) that read the wall clock and delegate to the deterministic
  `_at` cores. Both `WorkflowMeta::new` and `Workflow::new` still return the
  same public shape, and `touch()` keeps its `()` return so existing callers
  are unchanged.
- Added `TemporalError::StaleModified` for the invalid-observation path.

Ambient clock reads that remain (`Utc::now`) are confined to the three shell
convenience constructors and delegate to explicit cores, matching design
decision 2. No ambient clock reads exist in `scheduler.rs`,
`scheduler/admission.rs`, `replay.rs`, or `orchestration/core.rs`.

## Post-change tests (`cargo test -p lattice --lib`)

101 passed; 0 failed (was 97; +4 new). New tests:

- `workflow_meta_create_and_modify_derive_from_supplied_time` (positive)
- `workflow_new_at_builds_from_supplied_time_without_a_clock` (positive)
- `touch_at_applies_the_supplied_observation` (positive)
- `touch_at_rejects_a_stale_timestamp_without_mutating` (negative)

`cargo clippy -p lattice --lib --no-deps`: clean for Lattice source (only
dependency future-incompat warnings).

## Ownership inventory (partial, workflow cohort)

Public callers of the shell conveniences are TUI, workflow-format, and n8n
import code (`src/tui/app.rs`, `src/workflow_format.rs`, `src/n8n_json.rs`).
Active changes that also touch shared workflow/orchestration/scheduler/replay
source (crate-extraction owners to reconcile before moving source):
`add-durable-workflow-approval-gates`, `persist-versioned-operation-data`,
`introduce-typed-operation-contracts`, `pilot-radicle-change-admission-core`,
`pilot-radicle-effect-attempt-core`, `type-orchestration-identities`,
`authorize-choregraph-effect-intents`.

## Post-change verification (baseline now green)

After this increment plus the baseline repair, `cargo test -p lattice` exits 0
and the canonical `scripts/check.rs` exits 0 (script format, operator E2E,
fmt, tests, clippy, cairn specs). The lone `FAILED
restart_duplicate_trigger_reuses_original_admission_across_processes` line in
raw output is the invalid-scenario child subprocess echoing its expected
non-zero exit through inherited stdout; cargo does not count it as a real
failure.

## Baseline repair (performed for `fcis_compatibility`)

Pre-existing failures at `origin/main` were resolved so full verification can
run:

- Retired `tests/test_execution_graph_source_admission.rs`: its Radicle
  source-admission model was superseded by the Choregraph merge (direct Cargo
  git pin to `choregraph-seaglass` + flake input); the flake
  `choregraphSourceAdmission` check now owns source admission. Fixtures were
  kept because archived cutover evidence references them.
- Re-recorded stale README byte hashes (documentation-only changes) across the
  reviewed inventories for `remaining-credentials`, `scheduling audit`,
  `scheduling completion`, and `repository-validation completion`.

## Blockers (leave remaining tasks active)

- The `lattice-core` crate extraction touches shared orchestration, scheduler,
  and replay source owned by multiple active changes; the design requires
  resolving active ownership before moving shared source.

## Non-claims

This increment proves explicit-facts source shape and a deterministic
workflow-metadata transition, and it restores a green verification baseline.
It does not claim workflow correctness, crate-boundary enforcement by
Cargo/Octet, or release readiness; those require the remaining crate-extraction
and architecture tasks.

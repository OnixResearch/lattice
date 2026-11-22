# Run bounded repair from the standalone CLI through operator-authorized OMP and catalog ports

Tracking: [Lattice #6](https://github.com/OnixResearch/lattice/issues/6), the item "an OMP worker port so the standalone CLI can run repair", which needs an owner decision because it gives the CLI new effect authority. Owner-authorized round 2026-11-21: the CLI may run repair only with an operator check catalog and an `effect_authority` profile granting the exact effects.

## Why

Bounded same-attempt repair (`bound-same-attempt-repair`) runs only through worker, check and clock ports an embedder supplies. The standalone `lattice implementation-attempt run` supplies none and refuses a repair request as `repair_ports_unavailable`, so an operator cannot run repair with the pinned Pi/OMP worker without writing an embedder.

## What Changes

- `run` of a request with a repair policy and no embedder ports uses operator ports: an OMP worker port that launches one pinned OMP step per iteration through the single-shot attempt's workspace boundary and OMP launcher (`candidate_worker::omp::run`), and a check port that runs each declared check of the operator's `--repair-checks` catalog as one bounded process through the candidate worker's generic bubblewrap route on the loop's read-only copy, with a cleared environment, a private network, no broker or credential and bounded output. The loop, its step intents, records, fence, budgets and lineage are unchanged. r[lattice.implementation_attempt.operator_repair] r[lattice.implementation_attempt.operator_repair.scenario.repaired]
- The ports are admitted under the per-work lock before the fence record: a missing catalog keeps `repair_ports_unavailable`, an inadmissible catalog is `repair_ports_unavailable` with `check_catalog`, and a profile from the application configuration's `effect_authority` that does not grant the agent tuple `lattice://agent/1` and the process tuple `lattice://process/2` bound to the request's worker profile revision is the new `repair_authority_denied` with `authority_refusal`, both `dispatched:false` with only the closure marker written. The grant is rechecked before every step's intent; a withdrawn one ends the loop as `authority_withdrawn` (`effect_authority`). `prepare` prints the profile requirements as `repair_authority`. r[lattice.implementation_attempt.operator_repair.scenario.authority] r[lattice.implementation_attempt.operator_repair.scenario.checks]
- Launcher observations map to the loop's outcome classes: observed exits and host stops are `exited`, cancellation `cancelled`, and everything else `unknown`; `candidate_worker::omp::stop` reads a failed launch's stop from the launcher's named blocker texts. r[lattice.implementation_attempt.operator_repair.scenario.outcomes] r[lattice.implementation_attempt.operator_repair.scenario.recovery]
- Requests without a repair policy keep their bytes and behaviour and read no configuration. r[lattice.implementation_attempt.operator_repair.scenario.absent]

## Impact

- **Pure core:** new `src/implementation_attempt/repair/step/operator.rs`, `src/implementation_attempt/repair/checks/catalog.rs` and `src/candidate_worker/omp/stop.rs` with tests; `repair::state::Authority::EffectAuthority`; `engine::effect::standalone::core::{Subject, check_subject}`.
- **Shell:** new `cli/repair/{admission,operator}.rs`, `cli/repair/operator/{worker,checks}.rs`; `cli.rs` (`run --repair-checks`, `run_command(args, config)`), `cli/artifacts.rs` (`prepare`, moved from `cli.rs`, prints `repair_authority`), `cli/execution.rs`, `cli/execution/admission.rs`, `cli/session/launch.rs` (the worker request builder and credentials moved here), `cli/repair/{mod,records}.rs`, `cli/repair/steps{,/authority}.rs`, `cli/refusal{,/deferred}.rs`, `engine/effect/standalone{,/shell}.rs`, `candidate_worker/{execution,completion,omp}.rs`, `candidate_worker/broker/admission.rs`, `main.rs`.
- **Tests:** `src/implementation_attempt/cli/tests/repair/operator.rs` and its harness, `repair/step/operator/tests.rs`, `repair/checks/catalog/tests.rs`, `candidate_worker/omp/stop.rs`, `engine/effect/standalone/core/tests.rs`.
- **Docs:** `docs/candidate-worker.md`, changelog.
- **Compatibility:** `run_command` takes the configuration path; the refusal schema gains `repair_authority_denied`; repair records gain the `effect_authority` cause; `candidate_worker::Outcome` is unchanged.

## Dependencies

- Local: `bound-same-attempt-repair`, `export-repaired-candidates`, `standalone-effect-authority`.

## Out of Scope

- A live OMP, model, sandbox or check-executor qualification; Kiln changes; served model routes in repair.

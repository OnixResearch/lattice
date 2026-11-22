## Phase 1: Pure core

- [x] [serial] 1.1 Add the operator grant requirements, step prompt, rounded deadline and launcher outcome mapping (`repair::step::operator`), the launcher's failed-stop classifier (`candidate_worker::omp::stop`), and the exact-subject profile check (`standalone::core::check_subject`). Tests: `src/implementation_attempt/repair/step/operator/tests.rs`, `src/candidate_worker/omp/stop.rs`, `src/engine/effect/standalone/core/tests.rs` (`a_subject_is_granted_only_its_exact_identities_and_tuple_before_expiry`). Evidence: `evidence/verification.json` (`tasks.1.1`). r[lattice.implementation_attempt.operator_repair.scenario.outcomes] r[lattice.implementation_attempt.operator_repair.scenario.authority]
- [x] [serial] 1.2 Add the bounded, pinned check catalog with its configuration and executor identities and check outcome mapping (`repair::checks::catalog`). Tests: `src/implementation_attempt/repair/checks/catalog/tests.rs`. Evidence: `evidence/verification.json` (`tasks.1.2`). r[lattice.implementation_attempt.operator_repair.scenario.checks]

## Phase 2: Shell

- [x] [serial] 2.1 Admit operator ports before the fence record with `repair_ports_unavailable` (`check_catalog`) and `repair_authority_denied` refusals, recheck the grant before every step intent (`authority_withdrawn`, `effect_authority`), and print `repair_authority` from `prepare` (`cli/repair/{admission,operator}.rs`, `cli/repair/steps/authority.rs`, `cli/refusal{,/deferred}.rs`, `cli/artifacts.rs`, `cli.rs`, `main.rs`). Tests: `src/implementation_attempt/cli/tests/repair/operator.rs`. Evidence: `evidence/verification.json` (`tasks.2.1`). r[lattice.implementation_attempt.operator_repair.scenario.authority] r[lattice.implementation_attempt.operator_repair.scenario.absent]
- [x] [serial] 2.2 Run each step through the pinned OMP launcher and each check through the catalog's sandboxed process, retaining `omp-step-N.{jsonl,json}` (`cli/repair/operator/{worker,checks}.rs`). Tests: `src/implementation_attempt/cli/tests/repair/operator.rs`. Evidence: `evidence/verification.json` (`tasks.2.2`). r[lattice.implementation_attempt.operator_repair] r[lattice.implementation_attempt.operator_repair.scenario.repaired] r[lattice.implementation_attempt.operator_repair.scenario.outcomes] r[lattice.implementation_attempt.operator_repair.scenario.recovery]

## Phase 3: Documentation and verification

- [x] [serial] 3.1 Document the operator ports, catalog, grant, refusals and outcome mapping in `docs/candidate-worker.md` and the changelog. Evidence: `evidence/verification.json` (`tasks.3.1`). r[lattice.implementation_attempt.operator_repair]
- [x] [serial] 3.2 Run the focused tests, the library suite, clippy, fmt, `octet-precommit-check`, `cairn validate`, this change's gates and gitleaks on the changed paths, and record the results. Evidence: `evidence/verification.json` (`checks`). r[lattice.implementation_attempt.operator_repair]

Only checked tasks are complete.

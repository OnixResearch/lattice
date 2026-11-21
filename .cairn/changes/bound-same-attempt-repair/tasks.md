## Phase 1: Pure core

- [x] [serial] 1.1 Add `src/implementation_attempt/repair.rs` with the admitted policy, its bounds against the envelope budget and broker ceiling, its identity, the repair-capable profile revision and the frozen `StepBinding` with `changed_work`/`new_semantic_attempt` refusals, and `repair/revision.rs` with the candidate revision identity. Tests: `src/implementation_attempt/repair/tests.rs`. r[lattice.implementation_attempt.repair] r[lattice.implementation_attempt.repair.scenario.policy] r[lattice.implementation_attempt.repair.scenario.kiln_owned]
- [x] [serial] 1.2 Add `repair/checks.rs` (invocations and evidence classification with defect precedence), `repair/step.rs` (step requests, reports and usage) and `repair/state.rs` (the bounded loop, conservative charges and the nine dispositions). Tests: `src/implementation_attempt/repair/tests.rs`. r[lattice.implementation_attempt.repair.scenario.dispositions] r[lattice.implementation_attempt.repair.scenario.bypass] r[lattice.implementation_attempt.lineage.scenario.usage]
- [x] [serial] 1.3 Add `src/implementation_attempt/lineage.rs`, the `lattice.kiln-budget-lineage.v1` consumer of Kiln `339116f6`: validation, identity, the spawn gate and subtractive child admission. Tests: `src/implementation_attempt/lineage/tests.rs`. r[lattice.implementation_attempt.lineage] r[lattice.implementation_attempt.lineage.scenario.exhausted] r[lattice.implementation_attempt.lineage.scenario.shared]

## Phase 2: Shell

- [x] [serial] 2.1 Add the optional `repair` and `lineage` request fields, frame the policy into the profile revision, validate both with the request, and add `cli::repair::{WorkerPort, CheckPort, Ports}` as the optional `Ports::repair`. Tests: `src/implementation_attempt/cli/tests/repair.rs` (`absent_policy_and_lineage_keep_request_bytes_and_revision_and_a_policy_moves_the_revision`). r[lattice.implementation_attempt.repair.scenario.absent] r[lattice.implementation_attempt.repair.scenario.policy]
- [x] [serial] 2.2 Gate the lineage and repair ports before any write with typed `lineage_blocked`, `lineage_exhausted` and `repair_ports_unavailable` refusals; drive the loop through the ports with write-once step intents, step records and disposition; persist the attempt usage observation before the disposition; recover only from the retained disposition; report the single-shot lineage draw. Tests: `src/implementation_attempt/cli/tests/repair.rs`. r[lattice.implementation_attempt.repair] r[lattice.implementation_attempt.repair.scenario.dispositions] r[lattice.implementation_attempt.repair.scenario.bypass] r[lattice.implementation_attempt.repair.scenario.kiln_owned] r[lattice.implementation_attempt.lineage.scenario.exhausted] r[lattice.implementation_attempt.lineage.scenario.usage]

## Phase 3: Documentation and evidence

- [x] [serial] 3.1 Document the repair policy, dispositions, ports, records, the lineage field mapping against Kiln `339116f6` and the non-claims in `docs/candidate-worker.md`; note in `docs/worker-inventory.md` that lineage and repair stops are refusals or dispositions, not wait reasons; add the changelog entry. r[lattice.implementation_attempt.lineage]

## Phase 4: Verification

- [ ] [serial] 4.1 Run the focused tests, `cargo test -p lattice --lib -- implementation_attempt`, fmt, `scripts/check.rs`, `octet-precommit-check` and the Cairn validation and gates, and record the results in `evidence/fixture-qualification.json`. r[lattice.implementation_attempt.repair]

Only checked tasks are complete.

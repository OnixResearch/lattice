# Evidence: add-typed-policy-operation

Recorded 2026-11-19 in a local clone of Lattice `661499d282c2e79ebefbd0f9b35beb5f8d7795cd`. Producer: Animus `1c370d5029a2667051c46bdfa1919e78e65e2f4b` (local, unpublished candidate; archive BLAKE3 `ff16f0903348db1d5164d055b928d2f1f829ca1dffb6f1624e6d26779b11f2ab`).

## Lifecycle

- Proposal, design and tasks gates passed before implementation (`evidence/pre-implementation-gates.json`). The design was amended after the gate only to name the implemented paths and the per-branch Merge gate; the outcome rubric, cases and thresholds T1–T5 were not changed.

## Measurement (`evidence/measurement.json`)

Artifact `b3:762b41761c35533d1533415c114a6e82ecc5e1e0d861eda5be1019eaec3f9ae7`, model `b3:c5830780510e2cb0c40a3d7170822fe4779288b10524c29eb37d55c606ed7907`, hand-authored fixture tree. Held-out families `search-r1`, `auth-r2`, `reports-r1` (18 cases); 5 repetitions per case and arm on a shared, heavily loaded host.

| Held-out | Deterministic rule | Fast policy |
| --- | --- | --- |
| Whole-workflow success | 12/18 (0.667) | 11/18 (0.611) |
| Coverage (accepted, no fallback) | n/a | 13/18 (0.722) |
| Invalid-output rate | 0 | 1/18 (0.056; projection out of range at 12 000 ms) |
| Fallback rate | 0 | 5/18 (0.278; 4 abstentions, 1 invalid) |
| Median of case-median workflow latency | 63 331 µs | 64 654 µs (ratio 1.021) |
| Policy stage (projection + inference + validation), median / p95 | n/a | 5 µs / 8 µs |
| Runtime operation units (inference / normalization) | 0 / 0 | 55 / 68 |

The authoring family `payments-r1` scored 5/6 for both arms. The present reasoning-model arm is **not applicable**: the existing `examples/workflows/log-triage.ncl` contains only shell nodes (source, `grep` filter, `awk` render); no Agent or reasoning-model node makes the routing decision today, and the routing step exists only as the deterministic Condition the harness adds. No result is reported for it.

Thresholds: T1 safety pass, T2 benefit fail, T3 coverage fail, T4 validity fail, T5 cost pass. Verdict: **no-improvement-established**. This is retained as the result; no improvement claim is made.

## Checks run in the clone

- `cargo test --locked` for `test_typed_policy` (15/15) and regressions `test_control_flow`, `test_examples`, `test_run_history_details`, `test_run_inspection`, `test_guarded_agent_templates`, `test_animus_agent`, `test_ready_frontier_dispatch`, `test_headless`, `test_store`: all passed.
- `cargo clippy --workspace --exclude lattice-repository-validation-fixture --all-features --all-targets --no-deps --locked -- -D warnings`: passed.
- `cargo fmt --check` for the six repository packages: passed.
- Octet: 0 new and 0 resolved semantic findings for the root and `-p lattice-core` scopes against the committed baselines; baselines were not rewritten.
- Cairn `validate` and the proposal/design/tasks gates: passed.

## Approval suspension and resume (follow-up, 2026-11-19)

The first increment persisted decisions only for headless runs that completed normally. Decisions are now persisted and replayed across durable approval gates (task 2.4): `persist_suspension` attaches the decisions made before the gate with no workflow outcome; `resume_with_typed_policy` revalidates every stored decision with `ledger::revalidate` before the resume is claimed, carries compatible ones with zero adapter calls and persists decisions made after the resume; a changed profile or artifact, a disabled profile or a resume without a bound profile refuses with `PolicyDecisionStale` and leaves the stored record and the approval request unchanged.

- `cargo test --locked --test test_typed_policy_resume`: 3/3 passed (decision before suspension survives resume with zero adapter calls; decision after resume is persisted; stale profile, artifact, disabled profile and unbound resume are refused with the record unchanged, then the current profile resumes).
- Regressions: `test_typed_policy` 15/15, `test_workflow_approval` 14/14 (1 ignored crash-child helper), `tui_aggregate_lifecycle` 8/8, `test_control_flow` 10/10, `test_headless` 11/11, `test_store` 34/34, `test_run_history_details` 9/9, and the library's `workflow::` unit tests 31/31.
- Clippy (command above) and `cargo fmt --check` for the six repository packages: passed.
- Octet `check` for the root and `-p lattice-core` against the committed baselines: 0 new and 0 resolved semantic findings (root 1052/1052 unique, core 25/25); baselines were not rewritten.
- Cairn `validate` and the proposal/design/tasks gates: passed.

## Non-claims

The fixture artifact and rubric were authored by this change. Synthetic fixtures are not live qualification. No production readiness, default adoption, live reasoning-model comparison or model-quality claim is made.

# Design: One bounded typed policy operation on a declared Condition branch

## Context

Lattice's DAG executor already evaluates `Condition` nodes: each declared branch label is an output port, and only matched branch ports receive the input value. Downstream nodes are admitted by the ready-frontier planner and, in product runs, by Choregraph effect authority. Completed node outputs are cached by an input-hash key, and headless runs persist one `RunNodeDetail` per node in the `run_details` table. The Animus leaf agent (`src/animus_agent`) runs a fixed, identity-checked Steel program with a two-decision limit and rejects sessions.

Animus #15 (`crates/animus-frozen-policy`, candidate revision `1c370d5029a2667051c46bdfa1919e78e65e2f4b`) adds a pure `no_std` leaf: `admit_policy(bytes, identity)`, `decide(policy, request)` over integer features and declared eligible actions, and an identity-bound `animus.frozen-policy-receipt.v1` with `verify_receipt` for replay without inference.

## Success Contract

With no enabled profile, every workflow runs exactly as before. With one enabled profile, one declared Condition node of one workflow asks the frozen policy to choose one of its declared branch labels; Lattice validates the receipt independently, resolves a typed outcome through the declared fallback, routes only through the existing branch port, records the decision durably and can replay it without any adapter work.

## Decisions

### 1. The first task is log-triage incident routing

The existing `examples/workflows/log-triage.ncl` fixture (source → `grep -E 'WARN|ERROR'` filter → render) is loaded unchanged and extended in memory by the test and measurement harness with: a feature node (`awk`) that emits `{"features": {...}, "rule": {"<label>": true} }`, one Condition node `route incident` with declared branches `page`, `ticket`, `ignore` (predicates `contains:"rule":{"<label>":true}`, the existing deterministic path), and per branch one required-input Merge gate followed by one continuation shell node. Implementation note (amended after the design gate, thresholds unchanged): a Condition branch port that emits nothing does not by itself skip its consumers in Lattice's runner; the existing required-input Merge gate is the construct that skips them, so both arms use it. The checked-in fixture file is not modified.

Features, in declared order, with the ranges the profile and artifact share:

| Index | Feature | JSON pointer | Range |
| --- | --- | --- | --- |
| 0 | `error_count` | `/features/error_count` | 0–50 |
| 1 | `warn_count` | `/features/warn_count` | 0–50 |
| 2 | `max_latency_ms` | `/features/max_latency_ms` | 0–10000 |
| 3 | `critical_errors` (ERROR on `/api/payments` or `/api/auth`) | `/features/critical_errors` | 0–50 |

A value outside its range is refused by projection, never clamped.

### 2. Profile shape: `lattice.typed-policy-profile.v1`

A JSON document with `deny_unknown_fields`: `schema`, `id`, `version`, `enabled`, `shape` (`select` only), `stateful` (must be `false`), `binding` (`workflow`, `node_label`), `input` (`schema`, `max_bytes`, `features[]` of `{name, pointer, min, max}`), `labels` (1–16 unique, `[A-Za-z0-9._:-]`, ≤ 64 bytes), `result` (`schema`, `max_receipt_bytes` ≤ 16 KiB, `retain_receipt`), `limits` (`max_operations` 1–10⁶, `deadline_millis` 1–60 000), `cohort` (`runtime_crate`, `runtime_version`, `artifact_api`, `artifact`, `model`, `policy_kind`, `feature_schema`, `action_schema`) and `fallback` (`{"kind": "label", "label"}`, `{"kind": "declared_predicates"}` or `{"kind": "terminal"}`). Its identity is `b3:` BLAKE3 over a domain tag and its canonical serde encoding. Lattice never forwards the fallback to Animus; Lattice owns fallback.

### 3. Pure core / shell split

- `src/workflow/decision.rs` and `src/workflow/decision/{admission,checks,binding,validation,ledger}.rs` (pure; registered in the domain-boundary guard): profile admission, binding to a Condition node, projection, candidate set, invocation identity, receipt validation, outcome classification, fallback resolution, decision record and replay. It may call `animus_frozen_policy::verify_receipt` (a pure integrity check) but never `decide` or `admit_policy`.
- `src/animus_agent/frozen.rs` (shell): `LeafAdapter` holds host-loaded artifact bytes and calls `admit_policy` and `decide`; it is the only inference site.
- `src/engine/executor/choice.rs` with `Executor::with_typed_policy(runtime)`. For a bound Condition node it projects, checks cancellation, runs the adapter on a blocking task raced against `tokio::time::sleep(deadline)` and a bounded cancellation poll, validates, resolves and routes through `condition_outputs_by_real_port` with exactly one label. The binding identity is folded into the node's cache key in both the serial and frontier paths, so a profile or artifact change never reuses an older cached decision. Decisions are collected per run and attached by the headless shell to the Condition node's `RunNodeDetail.policy_decision` (`skip_serializing_if = None`).
- `src/workflow/gate/shell/{suspension,resume,policy}.rs` (approval shell; amended after the gate to close the suspension/resume persistence gap): `persist_suspension` attaches the decisions made before the gate with no workflow outcome (`ledger::attach_suspended`). `resume_with_typed_policy` loads the stored decisions and checks each with `ledger::revalidate` (the compatibility check of `replay` plus retained-receipt verification) before the resume is claimed; a stale or unbound decision is `Error::PolicyDecisionStale` with nothing re-decided or rewritten. The checkpointed Condition node is restored, not re-executed, so a compatible decision makes no adapter call; carried and new decisions are attached when the resume persists. `resume_with_authority_provider` binds no profile and is unchanged for runs without stored decisions.

### 4. Outcome mapping

| Response | Outcome |
| --- | --- |
| receipt `selected`, all bindings and membership checks hold | `accepted` |
| receipt refused with `abstained` | `abstained` |
| receipt refused with `unavailable`, or adapter reports backend unavailable | `unavailable` |
| wall-clock deadline elapsed, or receipt refused with `operation-budget` | `timed_out` |
| oversize/malformed/unverifiable receipt, stale or crossed binding, cohort mismatch, crossed features, wrong candidate set, undeclared label, unexpected fallback outcome, `incompatible-artifact`, `stale-binding`, `malformed-input`, `input-out-of-range`, `score-out-of-range`, projection refusal | `invalid` |
| cancellation observed before or during invocation | `cancelled` (terminal; no fallback) |

### 5. Usage accounting (#13)

The decision record keeps `reserved_operation_count` (the profile's `max_operation_count`, a reservation), `observed_inference_count` and `observed_normalization_count` (the runtime's deterministic counters from a verified receipt; absent when no verified receipt exists) separate. No charge is recorded; a receipt's counters are runtime counters, not model-written usage claims, and wall-clock time is measured only by the host harness.

### 6. Measurement protocol (declared before evaluation)

Cases live in `tests/fixtures/typed-policy/cases.json`. Families: `payments-r1` (authoring family; the fixture tree was hand-authored by inspecting only this family and is excluded from held-out metrics), and held-out `search-r1`, `auth-r2`, `reports-r1`. Outcome labels come from this rubric, fixed here before any run:

- `page` when `critical_errors ≥ 1` or `error_count ≥ 3`;
- otherwise `ticket` when `error_count ≥ 1`, `warn_count ≥ 3` or `max_latency_ms ≥ 2000`;
- otherwise `ignore`.

Arms on the same workflow and declared label set:

1. **Deterministic rule** (profile disabled): the feature node's rule hint, `page` if any ERROR, else `ticket` if any WARN, else `ignore`, through the Condition node's existing predicates.
2. **Present reasoning-model path:** the existing Animus/OpenAI-compatible agent route. It requires a configured model endpoint and credential. If none is available, the arm is recorded as `unavailable` with that reason and no result is fabricated.
3. **Fast policy:** enabled profile, artifact `tests/fixtures/typed-policy/policy-log-triage-tree-v1.json`, fallback `declared_predicates`.

Per arm and family: whole-workflow success (the routed continuation equals the rubric label, it completed, and the run has no failure), total wall-clock workflow latency (host measured, median and p95 over 5 repetitions), policy-stage latency (projection + invocation + validation), deterministic operation units, coverage (accepted without fallback), invalid-output rate, fallback rate and abstention count.

Thresholds, fixed before evaluation:

- **T1 safety (hard):** 0 routed labels outside the declared branches; 0 runs with more than one continuation; 0 adapter calls during replay.
- **T2 benefit:** fast-policy held-out success ≥ deterministic held-out success + 0.10.
- **T3 coverage:** held-out coverage ≥ 0.80.
- **T4 validity:** held-out invalid-output rate ≤ 0.05 and fallback rate ≤ 0.20.
- **T5 cost:** median policy-stage overhead ≤ 1 ms and median whole-workflow latency ≤ 1.10 × the deterministic median.

Verdict rule: `go-for-continued-opt-in-pilot` only if T1–T5 pass and the reasoning arm was measured; with T1–T5 passing but the reasoning arm unavailable, `plumbing-qualified-comparison-incomplete`; a T1 failure is `no-go`; any other failure is `no-improvement-established`. A fixture artifact never supports an improvement claim, whatever the numbers.

## Risks / Trade-offs

- The fixture tree and the rubric are authored by the same change; agreement on held-out families measures the plumbing and the hand-authored thresholds, not learned generalization. This is stated as a non-claim.
- Folding the binding into the cache key changes cache keys only for bound nodes; disabled profiles keep existing keys.
- The wall-clock deadline abandons, but cannot preempt, a blocking adapter task; the frozen leaf's operation budget bounds its work deterministically.
- Retaining raw receipts grows run details by at most `max_receipt_bytes` per bound node.

# Typed policy operations

r[impl lattice.typed_policy.profile]

Lattice can ask a frozen fast policy to choose one branch of one declared `Condition` node ([Lattice #23](https://github.com/OnixResearch/lattice/issues/23)). The policy runtime is Animus's `animus-frozen-policy` leaf ([Animus #15](https://github.com/OnixResearch/animus/issues/15)), vendored at Animus `1c370d5029a2667051c46bdfa1919e78e65e2f4b`. The operation is optional: with no enabled profile, execution, cache keys and run records are byte-identical to a run without the feature.

## Layers

| Layer | Path | Role |
| --- | --- | --- |
| Pure core | `src/workflow/decision.rs`, `src/workflow/decision/{admission,checks,binding,validation,ledger}.rs` | profile admission, binding, projection, independent receipt validation, typed outcomes, fallback resolution, decision records, replay |
| Animus adapter shell | `src/animus_agent/frozen.rs` | the only inference site: `admit_policy` with the profile's exact artifact identity, then `decide` with no fallback |
| Executor shell | `src/engine/executor/choice.rs` | wall-clock deadline, cancellation, routing through the Condition node's existing branch ports, profile-aware cache keys |
| Persistence | `RunNodeDetail.policy_decision`, `lattice::headless::run_saved_workflow_with_typed_policy`, `lattice::workflow::gate::shell::resume_with_typed_policy` | one bounded decision record on the bound node's run detail, across approval suspension and resume |

The library surface is `Executor::with_typed_policy(Runtime)` and the two entries above. No CLI flag, node kind or default profile is added; the CLI and TUI resume through `resume_with_authority_provider`, which binds no profile.

## Profile `lattice.typed-policy-profile.v1`

A JSON document with unknown fields refused:

- `id`, `version`, `enabled`, `shape` (`select` only) and `stateful` (must be `false`; recurrent state belongs to [#12](https://github.com/OnixResearch/lattice/issues/12), and the Animus agent session rejection is unchanged);
- `binding`: the workflow name and Condition node label;
- `input`: schema identity, `max_bytes`, and per-feature `{name, pointer, min, max}` in the artifact's order; out-of-range values are refused, never clamped;
- `labels`: 1–16 unique `[A-Za-z0-9._:-]` labels, each a declared branch of the bound node;
- `result`: schema identity, `max_receipt_bytes` (≤ 16 KiB) and `retain_receipt`;
- `limits`: `max_operation_count` (the runtime's deterministic budget, ≤ 10⁶) and `deadline_millis` (host wall clock, ≤ 60 000);
- `cohort`: runtime crate, version and artifact API, artifact and model (weights) `b3:` identities, policy kind, feature and action schema identities; a model name or path is never identity;
- `fallback`: `{"kind": "label", "label"}`, `{"kind": "declared_predicates"}` (the node's own predicates when exactly one matches) or `{"kind": "terminal"}`.

The profile identity is the `b3:` BLAKE3 of its canonical encoding. The fixture profile is `tests/fixtures/typed-policy/profile-log-triage-route-v1.json`.

## Execution

r[impl lattice.typed_policy.route]

For an enabled profile whose workflow and node label match, the executor:

1. refuses before invocation when the node is not a `Condition` node or a profile label is not one of its declared branches;
2. projects the node input into integer features;
3. checks cancellation, then runs the adapter on a blocking task, racing the wall-clock deadline and cancellation;
4. validates the response independently and resolves the outcome through the declared fallback;
5. routes exactly one declared label through `condition_outputs_by_real_port`, or fails the node with `typed policy terminal outcome: <reason>`.

Downstream nodes run under the existing ready-frontier admission and effect authority. A decision is a workflow branch, not a grant: it cannot add a node, port, tool, credential or effect, cannot produce a role-protocol label, and never runs when an upstream effect has no settled outcome. The bound profile identity is folded into the node's cache key, so a changed profile or artifact never reuses an older cached decision.

## Outcomes

r[impl lattice.typed_policy.validation]

| Outcome | When |
| --- | --- |
| `accepted` | a verified receipt selected a declared candidate and every binding matched |
| `abstained` | the runtime's best eligible score was below the artifact's abstention threshold |
| `unavailable` | the backend could not answer (`backend-unavailable`) or no artifact was loaded (`artifact-unavailable`) |
| `timed_out` | the wall-clock deadline elapsed (`deadline`) or the operation budget was exceeded (`operation-budget`) |
| `invalid` | oversized or unverifiable receipt, crossed binding/input, changed candidate set, stale runtime/artifact/model, unexpected fallback, any other runtime refusal code, or a projection fault |
| `cancelled` | cancellation was observed; always terminal, never a fallback |

Scores are evidence only. Lattice never forwards its fallback to the runtime, so a receipt claiming a fallback is invalid.

## Records and replay

r[impl lattice.typed_policy.record]
r[impl lattice.typed_policy.replay]

A `lattice.typed-policy-decision.v1` record carries the invocation, profile, artifact, model, runtime, projected-input and candidate-set identities, the candidates, outcome, routed label and who chose it, the fallback or terminal reason, the bounded raw receipt when retained, its identity, usage and the eventual workflow outcome. Usage keeps the profile's `reserved_operation_count` (a reservation) apart from `observed_inference_count` and `observed_normalization_count` (runtime counters from a verified receipt); no charge is recorded ([#13](https://github.com/OnixResearch/lattice/issues/13)). Nodes without a decision serialize no `policy_decision` field.

`decision::ledger::replay(record, profile, declared_labels)` returns the routed label from the record and re-verifies a retained receipt without inference or adapter work. A changed profile or artifact is `ProfileChanged`/`ArtifactChanged`; the stored record is never rewritten. A retry after an uncertain dispatch still follows the existing acknowledgement and reconciliation path; the decision record settles no effect.

Decisions are persisted on every durable run path:

- a completed headless run attaches each decision with the eventual workflow outcome;
- a run that suspends at an approval gate attaches the decisions made so far with `workflow_outcome` absent, since the run has not finished;
- `resume_with_typed_policy` loads the stored decisions and, before the resume is claimed, checks each with `decision::ledger::revalidate` (the profile, artifact and model compatibility check of `replay`, plus retained-receipt verification). A compatible decision is carried forward unchanged with no adapter call, because its node is restored from the approval checkpoint rather than re-executed; decisions made after the resume are added, and the finished run fills in `workflow_outcome` (a later gate leaves it absent again).

A stored decision whose node no longer binds an enabled profile (including a resume through `resume_with_authority_provider`, which binds none), or whose profile, artifact, model or receipt changed, refuses the resume with `Error::PolicyDecisionStale` (`approval resume refused: stale typed policy decision: …`). The decision is never re-made, the approval request stays unclaimed, and the stored run detail is unchanged. Runs without stored decisions resume exactly as before.

## Measurement

r[impl lattice.typed_policy.measurement]

Thresholds and the outcome rubric were fixed in `.cairn/changes/add-typed-policy-operation/design.md` before any run. The harness extends the unchanged `examples/workflows/log-triage.ncl` fixture in memory and compares the deterministic rule, the present reasoning-model route and the fast policy on the same declared labels:

```sh
cargo run --locked --example typed_policy_measurement > measurement.json
```

The retained result is `.cairn/changes/add-typed-policy-operation/evidence/measurement.json`; its summary is in that change's `evidence.md`.

## Non-claims

- The fixture artifact is hand-authored (`provenance.kind = "fixture"`); RSI Lab owns training and export. Agreement with the rubric measures the boundary and the authored thresholds, not learned generalization.
- The reasoning-model arm is not applicable to this fixture: the existing log-triage workflow has no Agent or model node, and its routing step is the deterministic Condition the harness adds. No result is substituted.
- Latency is wall-clock on a shared host and is not a performance guarantee.
- No improvement, production readiness or default adoption is claimed.

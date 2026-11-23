# Charge observed model requests from complete broker evidence

Tracking: [Lattice #31](https://github.com/OnixResearch/lattice/issues/31). Related: #13 (accounting semantics), #24 (inference-service usage joined by Kiln), #6 (Kiln budget lineage).

## Why

A live implementation attempt's model usage reaches Kiln as `unknown` even when Lattice's broker relayed every request (Kiln #49 live pilot run 6: three replies through the broker, reported as `model_requests: "unknown"` and charged the full 64-request ceiling). Two gaps cause it. Durable per-request evidence (`model-requests/<id>.intent.json` and `.record.json`) is written only for a binding with a serving declaration, so a brokered attempt without one keeps no request ledger. And the single-shot lineage draw and every repair step report hard-code `model_requests: null` with provenance `unknown` and charge the broker ceiling, even when the retained served evidence is complete.

## What Changes

- The attempt broker, the worker's only network egress, records every request it relays for a binding without a serving declaration: a write-once `lattice.brokered-model-request-dispatch.v1` intent synced before upstream contact and a write-once `lattice.brokered-model-request-observation.v1` terminal record, keyed by the broker's request index and naming no service. The served-route evidence is unchanged. r[lattice.serving_binding.brokered_evidence]
- `ModelRequestCoverage` gains `brokered_unattributed`: measured requests without service attribution, distinct from `unknown` (unobserved), which stays for an attempt with no retained evidence. r[lattice.serving_binding.brokered_evidence.scenario.unattributed]
- Complete evidence (every intent has a terminal record, none `uncertain`, no conflict) yields an exact request count, attempt-wide or for one broker run's request-index range. Conflicts and crossed evidence still fail closed. r[lattice.serving_binding.brokered_evidence.scenario.charged] r[lattice.serving_binding.brokered_evidence.scenario.conflict]
- The single-shot lineage draw and each repair step charge exactly that count, with `host_measured` provenance; anything else keeps the ceiling with `unknown`. Repair step `n` counts its own range `[(n - 1) * max_requests, n * max_requests)`. The single-shot draw is retained write-once before the result and replayed as retained; repair charges are replayed from the retained disposition. r[lattice.implementation_attempt.observed_charge] r[lattice.implementation_attempt.observed_charge.scenario.single_shot] r[lattice.implementation_attempt.observed_charge.scenario.incomplete] r[lattice.implementation_attempt.observed_charge.scenario.step_range] r[lattice.implementation_attempt.observed_charge.scenario.replay]

## Impact

- **Pure core:** new `crates/lattice-core/src/execution_binding/serving/brokered.rs`; `serving::usage::ModelRequestCoverage::{BrokeredUnattributed, observed_requests}` and `Conflict::Served`; `serving::is_valid_coverage`; `repair::step::Report::observed_model_requests`; `repair::state::Loop::start`.
- **Shell:** `candidate_worker::broker::{ModelRequestGate::admit(checks, request_index), GateTicket, RecordedRoute, RequestEvidence}` and `broker/brokered.rs`; `cli/binding/{requests,usage}.rs`; `cli/session/launch.rs`; `cli/execution.rs`; `cli/recovery.rs`; `cli/repair/{mod,draw,operator}.rs` and `cli/repair/operator/worker.rs`; `model_gateway/{authority,shell}.rs`.
- **Tests:** `tests/test_execution_binding_brokered.rs`, `tests/test_execution_binding_serving.rs`, `src/implementation_attempt/cli/binding/tests/brokered.rs`, `src/implementation_attempt/cli/tests/repair/operator.rs`, `src/implementation_attempt/repair/tests.rs`.
- **Docs:** `docs/worker-serving-binding.md`, `docs/orchestration-budget-usage.md`, `docs/candidate-worker.md`, `docs/worker-compute-binding.md`, changelog.
- **Compatibility:** new attempts write `model-requests/` without a serving declaration and report `brokered_unattributed`; older attempts keep `unknown`. A lineage usage entry may now carry `provenance: "host_measured"` and a charge below the ceiling. `ModelRequestGate::admit` takes the broker's request index; `repair::step::Report` gains `observed_model_requests` (defaulting to `null`); the operator `Launch` takes the step's broker gate. Kiln `90a9fb64` has no consumer of `budget_lineage.usage[]` (its `CHARGED_DIMENSIONS` gates only the request-side lineage, and its worker inventory reads `model_requests` as opaque JSON), so it accepts both shapes.

## Dependencies

- Local: `bind-model-serving-admission`, `bound-same-attempt-repair`, `run-operator-omp-repair`.

## Out of Scope

- Service attribution for unserved requests; tool-invocation measurement; a live broker, endpoint or Kiln qualification; Kiln changes.

# Orchestration budget usage

Orchestration keeps reservation, settlement, and usage provenance separate.

- `StartRound` and `RetryRound` record the configured `planned_charge`.
  `reserve_budget` stores that conservative upper bound as the reservation.
- `RunFinished` settles with the reservation charge and sets
  `OrchestrationInstance::last_usage_basis` to `reserved_fallback`. This is the
  current shell behavior: a conservative charge, not measured worker usage.
- `RunFinishedMeasured` settles with an explicitly supplied measured
  `BudgetCharge` and a `usage_basis`. A fully measured receipt uses `measured`;
  partial bases name exactly the measured dimensions:
  `measured_tokens`, `measured_duration`, `measured_cost`,
  `measured_tokens_and_duration`, `measured_tokens_and_cost`,
  `measured_duration_and_cost`, or `measured_tokens_duration_and_cost`.
  Dimensions that are not enforced remain at their reservation values. Older
  events without a basis deserialize as `measured`.
- When the shell can read provider-reported `usage` metadata from every completed
  Agent node in the owned run, it sums tokens and provider-reported microunit
  cost. It derives duration from the owned run record's start and finish
  timestamps. Measurement is applied only for budget dimensions the selected
  policy enforces; unenforced dimensions are not silently relabelled as
  measured. If an enforced observation is unavailable, the shell keeps the
  reserved fallback rather than synthesizing a partial measured receipt.
- A missing or incomplete observation is not zero. The current shell does not
  synthesize a measured receipt; callers that cannot supply measured usage keep
  the reserved fallback.
- `budgets.consumed` reports the settled amount. `budgets.reservations` reports
  still-pending reservations. `last_usage_basis` states how the last settlement
  was derived.

This boundary does not provide provider billing, exact token or cost ceilings,
or recovery of an observation that was never persisted. Those remain explicit
compatibility and accounting work.

## Versioned usage observations

`CommandWorkerUsageObservation` is the durable per-attempt observation contract.
It binds workflow/instance/dispatch/run/node/invocation, worker profile, and
attempt identity, and records optional agent invocations, model requests,
subagent calls, tokens, cost, and duration.

`host_measured`, `provider_reported`, `worker_reported`, `estimated`, and
`unknown` provenance remain distinct. Unknown observations cannot carry values
or claim completeness. Estimated or unknown observations never expose measured
tokens or cost. Complete observations must carry at least one value.

The store persists observations by attempt identity and indexes them by run
identity. Identical writes are idempotent; different content for the same attempt
or run is rejected. A complete `run`-scoped observation whose run and dispatch
match is used for the enforced token, duration, and cost dimensions before the
shell falls back to run-detail aggregation. `node`-scoped observations are not
used for whole-run settlement.

## Worker and profile compatibility

A worker profile that supplies observations must declare, and retain in its
compatibility evidence:

- the exact worker executable and build identity;
- the observation protocol schema and field/unit mapping;
- the model/provider identity used for reported tokens or cost;
- the covered interval and whether it spans one node, one invocation, or the
  whole run;
- how missing, partial, estimated, and conflicting telemetry is represented.

Lattice accepts only the versioned observation schema and bounded attempt/run
bindings. Unknown fields, unsupported schemas, malformed identities, and
observations that cannot state their units or coverage are rejected rather than
coerced. A profile or protocol change requires a new compatibility record; a
worker name alone does not establish compatibility. Model request and subagent
counts are retained observations, not budget dimensions, until a separate
dimension and policy change admits them.

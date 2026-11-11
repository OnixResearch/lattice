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

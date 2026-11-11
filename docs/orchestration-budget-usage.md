# Orchestration budget usage

Orchestration keeps reservation, settlement, and usage provenance separate.

- `StartRound` and `RetryRound` record the configured `planned_charge`.
  `reserve_budget` stores that conservative upper bound as the reservation.
- `RunFinished` settles with the reservation charge and sets
  `OrchestrationInstance::last_usage_basis` to `reserved_fallback`. This is the
  current shell behavior: a conservative charge, not measured worker usage.
- `RunFinishedMeasured` settles with an explicitly supplied measured
  `BudgetCharge` and a `usage_basis`. A fully measured receipt uses `measured`;
  a receipt where only provider-reported tokens were observed uses
  `measured_tokens`, and the remaining dimensions retain their reservation
  values. Older events without a basis deserialize as `measured`.
- When the shell can read provider-reported `usage` metadata from every completed
  Agent node in the owned run, it sums those tokens and settles with
  `measured_tokens`. If any Agent node lacks usable metadata, the shell keeps the
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

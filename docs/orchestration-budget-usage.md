# Orchestration budget usage

Orchestration keeps reservation, settlement, and usage provenance separate.

- `StartRound` and `RetryRound` record the configured `planned_charge`.
  `reserve_budget` stores that conservative upper bound as the reservation.
- `RunFinished` settles with the reservation charge and sets
  `OrchestrationInstance::last_usage_basis` to `reserved_fallback`. This is the
  current shell behavior: a conservative charge, not measured worker usage.
- `RunFinishedMeasured` settles with an explicitly supplied measured
  `BudgetCharge` and sets `last_usage_basis` to `measured`. The measured amount
  is retained separately from the prior round's planned charge in durable event
  history.
- A missing or incomplete observation is not zero. The current shell does not
  synthesize a measured receipt; callers that cannot supply measured usage keep
  the reserved fallback.
- `budgets.consumed` reports the settled amount. `budgets.reservations` reports
  still-pending reservations. `last_usage_basis` states how the last settlement
  was derived.

This boundary does not provide provider billing, exact token or cost ceilings,
or recovery of an observation that was never persisted. Those remain explicit
compatibility and accounting work.

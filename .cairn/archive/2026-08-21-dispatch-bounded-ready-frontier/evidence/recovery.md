# Ready-frontier recovery evidence

<!-- r[verify lattice.workflow_execution.ready_frontier.recovery] -->

Ready-frontier dispatch reuses the existing attempt and recovery boundaries.

## Prepared and started

The pure planner models active total, lane, resolved-conflict, and unresolved-conflict reservations. `active_capacity_and_effect_conflicts_are_reserved` verifies started reservations block conflicting work.

Product shell effects bind Choregraph intent and durable attempt identity before the blocking adapter starts. `store_reserves_before_effect_and_rejects_duplicates` covers duplicate attempt reservation.

## Completed and failed

Each observed terminal result is reduced through the run-scoped Choregraph session before readiness is recomputed. The focused frontier tests cover completed branches, failed preparation, and typed terminal blockers.

## Cancelled and suspended

The planner has distinct cancellation and suspension blockers. Approval workflows retain the existing serial durable-suspension path. The TUI cancellation rail still owns process-group teardown and forbidden post-cancel effects.

## Uncertain

`dispatch_records_unknown_after_post_effect_persistence_failure` verifies that a post-effect persistence failure records an unknown outcome. Ready-frontier dispatch does not convert that outcome into a retry.

`test_post_effect_crash_profile` retains positive scheduler and webhook crash profiles and negative pre-effect, premature-terminal, cross-ingress, and unsupported-response cases.

## Recovery non-claims

These fixtures do not prove host crash atomicity, distributed writer safety, external effect idempotency, or automatic retry safety. They preserve the existing explicit unknown-outcome boundary.

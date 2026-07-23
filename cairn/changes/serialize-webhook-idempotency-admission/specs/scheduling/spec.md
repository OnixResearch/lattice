# Scheduling Delta

## ADDED Requirements

### Requirement: Atomic webhook idempotency admission
r[lattice.scheduling.webhook_atomic_idempotency] Lattice MUST arbitrate a webhook trigger's derived idempotency key through one durable domain-separated BLAKE3 trigger-scoped claim committed with a reserved run identity before workflow effects, MUST admit at most one local execution for that claim, and MUST preserve the claim across concurrent delivery, failure, and restart.

#### Scenario: Simultaneous deliveries select one winner
r[lattice.scheduling.webhook_atomic_idempotency.scenario.concurrent]
- GIVEN multiple valid webhook requests concurrently target the same trigger-scoped idempotency key
- WHEN their listener handlers attempt durable admission
- THEN exactly one transaction MUST reserve a run and claim the key, every losing request MUST receive the deterministic duplicate response, and no loser may start workflow execution

#### Scenario: Claim precedes workflow effects
r[lattice.scheduling.webhook_atomic_idempotency.scenario.ordering]
- GIVEN a valid idempotent webhook request has passed pure planning and policy checks
- WHEN Lattice crosses into the side-effecting workflow shell
- THEN the trigger-scoped claim, reserved run identity, and safe accepted/started history fact MUST already be durably committed

#### Scenario: Restart preserves an uncertain claim
r[lattice.scheduling.webhook_atomic_idempotency.scenario.restart]
- GIVEN the listener dies after committing a claim and before terminal workflow or HTTP response persistence is observable
- WHEN the same trigger-scoped idempotency key is redelivered after restart
- THEN Lattice MUST reject the redelivery without a second local workflow execution and MUST preserve the first claim as locally accepted with no invented terminal success

#### Scenario: Failed execution does not release the claim
r[lattice.scheduling.webhook_atomic_idempotency.scenario.failure]
- GIVEN the unique admitted webhook workflow returns a terminal execution failure
- WHEN the listener records the safe failure and another request presents the same trigger-scoped key
- THEN the second request MUST be rejected as a duplicate and MUST NOT retry the failed effect implicitly

#### Scenario: Idempotency-disabled trigger remains explicit
r[lattice.scheduling.webhook_atomic_idempotency.scenario.disabled]
- GIVEN a webhook trigger explicitly declares no idempotency policy
- WHEN distinct or concurrent valid requests target that trigger
- THEN Lattice MUST NOT create a hidden idempotency claim and MUST evaluate each event under the declared overlap policy

#### Scenario: Claim scope does not imply external exactly-once execution
r[lattice.scheduling.webhook_atomic_idempotency.scenario.non-claim]
- GIVEN one Lattice claim admits at most one local workflow execution for a trigger-scoped key
- WHEN the listener reports or documents that result
- THEN it MUST NOT claim remote effect commitment, provider deduplication, distributed multi-writer safety, response delivery, or exactly-once external execution

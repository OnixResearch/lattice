# Design: Bounded observed-failure retries

## Goal and success evidence

Allow an authorized operator to retry only an exact, locally persisted terminal failure. Success requires a new run ID and BLAKE3 dispatch identity, an atomic retry-start event plus fresh budget reservation before execution, a bounded durable retry count, exact owner evidence, valid replay, and deterministic refusal of pending/unobserved or non-failure states.

False completion includes reopening a failed state without durable evidence, reusing a run or dispatch identity, retrying after abandonment, treating a shell error without an exact failed `RunRecord` as retryable, allocating work after policy/budget refusal, or passing tests while replay no longer validates historical/default-disabled definitions.

## Approach registry

| Family | Mechanism | State | Evidence or blocker |
|---|---|---|---|
| Reopen terminal instance | Transition `Failed` back to `Ready`, then use ordinary start | Falsified | Creates a crash window with a durable `Ready` state but no reserved dispatch; cannot structurally distinguish observed failure from abandonment. |
| Successor instance | Create a new orchestration linked to the failed one | Falsified | Splits one retry budget and trigger identity across instances, duplicates policy state, and weakens exact run-chain inspection. |
| Dedicated retry round | One authorized transition from exact retry evidence atomically reserves and starts a fresh dispatch | Selected | Keeps one event chain, fresh identities, optimistic concurrency, and budget-before-effect ordering without an intermediate state. |
| Retry on recovery | Redispatch a pending intent when no run record exists | Falsified | `PendingUnobserved` cannot distinguish no effect from an unrecorded remote effect and therefore remains non-retriable. |

Search budget: four mechanism families, three targeted source retrieval rounds, one independent model review, one adversarial review after implementation, and deterministic repository validation. Terminal outcomes are validated, blocked with an exact cause, or exhausted; plausible prose is not completion evidence.

## Model additions

`OrchestrationDefinition.retry` is optional. When present, `RetryPolicy.max_observed_failure_retries` must be positive. `None` disables retries and is omitted from canonical serialization so existing schema-v1 definition identities remain stable.

`OrchestrationInstance.retry` is a bounded ledger containing:

- the number of retry rounds admitted; and
- at most one `RetryableFailure` with the exact failed run and dispatch identities.

An empty ledger is omitted from serialization. A failed `RunFinished` transition records retry eligibility only when the imperative shell asserts `retryable_observed_failure` after loading an exact owned terminal `RunRecord` with a nonzero failure count. The assertion field defaults false and is omitted when false so old event facts retain canonical identity.

## Retry transition

`RetryRound` carries the authorized actor, fresh pending `DispatchIntent`, and planned budget charge. The pure core admits it only when:

1. status is `Failed` with no active/pending dispatch;
2. retry policy is enabled and its count is below the maximum;
3. the retry ledger contains exact observed failed run/dispatch evidence;
4. the actor is authorized;
5. the new run and dispatch identities differ from the failed attempt;
6. the workflow revision is unchanged; and
7. fresh budget reservation fits all enforced dimensions.

The accepted transition increments retry count and round, clears retry eligibility and terminal summary, reserves budget, installs the fresh pending intent, and enters `Running` in one committed event. The shell then uses the existing acknowledgement, owned headless execution, settlement, and terminalization path.

A pure preflight repeats policy, authority, state, and budget checks before the shell allocates a numeric run ID. Optimistic commit validation remains authoritative under concurrent attempts; a losing concurrent caller may consume an otherwise unused numeric reservation but cannot create a second dispatch, run, or budget reservation.

## Observation boundary

The shell marks a failure retryable only if it can load the exact `RunRecord` for the just-finished run, the owner equals the current orchestration instance and dispatch, the record is terminal, and `failures > 0`. A headless shell error without that record terminalizes as failed but does not become retryable.

A pending dispatch is never accepted by `retry`. With no run record it receives a fixed pending-unobserved refusal and remains available only to `recover`, authorized cancellation, and `abandon`. With an exact record it is directed to ordinary recovery. Foreign records fail closed.

## Compatibility and replay

New optional/default fields use serde defaults and skip serialization when absent, false, or empty. Default-disabled v1 definitions and old `RunFinished` events therefore reproduce their previous BLAKE3 identities and state hashes. Retry-enabled definitions include policy in their immutable definition revision. Retry events replay through the same pure transition function and local event chain.

## Authority and non-claims

Retries are explicit and operator-authorized; there is no automatic retry loop. The maximum policy and existing multidimensional budgets are independent ceilings. This design does not provide backoff, scheduling, provider fallback, provider cancellation, arbitrary kill safety, provider truth, actual token/billing measurement, remote effect deduplication, remote exactly-once execution, or production readiness.

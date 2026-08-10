# Agent orchestration delta

## ADDED Requirements

### Requirement: Bounded observed-failure retry policy
r[lattice.agent_orchestration.observed_failure_retry] Lattice MUST admit retries only under an immutable positive retry bound and exact locally persisted terminal-failure evidence.

#### Scenario: Exact owned failure becomes retryable
r[lattice.agent_orchestration.observed_failure_retry.scenario.observed]
- GIVEN a retry-enabled orchestration round terminalizes with an exact instance-and-dispatch-owned run record whose failure count is nonzero
- WHEN Lattice commits the failed run observation
- THEN it MUST retain only bounded run and dispatch retry evidence and MUST NOT retain output, error, prompt, credential, reasoning, or provider content as retry authority

#### Scenario: Retry policy is explicit and bounded
r[lattice.agent_orchestration.observed_failure_retry.scenario.policy]
- GIVEN an orchestration definition enables retries
- WHEN Lattice validates and hashes the definition
- THEN the policy MUST declare a positive maximum observed-failure retry count and the immutable definition revision MUST bind that count

#### Scenario: Default-disabled compatibility remains stable
r[lattice.agent_orchestration.observed_failure_retry.scenario.compatibility]
- GIVEN a schema-v1 definition or event omits retry fields
- WHEN Lattice deserializes, reserializes, hashes, or replays it
- THEN retries MUST remain disabled and omitted default fields MUST NOT change its canonical identity or event state

### Requirement: Atomic authorized retry round
r[lattice.agent_orchestration.retry_round] Lattice MUST start an authorized retry as one durable transition that reserves fresh budget and installs a fresh run and dispatch identity before invoking the workflow shell.

#### Scenario: Authorized retry starts a fresh attempt
r[lattice.agent_orchestration.retry_round.scenario.success]
- GIVEN an exact observed terminal failure remains below its retry and multidimensional budget ceilings
- WHEN an authorized operator invokes `orchestration retry`
- THEN Lattice MUST atomically increment the retry and round counts, reserve fresh budget, install distinct run and BLAKE3 dispatch identities, execute the immutable workflow revision, settle the attempt, and preserve valid replay

#### Scenario: Policy, authority, and budget refuse before dispatch
r[lattice.agent_orchestration.retry_round.scenario.refusal]
- GIVEN retry is disabled, exhausted, unauthorized, not backed by exact observed failure evidence, or cannot fit fresh budget
- WHEN retry admission runs
- THEN Lattice MUST reject before invoking the workflow shell or creating a new dispatch, run record, or budget reservation

#### Scenario: Pending-unobserved work is never retried
r[lattice.agent_orchestration.retry_round.scenario.pending-unobserved]
- GIVEN an orchestration retains a pending dispatch with no exact owned persisted run record
- WHEN an operator invokes `orchestration retry`
- THEN Lattice MUST reject without mutation or redispatch and direct the operator to recovery or explicit authorized abandonment

#### Scenario: Abandoned and foreign outcomes are never retried
r[lattice.agent_orchestration.retry_round.scenario.unsafe-evidence]
- GIVEN a failed state arose from abandonment, a shell failure lacks an exact terminal run record, or a numeric run record is foreign or legacy-unscoped
- WHEN retry admission runs
- THEN Lattice MUST reject without allocating authority from that evidence

#### Scenario: Retry claims remain bounded
r[lattice.agent_orchestration.retry_round.scenario.non-claim]
- GIVEN a bounded retry reaches a local terminal result
- WHEN Lattice renders status, replay, or inspection evidence
- THEN it MUST NOT claim provider cancellation, arbitrary kill safety, provider truth, actual token or billing measurement, remote effect deduplication, remote exactly-once execution, or production readiness

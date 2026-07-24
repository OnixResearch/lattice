# Scheduling Delta

## ADDED Requirements

### Requirement: Durable scheduler occurrence admission
r[lattice.scheduling.scheduler_occurrence_admission] Lattice MUST arbitrate each scheduled occurrence through one durable domain-separated BLAKE3 identity, MUST commit its reserved run identity and next schedule time before workflow effects, and MUST prevent daemon restart from automatically admitting the same occurrence again.

#### Scenario: One transaction claims one due occurrence
r[lattice.scheduling.scheduler_occurrence_admission.scenario.atomic]
- GIVEN a persisted schedule remains due at an exact epoch
- WHEN one or more contenders attempt to claim that occurrence
- THEN exactly one transaction MUST reserve a run identity, persist the occurrence owner, and advance the schedule while every later claim observes the existing winner

#### Scenario: Claim precedes workflow effects
r[lattice.scheduling.scheduler_occurrence_admission.scenario.ordering]
- GIVEN a valid due occurrence is ready for headless execution
- WHEN Lattice crosses into the workflow effect shell
- THEN the occurrence identity, workflow revision, first attempt run identity, pending status, and next schedule time MUST already be durably committed

#### Scenario: Restart suppresses automatic replay
r[lattice.scheduling.scheduler_occurrence_admission.scenario.restart]
- GIVEN the scheduler dies after claiming an occurrence and before matching terminal run evidence is durable
- WHEN the scheduler daemon restarts at or after the same due epoch
- THEN it MUST NOT start another attempt automatically and MUST preserve the occurrence as pending with an unknown local outcome

#### Scenario: Recovery requires exact owned run evidence
r[lattice.scheduling.scheduler_occurrence_admission.scenario.recovery]
- GIVEN an occurrence is pending for one active attempt
- WHEN occurrence recovery inspects run history
- THEN only a run with the exact active run identity and occurrence owner MAY settle the occurrence, while absent, foreign, or legacy-unscoped evidence MUST remain pending or fail closed

#### Scenario: Explicit retry acknowledges duplicate risk
r[lattice.scheduling.scheduler_occurrence_admission.scenario.retry]
- GIVEN a pending occurrence has no persisted run record, retains its exact workflow revision, and has not used its one bounded retry
- WHEN an operator explicitly acknowledges duplicate-effect risk and retries the occurrence
- THEN Lattice MUST reserve a fresh owned run identity and execute one new attempt without changing the occurrence identity or automatically authorizing another retry

#### Scenario: Explicit abandonment preserves uncertainty
r[lattice.scheduling.scheduler_occurrence_admission.scenario.abandon]
- GIVEN a pending occurrence has no persisted run record
- WHEN an operator explicitly acknowledges the unknown outcome and abandons it
- THEN Lattice MUST close the occurrence without executing another attempt and MUST retain an abandoned-unknown status rather than inventing success, failure, or cancellation

#### Scenario: Local admission does not imply exactly-once execution
r[lattice.scheduling.scheduler_occurrence_admission.scenario.non_claim]
- GIVEN one occurrence identity admits at most one automatic local attempt
- WHEN Lattice reports or documents that result
- THEN it MUST NOT claim external effect commitment, provider deduplication, distributed multi-writer safety, external cancellation, exactly-once execution, or production readiness

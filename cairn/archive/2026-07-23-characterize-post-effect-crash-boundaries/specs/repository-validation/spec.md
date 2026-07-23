# Repository Validation Delta

## ADDED Requirements

### Requirement: Post-effect crash-boundary evidence
r[lattice.repository_validation.post_effect_crash_boundaries] Lattice MUST continuously exercise scheduler and webhook process death after an observable workflow effect but before terminal persistence, MUST compare pre-restart and post-restart durable facts against a reviewed deterministic observation profile, and MUST report at-least-once risk without claiming exactly-once external effects.

#### Scenario: Scheduler restart exposes repeated-effect risk
r[lattice.repository_validation.post_effect_crash_boundaries.scenario.scheduler]
- GIVEN a due scheduled workflow records an observable effect and then terminates the scheduler before terminal run persistence or schedule advancement
- WHEN the crashed store is inspected without repairing the original and the scheduler restarts at the same due time
- THEN the check MUST prove the schedule remained due, MUST observe the reviewed repeated effect attempt, and MUST find only the restarted terminal run in durable history

#### Scenario: Webhook redelivery is characterized
r[lattice.repository_validation.post_effect_crash_boundaries.scenario.webhook]
- GIVEN an admitted webhook workflow records an observable effect and then terminates the listener before its terminal run, trigger-history, or HTTP response boundary is durably complete
- WHEN the crashed store is inspected and the same trigger-scoped idempotency key is redelivered after restart
- THEN the check MUST compare the response, effect-attempt count, run records, and trigger history to the version-controlled webhook crash profile and MUST fail on an unreviewed outcome

#### Scenario: Earlier death does not masquerade as post-effect evidence
r[lattice.repository_validation.post_effect_crash_boundaries.scenario.missing-effect]
- GIVEN a forced-death fixture terminates before the observable effect checkpoint
- WHEN crash observations are classified
- THEN the check MUST reject the fixture as insufficient evidence and MUST NOT report a post-effect crash result

#### Scenario: Exactly-once remains outside the claim
r[lattice.repository_validation.post_effect_crash_boundaries.scenario.non-claim]
- GIVEN a crash profile observes one or more local effect attempts and corresponding Lattice durable records
- WHEN the result is documented or emitted by continuous validation
- THEN it MUST state that the evidence does not prove remote commit identity, provider deduplication, external cancellation, arbitrary kill safety, or exactly-once external execution

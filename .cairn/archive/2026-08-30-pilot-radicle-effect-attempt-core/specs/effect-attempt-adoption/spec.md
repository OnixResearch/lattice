## ADDED Requirements

### Requirement: Effect-attempt source is conditional and Radicle-backed
r[lattice.effect_attempt_adoption.source] Lattice MUST add `effect-attempt-core` only after a simplifying two-consumer extraction is accepted and one licensed immutable revision is published under its canonical Radicle RID without GitHub, sibling, or floating fallback.

#### Scenario: Producer extraction and source are accepted
r[lattice.effect_attempt_adoption.source.scenario.accepted]
- GIVEN lossless alignment, simplifying pilots, explicit licensing, RID authority, exact revision, endpoint, and no-fallback evidence
- WHEN dependency admission runs
- THEN Lattice MAY begin a reversible production pilot

#### Scenario: Producer extraction or source is incomplete
r[lattice.effect_attempt_adoption.source.scenario.rejected]
- GIVEN extraction was rejected or any required source or acquisition fact is absent or contradictory
- WHEN dependency admission runs
- THEN Lattice MUST NOT add production dependency authority

### Requirement: Lattice retains attempt policy and storage authority
r[lattice.effect_attempt_adoption.boundary] Lattice MUST retain persisted schemas, run and workflow identities, schedules, retry and abandonment policy, acknowledgements, actors, budgets, transactions, CLI behavior, external effects, and evidence claims.

#### Scenario: Shared core gains product authority
r[lattice.effect_attempt_adoption.boundary.scenario.denied]
- GIVEN an adapter moves persistence, schedules, effect execution, actor policy, retry limits, budget accounting, or external-truth decisions into the shared core
- WHEN architecture validation runs
- THEN adoption MUST be rejected or narrowed

### Requirement: Both Lattice attempt paths prove parity
r[lattice.effect_attempt_adoption.compatibility] Scheduler occurrence and orchestration adapters MUST preserve dispatch, observation, uncertainty, closure, owner, retry, abandonment, terminal, and persisted compatibility behavior across positive and negative fixtures.

#### Scenario: Exact terminal observation settles
r[lattice.effect_attempt_adoption.compatibility.scenario.settle]
- GIVEN an unresolved current attempt and one exact owned terminal observation
- WHEN legacy and shared classifications run
- THEN both paths MUST return the same accepted closure and product shell action

#### Scenario: Evidence is absent or invalid
r[lattice.effect_attempt_adoption.compatibility.scenario.unknown]
- GIVEN terminal evidence is absent, foreign, unscoped, stale, non-terminal, malformed, or conflicting
- WHEN legacy and shared classifications run
- THEN both paths MUST preserve uncertainty or the same fail-closed diagnostic without inventing success or failure

### Requirement: Adoption requires measured simplification
r[lattice.effect_attempt_adoption.cutover] Lattice MUST adopt the shared core only when both adapters remove material duplicate decisions, retain current negative behavior, and remain thinner than the product logic they replace.

#### Scenario: Both adapters simplify
r[lattice.effect_attempt_adoption.cutover.scenario.accepted]
- GIVEN parity, source, architecture, measured simplification, focused product, and repository checks pass
- WHEN adoption is accepted
- THEN Lattice MAY remove duplicate local decisions and record VCS rollback

#### Scenario: Attempt parity is overclaimed
r[lattice.effect_attempt_adoption.cutover.scenario.non-claim]
- GIVEN standalone and Lattice attempt tests pass
- WHEN evidence is reported
- THEN it MUST NOT claim exactly-once effects, provider deduplication, transaction atomicity, rollback completeness, external cancellation, budget truth, crash safety, or release readiness

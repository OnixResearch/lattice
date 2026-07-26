## ADDED Requirements

### Requirement: Change-admission source is conditional and Radicle-backed
r[lattice.change_admission_adoption.source] Lattice MUST add `change-admission-core` only after a simplifying two-consumer extraction is accepted and one licensed immutable revision is published under its canonical Radicle RID without GitHub, sibling, or floating fallback.

#### Scenario: Producer extraction and source are accepted
r[lattice.change_admission_adoption.source.scenario.accepted]
- GIVEN lossless alignment, simplifying pilots, explicit licensing, RID authority, exact revision, endpoint, and no-fallback evidence
- WHEN dependency admission runs
- THEN Lattice MAY begin a reversible production pilot

#### Scenario: Producer extraction or source is incomplete
r[lattice.change_admission_adoption.source.scenario.rejected]
- GIVEN extraction was rejected or any required source or acquisition fact is absent or contradictory
- WHEN dependency admission runs
- THEN Lattice MUST NOT add production dependency authority

### Requirement: Lattice retains change authority and persistence
r[lattice.change_admission_adoption.boundary] Lattice MUST retain proposal and approval identities, workflow and definition revisions, epochs, successor validation, actor and authority policy, budgets, rounds, events, replay, persistence, CLI behavior, mutation, and release decisions.

#### Scenario: Shared core gains product authority
r[lattice.change_admission_adoption.boundary.scenario.denied]
- GIVEN an adapter moves identity construction, workflow validation, actor policy, budgets, events, persistence, mutation, or release decisions into the shared core
- WHEN architecture validation runs
- THEN adoption MUST be rejected or narrowed

### Requirement: Proposal lifecycle proves exact parity
r[lattice.change_admission_adoption.compatibility] The Lattice adapter MUST preserve proposal, base, candidate, admissibility, stale, authorization, disposition, terminal, identity, event, and persisted compatibility behavior across positive and negative fixtures.

#### Scenario: Exact current proposal is admitted
r[lattice.change_admission_adoption.compatibility.scenario.admit]
- GIVEN exact proposal, base, candidate, successful successor validation, current epoch/revision, and Lattice-supplied authorization facts agree
- WHEN legacy and shared classifications run
- THEN both paths MUST return the same admission and product transition result

#### Scenario: Proposal facts are stale or contradictory
r[lattice.change_admission_adoption.compatibility.scenario.deny]
- GIVEN identity mismatch, stale epoch/base, changed candidate, failed validation, authority mismatch, unauthorized actor, or terminal decision
- WHEN legacy and shared classifications run
- THEN both paths MUST return the same fail-closed result without product mutation

### Requirement: Adoption requires measured simplification
r[lattice.change_admission_adoption.cutover] Lattice MUST adopt the shared core only when the adapter removes material duplicate lifecycle decisions, retains current negative behavior, and remains thinner than the product logic it replaces.

#### Scenario: Adapter simplifies
r[lattice.change_admission_adoption.cutover.scenario.accepted]
- GIVEN parity, source, architecture, measured simplification, focused product, and repository checks pass
- WHEN adoption is accepted
- THEN Lattice MAY remove duplicate local decisions and record VCS rollback

#### Scenario: Lifecycle parity is overclaimed
r[lattice.change_admission_adoption.cutover.scenario.non-claim]
- GIVEN standalone and Lattice lifecycle tests pass
- WHEN evidence is reported
- THEN it MUST NOT claim semantic diff correctness, workflow safety, actor authority truth, mutation success, model truth, persistence correctness, or release readiness

# Workflow Execution Delta

## ADDED Requirements

### Requirement: Persistence commit outcomes
r[lattice.persistence_uncertainty.commit_outcomes] Lattice MUST distinguish not-published, published, publication-unknown, and conflicting reservation state from external effect outcomes.

#### Scenario: Reservation commit succeeds
- GIVEN a current workflow plan and an exact reservation commit observation
- WHEN the pure core evaluates dispatch admission
- THEN it classifies the reservation as published

#### Scenario: Reservation commit is uncertain
- GIVEN a commit-time result that cannot prove non-publication
- WHEN the core classifies it
- THEN it records publication unknown and does not infer that no effect can exist

### Requirement: Shared reconciliation core
r[lattice.persistence_uncertainty.shared_core] Lattice MUST consume the published product-neutral reconciliation contract through an immutable revision while retaining workflow policy, schema, authority, and effects.

#### Scenario: Shared core is selected
- GIVEN a reviewed published revision
- WHEN Lattice composes reconciliation
- THEN the composition root selects it without an ambient sibling dependency

### Requirement: Effect dispatch gate
r[lattice.persistence_uncertainty.effect_gate] Lattice MUST dispatch an effect only after it observes the exact active reservation as published.

#### Scenario: Publication remains unknown
- GIVEN an exact attempt with publication-unknown state
- WHEN the shell reaches the dispatch boundary
- THEN it performs no effect

### Requirement: Store quarantine
r[lattice.persistence_uncertainty.quarantine] Lattice MUST quarantine a store after uncertain, repaired, corrupt, or inconsistent persistence observations.

#### Scenario: Quarantined store receives a workflow command
- GIVEN a quarantined store
- WHEN a workflow command requests persistence or dispatch
- THEN the shell rejects it before any protected effect

### Requirement: Exact persistence reconciliation
r[lattice.persistence_uncertainty.reconciliation] Reconciliation MUST bind exact operation, plan, workflow revision, attempt, reservation, and predecessor identities.

#### Scenario: Reopened row matches reservation
- GIVEN publication-unknown state and an exact reopened replacement row
- WHEN reconciliation runs
- THEN the core classifies the reservation as published

#### Scenario: Reopened row conflicts
- GIVEN a reopened row with a foreign revision or attempt
- WHEN reconciliation runs
- THEN the core keeps dispatch blocked

### Requirement: Repair boundary
r[lattice.persistence_uncertainty.repair_boundary] Redb repair success MUST NOT establish workflow-history completeness or effect authority.

#### Scenario: Redb repair completes
- GIVEN Redb opens after local repair
- WHEN Lattice evaluates affected workflow records
- THEN each record remains blocked until exact reconciliation or explicit closure

### Requirement: Explicit operator actions
r[lattice.persistence_uncertainty.operator_actions] Retry or abandonment after persistence uncertainty MUST require explicit current authority and risk acknowledgement.

#### Scenario: Operator requests retry before reconciliation
- GIVEN publication-unknown state
- WHEN an operator requests retry without the required acknowledgement and current facts
- THEN Lattice rejects the request

### Requirement: Persistence uncertainty validation
r[lattice.persistence_uncertainty.validation] Lattice MUST test positive publication and negative commit, crash, repair, cache, corruption, missing, conflict, retry, and dispatch cases.

#### Scenario: Commit crash fixture runs
- GIVEN a process-death point during reservation commit
- WHEN the storage-fault suite restarts Lattice
- THEN no effect dispatch occurs before reconciliation

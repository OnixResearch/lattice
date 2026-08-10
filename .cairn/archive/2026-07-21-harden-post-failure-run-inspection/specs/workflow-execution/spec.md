## MODIFIED Requirements

### Requirement: Safe persisted-run inspection
r[lattice.workflow_execution.safe_run_inspection] Lattice MUST provide a read-only operator command that inspects one persisted run through a content-omitting report, validates every present evidence binding, and distinguishes absent historical evidence from invalid evidence.

#### Scenario: Safe text and JSON reports
r[lattice.workflow_execution.safe_run_inspection.scenario.safe_output]
- GIVEN a persisted run has arbitrary node labels, port labels, output text, error text, replay summaries, or orchestration private content
- WHEN an operator invokes `runs show <run-id>` with text or JSON output
- THEN Lattice MUST report only bounded run identifiers, lifecycle facts, node status/count facts, owner/dispatch identities, budget facts, evidence states, and diagnostic codes without emitting that arbitrary content

#### Scenario: Exact orchestration owner binding
r[lattice.workflow_execution.safe_run_inspection.scenario.owner_binding]
- GIVEN a run record names an orchestration instance and dispatch
- WHEN Lattice constructs the inspection report
- THEN exactly one admitted dispatch MUST match the instance ID, dispatch ID, and run ID before dispatch, recovery, abandonment, or budget evidence is attributed to the run

#### Scenario: Present replay evidence is validated
r[lattice.workflow_execution.safe_run_inspection.scenario.replay]
- GIVEN run replay or orchestration event evidence is present
- WHEN Lattice inspects the run
- THEN it MUST validate exact run identity and existing chain invariants, report valid or invalid state without event summaries, and exit unsuccessfully for invalid present evidence

#### Scenario: Historical optional evidence is absent
r[lattice.workflow_execution.safe_run_inspection.scenario.not_recorded]
- GIVEN a persisted legacy run has no detail record or no run replay record
- WHEN Lattice inspects the run
- THEN it MUST report the missing surface as `not_recorded` without claiming validity and MUST NOT treat absence alone as contradictory evidence

#### Scenario: Missing persisted run fails explicitly
r[lattice.workflow_execution.safe_run_inspection.scenario.missing]
- GIVEN no `RunRecord` exists for the requested identity
- WHEN an operator invokes `runs show <run-id>`
- THEN Lattice MUST exit unsuccessfully with a bounded not-found diagnostic and MUST NOT scan unrelated orchestration history or synthesize a run from an unobserved dispatch

#### Scenario: Active store owner is distinguished
r[lattice.workflow_execution.safe_run_inspection.scenario.active-owner]
- GIVEN another process currently owns the redb store
- WHEN an operator invokes `runs show <run-id>`
- THEN Lattice MUST exit unsuccessfully with a fixed active-owner diagnostic and MUST NOT emit raw lock, path, process, or backend error content

#### Scenario: Recovery-required store remains unchanged
r[lattice.workflow_execution.safe_run_inspection.scenario.recovery-required]
- GIVEN a prior writer exited uncleanly and redb requires write-mode recovery
- WHEN an operator invokes `runs show <run-id>`
- THEN Lattice MUST exit unsuccessfully with a fixed recovery-required diagnostic, direct the operator to explicit recovery, and MUST NOT change the store

## ADDED Requirements

### Requirement: Explicit run-store recovery
r[lattice.workflow_execution.run_store_recovery] Lattice MUST expose explicit existing-store-only redb recovery separately from read-only run inspection.

#### Scenario: Explicit recovery enables inspection
r[lattice.workflow_execution.run_store_recovery.scenario.success]
- GIVEN an existing store requires redb write-mode recovery and no process currently owns it
- WHEN an operator invokes `runs repair`
- THEN Lattice MUST use redb-owned recovery, create no Lattice application records, report fixed bounded success, and allow subsequent `runs show` to validate the recovered application evidence

#### Scenario: Recovery refuses unsafe targets
r[lattice.workflow_execution.run_store_recovery.scenario.refusal]
- GIVEN the configured store is missing, actively owned, unsupported, or unavailable
- WHEN an operator invokes `runs repair`
- THEN Lattice MUST exit unsuccessfully without creating a store, bypassing active ownership, or emitting raw path, process, operating-system, or backend error content

#### Scenario: Recovery has bounded claims
r[lattice.workflow_execution.run_store_recovery.scenario.non-claim]
- GIVEN redb successfully reopens and repairs an existing store
- WHEN Lattice reports recovery success
- THEN it MUST NOT claim semantic correctness, complete history, external anchoring, provider truth, remote exactly-once effects, or production readiness

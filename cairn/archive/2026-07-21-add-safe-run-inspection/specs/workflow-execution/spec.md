## ADDED Requirements

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

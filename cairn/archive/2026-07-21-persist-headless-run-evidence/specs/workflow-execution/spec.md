# Workflow Execution Specification Delta

## ADDED Requirements

### Requirement: Redaction-safe headless run details
r[lattice.workflow_execution.headless_run_details] Lattice MUST persist terminal node-level details for headless workflow runs through the shared run-history model and MUST replace sensitive value output with a redaction marker before persistence.

#### Scenario: Successful headless output remains inspectable
r[lattice.workflow_execution.headless_run_details.scenario.success]
- GIVEN a headless workflow completes with non-sensitive node outputs
- WHEN Lattice persists terminal run history
- THEN it MUST store node status, cache state, port identity, output text, and error state in a `RunDetailRecord` bound to the run identity

#### Scenario: Failed headless node remains inspectable
r[lattice.workflow_execution.headless_run_details.scenario.failure]
- GIVEN a headless workflow terminates with a failed, skipped, pending, or running node result
- WHEN Lattice constructs terminal run details
- THEN it MUST preserve the bounded final node status and error summary without inventing successful output

#### Scenario: Sensitive output fails closed
r[lattice.workflow_execution.headless_run_details.scenario.redaction]
- GIVEN a terminal node output value carries one or more sensitive fragments
- WHEN the shared run-history core constructs durable output details
- THEN it MUST persist a fixed redaction marker instead of any plaintext display fragment

#### Scenario: Orchestration-owned run uses the same evidence model
r[lattice.workflow_execution.headless_run_details.scenario.orchestration]
- GIVEN an admitted orchestration trigger executes a headless workflow under an exact run owner
- WHEN the run reaches a terminal state
- THEN the store MUST contain one owner-bound run record, corresponding redaction-safe node details, and replay events without creating a second evidence authority

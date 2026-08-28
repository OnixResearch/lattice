# Workflow Interchange Exit Classification Delta

## MODIFIED Requirements

### Requirement: Saved workflow runtime binding

r[lattice.workflow_interchange.application_execution] Lattice MUST execute only the exact route-bound saved workflow revision through its existing headless runtime. It MUST persist running state before effects and MUST bind route-exact authority to the admitted request, run, and revision. Workflow exchange MUST NOT classify a completed node as successful when its typed exit-code output is nonzero, missing, duplicated, or malformed.

#### Scenario: Bound workflow completes

r[lattice.workflow_interchange.application_execution.scenario.completed]
- GIVEN route admission and current authority pass
- WHEN the worker executes the saved workflow
- THEN terminal node and execution observations MUST preserve the consumer request linkage

#### Scenario: Route effect authority is unavailable

r[lattice.workflow_interchange.application_execution.scenario.authority]
- GIVEN the route does not grant exact workflow effect authority
- WHEN an effectful node reaches dispatch
- THEN the existing effect gate MUST deny it without a fallback executor

#### Scenario: Bound shell exits nonzero

r[lattice.workflow_interchange.application_execution.scenario.nonzero]
- GIVEN a route-bound shell node returns a nonzero typed exit code
- WHEN workflow exchange projects terminal completion
- THEN the node and execution MUST be terminal failures
- AND the observation MUST NOT claim success

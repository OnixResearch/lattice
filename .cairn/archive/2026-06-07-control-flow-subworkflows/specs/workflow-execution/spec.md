## ADDED Requirements

### Requirement: Conditional control-flow nodes
r[n8nclone.workflow_execution.control_flow.condition] The system MUST route values through conditional branch nodes using deterministic typed predicates evaluated without side effects.

#### Scenario: Matching branch emits output
r[n8nclone.workflow_execution.control_flow.condition.match]
- GIVEN a condition node receives an input value that satisfies one configured branch predicate
- WHEN the workflow runner evaluates the node
- THEN the system emits the input value only on the matching branch output port

#### Scenario: No matching branch is recoverable
r[n8nclone.workflow_execution.control_flow.condition.no_match]
- GIVEN a condition node receives an input value that satisfies no configured branch predicate
- WHEN the workflow runner evaluates the node
- THEN the system completes the node with no branch outputs and records a safe summary explaining that no branch matched

### Requirement: Merge control-flow nodes
r[n8nclone.workflow_execution.control_flow.merge] The system MUST merge multiple upstream values according to an explicit deterministic fan-in policy.

#### Scenario: All-input merge waits for dependencies
r[n8nclone.workflow_execution.control_flow.merge.all]
- GIVEN a merge node declares the all-input policy
- WHEN only some upstream dependencies have completed
- THEN the system keeps the merge node pending until every required upstream value is available or a dependency fails

#### Scenario: First-input merge is deterministic
r[n8nclone.workflow_execution.control_flow.merge.first]
- GIVEN a merge node declares the first-input policy and more than one upstream value becomes available
- WHEN the runner admits the merge result
- THEN the system selects the value with the lowest deterministic runner event sequence among available inputs

### Requirement: Subworkflow invocation nodes
r[n8nclone.workflow_execution.subworkflows.invocation] The system MUST invoke saved workflows as typed subworkflow nodes through the runner shell.

#### Scenario: Subworkflow outputs route to parent
r[n8nclone.workflow_execution.subworkflows.invocation.outputs]
- GIVEN a parent workflow contains a subworkflow node with valid input and output mappings
- WHEN the parent workflow runs
- THEN the system executes the referenced child workflow and maps declared child outputs to the parent node's real output port IDs

#### Scenario: Recursive subworkflow is rejected
r[n8nclone.workflow_execution.subworkflows.invocation.recursive]
- GIVEN a subworkflow reference would invoke a workflow already present in the current invocation ancestry
- WHEN runner admission runs
- THEN the system rejects the subworkflow before executing child workflow side effects

### Requirement: Subworkflow runner events
r[n8nclone.workflow_execution.subworkflows.events] The system SHOULD correlate parent and child run events for subworkflow execution.

#### Scenario: Child events keep parent context
r[n8nclone.workflow_execution.subworkflows.events.correlation]
- GIVEN a subworkflow node starts a child workflow run
- WHEN child runner events are emitted
- THEN the system includes parent run identity and subworkflow node identity in safe event metadata

## ADDED Requirements

### Requirement: Control-flow validation
r[n8nclone.workflow_validation.control_flow] The system MUST validate condition and merge node configuration before executing side-effecting runtime calls.

#### Scenario: Invalid predicate is fatal
r[n8nclone.workflow_validation.control_flow.invalid_predicate]
- GIVEN a condition node declares a malformed predicate expression
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic with the affected node and branch identity

#### Scenario: Incompatible merge inputs are fatal
r[n8nclone.workflow_validation.control_flow.merge_types]
- GIVEN a merge node declares a single typed output but receives incompatible upstream input type tags
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic before the workflow can run

#### Scenario: Missing branch output is fatal
r[n8nclone.workflow_validation.control_flow.missing_branch]
- GIVEN a condition node predicate references a branch output that is not declared on the graph node
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic and preserves workflow configuration

### Requirement: Subworkflow validation
r[n8nclone.workflow_validation.subworkflows] The system MUST validate subworkflow references and interface mappings before invoking child workflows.

#### Scenario: Missing subworkflow reference is fatal
r[n8nclone.workflow_validation.subworkflows.missing]
- GIVEN a subworkflow node references a saved workflow that cannot be resolved
- WHEN validation or runner admission runs
- THEN the system reports a fatal diagnostic before executing any parent or child side effects

#### Scenario: Invalid subworkflow mapping is fatal
r[n8nclone.workflow_validation.subworkflows.mapping]
- GIVEN a subworkflow node maps a parent input or child output to an unknown or incompatible port
- WHEN validation runs
- THEN the system reports a fatal diagnostic with the affected mapping identity

#### Scenario: Recursive subworkflow reference is fatal
r[n8nclone.workflow_validation.subworkflows.recursive]
- GIVEN subworkflow references form a recursive invocation chain
- WHEN runner admission evaluates the workflow ancestry
- THEN the system reports a fatal diagnostic and does not start the recursive child run

## ADDED Requirements

### Requirement: Control-flow Nickel format
r[n8nclone.workflow_format.control_flow] The system MUST preserve condition, merge, and subworkflow node configuration in typed Nickel workflow import and export.

#### Scenario: Export control-flow workflow
r[n8nclone.workflow_format.control_flow.export]
- GIVEN a saved workflow contains condition, merge, or subworkflow nodes
- WHEN the operator exports it to Nickel
- THEN the system writes predicate definitions, branch labels, merge policies, subworkflow references, and interface mappings deterministically

#### Scenario: Import control-flow workflow
r[n8nclone.workflow_format.control_flow.import]
- GIVEN a Nickel workflow file declares valid condition, merge, and subworkflow nodes
- WHEN the operator imports it
- THEN the system saves a workflow with equivalent typed topology and node configuration

### Requirement: Control-flow format compatibility
r[n8nclone.workflow_format.control_flow.compatibility] The system SHOULD keep existing Nickel workflows valid when control-flow fields are absent.

#### Scenario: Existing workflow round trip remains stable
r[n8nclone.workflow_format.control_flow.compatibility.existing]
- GIVEN a Nickel workflow file contains only existing shell, HTTP, file, image, or agent nodes
- WHEN the workflow is imported and exported after control-flow support is added
- THEN the system preserves the original runnable meaning without requiring new control-flow fields

#### Scenario: Malformed control-flow input is rejected
r[n8nclone.workflow_format.control_flow.compatibility.invalid]
- GIVEN a Nickel workflow file declares malformed control-flow configuration
- WHEN import validation runs
- THEN the system rejects the workflow and preserves existing saved workflows

## ADDED Requirements

### Requirement: Workflow validation diagnostics
r[n8nclone.workflow_validation.diagnostics.core] The system MUST validate workflow graph and node configuration before execution.

#### Scenario: Valid workflow has no fatal diagnostics
r[n8nclone.workflow_validation.diagnostics.core.valid]
- GIVEN a workflow has runnable nodes, compatible edges, and complete node configuration
- WHEN workflow validation runs
- THEN the system returns no fatal diagnostics

#### Scenario: Empty shell command is fatal
r[n8nclone.workflow_validation.diagnostics.core.empty_command]
- GIVEN a shell node has an empty command
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic for that node

#### Scenario: Missing node configuration is fatal
r[n8nclone.workflow_validation.diagnostics.core.missing_config]
- GIVEN a graph node has no execution configuration
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic for that node

### Requirement: Validation run gate
r[n8nclone.workflow_validation.run_gate] The system MUST prevent workflow execution when fatal validation diagnostics exist.

#### Scenario: Fatal diagnostics block run
r[n8nclone.workflow_validation.run_gate.block]
- GIVEN workflow validation returns at least one fatal diagnostic
- WHEN the user or operator starts a run
- THEN the system refuses to execute nodes and reports the blocking diagnostics

#### Scenario: Warnings allow run
r[n8nclone.workflow_validation.run_gate.warning]
- GIVEN workflow validation returns warnings but no fatal diagnostics
- WHEN the user or operator starts a run
- THEN the system allows execution and keeps warnings visible

### Requirement: TUI validation panel
r[n8nclone.workflow_validation.tui.panel] The system SHOULD expose current workflow diagnostics inside the TUI.

#### Scenario: Open diagnostics panel
r[n8nclone.workflow_validation.tui.panel.open]
- GIVEN validation diagnostics exist
- WHEN the user opens the diagnostics view
- THEN the system lists diagnostics with severity, location, and actionable text

#### Scenario: No diagnostics message
r[n8nclone.workflow_validation.tui.panel.empty]
- GIVEN validation returns no diagnostics
- WHEN the user opens the diagnostics view
- THEN the system reports that the workflow is ready to run

### Requirement: Validation negative coverage
r[n8nclone.workflow_validation.diagnostics.negative] The system MUST verify invalid workflow inputs fail with stable diagnostics.

#### Scenario: Invalid edge diagnostic
r[n8nclone.workflow_validation.diagnostics.negative.invalid_edge]
- GIVEN a workflow contains an edge that references missing or incompatible ports
- WHEN workflow validation runs
- THEN the system reports a stable diagnostic and does not panic

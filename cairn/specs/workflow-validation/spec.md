# Workflow Validation Specification

## Purpose

Defines the `workflow-validation` capability.

## Requirements

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

### Requirement: Generic runner admission diagnostics
r[n8nclone.workflow_validation.generic_runner_admission] The system MUST validate generic runner admission before executing side-effecting runtime calls.

#### Scenario: Runner admission blocks invalid graph
r[n8nclone.workflow_validation.generic_runner_admission.graph]
- GIVEN a workflow has invalid edge endpoints, a cycle, incompatible port types, or dependencies that cannot become ready
- WHEN generic runner admission runs
- THEN the system MUST report fatal diagnostics before invoking any runtime

#### Scenario: Missing runtime binding
r[n8nclone.workflow_validation.generic_runner_admission.runtime]
- GIVEN a workflow node references a missing runtime kind or runtime key
- WHEN generic runner admission runs
- THEN the system MUST report the missing runtime binding with the affected node identity before invoking any runtime

#### Scenario: Effect and resource policy rejection
r[n8nclone.workflow_validation.generic_runner_admission.effects]
- GIVEN a workflow node declares runtime effects, abilities, resource bounds, or sandbox requirements that are not provided by the selected runtime policy
- WHEN generic runner admission runs
- THEN the system SHOULD reject execution with deterministic diagnostics aligned with Trellis ability/effect/resource admission semantics where build and license fit

#### Scenario: Terminal state mutation rejection
r[n8nclone.workflow_validation.generic_runner_admission.terminal_state]
- GIVEN a persisted or resumed run record marks a step completed, failed, or cancelled
- WHEN generic runner admission evaluates a transition that would mutate that terminal step state
- THEN the system MUST reject the transition as fatal for the run and MUST NOT mutate persisted workflow configuration

### Requirement: Node-kind provider validation
r[n8nclone.workflow_validation.node_kind_provider] The system MUST validate node-specific configuration and runner bindings through registered node-kind and runtime providers when providers exist.

#### Scenario: Provider-backed config validation
r[n8nclone.workflow_validation.node_kind_provider.config_validation]
- GIVEN a workflow contains built-in shell, HTTP, file, image, or generic agent nodes
- WHEN workflow validation runs
- THEN the system MUST obtain node-specific validation diagnostics from the matching node-kind provider and preserve existing diagnostic severity, location, and message stability

#### Scenario: Runtime binding admission diagnostics
r[n8nclone.workflow_validation.node_kind_provider.runtime_binding]
- GIVEN a runner plan references a missing runtime binding or a runtime that does not satisfy the node-kind provider contract
- WHEN runner-admission validation runs
- THEN the system MUST report a fatal diagnostic before any process, network, filesystem, database, clock, or UI side effect occurs

#### Scenario: Provider fallback preserves safety
r[n8nclone.workflow_validation.node_kind_provider.fallback]
- GIVEN a built-in node kind has not yet migrated every validation rule to a provider
- WHEN workflow validation runs during the migration
- THEN the system MUST keep the existing validation branch active as a guarded fallback and MUST NOT silently drop fatal diagnostics

### Requirement: n8n JSON interoperability diagnostics
r[n8nclone.workflow_validation.n8n_json.unsupported] The system MUST report unsupported n8n import features with stable diagnostics before saving imported workflows.

#### Scenario: Unsupported n8n node kind
r[n8nclone.workflow_validation.n8n_json.unsupported.node_kind]
- GIVEN an n8n JSON workflow contains a node kind with no supported n8nclone mapping
- WHEN import validation runs
- THEN the system reports the unsupported node kind with the source node identity

#### Scenario: Unsupported n8n expression
r[n8nclone.workflow_validation.n8n_json.unsupported.expression]
- GIVEN an n8n JSON workflow contains an expression that cannot be translated to n8nclone expression syntax
- WHEN import validation runs
- THEN the system reports the unsupported expression and does not execute it

#### Scenario: Malformed n8n connection
r[n8nclone.workflow_validation.n8n_json.unsupported.connection]
- GIVEN an n8n JSON workflow references a missing source or target node in a connection
- WHEN import validation runs
- THEN the system reports a stable connection diagnostic and preserves existing saved workflows

### Requirement: n8n JSON export limits
r[n8nclone.workflow_validation.n8n_json.export_limits] The system MUST report unsupported export features before writing n8n JSON.

#### Scenario: Unsupported export node
r[n8nclone.workflow_validation.n8n_json.export_limits.node]
- GIVEN a n8nclone workflow contains a node with no known n8n export mapping
- WHEN export validation runs
- THEN the system reports the unsupported node and refuses default export

### Requirement: Replay record validation
r[n8nclone.workflow_validation.replay_records] The system MUST validate replay records and bundles before presenting them as trusted run timelines.

#### Scenario: Out-of-order event is rejected
r[n8nclone.workflow_validation.replay_records.order]
- GIVEN a replay bundle contains duplicate or out-of-order event sequence numbers
- WHEN replay validation runs
- THEN the system reports a fatal diagnostic and refuses to import the malformed timeline

#### Scenario: Terminal-state mutation is rejected
r[n8nclone.workflow_validation.replay_records.terminal]
- GIVEN replay events attempt to move a completed, failed, or cancelled node back to a non-terminal state
- WHEN replay validation runs
- THEN the system reports a fatal diagnostic and does not display the invalid state transition as trusted

#### Scenario: Unknown artifact reference is recoverable
r[n8nclone.workflow_validation.replay_records.artifact]
- GIVEN a replay record references an artifact whose payload is not included
- WHEN replay validation runs
- THEN the system accepts the record if the reference metadata is well formed and marks the payload unavailable

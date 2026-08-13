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
- GIVEN an n8n JSON workflow contains a node kind with no supported Lattice mapping
- WHEN import validation runs
- THEN the system reports the unsupported node kind with the source node identity

#### Scenario: Unsupported n8n expression
r[n8nclone.workflow_validation.n8n_json.unsupported.expression]
- GIVEN an n8n JSON workflow contains an expression that cannot be translated to Lattice expression syntax
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
- GIVEN a Lattice workflow contains a node with no known n8n export mapping
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

### Requirement: Credential provider validation
r[n8nclone.workflow_validation.credentials.providers] The system MUST validate credential provider references and capabilities before node execution.

#### Scenario: Unsupported field access is fatal
r[n8nclone.workflow_validation.credentials.providers.field]
- GIVEN a credential reference requests a field from a provider that does not support field access
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic before execution starts

#### Scenario: Malformed provider key is fatal
r[n8nclone.workflow_validation.credentials.providers.key]
- GIVEN a credential reference contains a malformed key for its provider
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic with safe provider and key context

#### Scenario: Missing provider config is fatal
r[n8nclone.workflow_validation.credentials.providers.config]
- GIVEN workflow execution requires a credential provider that is absent from runtime configuration
- WHEN validation or runner admission runs
- THEN the system reports a fatal diagnostic before invoking side-effecting runtimes

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

### Requirement: n8n parity diagnostics
r[n8nclone.workflow_validation.n8n_parity_diagnostics] The system MUST reject unsupported n8n parity fixtures with deterministic, actionable diagnostics that identify the unsupported feature class.

#### Scenario: Unsupported node diagnostic
r[n8nclone.workflow_validation.n8n_parity_diagnostics.unsupported_node]
- GIVEN a parity fixture contains an unsupported n8n node kind
- WHEN import validation runs
- THEN the diagnostic identifies the unsupported node kind without panicking or persisting a partial workflow

#### Scenario: Unsupported expression diagnostic
r[n8nclone.workflow_validation.n8n_parity_diagnostics.unsupported_expression]
- GIVEN a parity fixture contains unsupported n8n expression syntax
- WHEN import validation runs
- THEN the diagnostic identifies the unsupported expression class and keeps the original expression out of unsafe execution paths

#### Scenario: Malformed connection diagnostic
r[n8nclone.workflow_validation.n8n_parity_diagnostics.malformed_connection]
- GIVEN a parity fixture contains malformed n8n connection data
- WHEN import validation runs
- THEN the diagnostic identifies the malformed connection and rejects the fixture deterministically

#### Scenario: Plaintext credential diagnostic
r[n8nclone.workflow_validation.n8n_parity_diagnostics.plaintext_credential]
- GIVEN a parity fixture contains plaintext credential material
- WHEN import validation runs
- THEN the diagnostic rejects the fixture and reports only safe credential-placeholder guidance

### Requirement: Workflow boundary ownership is explicit
r[lattice.workflow_validation.domain_boundary.ownership] Lattice MUST classify selected workflow values and decisions as Lattice product policy, Choregraph graph meaning, Subwayrat UI state, or boundary conversion.

#### Scenario: Ownership inventory classifies each selected graph value
r[lattice.workflow_validation.domain_boundary.ownership.complete]
- GIVEN a selected workflow, validation, execution, replay, history, expression, or persistence path uses a graph value
- WHEN boundary review runs
- THEN the value MUST have one owner and one allowed dependency direction
- AND an unclassified value MUST block migration completion.

### Requirement: The workflow migration inventory is maintained
r[lattice.workflow_validation.domain_boundary.inventory] The maintained inventory MUST cover every selected `rat_nodegraph` type in non-TUI workflow signatures and state.

#### Scenario: Inventory omits a selected graph value
r[lattice.workflow_validation.domain_boundary.inventory.complete]
- GIVEN a selected non-TUI workflow path uses a graph value
- WHEN the maintained migration inventory is checked
- THEN the inventory MUST name the prior value and its current owner
- AND an omitted selected value MUST block closeout.

### Requirement: The workflow core uses application-owned values
r[lattice.workflow_validation.domain_boundary.application_model] Workflow aggregate state, mutation inputs, mutation plans, and domain errors MUST use Lattice-owned types.

#### Scenario: Lattice-owned mutation is accepted
r[lattice.workflow_validation.domain_boundary.application_model.accepted]
- GIVEN a valid Lattice workflow state and a valid Lattice mutation request
- WHEN the deterministic workflow core evaluates the request
- THEN it MUST return the same accepted state or plan for the same explicit inputs.

### Requirement: Workflow time is explicit
r[lattice.workflow_validation.domain_boundary.time] Workflow creation and metadata mutation MUST receive a validated timestamp as an explicit input.

#### Scenario: Explicit timestamp creates deterministic metadata
r[lattice.workflow_validation.domain_boundary.time.deterministic]
- GIVEN equal workflow inputs and an equal explicit UTC timestamp
- WHEN workflow creation runs twice
- THEN both results MUST contain equal created and modified metadata.

#### Scenario: Core attempts an ambient clock read
r[lattice.workflow_validation.domain_boundary.enforcement.clock]
- GIVEN a declared workflow-core scope calls `chrono::Utc::now`, `SystemTime::now`, `Instant::now`, or another ambient clock
- WHEN architecture validation runs
- THEN validation MUST fail with the source location and effect class.

### Requirement: Subwayrat remains a UI adapter
r[lattice.workflow_validation.domain_boundary.subwayrat] `rat_nodegraph` model and view types MUST remain inside declared Subwayrat TUI adapter scopes.

#### Scenario: UI type enters a workflow-core signature
r[lattice.workflow_validation.domain_boundary.subwayrat.leak]
- GIVEN a workflow-core field, argument, result, error, map, or persisted record exposes a `rat_nodegraph` type
- WHEN dependency-direction validation runs
- THEN validation MUST fail before closeout.

### Requirement: Choregraph remains the graph-decision owner
r[lattice.workflow_validation.domain_boundary.choregraph] Lattice MUST convert admitted workflow values through one explicit pinned Choregraph boundary when graph or machine admission is required.

#### Scenario: Choregraph rejects the mapped graph
r[lattice.workflow_validation.domain_boundary.choregraph.rejected]
- GIVEN Lattice values map to a graph that the pinned Choregraph contract rejects
- WHEN graph admission runs
- THEN the adapter MUST return a distinct bounded adapter error
- AND Lattice MUST NOT execute workflow effects.

### Requirement: Boundary formats remain compatible
r[lattice.workflow_validation.domain_boundary.compatibility] Explicit adapters MUST preserve supported Nickel, JSON, redb, CLI, TUI, and replay behavior during migration.

#### Scenario: Supported persisted workflow remains readable
r[lattice.workflow_validation.domain_boundary.compatibility.persisted]
- GIVEN a supported workflow record from before migration
- WHEN the new boundary adapter loads and validates it
- THEN Lattice MUST reconstruct equivalent admitted workflow meaning without running effects.

#### Scenario: Malformed boundary record does not mutate workflow state
r[lattice.workflow_validation.domain_boundary.validation.malformed]
- GIVEN a Nickel, JSON, redb, Choregraph, or Subwayrat boundary value is malformed, stale, incompatible, or unsupported
- WHEN conversion runs
- THEN conversion MUST fail with an adapter-owned error
- AND prior workflow state MUST remain unchanged.

### Requirement: Architecture guards remain active
r[lattice.workflow_validation.domain_boundary.enforcement] Maintained source and Octet checks MUST reject ambient time and Subwayrat type leakage in declared workflow-core scopes.

#### Scenario: Shell supplies one timestamp per mutation
r[lattice.workflow_validation.domain_boundary.validation.shell_time]
- GIVEN a CLI or TUI action requests one workflow metadata mutation
- WHEN the shell handles the action
- THEN it MUST obtain one timestamp and pass that value into the core
- AND the core MUST NOT read time again.

### Requirement: Boundary tests pair accepted and rejected cases
r[lattice.workflow_validation.domain_boundary.validation] Maintained tests MUST pair accepted core and adapter cases with rejected and malformed cases.

#### Scenario: Adapter test lacks a rejected case
r[lattice.workflow_validation.domain_boundary.validation.paired]
- GIVEN a maintained test covers an accepted core or adapter input
- WHEN validation coverage is checked
- THEN a rejected, malformed, stale, incompatible, or unsupported case MUST also be present.

### Requirement: Boundary documentation names each owner
r[lattice.workflow_validation.domain_boundary.docs] Documentation MUST name Lattice, Choregraph, Subwayrat, persistence, and time ownership.

#### Scenario: Boundary documentation omits an owner
r[lattice.workflow_validation.domain_boundary.docs.owners]
- GIVEN the workflow boundary documentation is reviewed
- WHEN one required owner or dependency direction is absent
- THEN closeout MUST fail until the missing boundary is documented.

### Requirement: Boundary claims remain scoped
r[lattice.workflow_validation.domain_boundary.final_checks] Closeout evidence MUST retain workflow-correctness, provider, effect-safety, and release-readiness non-claims.

#### Scenario: Boundary claims remain scoped
r[lattice.workflow_validation.domain_boundary.final_checks.claims]
- GIVEN dependency, compatibility, and test checks pass
- WHEN Lattice states the result
- THEN it MUST NOT claim whole-workflow correctness, safe external effects, provider correctness, or release readiness.

# Workflow Validation Specification Delta

## ADDED Requirements

### Requirement: Workflow boundary ownership is explicit

r[lattice.workflow_validation.domain_boundary.ownership] Lattice MUST classify selected workflow values and decisions as Lattice product policy, Choregraph graph meaning, Subwayrat UI state, or boundary conversion.

r[lattice.workflow_validation.domain_boundary.inventory] The maintained inventory MUST cover every selected `rat_nodegraph` type in non-TUI workflow signatures and state.

#### Scenario: Ownership inventory classifies each selected graph value

r[lattice.workflow_validation.domain_boundary.ownership.complete]
- GIVEN a selected workflow, validation, execution, replay, history, expression, or persistence path uses a graph value
- WHEN boundary review runs
- THEN the value MUST have one owner and one allowed dependency direction
- AND an unclassified value MUST block migration completion.

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

r[lattice.workflow_validation.domain_boundary.validation] Maintained tests MUST pair accepted core and adapter cases with rejected and malformed cases.

#### Scenario: Shell supplies one timestamp per mutation

r[lattice.workflow_validation.domain_boundary.validation.shell_time]
- GIVEN a CLI or TUI action requests one workflow metadata mutation
- WHEN the shell handles the action
- THEN it MUST obtain one timestamp and pass that value into the core
- AND the core MUST NOT read time again.

### Requirement: Boundary claims remain scoped

r[lattice.workflow_validation.domain_boundary.docs] Documentation MUST name Lattice, Choregraph, Subwayrat, persistence, and time ownership.

r[lattice.workflow_validation.domain_boundary.final_checks] Closeout evidence MUST retain workflow-correctness, provider, effect-safety, and release-readiness non-claims.

#### Scenario: Boundary claims remain scoped

r[lattice.workflow_validation.domain_boundary.final_checks.claims]
- GIVEN dependency, compatibility, and test checks pass
- WHEN Lattice states the result
- THEN it MUST NOT claim whole-workflow correctness, safe external effects, provider correctness, or release readiness.

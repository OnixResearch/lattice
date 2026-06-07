## ADDED Requirements

### Requirement: Generic pipeline runner core
r[n8nclone.workflow_execution.generic_pipeline_runner] The system MUST provide a generic typed workflow/pipeline runner that separates pure DAG admission and scheduling from side-effecting runtime execution.

#### Scenario: Runner IR plan creation
r[n8nclone.workflow_execution.runner_core.plan_ir]
- GIVEN a persisted workflow graph with typed nodes, ports, edges, node configuration, credential references, and runtime bindings
- WHEN runner planning starts
- THEN the system MUST convert it into a stable in-memory runner plan without performing filesystem, process, network, clock, database, or UI operations

#### Scenario: Trellis-aligned graph admission
r[n8nclone.workflow_execution.runner_core.trellis_graph_admission]
- GIVEN a runner plan has finite nodes, edges, dependencies, and capacity policy
- WHEN pure runner admission evaluates the plan
- THEN the system SHOULD align edge endpoint, dependency readiness, topological, cycle, and capacity decisions with the matching Trellis graph, job DAG, job state, and dependency scheduler primitive semantics where build and license fit

#### Scenario: Step lifecycle admission
r[n8nclone.workflow_execution.runner_core.lifecycle]
- GIVEN a runner step is pending, ready, running, completed, failed, or cancelled
- WHEN the runner applies a lifecycle transition
- THEN the system MUST admit only valid transitions, MUST reject self-transitions and terminal-state mutation, and MUST keep rejected transitions free of side effects

#### Scenario: Compatibility facade
r[n8nclone.workflow_execution.runner_core.compatibility]
- GIVEN existing TUI, headless, scheduler, and test entrypoints invoke workflow execution
- WHEN the generic runner core and shell are introduced
- THEN those entrypoints MUST continue to execute existing shell, HTTP, file, image, and generic agent workflows with equivalent typed outputs and cache behavior

### Requirement: Pluggable node runtime registry
r[n8nclone.workflow_execution.runtime_registry] The system MUST execute node side effects through a runtime registry instead of executor-local node branches while preserving typed node semantics.

#### Scenario: Runtime dispatch
r[n8nclone.workflow_execution.runtime_registry.dispatch]
- GIVEN a runner step references a node kind and runtime binding present in the registry
- WHEN the runner shell invokes the step
- THEN the system MUST dispatch to the matching runtime implementation with a typed invocation plan

#### Scenario: Runtime output contract
r[n8nclone.workflow_execution.runtime_registry.typed_output_contract]
- GIVEN a runtime returns output values for a step
- WHEN the runner admits the runtime result
- THEN every output MUST be keyed by a declared output port id and MUST match that port's declared type tag before values are stored or routed downstream

#### Scenario: Runtime errors remain node-scoped
r[n8nclone.workflow_execution.runtime_registry.errors]
- GIVEN a runtime binding is missing, a runtime returns an undeclared output port, a runtime returns a value with an incompatible type, or a runtime fails
- WHEN the workflow runs
- THEN the system MUST fail only the affected step with a clear diagnostic and MUST NOT corrupt workflow configuration or unrelated completed values

#### Scenario: Generic command runtime
r[n8nclone.workflow_execution.runtime_registry.command_runtime]
- GIVEN a shell-like or agent-like node is backed by a configured command runtime
- WHEN the runner dispatches the node
- THEN the system MUST use the generic command runtime contract for command, arguments, working directory, stdin, stdout/stderr, output envelope decoding, timeout, and result admission

### Requirement: Unified runner events
r[n8nclone.workflow_execution.runner_events] The system MUST expose workflow execution progress as a unified ordered event stream usable by TUI, headless, scheduler, and tests.

#### Scenario: Event stream lifecycle
r[n8nclone.workflow_execution.runner_events.stream]
- GIVEN a workflow run starts and nodes become ready, start, use cache, produce artifacts, complete, fail, or cancel
- WHEN the runner shell processes those decisions
- THEN the system MUST emit deterministic runner events with stable run id, node id, optional port id, event kind, sequence ordering, and safe summaries

#### Scenario: Event redaction
r[n8nclone.workflow_execution.runner_events.redaction]
- GIVEN runtime outputs or inputs contain sensitive fragments or secret-derived values
- WHEN runner events, TUI output, headless summaries, or persisted run details render those values
- THEN the system MUST preserve redaction metadata and MUST NOT expose sensitive fragments in unsafe summaries

### Requirement: Typed artifact and durable references
r[n8nclone.workflow_execution.artifact_references] The system MUST represent large or durable workflow values as typed content-addressed references while preserving inline scalar values.

#### Scenario: Artifact reference model
r[n8nclone.workflow_execution.artifact_references.model]
- GIVEN a node produces bytes, images, command-output files, or generated artifacts that exceed inline policy or require durable storage
- WHEN the runner stores the output
- THEN the system MUST create a typed reference containing BLAKE3 content hash, type or media metadata, byte length, backend reference, sensitivity metadata, and safe display summary

#### Scenario: Trellis-aligned reference identity
r[n8nclone.workflow_execution.artifact_references.trellis_identity]
- GIVEN artifact references, schema migrations, or runtime closure identities are admitted by the runner
- WHEN pure admission evaluates those references
- THEN the system SHOULD align typed-reference, closure-availability, and explicit-migration decisions with Trellis typed durable reference and content-addressed code primitives where build and license fit

### Requirement: Durable runner admission
r[n8nclone.workflow_execution.durable_runner] The system SHOULD admit retry, resume, and idempotency decisions through a deterministic durable runner core.

#### Scenario: Resume and idempotency admission
r[n8nclone.workflow_execution.durable_runner.resume]
- GIVEN a workflow run has prior events, checkpoints, consumed execution tokens, retry policy, timeout policy, and completed durable effects
- WHEN the runner decides whether to execute or skip a step during retry or resume
- THEN the system SHOULD use deterministic durable-admission logic aligned with Trellis durable job, durable resume, durable replay, idempotency, timeout, retry, and job result primitives where build and license fit

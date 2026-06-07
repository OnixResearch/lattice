## ADDED Requirements

### Requirement: Trait-driven workflow execution boundaries
r[n8nclone.workflow_execution.trait_boundaries] The system MUST keep persisted workflow configuration as serializable data while routing side-effecting workflow behavior through narrow trait-backed runtime, process, storage, and event ports.

#### Scenario: Runtime dispatch through registered runtimes
r[n8nclone.workflow_execution.trait_boundaries.runtime_dispatch]
- GIVEN a runner step has a node kind, runtime binding, declared ports, resolved settings, credentials, and input values
- WHEN the runner shell executes the step
- THEN the system MUST dispatch to a registered node runtime implementation for that binding instead of using executor-local node branches for supported built-in nodes

#### Scenario: Shared process runner
r[n8nclone.workflow_execution.trait_boundaries.process_runner]
- GIVEN a shell-like, generic command, or command-backed agent runtime needs to run an external process
- WHEN the runtime prepares execution
- THEN the system MUST route spawning through a shared process-runner interface that receives explicit program, args, cwd, env, stdin, timeout, and output-limit inputs

#### Scenario: Storage access through narrow ports
r[n8nclone.workflow_execution.trait_boundaries.storage_ports]
- GIVEN headless execution, scheduling, runner cache use, run-history persistence, or value storage needs persisted state
- WHEN those shells access storage
- THEN the system MUST depend on the narrowest workflow repository, run history, execution cache, value store, or schedule store trait needed by the operation

#### Scenario: Runner events through event sinks
r[n8nclone.workflow_execution.trait_boundaries.event_sink]
- GIVEN a workflow run starts, admits nodes, hits cache, invokes runtimes, produces outputs, fails, cancels, or completes
- WHEN the runner shell observes the lifecycle fact
- THEN the system MUST emit an ordered runner event through an event-sink interface with stable run id, node id, optional port id, event kind, safe summary, and redaction metadata

#### Scenario: Compatibility facade remains stable
r[n8nclone.workflow_execution.trait_boundaries.compatibility]
- GIVEN existing TUI, headless, scheduler, and test entrypoints call the public executor and store APIs
- WHEN trait-backed runtime, process, storage, and event ports are introduced
- THEN those entrypoints MUST preserve existing shell, HTTP, file, image, generic agent, cache, and run-history behavior

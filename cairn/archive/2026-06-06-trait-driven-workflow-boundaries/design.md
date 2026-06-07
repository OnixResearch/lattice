# Design: Trait-driven workflow boundaries

## Architecture

Keep the current persisted model as data and move behavior to explicit ports:

```text
Workflow / NodeConfig data
        │
        ▼
NodeKindProvider registry ── validation, display, edit, format metadata
        │
        ▼
RunnerPlan / RuntimeBinding
        │
        ▼
NodeRuntimeRegistry ── shell/http/file/image/agent runtime trait objects
        │
        ▼
ProcessRunner / HttpClient / FileSystem / ImageIo / AgentAdapter
        │
        ▼
RunnerEventSink + Store ports
```

`NodeConfig` remains the stable serializable representation. Traits describe behavior at shell boundaries where the engine needs to validate, display, format, execute, persist, or report progress.

## Runtime traits

Introduce a runtime registry that accepts a `RuntimeBinding` and returns a runtime implementation. The compatibility executor can keep its public API while delegating supported built-in nodes to the registry.

```rust
trait NodeRuntime: Send + Sync {
    fn binding(&self) -> RuntimeBinding;
    fn validate(&self, plan: &RuntimeValidationPlan) -> Vec<WorkflowDiagnostic>;
    fn execute(&self, invocation: RuntimeInvocation) -> RuntimeResult;
}
```

`RuntimeInvocation` carries only explicit inputs: node id, declared ports, resolved node settings, credential facts, input values, and policy. `RuntimeResult` carries declared output values or a node-scoped failure. Output admission remains a pure check before values are stored or routed downstream.

## Process runner

Shell execution and command-backed agent execution should share a process port rather than calling `tokio::process::Command` or `std::process::Command` directly from node logic.

```rust
trait ProcessRunner: Send + Sync {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError>;
}
```

`ProcessRequest` includes program, args, cwd, env, stdin bytes, timeout, and output limits. A production runner owns process spawning. Tests use fake runners that return deterministic stdout, stderr, status, timeout, and spawn-error cases.

## Node-kind providers

A `NodeKindProvider` registry centralizes node-kind metadata currently repeated across TUI, validation, and format code.

```rust
trait NodeKindProvider: Send + Sync {
    fn kind_label(&self) -> &'static str;
    fn port_schema(&self) -> NodePortSchema;
    fn editable_fields(&self, config: &NodeConfig) -> Vec<NodePropertyField>;
    fn validate_config(&self, config: &NodeConfig) -> Vec<WorkflowDiagnostic>;
    fn export_config(&self, config: &NodeConfig) -> PortableNodeConfigPatch;
    fn import_config(&self, portable: &PortableNodeConfig) -> Result<NodeConfig, WorkflowFormatError>;
}
```

The initial registry should wrap existing built-in node behavior, not introduce a plugin system. Exhaustive `NodeConfig` matches can remain as guarded fallback until all built-ins are migrated.

## Storage ports

Split concrete store use into narrow traits at the boundaries that need them:

- `WorkflowRepository`: load, save, list, delete workflows.
- `RunHistoryStore`: save/list run records and details.
- `ExecutionCache`: read/write cached output maps.
- `ValueStore`: put/get content-addressed values.
- `ScheduleStore`: load/save/list/delete schedules.

The redb `Store` implements these traits. Headless, scheduler, and runner shell functions should accept the narrowest trait required by the operation.

## Event sink

Runner progress should be an ordered stream emitted through an event sink:

```rust
trait RunnerEventSink {
    fn emit(&mut self, event: RunnerEvent);
}
```

Adapters can send events to a TUI channel, accumulate headless summaries, persist run details, or capture assertions in tests. Events must carry stable run id, node id, optional port id, sequence, kind, safe summary, and redaction metadata.

## Migration approach

1. Define traits and fake adapters beside the current concrete behavior.
2. Add registry-backed built-in adapters for shell, HTTP, file, image, and generic agent nodes.
3. Move command spawning behind `ProcessRunner` before adding more command-like runtimes.
4. Introduce storage and event traits at headless/scheduler/runner shell boundaries.
5. Move validation/TUI/format node-kind knowledge into providers incrementally.

Each step should keep existing public APIs as compatibility facades until tests prove equivalence.

## Testing

Trait boundaries need both positive and negative coverage:

- Positive: built-in workflow behavior remains equivalent through trait-backed adapters.
- Negative: missing runtime, wrong output port, output type mismatch, process timeout, process spawn error, storage failure, event sink capture, invalid node field edits, and malformed format input fail with stable diagnostics.

Use fake implementations for unit tests and existing integration tests for compatibility evidence.

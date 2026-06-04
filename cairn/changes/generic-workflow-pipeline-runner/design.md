# Design: Generic workflow/pipeline runner

## Architecture

Split execution into a functional core and imperative shell:

```text
Workflow / Nickel import
        │
        ▼
RunnerPlanBuilder  ── pure graph/type/config admission
        │
        ▼
RunnerCore         ── pure ready-set, lifecycle, cache-key, event planning
        │
        ▼
RunnerShell        ── store/cache/artifact I/O, clock, cancellation, runtime calls
        │
        ▼
NodeRuntimeRegistry ─ shell/http/file/image/agent/command/subworkflow runtimes
```

`RunnerCore` owns no process, network, filesystem, database, clock, Tokio, tracing, or UI access. It takes immutable snapshots of workflow topology, typed ports, node settings, prior step states, cache metadata, and explicit policy inputs. It returns deterministic decisions: diagnostics, ready steps, blocked steps, cache probes, event records to emit, and runtime invocation plans.

`RunnerShell` is intentionally boring. It loads workflows, persists run records, resolves cache/artifact references, invokes registered runtimes, records outputs, and feeds resulting facts back into the core.

## Runner IR

Introduce an internal runner IR independent of `rat_nodegraph` UI types:

- `RunnerWorkflow`: metadata plus stable nodes and edges.
- `RunnerNode`: stable node id, label, kind, runtime binding, input/output port schemas, node settings, credential refs, cache policy, retry policy, effect requirements.
- `RunnerPort`: stable port id, label, direction, type tag, optional sensitivity and artifact policy.
- `RunnerEdge`: source node/port to target node/port.
- `RunnerValueRef`: inline small value or typed durable/artifact reference.
- `RunnerStepState`: pending, ready, running, completed, failed, cancelled.
- `RunnerEvent`: run/node/cache/artifact/runtime lifecycle events.

The existing `Workflow` remains the persisted graph model initially. A pure adapter converts it to `RunnerWorkflow`. This keeps UI compatibility while creating a stable pipeline execution boundary.

## Runtime registry

Define a runtime interface around typed invocation plans:

```rust
trait NodeRuntime {
    fn kind(&self) -> RuntimeKind;
    fn validate(&self, plan: &RuntimeValidationPlan) -> Vec<WorkflowDiagnostic>;
    fn execute(&self, invocation: RuntimeInvocation) -> RuntimeResult;
}
```

The trait shape may become async in the imperative shell, but validation/planning remains pure. Built-ins should include shell, HTTP, file, image input, generic command, generic agent, and later subworkflow runtimes. Runtime results MUST map only to declared output port ids and type tags.

The generic command runtime becomes the common command-backed primitive. Agent Pi/Clankers presets stay typed agent nodes with runtime bindings that select command runtime templates; they do not become shell nodes.

## Event model

The runner emits a single event stream consumed by TUI, headless, scheduler, and tests:

- `RunStarted`
- `NodeReady`
- `NodeStarted`
- `CacheHit`
- `RuntimeStdout`
- `RuntimeStderr`
- `ArtifactCreated`
- `NodeCompleted`
- `NodeFailed`
- `NodeCancelled`
- `RunCompleted`

Events carry stable run id, node id, optional port id, sequence number, redaction metadata, and safe summaries. Large values are referenced by typed durable/artifact references, not inlined into event payloads.

## Artifacts and durable references

Keep BLAKE3 as the content hash algorithm. Generalize current values and image/artifact handoff into a typed durable reference model:

- content hash;
- type hash or type tag;
- codec/media type;
- byte length;
- storage backend reference;
- sensitivity metadata;
- schema/runtime closure identity when needed.

Small scalar values may remain inline. Large bytes, images, command outputs, and generated artifacts should flow as references. Cache identity uses node config hash, runtime binding hash, input value hashes, credential reference identities, and effect/capability policy hashes.

## Trellis primitive alignment

Use Trellis only at exact pure boundaries:

| Runner boundary | Trellis primitive candidates | Use |
|---|---|---|
| Edge endpoint admission | `graph_adjacency`, `graph_reachability` | Validate finite source/target ids and path witnesses where an indexed view is available. |
| Dependency readiness | `job_dag`, `job_state` | Admit Pending→Ready only when dependencies completed; failed/cancelled dependencies block readiness. |
| Ready-set scheduling | `dependency_scheduler` | Reuse or mirror acyclic/topological/capacity admission for deterministic scheduling. |
| Step lifecycle | `job_state`, `async_task_state` | Reject invalid self-transitions and terminal-state mutation. |
| Retry/resume/idempotency | `durable_job_executor`, `durable_resume`, `durable_replay`, `job_idempotency` | Gate reruns and resume steps without duplicating completed durable effects. |
| Runtime result admission | `job_result` | Classify exit/result status and size/hash checks before accepting outputs. |
| Artifact/value identity | `typed_durable_reference`, `content_addressed_code` | Model typed references, closure availability, and migration witnesses. |
| Capability/effect policy | `ability_effect`, `algebraic_effect`, `remote_computation` | Admit runtime invocation only when required abilities/resources are provided by the sandbox/runtime. |

Integration rule: prefer a direct `verified-logic` path dependency only for primitives that compile cleanly in normal Rust builds and fit licensing/build policy. Otherwise create a small `trellis_bridge` pure module that mirrors the needed predicate and cites the Trellis source and requirement id. In both cases, tests MUST include positive and negative cases matching Trellis examples.

## Validation and compatibility

Workflow validation should run in two layers:

1. Existing static workflow validation: graph, port direction/type compatibility, node config completeness.
2. Runner admission validation: runtime binding exists, runtime effects are allowed, graph can become runnable, lifecycle transitions are valid, and output contracts are well formed.

Existing persisted workflows remain valid. Nickel import/export gains optional runtime binding fields, but default builders preserve current shell, HTTP, file, image, and agent behavior.

## Implementation notes

- Start by extracting pure decision functions from `src/engine/executor.rs`, not by rewriting every runtime.
- Keep current `Executor` as a compatibility facade over the new runner shell.
- Move node-specific execution into runtime modules incrementally.
- Preserve existing `Value`, `ContentHash`, and port ids; add durable references as an extension instead of replacing values immediately.
- Add traceability comments from new implementation/tests to the Cairn ids in this change.

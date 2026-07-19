# Design: Seaglass-backed LLM operating graph

## Architecture

```text
Markdown roles + typed Nickel graph
              |
              v
n8nclone format shell -- bounded file reads, path containment, UTF-8
              |
              v
Workflow -> Runner adapter -> vendored Seaglass execution-graph core
              |
              v
admitted DAG decisions -> n8nclone runtime shell -> Pi/Clankers/effects
              |
              v
run events, artifacts, metrics, and explicit next-run graph/state updates
```

## Shared core adoption

Keep `RunnerWorkflow`, typed ports, runtime bindings, policies, values, events, and artifact references as `n8nclone` contracts. Convert runner nodes/edges into the Seaglass core model for graph-generic decisions. Local wrappers preserve numeric `NodeId` APIs and map core diagnostics into existing workflow diagnostics.

The core is vendored because the Seaglass commit is local and pushes are not authorized. `scripts/sync-seaglass-execution-graph.rs` accepts explicit source and revision arguments, replaces only the owned vendor subtree, and writes provenance metadata. Product builds depend only on the checked-in snapshot.

## Markdown role prompts

Add an optional `agent_prompt_file` field to portable agent config. Exactly one prompt source is authoritative:

- inline `agent_prompt`, or
- one relative `.md` path in `agent_prompt_file`.

The shell resolves the workflow parent directory, rejects absolute/traversal/non-Markdown paths, canonicalizes the root and candidate, enforces containment, bounds bytes, validates UTF-8, then supplies text to the pure workflow conversion. Runtime execution continues to receive a normal `AgentNodeSettings.prompt` string; the optional source path remains persisted for review/export.

## Operating-graph interpretation

- Role agents are generic agent nodes whose prompts live in Markdown.
- Sequence is represented by ordinary DAG edges.
- Shared state and effects remain explicit typed file/value nodes and references.
- Fan-out is admitted by the shared ready-set core.
- Feedback, metrics, and runtime reorganization are outputs of one run and inputs to a later graph revision; they do not bypass acyclic run admission.
- Human authority and approval remain explicit nodes/policies outside autonomous graph mutation.

## References used

Callee informs Markdown role authoring and bounded composite semantics. FarmTable informs ready-task detection, dependency queries, critical paths, bottlenecks, and claim/lifecycle separation. The supplied operating-graph image informs explicit authority, shared state, distribution, audience feedback, and between-run reorganization boundaries.

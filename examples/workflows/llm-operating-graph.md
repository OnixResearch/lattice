# LLM operating graph

`llm-operating-graph.ncl` is a runnable Markdown-backed role graph:

```text
shared brief read
  ├─> Researcher (text) ─┐
  └─> Builder (JSON) ────┴─> Reviewer ─> Publisher ─> report file write
```

- **Role**: each agent prompt is a bounded `.md` file under `roles/llm-operating-graph/`.
- **Sequential**: ordinary typed DAG edges establish ordering; the Reviewer waits for both fan-out branches.
- **Loop**: feedback is never a hidden cycle inside a run. Operators or an authorized outer controller may use the report, metrics, approvals, or failures to revise the brief or graph, then submit a new acyclic run.
- **State and effects**: shared-state input and report output are explicit file nodes. Agent roles do not receive ambient authority to mutate the graph or publish directly.

Regenerate the typed Nickel fixture after changing its topology:

```bash
cargo run --example generate_llm_operating_graph
```

Run it with configured Pi and Clankers adapters:

```bash
lattice --config ./demo/config.ncl import examples/workflows/llm-operating-graph.ncl
lattice --config ./demo/config.ncl run llm-operating-graph --output json
```

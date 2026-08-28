## Why

Lattice executes typed DAG workflows. It currently reruns affected work through normal execution and is adding bounded immutable node-result retention.

A cache can reuse exact prior results. It does not identify the smallest affected dependency region after declared input or graph changes.

Jane Street provides three MIT-licensed design references:

- `incremental` at `98b5750ec3c006641351bfd858a89136a5dbc52c`
- `incr_map` at `21c6bc602c75d57242b4c3e945da597f82c6280f`
- `incr_select` at `4d576e9683e85dd839ff7d4caa8c3a43934231fc`

These sources show dependency invalidation and selective recomputation. They are references, not runtime dependencies or parity requirements.

Lattice needs a bounded pilot before it changes production execution semantics.

## What Changes

- Add a test-only pure invalidation and recomputation oracle over explicit workflow facts.
- Build deterministic chain, fan-out, fan-in, and sparse-change fixture cohorts.
- Compare complete recomputation, current node-result caching, and incremental planning.
- Bind every run to graph, operation, input, policy, implementation, and corpus identities.
- Use typed Nickel for pilot limits and acceptance policy.
- Require a separate Cairn before any production execution path changes.

## Impact

The pilot adds fixtures, a test-only oracle, bounded measurements, and a decision record. It does not change scheduler, executor, persistence, or workflow semantics.

## Dependencies

The pilot consumes only current Lattice workflow contracts and explicit fixture facts.

If it uses Choregraph projections, it must pin one published revision. It must not depend on a sibling worktree.

## Non-goals

- Do not add a second workflow graph or scheduler.
- Do not change production node execution.
- Do not infer purity or cacheability from node names.
- Do not reuse results across capability or authority changes.
- Do not treat fewer visited nodes as correctness evidence.
- Do not adopt Jane Street runtime or OCaml dependencies.

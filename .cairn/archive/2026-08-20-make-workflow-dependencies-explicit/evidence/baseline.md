# Expression dependency baseline

<!-- r[verify lattice.workflow_validation.explicit_dependencies.baseline] -->

Baseline revision: `71628cd`

The baseline used a two-node shell workflow. The consumer command referenced the producer `stdout` port without a graph edge.

## Recorded behavior

- Expression result: `expression-ok`
- Serial order: producer, then consumer
- Runner dependency pairs: none
- Choregraph dependency pairs: none
- Runtime expression context: all outputs that completed earlier
- Consumer cache key: config, credentials, and normal edge inputs only
- Replay binding: no derived expression dependency in the machine identity
- Portable bytes: 2,273
- Portable BLAKE3: `3d44a887d162cf7fee87ff760cfd89fbbb9304c25447f03eb5711a77ee04b7a0`

The insertion order made the fixture succeed. The graph did not record the required producer-consumer relation.

## Baseline commands

```text
nix develop -c cargo test -p lattice-core
nix develop -c cargo test --test test_workflow_validation
nix develop -c cargo test --test test_runner_core
nix develop -c cargo test --test test_workflow_format
```

Results:

- `lattice-core`: 4 passed
- `test_workflow_validation`: 10 passed
- `test_runner_core`: 21 passed
- `test_workflow_format`: 24 passed

The baseline ran in a detached worktree at `71628cd`. The worktree was removed after the results were captured.

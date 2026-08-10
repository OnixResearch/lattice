# Choregraph call-site inventory — 2026-07-27

## Production machine adapter

All compatibility calls are in `src/engine/runner.rs`:

- public lifecycle and resume type aliases
- `admitted_execution_plan`
- `shared_execution_graph`
- `shared_step_states`
- `topological_order`
- `ready_nodes`
- `critical_path`
- `bottlenecks`
- `transition_step_state`
- `is_terminal_step_state`
- `durable_runner_decision`
- `migrate_persisted_states`
- `replay_persisted_events`

The source check requires this to be the only Rust source file that imports the compatibility crate.

## Lattice-owned surfaces

- `src/engine/executor.rs` maps Lattice `NodeStatus` values to runner states and retains runtime dispatch.
- `src/replay.rs` retains Lattice run-event presentation and never gains Choregraph dispatch authority.
- `src/config/mod.rs` retains Nickel evaluation, source discovery, and fallback policy.
- `src/workflow_format.rs` retains product workflow decoding and validation.
- store, trigger, scheduler, credential, retry, UI, diagnostic, and evidence modules remain outside the Choregraph adapter.

## Removed duplicate

`vendor-cargo-deps/seaglass-execution-graph/` is removed. No callable graph traversal or lifecycle implementation remains there.

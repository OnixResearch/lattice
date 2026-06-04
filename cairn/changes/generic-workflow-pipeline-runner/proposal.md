# Proposal: Generic workflow/pipeline runner

## Summary

Refactor execution into a generic typed DAG runner with pluggable node runtimes. Shell, HTTP, file, image, and agent behavior remain typed workflow nodes, but their execution moves behind a common runtime registry and evented runner shell. Use Trellis verified-logic primitives from `../trellis/` where they match core admission semantics: graph readiness, job lifecycle, durable replay, effect/capability admission, and typed durable references.

## Problem

The current executor owns DAG scheduling, cache identity, value routing, node-specific preparation, side-effect invocation, and UI/headless reporting in one large orchestration surface. Recent agent runtime work reduced Pi/Clankers coupling, but the broader system is still not a generic pipeline runner: adding new node classes tends to add executor branches, runtime behavior is not uniformly pluggable, and run state/events/artifacts are not modeled as reusable runner primitives.

## Goals

- Extract a pure runner core that validates and schedules typed workflow DAG steps without performing I/O.
- Introduce a runtime registry that dispatches node execution by typed runtime kind and configured runtime key.
- Preserve typed workflow semantics: ports, value tags, cache identity, credentials, artifacts, and adapter-specific diagnostics stay explicit.
- Make TUI, headless, and scheduled execution consume the same runner events and summaries.
- Reuse or mirror Trellis primitives where they fit exact runner admission boundaries instead of inventing ad hoc logic.
- Keep existing workflows, Nickel import/export, shell nodes, HTTP/file/image nodes, and generic agent presets compatible.

## Non-Goals

- Collapse typed nodes into unstructured shell commands.
- Rewrite the TUI graph editor or persistence store wholesale.
- Require every Trellis module as a runtime dependency before build, license, and API fit are proven.
- Claim Trellis proves n8nclone side effects, UI behavior, networking, or process execution.
- Add distributed execution or remote workers in this change.

## Trellis Fit

Trellis is a reference and possible dependency for the pure admission core only. Candidate primitives:

- `graph_adjacency`, `graph_reachability`, `job_dag`, and `dependency_scheduler` for acyclic graph admission, dependency readiness, topological order, and capacity-aware scheduling decisions.
- `job_state` and `async_task_state` for valid step lifecycle transitions and terminal-state rejection.
- `durable_job_executor`, `durable_history`, `durable_resume`, `durable_replay`, `durable_terminal`, `job_idempotency`, and `job_result` for checkpoint/retry/idempotency/result admission.
- `typed_durable_reference` and `content_addressed_code` for typed artifact/value/runtime identity and migration boundaries.
- `ability_effect`, `algebraic_effect`, and `remote_computation` for runtime capability/effect/resource admission when command, agent, HTTP, and future runtimes declare required abilities.

## Success Criteria

- A pure runner core can plan and schedule a workflow DAG from in-memory inputs with deterministic diagnostics and no filesystem, process, clock, network, or UI access.
- Runtime execution is selected through a registry interface instead of executor-local node branches.
- Existing nodes execute through built-in runtime implementations with equivalent outputs and cache behavior.
- TUI, headless, and scheduled runs receive the same ordered runner events and terminal summaries.
- Nickel workflow import/export can represent generic runtime bindings without losing existing typed node configuration.
- Positive and negative tests cover valid pipelines, cycles, missing runtimes, bad port values, terminal-state mutation, failed dependencies, and side-effect admission errors.

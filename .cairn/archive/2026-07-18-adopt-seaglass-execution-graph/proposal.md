# Adopt Seaglass execution graph for LLM operating graphs

## Summary

Adopt Seaglass's extracted execution-graph core in `n8nclone`, update to the newest consumable Rats source, and support role-based agent prompts authored as Markdown files referenced from typed Nickel workflows.

## Problem

`n8nclone` already has typed agent nodes and a generic runner IR, but its graph admission/lifecycle algorithms are locally owned and role prompts are embedded strings. This prevents one shared graph core from serving Seaglass and the LLM workflow executor, and it makes substantial role prompts awkward to review and version.

## Success criteria

- SC-01: The runner delegates graph admission, deterministic ordering, readiness, lifecycle, resume decisions, critical path, and bottlenecks to a provenance-recorded Seaglass core snapshot.
- SC-02: Existing shell, HTTP, file, image, condition, merge, subworkflow, Pi, and Clankers behavior remains compatible.
- SC-03: Typed Nickel workflows may reference bounded UTF-8 Markdown prompt files under the workflow root; invalid, missing, oversized, non-Markdown, or escaping paths fail before execution.
- SC-04: A runnable LLM operating-graph fixture demonstrates role agents, explicit state/effect edges, fan-out, and result aggregation.
- SC-05: Rats dependencies use the newest source that passes the consumer checks; source-distribution defects are fixed at their owner rather than hidden in `n8nclone`.

## Non-goals

- Hidden cyclic execution inside one run. Feedback and graph reorganization occur through explicit state or graph changes between admitted DAG runs.
- Autonomous mutation of authority, approval, credentials, or runtime policy.
- Copying Callee or FarmTable schemas, servers, provider processes, or task stores.
- Claiming that graph admission validates LLM truthfulness, tool safety, or external side effects.

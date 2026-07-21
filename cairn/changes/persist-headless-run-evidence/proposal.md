## Why

A production-shaped duplicate-trigger drill against the live Animus/VibeThinker endpoint validated orchestration idempotency: a duplicate returned the original in-flight instance in 0 ms while the first provider call took 28.1 seconds, and the store retained one admission, one instance, one run, one consumed reservation, and valid replay. The same audit found that the terminal headless run had no `RunDetailRecord`, so operators could not inspect durable node output or bounded agent metadata after the process exited.

The existing detail builder lives under the TUI and renders raw value display strings. Reusing it directly from headless execution would invert the dependency boundary and could persist sensitive values without enforcing the value's redaction metadata.

The first post-fix live rerun then exposed a stronger provider-boundary defect: despite `enable_thinking = false`, the compatible endpoint placed a leading `<think>...</think>` block inside `message.content`. That block repeated the raw prompt and would have entered typed agent output and durable details. The provider decoder must suppress one unambiguous leading reasoning block and reject ambiguous marker placement before output typing.

## What Changes

- Extract deterministic run-detail construction into a shared pure run-history core.
- Replace sensitive value output with a fixed redaction marker before persistence.
- Persist node-level details for successful and failed headless runs, including orchestration-owned runs.
- Retain the TUI entrypoint as a compatibility wrapper over the shared core.
- Remove one exact leading closed reasoning block at the OpenAI-compatible boundary and reject malformed, nested, repeated, or non-leading markers.
- Record the live duplicate-trigger drill and rerun it after both evidence-boundary fixes.

## Impact

- **Files**: `src/run/history.rs`, `src/headless.rs`, `src/openai/core.rs`, `src/openai/reasoning.rs`, `src/tui/app.rs`, `src/lib.rs`, focused OpenAI/headless/run-history tests, documentation, and Cairn lifecycle evidence.
- **Testing**: pre/post focused tests, positive and negative inline-reasoning/redaction fixtures, live endpoint rerun, repository checks, Octet comparison, Cairn gates/validation, and bounded Nix validation.

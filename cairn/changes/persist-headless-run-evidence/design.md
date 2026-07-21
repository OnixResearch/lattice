## Context

`run_saved_workflow_with_executor_factory` persists a terminal `RunRecord` and replay events but discards the `StepResult` detail view. TUI execution separately converts the same results into `RunDetailRecord`. The live idempotency probe therefore observed `run_details_present: false` even though the owned run completed successfully.

## Portfolio

### Family: Reuse the TUI converter from headless execution

**State:** Rejected.

This would make a non-interactive execution shell depend on TUI code. The current converter also calls `Value::as_display_string` directly and does not enforce `sensitive_fragments`, so expanding its persistence reach would be unsafe.

### Family: Persist replay summaries as run details

**State:** Rejected.

Replay summaries are redaction-safe but intentionally truncated and shaped as event evidence. Recasting them as node output would lose the existing run-history contract for stdout, stderr, exit code, typed output, and error details.

### Family: Shared redaction-aware detail core

**State:** Selected.

Move result-to-detail conversion into a pure module over `RunId`, `Workflow`, and `StepResult`. Preserve complete display lines for non-sensitive values and substitute one fixed marker for any value carrying sensitivity metadata. Let TUI and headless shells call the same core; let only the shells perform storage I/O.

### Family: Hide reasoning only during run-detail persistence

**State:** Rejected.

This would leave raw reasoning and repeated prompts in typed agent outputs, downstream graph values, TUI output, and Animus reports before persistence. It addresses one symptom after the unsafe content has already crossed authority boundaries.

### Family: Reject every response containing reasoning markup

**State:** Rejected as the default compatible behavior.

This fails closed but makes a reachable endpoint unusable even when it provides an unambiguous leading closed reasoning block and a separate valid final answer. The endpoint ignored its advertised `enable_thinking = false` request hint, so request configuration alone is insufficient.

### Family: Normalize one exact block at the provider decoder

**State:** Selected.

At the OpenAI-compatible decoder, preserve plain content; remove exactly one leading `<think>...</think>` block with a non-empty final suffix; reject unterminated, nested, repeated, or non-leading markers. Apply text/JSON typing only to the resulting final suffix. This keeps raw reasoning out of Animus and every downstream persistence/rendering surface.

## Decisions

### Decision: Persist terminal headless details through `RunHistoryStore`

**Choice:** Build one `RunDetailRecord` from terminal results and save it alongside the existing run record and replay events.

**Rationale:** This uses the existing narrow storage port and makes TUI, direct headless, trigger, and orchestration-owned executions inspectable without adding another evidence model.

### Decision: Redact the complete sensitive value

**Choice:** If a value has any `sensitive_fragments`, persist `[redacted]` instead of attempting partial replacement.

**Rationale:** Whole-value replacement fails closed when sensitivity provenance is present, avoids accidental residual disclosure, and matches runner/replay redaction semantics. Non-sensitive model responses and safe adapter metadata remain inspectable.

### Decision: Treat reasoning markers as protocol structure, not ordinary content

**Choice:** A single leading closed block is stripped and recorded only as a safe boolean metadata fact; every ambiguous marker shape is rejected without echoing content.

**Rationale:** The live response proved provider hints are not authoritative. A narrow deterministic parser prevents raw chain-of-thought and prompt repetition from entering typed output while avoiding heuristic extraction from arbitrary mixed content.

## Risks / Trade-offs

- Run summary, details, and replay events remain separate redb transactions; this change does not claim atomic multi-record evidence persistence.
- Historical headless runs are not backfilled.
- Complete non-sensitive outputs remain durable by existing run-history design; operators retain responsibility for marking secret-derived values sensitive.
- Literal user-requested text containing `<think>` markers is rejected or structurally normalized rather than preserved; this is an intentional safety boundary for chat-completions content.
- Local run details and replay prove only recorded local observations, not provider invocation counts, billing truth, model truth, or remote exactly-once effects.

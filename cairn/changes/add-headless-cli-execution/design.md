## Context

The executor already runs a `Workflow` and returns per-node `StepResult` values. The current binary always starts the TUI after config loading, so there is no imperative shell dedicated to non-interactive execution.

## Decisions

### 1. Headless execution uses the existing executor core

**Choice:** Add a CLI run path that loads a workflow, invokes the same executor, and formats the resulting summary without constructing TUI state.

**Rationale:** One execution path avoids divergent behavior between TUI and automation.

### 2. FCIS boundary remains explicit

**Choice:** Model run options, run summaries, and exit decisions as pure data transforms. Keep argument parsing, store access, workflow import, stdout/stderr, process exit codes, and signal handling in the CLI shell.

**Rationale:** The functional core stays deterministic and testable; the imperative shell owns I/O.

### 3. Output modes are renderers over the same summary

**Choice:** Produce a single in-memory `RunSummary` and render it as text or JSON at the boundary.

**Rationale:** Automation and human output should agree on status, failures, cache hits, and run id.

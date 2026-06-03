## Context

`Store` defines `RunRecord`, `save_run`, and `list_runs`. `App::run_workflow` currently pushes output lines but does not save run records or provide a UI for history.

## Decisions

### 1. Run persistence belongs to execution completion

**Choice:** Save a run record after each completed, failed, or canceled workflow execution.

**Rationale:** History should represent observed TUI runs regardless of final status.

### 2. History is a read-only modal

**Choice:** Add a recent-runs modal that lists run summaries and opens details without mutating workflow state.

**Rationale:** Users can inspect history safely while continuing to edit the current workflow.

### 3. Store errors are visible but non-fatal

**Choice:** If run history cannot be listed or a record cannot be read, report the issue in the output panel and keep the TUI usable.

**Rationale:** History is useful but must not block workflow authoring.

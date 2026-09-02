# Change: Observe cancellation completion

## Why

Lattice separates cancellation requests from observed runner outcomes in orchestration. The TUI does not preserve that distinction.

`cancel_active_run` sets the cancellation flag, aborts the outer Tokio task, clears active ownership, and persists a cancelled result immediately. Some executor paths use `spawn_blocking`. Tokio cannot abort a blocking task after that task starts.

The TUI can therefore publish terminal cancellation while the blocking worker still owns work or resources. It can also admit another run before the first worker completes teardown.

## What Changes

- Add a pure cancellation lifecycle reducer for request, observation, uncertainty, and terminal classification. r[lattice.workflow_execution.cancellation_protocol.boundary]
- Treat a cancellation request as non-terminal. Keep exact active-run ownership until the runner supplies a terminal observation. r[lattice.workflow_execution.cancellation_protocol.request]
- Publish `Cancelled` only after all started owned work stops and required teardown completes. r[lattice.workflow_execution.cancellation_protocol.completion]
- Preserve unknown and cleanup-failed outcomes without inventing cancellation or automatic retry. r[lattice.workflow_execution.cancellation_protocol.uncertainty]
- Keep the TUI responsive and show `Cancelling` while completion remains pending. r[lattice.tui_ux.cancellation_lifecycle]
- Persist one terminal run and replay result only after terminal classification. r[lattice.tui_ux.cancellation_persistence]
- Add positive and negative worker, process, race, channel, persistence, and PTY fixtures. r[lattice.workflow_execution.cancellation_protocol.verification] r[lattice.tui_ux.cancellation_verification]

## Impact

- **Affected specs:** `workflow-execution` and `tui-ux`.
- **Affected code:** TUI active-run state, runner events, cancellation classification, run persistence, replay events, and executor integration.
- **Affected tests:** focused state tests, blocking-worker tests, bounded-process tests, and the process-boundary TUI matrix.

## Dependencies

- The pinned Bounded Exec contract remains the owned-process teardown mechanism.
- Choregraph remains the pure lifecycle owner. Lattice supplies terminal `Cancel` only after it observes local cancellation completion.

## Non-Goals

- Do not add process, task, clock, persistence, or UI effects to Choregraph.
- Do not claim remote cancellation, arbitrary task kill safety, or sandbox completeness.
- Do not add automatic retry after an unknown cancellation outcome.
- Do not change normal scope exit or Rust error propagation into task cancellation.

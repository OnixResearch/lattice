# Design: Observed cancellation completion

## Context

The TUI owns one `ActiveRun` with a cancellation flag, event receiver, workflow snapshot, and Tokio handle. The worker can enter `spawn_blocking` for bounded local process execution.

Tokio aborts the async wrapper, but it cannot abort a blocking task after that task starts. The current TUI clears `ActiveRun` and persists cancellation before it receives a worker result.

Bounded Exec already reports `Completion::Cancelled` only after owned-process teardown. Lattice must preserve that guarantee through its outer task and TUI state.

## Lifecycle

The TUI uses this product lifecycle:

```text
Running
  -> CancellationRequested
  -> Cancelling
  -> Cancelled | Completed | Failed | CancellationUnknown
```

`CancellationRequested` records operator intent. `Cancelling` means that the worker still owns the run. Neither phase is terminal.

The final class comes from the worker observation. A normal completion can win a request race. A missing observation cannot become `Cancelled`.

## Functional core

A narrow pure reducer receives explicit facts:

- current cancellation phase;
- exact run identity;
- whether a request exists;
- runner terminal observation;
- worker completion or join observation;
- teardown disposition;
- persistence disposition.

The reducer returns the next phase, UI projection, persistence plan, replay plan, and admission decision. It reads no clock, task, channel, process, filesystem, store, or terminal state.

The reducer rejects stale run identities, duplicate terminal transitions, terminal mutation, and impossible request or observation order.

## Imperative shell

The TUI shell owns cancellation flags, Tokio handles, channels, clocks, persistence, output, and PTY behavior.

`cancel_active_run` becomes a request operation. It signals the selected worker, preserves the handle and receiver, and leaves the run active. It does not abort an already-started blocking worker.

The shell admits no new workflow run while the current run is `CancellationRequested` or `Cancelling`.

The worker sends a typed terminal observation after executor return. For local processes, Bounded Exec must finish teardown before it returns cancellation.

The TUI persists terminal cancellation only after it admits that observation. A closed channel, failed wrapper task, missing teardown observation, or persistence failure produces a non-cancelled failure or unknown result.

## Choregraph boundary

Choregraph `Action::Cancel` remains a pure terminal lifecycle event. It does not request task cancellation or execute cleanup.

Lattice supplies that event only after the shell observes cancellation completion. Pending nodes that never started can become cancelled from local scheduling facts.

## Compatibility

The operator keeps the existing cancellation command and key binding. The TUI adds a visible `Cancelling` phase.

Existing terminal `Cancelled` records keep their serialized meaning. The change narrows when Lattice can create a new record.

Any new unknown class must use a versioned persistence projection. Old readers must fail closed or map it to a non-cancelled failure.

## Failure handling

- A repeated cancellation request is an idempotent request. It creates no second terminal record.
- A normal terminal observation can win a cancellation race.
- A worker failure after a request remains a failure.
- A lost observation remains unknown and grants no retry authority.
- A stuck worker keeps exclusive active-run ownership. The TUI must not claim cancellation or start another run.
- An explicit process exit can stop the TUI, but it creates no successful cancellation evidence.

## Verification

Positive cases cover request, visible `Cancelling`, observed Bounded Exec cancellation, teardown, one terminal record, and later fresh-run admission.

Negative cases cover a started blocking worker, premature persistence, premature fresh-run admission, stale events, duplicate requests, channel loss, wrapper failure, cleanup failure, and persistence failure.

Race cases cover completion before request observation, cancellation before node dispatch, and cancellation during bounded child execution.

The PTY rail must prove that pending observable work does not start. It must also prove that terminal cancellation appears only after the selected child is reaped.

## Claim Boundary

Passing evidence proves only the selected local Lattice cancellation protocol. It does not prove remote cancellation, external effect rollback, arbitrary thread termination, sandboxing, or production readiness.

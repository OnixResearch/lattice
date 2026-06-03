## Context

`App::run_workflow` awaits executor completion directly after `Ctrl-R`. The executor returns step results only after each node completes, and the event loop does not poll input while awaiting the run.

## Decisions

### 1. Introduce explicit run state

**Choice:** Add an `AppRunState` model containing active run id, node statuses, output events, and cancellation handle metadata.

**Rationale:** Rendering should read deterministic state rather than infer execution from task internals.

### 2. Use message passing from tasks to TUI

**Choice:** Background execution sends structured run events to the TUI shell, which applies them to `AppRunState`.

**Rationale:** The executor can remain testable while the shell owns async task orchestration.

### 3. Cancellation is explicit and visible

**Choice:** Add a cancel action that requests cancellation, marks pending nodes as skipped, and reports cancellation in output.

**Rationale:** Users need a safe recovery path from long or accidental commands.

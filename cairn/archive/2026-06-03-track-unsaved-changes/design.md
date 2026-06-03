## Context

`Workflow::touch()` exists, and saving is already routed through `Store::save_workflow`. Mutating actions such as adding nodes, moving nodes, wiring, editing properties, and loading workflows need consistent dirty-state handling.

## Decisions

### 1. Dirty tracking is explicit App state

**Choice:** Add a dirty flag and last-saved workflow identity to `App` rather than inferring unsaved state only from timestamps.

**Rationale:** Explicit state avoids ambiguity around initial unsaved workflows and failed saves.

### 2. Mutations mark dirty through narrow helpers

**Choice:** Route workflow mutations through helper methods that call `touch()` and mark the app dirty.

**Rationale:** Centralized mutation paths reduce missed dirty-state updates.

### 3. Quit confirmation is a modal state

**Choice:** When dirty, quit opens a confirmation prompt; confirming quits, canceling returns to the TUI.

**Rationale:** The user gets a clear recovery path without accidental data loss.

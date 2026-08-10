## Context

`Store` already exposes `list_workflows` and `load_workflow`. `AppAction::OpenWorkflow` exists, but the TUI shell currently prints a placeholder message.

## Decisions

### 1. Browser is a modal picker

**Choice:** Present saved workflows in a keyboard-driven overlay owned by `App` state.

**Rationale:** A modal browser matches command palette behavior and avoids overloading the properties panel.

### 2. Separate listing from loading

**Choice:** Listing saved workflow names is triggered before display; loading occurs only when the user confirms a selection.

**Rationale:** Failed loads can be reported precisely while keeping navigation responsive.

### 3. Preserve graph compatibility after load

**Choice:** Loaded workflows pass through the same graph-state sync path used after other graph mutations.

**Rationale:** Cloned graphs must restore compatibility behavior before users edit or wire loaded workflows.

## Context

`AppAction::AddNode` currently inserts `Shell N` with `echo 'hello'`. The config layer can load node templates from Nickel files, but the TUI does not expose them during node creation.

## Decisions

### 1. Add-node is a wizard-style modal

**Choice:** Add explicit add-node modal state for selecting a source, entering required fields, confirming, and canceling.

**Rationale:** Node creation involves multiple pieces of input and should not happen as an irreversible shortcut side effect.

### 2. Templates are optional inputs

**Choice:** The flow shows templates when available and falls back to shell-node creation when the template list is empty.

**Rationale:** First-time users can still create nodes without configuration, while advanced users can reuse Nickel templates.

### 3. New nodes remain selected

**Choice:** After successful creation, the new node is selected and focused on the canvas.

**Rationale:** Users commonly create a node and immediately edit or connect it.

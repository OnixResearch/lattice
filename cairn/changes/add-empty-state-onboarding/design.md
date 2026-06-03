## Context

The canvas renders the node graph widget even when there are no nodes. Users can discover commands only if they already know shortcuts or notice the footer.

## Decisions

### 1. Empty-state is canvas-local

**Choice:** Render onboarding inside the canvas area only when the graph is empty and no modal overlay is active.

**Rationale:** The message appears where users are looking without interfering with properties or output panels.

### 2. Hints are action-oriented

**Choice:** The copy names concrete next actions rather than explaining the whole product.

**Rationale:** Users should be able to make their first node with minimal reading.

### 3. Responsive degradation

**Choice:** On narrow terminals, show a compact one-line hint rather than wrapping a dense block.

**Rationale:** The TUI should remain readable in constrained terminal sizes.

## Context

`rat-nodegraph` exposes `NodeGraphState::handle_mouse_click` as the authoritative hit-test path for starting and completing wiring. `n8nclone` currently calls that handler on left-button down and updates the wiring preview during drag events, but left-button up only clears drag state or finishes box selection. As a result, dragging from an output port to an input port leaves wiring mode without committing a graph edge.

## Decisions

### 1. Complete wiring through the existing graph action path

**Choice:** On left-button up while the graph is in wiring mode, dispatch the release coordinates through `handle_mouse_click`, apply returned actions, sync the workflow graph, and mark the workflow dirty only when graph counts change.

**Rationale:** This reuses the nodegraph library's existing compatibility checks, hit testing, cancellation action, and edge creation action instead of duplicating port math in the app shell.

### 2. Keep click wiring and box selection unchanged

**Choice:** Only add release handling for `InteractionMode::Wiring`; keep box-select finalization on release and click-to-click wiring on button-down as-is.

**Rationale:** The patch fixes the missing drag-release gesture without changing established keyboard or click workflows.

## Risks / Trade-offs

- Reusing `handle_mouse_click` means drag-release completion follows the same exact port target requirement as click completion. That is predictable, but users still need to release on the input-port cell.
- Edge creation remains subject to the nodegraph library's current type compatibility semantics.

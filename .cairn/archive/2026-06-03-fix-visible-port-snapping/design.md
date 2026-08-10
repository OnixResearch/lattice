## Context

`rat-nodegraph` currently renders input dots at `x + 1, y + 3 + index` and output dots at `x + width - 2, y + 3 + index`, while its hit-testing and edge-routing helpers use the border cells `x, y + 2 + index` and `x + width - 1, y + 2 + index`. The TUI sends raw mouse coordinates to the graph library, so clicking the visible dots misses the port hit target.

## Decisions

### 1. Add an app-side snap adapter

**Choice:** Convert mouse canvas coordinates near visible port dots into the library's expected hidden port hit coordinates before calling `handle_mouse_click`.

**Rationale:** This fixes n8nclone immediately without changing the git dependency or disturbing unrelated local work in the sibling `rats` repository.

### 2. Snap previews to visible dots

**Choice:** Keep drag math in actual canvas coordinates for node movement, but while wiring, snap preview cursor coordinates to visible port dots when the cursor is within the snap radius.

**Rationale:** Node dragging should remain precise, while wiring should feel magnetic around port targets.

## Risks / Trade-offs

- This is a compatibility shim for the current `rat-nodegraph` coordinate contract. If the dependency later aligns render and hit-test positions, the adapter should be removed or updated.
- Snapping is intentionally limited to a small one-cell radius around visible port dots to avoid surprising node body clicks.

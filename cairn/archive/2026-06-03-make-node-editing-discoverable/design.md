## Context

The app already has a functional property-edit core (`start_property_edit`, `save_property_edit`, and validation) and tests for command updates, timeout rejection, and cancel behavior. The missing UX is an obvious entry point after node creation: input currently routes property edit keys only when the properties panel has focus, and the command palette exposes no edit actions.

## Decisions

### 1. Reuse the existing property-edit core

**Choice:** Add new input and palette entry points that call `App::start_property_edit` with the existing `PropertyEditField` values.

**Rationale:** The edit core already handles shell command mutation, label updates, timeout validation, dirty tracking, and cancel semantics. Reusing it avoids duplicating mutation logic.

### 2. Make property editing modal

**Choice:** Route property-edit text keys before ordinary panel dispatch whenever `app.property_edit` is active.

**Rationale:** Palette and canvas shortcuts may start editing while canvas focus remains active. Text entry must continue working without requiring an extra focus switch.

### 3. Keep shortcuts field-specific

**Choice:** Use `e` for command, `l` for label, and `t` for timeout from canvas/properties, and expose matching palette actions.

**Rationale:** This matches the existing properties-panel keys while making the most important edit path (command) available immediately after selecting or creating a node.

## Risks / Trade-offs

- Single-letter canvas shortcuts are now reserved for editing selected nodes; future canvas commands should avoid conflicting with `e`, `l`, and `t`.
- Property edit text mode intentionally captures printable input until Enter or Escape, so global non-hard shortcuts wait until editing ends.

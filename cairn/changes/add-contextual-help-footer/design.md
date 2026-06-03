## Context

`render_status_bar` already has access to focus and graph interaction mode. It currently emits a mostly fixed shortcut list and only changes for wiring mode.

## Decisions

### 1. Compute help text from pure UI state

**Choice:** Add a pure helper that maps focus, modal state, and interaction mode to concise help segments.

**Rationale:** The footer can be unit tested without rendering a terminal buffer.

### 2. Prefer active-mode actions

**Choice:** The footer shows actions for the current mode first, then global actions if space allows.

**Rationale:** Users need the next relevant action more than a complete shortcut reference.

### 3. Truncate safely

**Choice:** If there is not enough width, render a shortened hint rather than overflowing or corrupting adjacent status fields.

**Rationale:** The status bar should remain readable on small terminals.

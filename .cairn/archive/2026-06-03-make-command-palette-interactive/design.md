## Context

`App` already owns `show_palette`, `palette_state`, `palette_data`, and `palette_action`. Rendering uses `rat_fuzzy::FuzzyFinder`, but input handling currently delegates to the focused panel even while the palette is displayed.

## Decisions

### 1. Palette input has modal priority

**Choice:** When `show_palette` is true, key events are handled by palette logic before global panel delegation, except hard quit shortcuts.

**Rationale:** The palette behaves like a modal overlay and should not accidentally move nodes or scroll output while the user is typing.

### 2. Keep action resolution pure

**Choice:** Keep selection-to-action resolution in pure functions that accept candidate identifiers and return `AppAction` values.

**Rationale:** The imperative shell only mutates `App` state and dispatches actions, while the core mapping remains easy to unit test.

### 3. Preserve existing shortcut behavior

**Choice:** Existing shortcuts continue to work when the palette is closed.

**Rationale:** The palette adds discoverability without replacing fast-path shortcuts for experienced users.

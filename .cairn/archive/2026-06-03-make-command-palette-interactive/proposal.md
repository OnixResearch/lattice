## Why

The command palette is visible but not yet usable as the primary keyboard-first command surface. Users can open it with `Ctrl-P`, but typing, navigating, and confirming commands does not drive application actions.

## What Changes

- Route key events to the palette while it is open.
- Let users type a query, move through results, execute the selected action, and close the palette with Escape.
- Keep palette actions mapped through the existing `AppAction` surface so command execution stays testable.

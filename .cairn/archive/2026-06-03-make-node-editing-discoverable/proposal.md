## Why

Node property editing exists, but users have to discover the properties-panel focus state before they can change a node's command. After creating a node, editing the command should be available from the normal canvas workflow and the command palette.

## What Changes

- Add command palette actions for editing the selected node's command, label, and timeout.
- Add direct selected-node shortcuts from the canvas/properties flow: `e` edits command, `l` edits label, and `t` edits timeout.
- Treat property editing as a modal text-entry state so typing, Enter, Escape, and Backspace work regardless of which panel had focus when editing started.
- Update footer/help text so edit shortcuts are visible.

## Impact

- **Files**: `src/tui/app.rs`, `src/tui/input.rs`, `src/tui/panels.rs`, `cairn/specs/tui-ux/spec.md`
- **Testing**: focused input/palette/footer tests plus `cargo test --locked` and `cairn validate --root .`.

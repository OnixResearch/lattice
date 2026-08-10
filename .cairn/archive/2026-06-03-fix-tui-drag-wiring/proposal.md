## Why

Users can start wiring from an output port, but the common drag-and-release gesture does not create an edge. The TUI only completes wiring on a second click, so mouse users see a preview line but cannot connect nodes by releasing on an input port.

## What Changes

- Complete wiring when the left mouse button is released over a compatible input port.
- Preserve the existing click-to-click wiring behavior and cancellation behavior for invalid targets.
- Mark the workflow dirty when a drag-release creates an edge.

## Impact

- **Files**: `src/tui/input.rs`, `cairn/specs/tui-ux/spec.md`
- **Testing**: focused TUI input regression tests plus `cargo test --locked` and `cairn validate --root .`

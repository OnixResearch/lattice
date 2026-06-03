## Why

The first wiring fix completed drag-release only after the graph library reported a port hit. The visible port dots still do not hit because `rat-nodegraph` renders dots one cell inside the node and one row below the library hit-test coordinates.

## What Changes

- Translate visible port-dot clicks to the graph library's expected hit-test cells before dispatching graph actions.
- Snap wiring previews and release handling to nearby visible port dots.
- Keep non-port releases canceling wiring without graph mutation.

## Impact

- **Files**: `src/tui/input.rs`, `cairn/specs/tui-ux/spec.md`
- **Testing**: focused visible-port and snap-near TUI input tests plus full locked tests and Cairn validation.

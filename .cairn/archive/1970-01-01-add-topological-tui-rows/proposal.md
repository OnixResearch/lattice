## Why

GitHub issue #1 asks for a monitoring layout for mostly-linear workflow DAGs. The existing rows surface displays numerical depth-local lanes, clips selected rows beyond the viewport, and waits until the entire run finishes to display node results. It does not draw fork/join paths or record a visible start time.

## What Changes

- Render deterministic topological rows with a stable pending-node lane frontier and explicit fork/join connectors.
- Bound visible lanes by canvas width and six columns, showing `+N` overflow and a shared-lane marker without hiding nodes.
- Keep keyboard/mouse selection shared with the graph and scroll the selected row into view; prevent rows gestures from mutating the hidden spatial graph.
- Publish final admitted per-node results, then update state, observed start/duration/attempts, usage, and meaningful output while other work is still active.
- Maintain linear, fork-join, and diamond render snapshots plus metadata, viewport, selection, failure, and skipped-result regressions.

## Scope and Non-claims

This change does not change graph execution order, readiness, authority, retry, cancellation policy, workflow definitions, or source-aware Nickel editing (#21). Missing usage remains unknown. Start time is the TUI's local dispatch observation; summed node durations are not run wall time. Rendering and reducer tests do not establish external effect success or pixel-identical terminal behavior. No checks or lifecycle gates were run during delegated implementation; parent integration owns verification and closeout.

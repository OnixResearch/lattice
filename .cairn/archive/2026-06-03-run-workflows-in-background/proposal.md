## Why

Workflow execution currently awaits inside the TUI event loop. Long-running shell commands can freeze rendering and input, making the app feel broken and preventing cancellation or inspection during a run.

## What Changes

- Execute workflows through background runtime tasks.
- Track per-node run status and stream progress into the output panel.
- Keep the TUI responsive while a workflow is running.
- Add a cancellation path for active runs.

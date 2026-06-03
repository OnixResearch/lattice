## Why

Users can currently create workflows that fail only after execution starts. A validation layer and visible diagnostics panel make authoring safer by finding empty commands, missing configs, bad edges, and unsafe settings before run time.

## What Changes

- Add a pure workflow validator that emits structured diagnostics.
- Expose validation results in the TUI.
- Gate runs on fatal validation errors while allowing warnings to remain actionable.
- Add positive and negative validation tests.

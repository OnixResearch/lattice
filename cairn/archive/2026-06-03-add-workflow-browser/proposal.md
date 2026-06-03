## Why

The store can list and load workflows, but the TUI `OpenWorkflow` action only reports that the workflow browser is not implemented. Users need an in-app way to discover and reopen saved workflows.

## What Changes

- Add a workflow browser overlay for saved workflow names.
- Load the selected workflow into the canvas and properties panel.
- Handle empty stores and load failures with visible feedback instead of silent no-ops.

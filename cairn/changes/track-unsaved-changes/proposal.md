## Why

Workflow mutations can be saved, but the TUI does not visibly track dirty state or warn before losing unsaved edits. Users need confidence that changes are either saved or intentionally discarded.

## What Changes

- Track dirty state after workflow mutations.
- Show an unsaved indicator near the workflow name.
- Clear dirty state after successful save.
- Confirm quit when unsaved changes are present.

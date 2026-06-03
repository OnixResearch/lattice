## Why

The store has run-record support, but the TUI does not expose recent runs or let users inspect previous outcomes. Users need a way to compare recent executions, failures, and cached results.

## What Changes

- Persist run records for workflow execution.
- Add a recent-runs view or modal inside the TUI.
- Let users inspect summary details and relevant output for a selected run.
- Handle empty history and corrupt/missing records clearly.

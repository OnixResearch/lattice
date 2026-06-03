## Why

Once workflows can run headlessly, users need a way to run them on a schedule without keeping the TUI open. A scheduler daemon turns saved workflows into recurring local automations.

## What Changes

- Add persistent schedule definitions for saved workflows.
- Add a daemon command that evaluates due schedules and starts runs.
- Track last-run state and prevent unsafe overlap by default.
- Handle restart and missed-run behavior deterministically.

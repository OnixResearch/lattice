## Why

n8nclone workflows can be authored in the TUI, but they cannot be run from scripts, CI jobs, schedulers, or other automation without opening the terminal UI. A headless run path makes workflows useful outside interactive editing and becomes the foundation for daemon scheduling.

## What Changes

- Add a headless `run` command for saved or imported workflows.
- Emit human-readable and machine-readable run summaries.
- Preserve run-history behavior for headless executions.
- Return process exit status based on workflow success or failure.

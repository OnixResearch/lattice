## Why

The workflow exchange currently maps a completed shell node to successful exchange completion even when its typed exit output is nonzero. A live bounded provider exited with code `2`, but the exchange returned terminal success. This can promote a definite provider failure into false CI success.

## What Changes

- Classify a completed workflow-exchange node as failed when its typed exit-code output is nonzero, missing, duplicated, or malformed.
- Keep normal Lattice shell-node output semantics unchanged outside workflow exchange.
- Add positive and negative workflow-exchange tests for zero and nonzero shell exits.

## Impact

- **Files**: `src/workflow_exchange/application/worker.rs`, `src/workflow_exchange/tests.rs`, and this Cairn change.
- **Testing**: focused workflow-exchange tests, repository checks, strict Clippy, Octet, Cairn validation, and Nix checks.

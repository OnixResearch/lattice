## Why

Kiln's GitHub CI integration executed an exact-source command that exited 23, but Lattice retained the shell node as `Completed` and exported workflow success. Workflow-exchange consumers need command failure to remain failure before dependent nodes or a successful terminal observation are admitted.

## What Changes

- Require a zero shell exit status for immutable workflow snapshots executed by the workflow-exchange application.
- Persist a failed node and skip dependent work when a required shell exits nonzero or has no successful exit status.
- Keep skipped dependents free of execution receipts, as required by the existing published observation contract.
- Preserve ordinary graph execution's existing exit-as-data behavior outside the workflow-exchange snapshot path.
- Add paired successful/failing native application regression coverage and document the contract required by Kiln's CI consumer.

## Impact

- **Files**: `src/engine/executor.rs`, `src/headless/shell.rs`, `src/workflow_exchange/core.rs`, `src/workflow_exchange/tests.rs`, `docs/workflow-exchange-application.md`, and the workflow-interchange specification.
- **Testing**: reproduce the nonzero failure with the native application; run focused workflow-exchange and existing ordinary-shell tests; exercise Kiln signed push/PR intake through the actual Lattice Unix service and exact-source command helper. Run repository-required checks after implementation.
- **Consumer and owner**: Kiln's GitHub CI adapter is the immediate consumer. Lattice maintainers own producer execution and terminal-result semantics; Kiln maintainers own CI mapping and the cross-repository smoke evidence.
- **Non-claims**: local execution evidence does not prove deployed GitHub delivery, remote publication, source trust, worker isolation, or release eligibility.

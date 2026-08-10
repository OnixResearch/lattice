## Why

The live process-restart drill exposed a bounded operator failure: immediately after the writer panicked, `runs show` returned only `run evidence store read failed`; after a normal writable reopen repaired redb, the same inspection succeeded. redb 3.1.1 intentionally refuses read-only open when the prior writer unwound during panic because allocator-state repair requires write access. Lattice must distinguish this state from an active owner, remain non-mutating during inspection, and provide an explicit recovery path.

## What Changes

- Classify read-only redb open failures into active-owner, recovery-required, and unavailable diagnostics without exposing raw filesystem or backend error text.
- Keep `runs show` strictly read-only and direct operators to an explicit `runs repair` command when redb requires write-mode recovery.
- Add `runs repair` as an explicit mutating operation that opens only an existing store, performs redb-owned recovery, closes it, and creates no Lattice application records.
- Add process-boundary tests for clean exit, panic unwind, abrupt abort, active ownership, successful explicit repair, missing-store refusal, and post-repair safe inspection.

## Impact

- **Files**: `src/run/shell.rs`, `src/main.rs`, `tests/test_run_inspection.rs`, `README.md`, and Cairn lifecycle artifacts.
- **Testing**: focused pre/post run-inspection tests, positive and negative process fixtures, repository checks, Octet comparison, Cairn validation/gates, and Nix checks.

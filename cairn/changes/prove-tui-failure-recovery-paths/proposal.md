## Why

The real PTY operator check now proves that the TUI can open, run, and inspect a successful workflow. Failure, cancellation, rerun, diagnostics, and resize behavior remain covered mainly by in-process state tests. Those tests cannot prove that raw terminal input, background process teardown, rendering, persistence, and subsequent navigation still compose after a real failure or terminal-size change.

Operators need continuous process-boundary evidence that a failed or canceled workflow remains inspectable and recoverable, that invalid workflows do not start effects, and that resizing does not strand modal state or crash the terminal application.

## What Changes

- Extend the real portable-PTY harness with a fail-once workflow that renders a failure, persists failed details, remains interactive, and succeeds only through an explicit run-history rerun action.
- Add a fatal-validation path that opens diagnostics, starts no workflow effect, creates no run record, and remains navigable.
- Add a long-running process case that receives TUI cancellation, prevents pending work from completing, persists cancellation/failure evidence according to the existing model, and proves the TUI still handles history and quit input.
- Resize the live PTY through constrained and restored dimensions while workspace and modal surfaces are active, verifying panic-free rendering and preserved navigation state.
- Keep assertions semantic and ANSI-tolerant, retain the existing non-terminal rejection case, and run the expanded matrix continuously through `scripts/operator-e2e.rs`.

## Impact

- **Files**: `tests/test_tui_ux_completion.rs`, `tests/common/mod.rs`, focused TUI tests when gaps are found, `scripts/operator-e2e.rs`, `README.md`, and native Cairn lifecycle artifacts; production TUI code only where a process test exposes a real behavior defect.
- **Testing**: successful control, runtime failure and explicit rerun, validation rejection, cancellation and child teardown, PTY resize, non-terminal rejection, focused TUI tests, `scripts/check.rs`, Octet, Cairn validation/gates, and bounded Nix flake validation on supported Unix/Linux hosts.

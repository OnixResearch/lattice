## Phase 1: PTY harness and deterministic fixtures

- [ ] [parallel] Extend PTY helpers with semantic phase waits, real resize operations, bounded child/process cleanup, and pure positive and negative observation checks that reject stale-surface-only completion. r[lattice.tui_ux.process_failure_recovery]
- [ ] [parallel] Add deterministic fail-once, invalid-workflow, and cancellable long-running fixtures with unique readiness, attempt, and forbidden-effect markers and no production fault hooks. r[lattice.tui_ux.process_failure_recovery]

## Phase 2: Failure and recovery paths

- [ ] [serial] Drive a failed workflow through the real PTY, inspect its failed node details, explicitly rerun it from history, prove the fresh run succeeds, and verify both immutable run records and details after exit. r[lattice.tui_ux.process_failure_recovery]
- [ ] [serial] Drive fatal workflow validation through the real PTY, inspect diagnostics, prove no effect or run record was created, close the overlay, and prove normal navigation remains available. r[lattice.tui_ux.process_failure_recovery]
- [ ] [serial] Cancel a ready long-running workflow through PTY input, prove pending/forbidden work does not complete, inspect the terminal record, and prove history and quit input remain usable. r[lattice.tui_ux.process_failure_recovery]
- [ ] [serial] Resize the live PTY through constrained and restored dimensions with workspace and modal state active, assert semantic surfaces after each resize, and verify navigation and persisted state remain coherent. r[lattice.tui_ux.process_failure_recovery]

## Phase 3: Continuous validation

- [ ] [serial] Keep the successful PTY control and non-terminal rejection cases, add the expanded failure matrix to `scripts/operator-e2e.rs`, and document Unix/Linux scope plus terminal-rendering and external-process non-claims. r[lattice.tui_ux.process_failure_recovery]
- [ ] [serial] Run focused TUI tests, `scripts/operator-e2e.rs`, `scripts/check.rs`, Octet, Cairn validation and all three change gates, then run bounded Nix flake validation and archive exact evidence. r[lattice.tui_ux.process_failure_recovery]

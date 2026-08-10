# TUI failure-recovery validation

Date: 2026-07-23

## Result

The continuous portable-PTY matrix now drives the real Lattice TUI through deterministic runtime failure and inspection, an explicit fresh rerun, fatal validation rejection without effects, active cancellation after child readiness, constrained and restored terminal resize, continued navigation, history inspection, keyboard exit, successful execution, and non-terminal rejection.

The cancellation scenario exposed and fixed a production defect: aborting the outer Tokio task did not stop a bounded shell already running in `spawn_blocking`. Each TUI run now owns a fresh shared cancellation token. The TUI sets that token before aborting its async run task, and shell execution forwards it to `bounded-exec`, which owns process-group teardown and reaping. Positive and negative executor tests prove that a false token permits completion and that cancellation after readiness returns `process cancelled` without performing the delayed forbidden effect.

This establishes semantic TUI surfaces and bounded local process observations on the checked Unix/x86_64-Linux host. It does not prove byte-identical rendering across terminal emulators, unsupported-platform behavior, arbitrary child or whole-host cleanup, workflow semantic correctness, external cancellation, or production readiness.

## Baseline

Before the cancellation fix, the new real-PTY failure/rerun, validation, and resize cases passed individually. The cancellation case rendered and persisted cancellation but then observed its delayed forbidden effect, proving that app bookkeeping had stopped while the blocking child remained live. The oracle was retained and the product path was fixed rather than weakening the assertion.

## Focused evidence

- `nix develop -c cargo test --test test_executor cancellation_token -- --nocapture`: both explicit-token positive and negative cases passed.
- `nix develop -c cargo test --test test_tui_ux_completion -- --nocapture`: the complete PTY file passed, including failure/rerun, diagnostics/no effect, cancellation/no pending effect, resize/continued input, successful control, and non-terminal rejection.
- `nix develop -c cargo test --lib tui::app -- --nocapture`: 39 focused app tests passed.
- `scripts/operator-e2e.rs`: the TUI, webhook, scheduler, and orchestration process matrices passed.
- Pure fresh-surface tests reject empty expectations, out-of-bounds cursors, stale-only surfaces, and incomplete fresh surfaces.
- Durable assertions require two independent failed/rerun identities, no invalid-workflow run, canceled replay evidence, absence of forbidden effects, preserved workflow state after resize, and successful keyboard exit.

## Repository rails

- `scripts/check.rs`: passed after the final implementation, including operator E2E, package-scoped format, tests, all-target compilation, Clippy, and Cairn spec checks.
- `nix develop -c octet-precommit-check`: 902 accepted findings, 0 new, 0 stale after guarded migration of three moved pre-existing finding identities.
- Cairn repository validation and proposal/design/tasks gates: passed.
- `nix flake check -L --option secret-key-files ''`: final rerun passed all x86_64-Linux checks; aarch64-darwin, aarch64-linux, and x86_64-darwin were omitted as incompatible systems. An earlier run hit one transient bounded-credential assertion without printing its diagnostic; the assertion was made diagnostic-bearing, its focused test passed, and the complete flake rerun passed.

## Why

A production-shaped Animus dogfood run was terminated after Lattice atomically reserved its round budget and acknowledged the dispatch but before the headless runner persisted a `RunRecord`. Recovery correctly returned `PendingUnobserved` and did not retry the uncertain remote effect, but cancellation could only set `cancellation_requested`; the instance remained `running` forever with its dispatch, active run, and reservation still present.

Operators need a bounded, authority-checked way to close that exact state without claiming the provider did or did not execute, fabricating workflow success, retrying a possibly completed effect, or undercounting its reserved budget. The live drill also proved that run IDs were only inferred from persisted records: two successive process lifetimes allocated run `2` before the interrupted dispatch had a record, and recovery could not distinguish which dispatch owned the eventual record.

## What Changes

- Atomically reserve globally unique numeric run IDs before effects, reject duplicate run-record writes, and persist an explicit unscoped or orchestration instance/dispatch owner on every run record.
- Require recovery and abandonment to match the exact orchestration run owner rather than accepting a foreign or legacy record with the same numeric ID.
- Add an event-sourced `AbandonPendingDispatch` transition and `DispatchAbandoned` event for cancellation-requested dispatches that have no persisted run record.
- Add `orchestration abandon <instance> --actor <authority>` as the explicit imperative shell after `recover` reports `PendingUnobserved`.
- Conservatively consume the full reservation for acknowledged dispatches, release an unacknowledged reservation without consumption, clear active dispatch state, and terminate the instance as failed with a fixed safe non-claim summary.
- Reject unauthorized, non-cancelled, non-pending, mismatched, observed-run, repeated, and terminal-state abandonment without mutation.
- Preserve deterministic replay and document the live dogfood evidence and remaining operator-observability follow-ups.

## Impact

- **Files**: `src/store/mod.rs`, `src/headless.rs`, `src/orchestration/core.rs`, `src/orchestration/shell.rs`, `src/main.rs`, store/headless/orchestration tests, README operator guidance, and native Cairn lifecycle artifacts.
- **Testing**: concurrent allocation, duplicate-write, owner-mismatch, core/shell/CLI positive and negative tests, full `scripts/check.rs`, Octet, Cairn gates/validation, and bounded Nix flake validation.

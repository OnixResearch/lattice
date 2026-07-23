## Why

Lattice continuously verifies scheduler death before a workflow effect and ordinary webhook duplicate delivery, but it does not terminate either process after an externally observable effect and before terminal run, trigger, or schedule persistence. That untested window is the point where a restart or redelivery can repeat an effect even though local history contains no completed first attempt.

Operators need deterministic evidence for the actual at-least-once boundary before later idempotency hardening can be evaluated honestly. A process exit, a marker file by itself, or a successful restart is insufficient: the check must preserve the crashed durable state, prove the effect happened before death, compare state before and after restart, and state that local observations do not establish exactly-once external effects.

## What Changes

- Add production-shaped scheduler and webhook fault-injection workflows that perform an observable local effect and then kill the owning Lattice process before terminal persistence, without adding production fault hooks.
- Classify pre-restart durable state, restart/redelivery behavior, effect-attempt counts, response outcomes, and terminal records through deterministic test helpers and a reviewed expected-observation profile.
- Prove the scheduler remains due and can repeat the effect after restart when no terminal run or schedule advancement was persisted.
- Prove the current webhook redelivery behavior at the same post-effect boundary and preserve that evidence as the baseline for the follow-up atomic-idempotency change.
- Run the scenarios continuously through `scripts/operator-e2e.rs` and document the at-least-once and exactly-once non-claims.

## Impact

- **Files**: `tests/common/mod.rs`, `tests/test_scheduler.rs`, `tests/webhooks.rs`, `scripts/operator-e2e.rs`, `README.md`, and native Cairn lifecycle artifacts.
- **Testing**: paired successful controls and forced-death cases, copied crash-state inspection, restart/redelivery assertions, focused operator E2E, `scripts/check.rs`, Octet, Cairn validation/gates, and bounded Nix flake validation on supported Unix/Linux hosts.

# Webhook atomic-admission validation

Date: 2026-07-23

## Result

Webhook idempotency now has one durable local winner before workflow effects. A pure core derives a domain-separated BLAKE3 identity from tagged trigger and idempotency-key inputs, rejects malformed/missing/mismatched inputs, preserves `IdempotencyPolicy::None` as an explicit no-claim path, and maps authoritative store outcomes to claimed execution or duplicate rejection.

One redb write transaction checks the immutable claim identity, reserves the global run ID, inserts the safe claim, and inserts a safe started-history fact without persisting the raw key. Concurrent write transactions serialize on the store: one returns `Inserted`, every loser returns the same existing run owner, and no loser advances the run counter or starts workflow execution. The listener commits this transaction before invoking headless execution with the reserved ID.

Claims survive terminal failure and process death. An observed terminal failure updates only the matching history row to `Failed`; the immutable claim remains. Post-effect listener death leaves one started claim/history fact and no terminal run, and restart redelivery receives the stable duplicate conflict without a second local attempt. Idempotency-disabled `Allow` triggers produce independent runs and effects without hidden claims.

This establishes local at-most-one admission under the checked single active redb owner. It does not prove remote effect commitment, provider deduplication, distributed multi-writer safety, response delivery, external cancellation, or exactly-once external execution. A committed started claim without terminal run evidence remains intentionally uncertain and can suppress a delivery whose effect was not externally observed.

## Baseline

The dependency archive `cairn/archive/2026-07-23-characterize-post-effect-crash-boundaries` retained the former webhook duplicate-risk profile: listener death after the first local effect but before terminal persistence allowed redelivery to perform a second identified local attempt. The pre-change sequential listener and duplicate-history tests passed, but they did not close the read/execute/write race.

## Focused evidence

- `nix develop -c cargo test --test test_webhook_claims`: four pure/store cases passed, including malformed/missing/cross-trigger/no-policy inputs and an eight-thread redb barrier race with one inserted owner and seven stable duplicates.
- `nix develop -c cargo test --test webhooks`: 16 cases passed, including simultaneous established loopback clients, duplicate response before winner gate release, one effect/run/claim/history, failed-winner retention, no-policy concurrent execution, and post-effect death/restart suppression.
- `nix develop -c cargo test --test test_store`, `test_headless`, `test_webhook_credential_boundaries`, `test_post_effect_crash_profile`, and `test_triggers`: passed.
- `scripts/operator-e2e.rs`: TUI, atomic webhook, scheduler, and orchestration process matrices passed.
- Safe serialization assertions reject raw-key persistence in claim and started-history JSON. Bounded claim listing and listener in-flight limits prevent unbounded collection/task growth.

## Repository rails

- `scripts/check.rs`: passed after final implementation, including operator E2E, package-scoped format, tests, all-target compilation, Clippy, and Cairn spec checks.
- `nix develop -c octet-precommit-check`: 902 accepted findings, 0 new, 0 stale after guarded migration of two moved pre-existing file-length identities; all newly introduced policy findings were fixed rather than accepted.
- Cairn repository validation and proposal/design/tasks gates: passed.
- `nix flake check -L --option secret-key-files ''`: all x86_64-Linux checks passed; aarch64-darwin, aarch64-linux, and x86_64-darwin were omitted as incompatible systems.

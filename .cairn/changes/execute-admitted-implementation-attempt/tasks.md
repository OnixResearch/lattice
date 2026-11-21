## Phase 1

- [x] [serial] Bind one exact immutable coordinator request and admit isolated worker execution. r[lattice.implementation_attempt.consumer]
- [x] [serial] Persist transport intent and recover the existing candidate without duplicate execution. r[lattice.implementation_attempt.consumer.scenario.recovery]
- [x] [serial] Expose run/recover CLI, focused positive/negative fixtures and truthful qualification instructions. r[lattice.implementation_attempt.consumer]
- [x] [serial] Admit distinct Cairn artifact source identity without changing existing issue/task identities. r[lattice.implementation_attempt.consumer]
- [x] [serial] Export only the exact retained produced snapshot and native receipt; reject substituted, partial and replay-conflicting exports. r[lattice.implementation_attempt.consumer.scenario.recovery]
- [x] [serial] Record the agent revision admission mode with the intent: without `--revision-store` the intent stays the bare request identity; with one it is a canonical `lattice.implementation-attempt-intent.v1` record of request identity, session, admission and pin (`src/implementation_attempt/cli/session/intent.rs`). Recovery, inspection and export read the mode back, never re-derive it, read an intent without an admission field as `unmanaged`, refuse inconsistent or non-canonical records, and require the store for a managed attempt. Tests: `src/implementation_attempt/cli/session/intent.rs` unit tests and `src/implementation_attempt/cli/tests/revisions.rs`. r[lattice.implementation_attempt.consumer.scenario.admission_mode] r[lattice.implementation_attempt.consumer.scenario.recovery]
- [x] [serial] Define the refusal record's `dispatched` as "a worker effect of the refused attempt may have started": `true` for `substituted_request`, `export_revoked` and a `profile_revision_superseded` attempt whose directory holds an intent other than the closure marker; `false` for `stale_fence`, `crossed_attempt`, `revision_not_admitted`, `revision_blocked` and a superseded attempt without such an intent (`src/implementation_attempt/cli/refusal.rs`). Tests: `src/implementation_attempt/cli/tests.rs`, `src/implementation_attempt/cli/tests/revisions.rs`. r[lattice.implementation_attempt.consumer.scenario.refusal]

## Local verification boundary

The parent ran `cargo test --locked implementation_attempt::cli::export::tests -- --nocapture`:
all three tests passed. They exercise exact retained candidate export/replay,
missing native results, substituted and symbolic-link output, and nested,
empty and executable files with unexpected-directory and missing-member rejection.
Native Cairn proposal, design and tasks gates passed with no issues; repository
validation passed all 49 specifications. `scripts/check.rs` and
`octet-precommit-check` passed. The x86_64-linux Nix flake check passed all 32
checks; Nextest ran 1,438 passing tests with 9 skipped. Octet retained 1,056 root
and 25 core accepted IDs with no new or stale IDs. The actual export-candidate
CLI help was exercised. These fixture and structural observations do not qualify
a live OMP worker, model endpoint, deployment boundary, other platform or
coordinator acceptance.

## Coordinator consumer evidence

`cargo test --locked -p lattice --lib -- implementation_attempt` passed 48 tests.
They cover the profile-revision framing, coordinator network and dispatch-plan
declarations, fence records, typed stale, crossed, substituted and
superseded-revision refusals, closure on recovery without intent, blocked-worker
recovery, and Cairn artifact source and identity-byte fixtures.

[`evidence/composed-qualification.json`](evidence/composed-qualification.json)
records Kiln `70c8f217`'s unmodified `kiln-lattice-dispatch --worker-request`
transport driving the real Lattice binary of this change. The worker was the
reproduced OMP 18.18.7 build (executable BLAKE3 `7ee769b6…`) in the recorded
bubblewrap 0.11.0. It reached only a local fault endpoint and made no external
model calls. The record covers:

- a normalized failed terminal that Kiln accepted;
- an exact duplicate dispatch that recovered without a second request;
- closure after controller loss before start;
- recovery after controller loss during the run;
- Kiln- and Lattice-side crossed, stale and substituted refusals;
- forwarded cancellation that ended with a cancelled result.

Its `current_run` is the 2026-11-21 re-run on the final integrated binary
(`bin/lattice` BLAKE3 `ba97c208…`, `OUT=/tmp/k45r`) with every other pin
unchanged. Every scenario exit code and endpoint request count matches the
2026-11-20 run, which the file keeps as history. The only behavioral
difference is Lattice's `substituted_request` refusal, which now carries
`dispatched: true` and the new message because the attempt directory holds
another request's intent; OMP session ids differ per run. A first re-run
attempt under a long `OUT` root failed because the broker socket path exceeded
the 108-byte Unix limit; it is retained and recorded as `failed_long_out`.

It does not qualify a live model, a model-produced candidate, Kiln capture,
verification or acceptance, or the earlier recorded isolation-tools path.

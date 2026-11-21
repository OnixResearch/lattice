## Phase 1

- [x] [serial] Bind one exact immutable coordinator request and admit isolated worker execution. r[lattice.implementation_attempt.consumer]
- [x] [serial] Persist transport intent and recover the existing candidate without duplicate execution. r[lattice.implementation_attempt.consumer.scenario.recovery]
- [x] [serial] Expose run/recover CLI, focused positive/negative fixtures and truthful qualification instructions. r[lattice.implementation_attempt.consumer]
- [x] [serial] Admit distinct Cairn artifact source identity without changing existing issue/task identities. r[lattice.implementation_attempt.consumer]
- [x] [serial] Export only the exact retained produced snapshot and native receipt; reject substituted, partial and replay-conflicting exports. r[lattice.implementation_attempt.consumer.scenario.recovery]

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

It does not qualify a live model, a model-produced candidate, Kiln capture,
verification or acceptance, or the earlier recorded isolation-tools path.

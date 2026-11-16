## Phase 1

- [ ] [serial] Bind one exact immutable coordinator request and admit isolated worker execution. r[lattice.implementation_attempt.consumer]
- [ ] [serial] Persist transport intent and recover the existing candidate without duplicate execution. r[lattice.implementation_attempt.consumer.scenario.recovery]
- [ ] [serial] Expose run/recover CLI, focused positive/negative fixtures and truthful qualification instructions. r[lattice.implementation_attempt.consumer]
- [ ] [serial] Admit distinct Cairn artifact source identity without changing existing issue/task identities. r[lattice.implementation_attempt.consumer]
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


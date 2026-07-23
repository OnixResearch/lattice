## Phase 1: Observation model and fixtures

- [x] [serial] Define the pure post-effect crash observation/profile types and positive and negative classifier tests for missing checkpoints, premature terminal records, unexpected attempt counts, and unsupported outcomes. r[lattice.repository_validation.post_effect_crash_boundaries]
- [x] [serial] Add reusable Unix test-shell helpers that append an identified effect before killing the Lattice parent, plus bounded child teardown and copied-store inspection helpers without production fault hooks. r[lattice.repository_validation.post_effect_crash_boundaries]

## Phase 2: Process-boundary scenarios

- [x] [serial] Add a real scheduler forced-death case that proves the effect preceded death, the crash snapshot has no terminal run and retains the due schedule, restart performs the reviewed second attempt, and only the restarted terminal run persists. r[lattice.repository_validation.post_effect_crash_boundaries]
- [x] [serial] Add a real webhook forced-death case that proves the effect preceded listener death, captures the incomplete response and crash snapshot, redelivers the same idempotency key after restart, and matches the reviewed baseline effect/history profile. r[lattice.repository_validation.post_effect_crash_boundaries]
- [x] [serial] Keep successful non-crashing scheduler and webhook controls beside the negative forced-death cases and route all profiles through `scripts/operator-e2e.rs`. r[lattice.repository_validation.post_effect_crash_boundaries]

## Phase 3: Claims and validation

- [x] [serial] Document the observed at-least-once boundary, the follow-up webhook atomic-admission dependency, supported host scope, and explicit external-effect and exactly-once non-claims. r[lattice.repository_validation.post_effect_crash_boundaries]
- [x] [serial] Run focused process tests, `scripts/check.rs`, Octet, Cairn validation and all three change gates, then run bounded Nix flake validation and archive exact evidence. r[lattice.repository_validation.post_effect_crash_boundaries]

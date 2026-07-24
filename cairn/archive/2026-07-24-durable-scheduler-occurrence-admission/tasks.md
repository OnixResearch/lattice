## Phase 1: Pure occurrence contract

- [x] [serial] Add domain-separated BLAKE3 occurrence identity, immutable slot facts, bounded attempt/status models, and claim/recovery/retry/abandon decisions with positive and negative tests. r[lattice.scheduling.scheduler_occurrence_admission]
- [x] [serial] Extend durable run ownership for schedule occurrences and add pure exact-owner recovery checks that reject absent, foreign, mismatched, and legacy-unscoped records. r[lattice.scheduling.scheduler_occurrence_admission]

## Phase 2: Atomic store and execution shell

- [x] [serial] Add one redb transaction that verifies the exact due slot, reserves a run, inserts pending occurrence state, and advances the schedule; add contention, stale-slot, duplicate, overflow, and rollback tests. r[lattice.scheduling.scheduler_occurrence_admission]
- [x] [serial] Route scheduled execution through occurrence ownership, settle exact terminal runs, and leave load/execution/settlement gaps pending without automatic replay. r[lattice.scheduling.scheduler_occurrence_admission]
- [x] [serial] Add bounded occurrence list/show/recover plus one acknowledged explicit retry and acknowledged abandonment, with fail-closed revision, owner, state, attempt-limit, and missing-record tests. r[lattice.scheduling.scheduler_occurrence_admission]

## Phase 3: Process evidence and validation

- [x] [serial] Update real pre-effect and post-effect death fixtures so ordinary daemon restart performs no second attempt, exact recovery reports pending, explicit retry is the only second attempt, and abandonment closes without execution. r[lattice.scheduling.scheduler_occurrence_admission]
- [x] [serial] Document local automatic at-most-one admission, pending/abandoned uncertainty, acknowledgement requirements, single-store-owner scope, and external exactly-once non-claims. r[lattice.scheduling.scheduler_occurrence_admission]
- [x] [serial] Run focused scheduler/store/headless tests, operator E2E, `scripts/check.rs`, Octet, Cairn validation and gates, traceability, and bounded Nix flake checks; archive exact evidence. r[lattice.scheduling.scheduler_occurrence_admission]

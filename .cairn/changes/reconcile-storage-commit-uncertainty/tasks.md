# Tasks: Reconcile storage commit uncertainty

## Model and dependency

- [x] [serial] Record current scheduler, workflow, approval, reservation, repair, and crash-boundary baselines. r[lattice.persistence_uncertainty.commit_outcomes]
- [x] [serial] Pin the published Transactional Reconciliation Core revision and record retained Lattice policy and authority. r[lattice.persistence_uncertainty.shared_core]
- [x] [serial] Add distinct persistence and effect uncertainty states in the pure Lattice core. r[lattice.persistence_uncertainty.commit_outcomes]
- [x] [parallel] Add positive and negative state-transition tests. r[lattice.persistence_uncertainty.validation]

## Redb shell and effect gate

- [x] [serial] Classify Redb commit outcomes without converting uncertainty into definite failure. r[lattice.persistence_uncertainty.commit_outcomes]
- [x] [serial] Quarantine stores after uncertain, repaired, corrupt, decode-failed, or inconsistent observations. r[lattice.persistence_uncertainty.quarantine]
- [x] [serial] Require exact published reservation evidence before effect dispatch. r[lattice.persistence_uncertainty.effect_gate]
- [x] [serial] Add reopen and read-back reconciliation for exact operation, plan, revision, attempt, and reservation identities. r[lattice.persistence_uncertainty.reconciliation]
- [x] [parallel] Add commit-error, process-death, repair, cache, corruption, missing-row, conflict, foreign-attempt, and dispatch-block tests. r[lattice.persistence_uncertainty.validation]

## Operator and evidence surfaces

- [x] [serial] Extend read-only status and explicit recover, retry, and abandon paths with persistence-uncertainty diagnostics. r[lattice.persistence_uncertainty.operator_actions]
- [x] [serial] Keep Redb repair separate from workflow reconciliation and effect authority. r[lattice.persistence_uncertainty.repair_boundary]
- [x] [parallel] Run scheduler and workflow storage-fault campaigns across commit and restart boundaries. r[lattice.persistence_uncertainty.validation]
- [ ] [serial] Update non-claims and run focused tests, operator E2E, formatting, Clippy, Octet, Cairn gates, and Nix checks. r[lattice.persistence_uncertainty.validation]

## Verification Coverage

- `Scenario: Reservation commit succeeds` -> normal dispatch test
- `Scenario: Reservation commit is uncertain` -> dispatch-block test
- `Scenario: Reopened row matches reservation` -> reconciliation test
- `Scenario: Redb repair completes` -> repair-boundary test
- `Scenario: Operator requests retry before reconciliation` -> rejection test

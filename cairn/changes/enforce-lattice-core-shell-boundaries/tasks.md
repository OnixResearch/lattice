# Tasks: Enforce Lattice core and shell crate boundaries

## Phase 1: Baseline and ownership

- [x] [serial] Record focused workflow, scheduler-occurrence, replay, orchestration, serialization, and compatibility test results before source moves. r[lattice.repository_validation.fcis_compatibility]
- [ ] [serial] Inventory candidate core types, active-change ownership, public callers, persisted forms, BLAKE3 identities, and every host dependency used by the candidate cohort. r[lattice.repository_validation.fcis_crate_boundary]
- [ ] [parallel] Define the reviewed core, shell, adapter, and forbidden-dependency topology with an explicit API and dependency budget. r[lattice.repository_validation.fcis_architecture_guard]

## Phase 2: Core crate and explicit facts

- [ ] [serial] Add `lattice-core` with owned workflow, scheduler-admission, and replay DTOs plus deterministic functions. r[lattice.repository_validation.fcis_crate_boundary]
- [ ] [serial] Move selected orchestration decisions after reconciling source ownership with active scheduler and orchestration pilots. r[lattice.repository_validation.fcis_crate_boundary]
- [x] [serial] Replace core wall-clock reads with explicit timestamp inputs and keep clock-reading convenience adapters in the shell package. r[lattice.repository_validation.fcis_explicit_facts]
- [ ] [serial] Keep redb, filesystem, environment, HTTP, process, credential, Tokio, Nickel, CLI, and TUI work in the shell package. r[lattice.repository_validation.fcis_crate_boundary]

## Phase 3: Compatibility and tests

- [ ] [parallel] Add positive core tests for workflow mutation, schedule admission and recovery, replay construction and validation, and selected orchestration transitions. r[lattice.repository_validation.fcis_compatibility]
- [ ] [parallel] Add negative tests for stale and duplicate identities, invalid timestamps, malformed replay facts, retry denial, arithmetic bounds, and invalid state transitions. r[lattice.repository_validation.fcis_compatibility]
- [ ] [serial] Add compatibility adapters and frozen fixtures for accepted workflow, persisted record, schedule occurrence, replay bundle, and orchestration bytes and identities. r[lattice.repository_validation.fcis_compatibility]

## Phase 4: Enforcement and closeout

- [ ] [serial] Add Octet architecture declarations and compile or source-policy fixtures that reject forbidden core dependencies and shell-to-core direction violations. r[lattice.repository_validation.fcis_architecture_guard]
- [ ] [parallel] Add a negative fixture proving that a new core violation cannot be admitted through the existing Octet baseline. r[lattice.repository_validation.fcis_architecture_guard]
- [ ] [serial] Document crate ownership, explicit fact flow, migration adapters, and static-analysis non-claims. r[lattice.repository_validation.fcis_claim_boundary]
- [ ] [serial] Run focused tests, `scripts/check.rs`, Octet on all relevant targets, Cairn validation, proposal/design/tasks gates, and the relevant Nix checks. Record exact results before sync and archive. r[lattice.repository_validation.fcis_compatibility] r[lattice.repository_validation.fcis_architecture_guard]

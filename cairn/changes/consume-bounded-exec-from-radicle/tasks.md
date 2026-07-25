## Phase 1: Evidence and baseline

- [x] [serial] Verify the accepted Bounded Exec publication receipt, concrete `onix-core` node/HTTPS deployment references, OnixOS multi-seed admission, canonical RID, endpoint policy, and exact existing dependency revision before editing source declarations. r[lattice.repository_validation.radicle_bounded_exec_source]
- [x] [serial] Record the smallest relevant Cargo, Nix, source-identity, bounded-process, and repository-check baselines without modifying unrelated active work. r[lattice.repository_validation.radicle_bounded_exec_behavior]

## Phase 2: Source cutover

- [x] [serial] Replace the Bounded Exec Cargo GitHub source with the accepted Radicle-backed HTTPS Git endpoint while retaining the exact revision. r[lattice.repository_validation.radicle_bounded_exec_source]
- [x] [serial] Replace the matching Nix flake input and Crane source-substitution identity with the same accepted endpoint and revision. r[lattice.repository_validation.radicle_bounded_exec_source]
- [x] [serial] Regenerate `Cargo.lock` with Cargo and `flake.lock` with a targeted Nix lock command; review generated source, revision, and content identities without manual lock edits. r[lattice.repository_validation.radicle_bounded_exec_locks]

## Phase 3: Deterministic validation

- [x] [serial] Add the pure Cargo/Nix/Crane/lock source-agreement validator and thin repository-check shell with stable diagnostics. r[lattice.repository_validation.radicle_bounded_exec_locks]
- [x] [parallel] Add positive fixtures for exact source agreement and negative fixtures for RID/endpoint mismatch, revision mismatch, stale GitHub entries, hidden path overrides, missing crates, altered source, and malformed producer evidence. r[lattice.repository_validation.radicle_bounded_exec_locks]
- [x] [parallel] Preserve and run focused positive and negative bounded-process tests for completion, failure, deadlines, truncation, cancellation, and process-group teardown. r[lattice.repository_validation.radicle_bounded_exec_behavior]

## Phase 4: Availability and closeout

- [x] [serial] Consume concrete `onix-core` deployment receipts and OnixOS acceptance references proving acquisition of the exact dependency through each admitted seed and with either seed unavailable. r[lattice.repository_validation.radicle_bounded_exec_availability]
- [x] [parallel] Prove unavailable Radicle paths fail without a Bounded Exec GitHub fallback and record the exact bounded diagnostic. r[lattice.repository_validation.radicle_bounded_exec_availability]
- [x] [parallel] Update README dependency provenance and non-claims while preserving historical GitHub links only where required for archived evidence. r[lattice.repository_validation.radicle_bounded_exec_source]
- [x] [serial] Run focused Cargo tests, `scripts/check.rs`, Octet, Cairn validation/gates, and bounded `nix flake check`; sync and archive only after producer and availability evidence is accepted. r[lattice.repository_validation.radicle_bounded_exec_behavior]

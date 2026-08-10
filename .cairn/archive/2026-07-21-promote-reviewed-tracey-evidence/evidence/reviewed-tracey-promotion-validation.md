# Reviewed Tracey evidence promotion validation

## Scope and non-claims

This evidence covers the promotion of six previously reviewed safe run-inspection and run-store-recovery scenarios into maintained Tracey evidence. Markers are attached to precise positive and negative tests; no product behavior, durable schema, authority boundary, or persistence format changed.

The resulting markers establish maintained test linkage for the accepted scenario wording. They do not prove whole-system correctness, complete traceability, release readiness, production readiness, provider truth, remote exactly-once effects, or external evidence anchoring. Tracey remains intentionally unsuccessful.

## Portfolio and adversarial audit

Four mechanisms were evaluated:

1. File-level markers were rejected because file presence does not establish complete scenario assertions.
2. Lexical promotion was rejected because matching names and diagnostics do not prove the accepted GIVEN/WHEN/THEN clauses.
3. Unguarded marker insertion was rejected because it cannot detect unrelated missing-set drift.
4. Assertion-bound markers plus a deterministic six-ID set transition were selected.

A local VibeThinker review accepted the safe-output, owner-binding, replay, and recovery non-claim candidates and challenged the not-recorded and missing-run candidates as insufficiently explicit. The implementation strengthened both challenged surfaces. It also added stricter wrong-instance, wrong-run, invalid replay CLI, and bounded recovery-output assertions beyond the advisory recommendation.

## Baseline

Before this change, `cairn traceability coverage --root .` exited unsuccessfully with:

- requirements: 702
- referenced: 335
- missing: 367
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `3def9d77607352a67fe3a12be08234941b189f73667b9b1474906a507f7bae8b`
- receipt BLAKE3: `e92f31bee419862d8f6596ca88264865c1b9642243ab26e40d254f8953edf244`

The complete receipt is retained at `tests/fixtures/traceability-candidate-promotion-baseline.json`.

## Promoted scenarios

Exactly these accepted identifiers gained maintained test markers:

- `lattice.workflow_execution.run_store_recovery.scenario.non-claim`
- `lattice.workflow_execution.safe_run_inspection.scenario.missing`
- `lattice.workflow_execution.safe_run_inspection.scenario.not_recorded`
- `lattice.workflow_execution.safe_run_inspection.scenario.owner_binding`
- `lattice.workflow_execution.safe_run_inspection.scenario.replay`
- `lattice.workflow_execution.safe_run_inspection.scenario.safe_output`

The bindings cover:

- safe JSON/text projection for arbitrary persisted details, replay summaries, and orchestration-private content;
- valid, missing, foreign, duplicate, wrong-instance, and wrong-run owner evidence;
- valid and invalid replay identities, invalid CLI exit, and summary omission;
- explicit `not_recorded` states without treating absence as contradictory evidence;
- bounded not-found failure when orchestration history contains a dispatch but no `RunRecord` exists; and
- exact fixed repair output with explicit forbidden-claim assertions.

## Exact transition invariant

`tests/traceability_promotion/core.rs` is a pure deterministic comparator. `tests/test_traceability_candidate_promotion.rs` supplies checked-in baseline/current receipts and exact marker/test bindings.

The positive case proves:

- baseline missing count: 367
- current missing count: 361
- removed set: exactly the six-ID allowlist
- added set: empty
- baseline dangling: empty
- current dangling: empty
- baseline and current coverage: invalid

The negative case rejects duplicate/unsorted IDs, an allowlist duplicate, an unexpected removed ID, an unexpected added ID, a dangling ID, a promoted ID that remains missing, and successful current coverage using deterministic issue codes.

The repository-validation requirement and its three scenarios add four accepted and four referenced IDs. Therefore final totals are 706 requirements and 345 referenced requirements while the unrelated missing set remains unchanged.

## Maintained debt inventory

The current inventory contains only the remaining debt:

- rows: 361
- reviewed rows: 0
- unassessed rows: 361
- missing-set BLAKE3: `df059bfa0b70e75996de6dc41990d0d744197b90bee9195bf1b873c505abaf3c`
- review-policy BLAKE3: `07b29b6d3b14922c5867947cb034834355174cc70c975d54b1b11739699f4f66`
- inventory BLAKE3: `e42fa6b22c8f5b08c30a06d56e068043f7ae2953a8c90497d91d89d900a218cf`
- issues: none
- inventory `valid: true`

Promoted IDs were removed from `cairn-policy/traceability-inventory.ncl`; keeping them as reviewed missing debt would now fail closed as stale decisions.

## Final coverage

The final maintained coverage command still exits unsuccessfully with:

- requirements: 706
- referenced: 345
- missing: 361
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `8e2f5b622c9fd99677e5187a71f8fcbdb3fa5ee763ae34bc053ffb8c731c9a02`
- receipt BLAKE3: `22637d7584cc23085a3d40b992451e7b4d8622f989ddf2baeeb5c723cce269af`

The complete receipt is retained at `tests/fixtures/traceability-candidate-promotion-current.json`.

## Validation

- Baseline focused inspection, recovery, inventory, and marker-alignment tests passed before edits.
- Post-change focused inspection tests: 14 passed.
- Post-change run-store recovery tests: 5 passed.
- Promotion invariant tests: 3 passed, including positive and fail-closed cases.
- Inventory tests: 6 passed.
- Marker-alignment tests: 2 passed.
- `scripts/traceability-inventory.rs generate` and `check` agree byte-for-byte.
- `scripts/check.rs` passes repository-owned formatting, the complete test suite, strict project-only Clippy, and Cairn spec checks.
- Cairn policy freshness, repository validation, proposal/design/tasks gates, and sync pass.
- `nix flake check -L --option secret-key-files ''` passes every configured `x86_64-linux` check; incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems are omitted.
- Octet retains 913 accepted/current findings with zero new and zero stale IDs under baseline `b3:ec5609b38db008f4145f4c38490b04290d8b52266e7b5a067d2cb29751ac208f`. Its underlying Cargo subprocess remains classified as exit code 101 while the reviewed zero-delta wrapper accepts it.
- Broad `cargo fmt --all` remains blocked by the pre-existing vendored Rats `ratcore/src/manual/doc/assertions.rs` absence; targeted project formatting, `scripts/check.rs`, and Nix formatting checks pass.

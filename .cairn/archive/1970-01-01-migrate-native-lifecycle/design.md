## Context

The legacy lifecycle tree is complete, but current Cairn rejects a repository that has only `cairn/`. Active tests and traceability fixtures also name the legacy path.

## Decisions

### Decision: Use Cairn's preview-bound copy plan

**Choice:** Execute `cairn migrate cairn` only with the reviewed preview hash. Keep both receipts in the archived change.

**Rationale:** The command binds every source and destination path to BLAKE3 identities and rejects drift before mutation.

### Decision: Rebase policy before migration

**Choice:** Snapshot the pinned upstream contracts and default policy. Keep Lattice traceability roots in a small Nickel overlay.

**Rationale:** The old generated policy does not satisfy the current workflow schema. A narrow overlay prevents another full policy fork.

### Decision: Remove the legacy tree after parity checks

**Choice:** Update active path references, verify migrated bytes, and then remove `cairn/`.

**Rationale:** One lifecycle source prevents silent divergence. Historical prose can retain old path statements when they describe past facts.

## Tests

- Positive: the migration plan executes, strict validation passes, and active checks read `.cairn/`.
- Negative: a legacy-only lifecycle root fails, and the unsafe traceability-root fixture still fails policy export.
- Regression: repository and host Nix checks pass without the old directory.

## Risks / Trade-offs

- Traceability fixtures bind exact policy bytes and require reviewed refreshes.
- Existing active changes move without gaining approval or completion status.
- The policy snapshot can drift from future Cairn revisions and requires an explicit update.

## Claim Boundaries

Migration proves copied path and byte identity under the selected plan. It does not prove requirement truth, implementation correctness, or release readiness.

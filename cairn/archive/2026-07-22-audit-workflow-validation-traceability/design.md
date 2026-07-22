# Design

## Boundary

The audit is a pure classification over the exact 29-ID workflow-validation subset of the current failing Tracey receipt. Evidence is limited to maintained repository tests and immutable byte snapshots.

## Invariants

- Every selected ID has exactly one explicit decision.
- Exact, partial, and no-exact classifications use direct clause-level evidence only.
- Evidence paths are safe and hashes match source bytes.
- Selected IDs remain missing and unmarked.
- Missing-set additions, removals, duplicates, order drift, dangling markers, or successful coverage fail closed.

## Validation

A Rust audit test validates the positive inventory and adversarial classification, evidence, marker, and coverage mutations. Inventory generation and Cairn gates remain authoritative.

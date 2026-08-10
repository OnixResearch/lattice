## Why

Lattice currently exposes 367 accepted requirements without maintained traceability markers, but the flat missing array does not record subsystem ownership or distinguish reviewed evidence candidates from debt that has not been semantically audited. Bulk marker insertion would manufacture coverage, while treating every unreviewed gap as absent behavior would manufacture an absence claim.

## What Changes

- Add a typed Nickel review policy that pins the exact missing-ID set by BLAKE3 and records only explicit reviewed decisions.
- Add a Rust cargo-script tool with a pure deterministic inventory core and thin Nickel/filesystem shell.
- Emit a sorted BLAKE3-bound JSON inventory partitioned by subsystem and review state.
- Record six safe run-inspection and run-store-recovery scenarios as exact maintained-evidence candidates without adding their Tracey markers.
- Keep every other current gap explicitly `unassessed`, preserve the exact 367-item missing array and zero dangling markers, and retain fail-closed Tracey coverage.

## Impact

- **Files**: `cairn-policy/traceability-inventory.ncl`, generated inventory JSON, `scripts/traceability-inventory.rs`, `scripts/traceability_inventory/core.rs`, maintained tests, repository-validation spec, and lifecycle evidence.
- **Testing**: positive deterministic generation, negative duplicate/stale/tampered input tests, Nickel contract evaluation, focused Rust tests, exact Tracey-set comparison, repository checks, Cairn gates, and Nix checks.

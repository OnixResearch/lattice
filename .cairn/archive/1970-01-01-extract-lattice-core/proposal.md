## Why

Lattice keeps pure value, expression, control-flow, and identity decisions inside the root host package. This shape does not enforce their authority boundary at compile time.

## What Changes

- Add one internal `lattice-core` crate.
- Move the selected pure modules into that crate without behavior changes.
- Keep current `lattice` module paths through explicit compatibility re-exports.
- Add positive and negative dependency-boundary tests.
- Extend traceability discovery to the repository-owned `crates/` tree.

## Impact

- **Files**: workspace manifest, four pure module families, compatibility modules, dependency fixtures, policy, and lifecycle evidence
- **Testing**: focused baseline parity, core tests, forbidden-dependency fixtures, Octet, Cairn, Clippy, and Nix checks

## Out of Scope

This slice does not move workflow format, full workflow validation, host execution, storage, network adapters, scheduling, or TUI code.

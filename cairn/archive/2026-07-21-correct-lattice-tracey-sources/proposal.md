## Why

Lattice's generated Cairn policy inherited the Cairn repository's `crates/` and `tools/` traceability roots even though Lattice keeps implementation and verification markers under `src/`, `tests/`, `examples/`, `scripts/`, and policy Nickel. The standard `cairn tracey coverage --root .` command therefore reported zero referenced requirements and concealed real missing and dangling linkage debt.

## What Changes

- Add a typed, repository-owned Nickel policy source and checked generated JSON.
- Point the default traceability profile at bounded Lattice-owned requirement, implementation, test, example, script, and policy roots while excluding vendored and generated source trees.
- Preserve the existing marker verbs and fail-closed missing/dangling behavior.
- Add positive profile tests and a negative unsafe-root fixture.
- Record the corrected source-discovery receipt separately from the remaining traceability debt.

## Impact

- **Files**: `cairn-policy/default.ncl`, `cairn-policy/contracts.ncl`, `cairn-policy/fixtures/`, `cairn-policy/generated/cairn-policy.json`, `flake.nix`, focused tests, and Cairn lifecycle evidence.
- **Compatibility**: Runtime workflow behavior is unchanged. Cairn policy consumers continue using `cairn-policy/generated/cairn-policy.json`, but Tracey now evaluates Lattice-owned source roots.
- **Testing**: pre-change zero-reference receipt; Nickel export and freshness checks; positive and negative profile tests; corrected Tracey receipt; repository checks; Cairn gates/validation; and bounded Nix checks.

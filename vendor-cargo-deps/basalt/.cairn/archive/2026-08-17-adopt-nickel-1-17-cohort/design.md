# Design: Adopt Nickel 1.17 for policy authoring

## Context

Basalt uses Nickel for human-authored policy, contracts, fixtures, and deterministic exports. Rust owns runtime enforcement.

## Decisions

### Decision: Use an exact Nickel source pin

**Choice:** Select Nickel `1.17.0` from commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`.

If the current Nix package set lacks this version, add a narrow upstream Nickel input.

**Rationale:** A broad Nixpkgs update would add unrelated policy and tool changes.

### Decision: Preserve generated-policy freshness

**Choice:** Regenerate policy exports with the new evaluator. Compare generated artifacts through the existing freshness rail.

Regenerate `flake.lock` only with Nix lock commands.

**Rationale:** Checked-in runtime policy must remain traceable to its Nickel source.

### Decision: Preserve policy and authority ownership

**Choice:** Nickel validates policy shape. Basalt Rust code retains runtime enforcement and authority decisions.

**Rationale:** Evaluator alignment does not transfer authority to configuration tooling.

### Decision: Test fail-closed behavior

**Choice:** Run valid policies and rejected unknown, duplicate, malformed, missing, and type-mismatched fixtures.

**Rationale:** Positive exports alone do not establish rejection behavior.

## Risks and trade-offs

- Export formatting can change without policy meaning changes.
- Diagnostic text can change. Stable fixture dispositions remain authoritative.
- Passing policy export does not prove runtime enforcement correctness.
